use regex::Regex;
use std::sync::OnceLock;

#[derive(Debug, Clone)]
pub struct ParsedFileBlock {
    pub path: String,
    pub content: String,
}

#[derive(Debug)]
pub struct ParseFileBlocksResult {
    pub blocks: Vec<ParsedFileBlock>,
    pub warnings: Vec<String>,
}

static OPENER_LINE: OnceLock<Regex> = OnceLock::new();
static CLOSER_LINE: OnceLock<Regex> = OnceLock::new();
static FENCE_LINE: OnceLock<Regex> = OnceLock::new();

fn opener_line() -> &'static Regex {
    OPENER_LINE.get_or_init(|| Regex::new(r"^---\s*FILE:\s*(.+?)\s*---\s*$").unwrap())
}

fn closer_line() -> &'static Regex {
    CLOSER_LINE.get_or_init(|| Regex::new(r"^---\s*END\s+FILE\s*---\s*$").unwrap())
}

fn fence_line() -> &'static Regex {
    FENCE_LINE.get_or_init(|| Regex::new(r"^\s{0,3}(```+|~~~+)").unwrap())
}

pub fn is_safe_ingest_path(p: &str) -> bool {
    if p.trim().is_empty() {
        return false;
    }

    if p.chars().any(|c| c.is_control() || c == '\0') {
        return false;
    }

    if p.starts_with('/') || p.starts_with('\\') {
        return false;
    }

    if p.len() >= 2 && p.chars().nth(1) == Some(':') {
        let first = p.chars().next().unwrap();
        if first.is_ascii_alphabetic() {
            return false;
        }
    }

    let normalized = p.replace('\\', "/");

    if normalized.split('/').any(|seg| seg == "..") {
        return false;
    }

    if !normalized.starts_with(".wiki/") {
        return false;
    }

    true
}

pub fn parse_file_blocks(text: &str) -> ParseFileBlocksResult {
    let normalized = text.replace("\r\n", "\n");
    let lines: Vec<&str> = normalized.split('\n').collect();

    let mut blocks = Vec::new();
    let mut warnings = Vec::new();

    let mut i = 0;
    while i < lines.len() {
        let line = lines[i];
        if let Some(caps) = opener_line().captures(line) {
            let path = caps.get(1).unwrap().as_str().trim().to_string();
            i += 1;

            let mut content_lines = Vec::new();
            let mut fence_marker: Option<char> = None;
            let mut fence_len = 0;
            let mut closed = false;

            while i < lines.len() {
                let line = lines[i];

                if let Some(caps) = fence_line().captures(line) {
                    let run = caps.get(1).unwrap().as_str();
                    let char = run.chars().next().unwrap();
                    let len = run.len();

                    if fence_marker.is_none() {
                        fence_marker = Some(char);
                        fence_len = len;
                    } else if Some(char) == fence_marker && len >= fence_len {
                        fence_marker = None;
                        fence_len = 0;
                    }
                    content_lines.push(line);
                    i += 1;
                    continue;
                }

                if fence_marker.is_none() && closer_line().is_match(line) {
                    closed = true;
                    i += 1;
                    break;
                }

                content_lines.push(line);
                i += 1;
            }

            if !closed {
                let path_label = if path.is_empty() { "(unnamed)" } else { &path };
                let msg = format!(
                    "FILE block \"{}\" was not closed before end of stream — likely truncation. Block dropped.",
                    path_label
                );
                eprintln!("[ingest] {}", msg);
                warnings.push(msg);
                continue;
            }

            if path.is_empty() {
                let msg = "FILE block with empty path skipped (LLM omitted the path after `---FILE:`).".to_string();
                eprintln!("[ingest] {}", msg);
                warnings.push(msg);
                continue;
            }

            if !is_safe_ingest_path(&path) {
                let msg = format!(
                    "FILE block with unsafe path \"{}\" rejected (must be under .wiki/, no .., no absolute paths).",
                    path
                );
                eprintln!("[ingest] {}", msg);
                warnings.push(msg);
                continue;
            }

            blocks.push(ParsedFileBlock {
                path,
                content: content_lines.join("\n"),
            });
        } else {
            i += 1;
        }
    }

    ParseFileBlocksResult { blocks, warnings }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_safe_ingest_path() {
        assert!(is_safe_ingest_path(".wiki/concepts/foo.md"));
        assert!(is_safe_ingest_path(".wiki/entities/bar.md"));
        
        assert!(!is_safe_ingest_path(""));
        assert!(!is_safe_ingest_path("   "));
        assert!(!is_safe_ingest_path("/etc/passwd"));
        assert!(!is_safe_ingest_path("C:/Windows/system32"));
        assert!(!is_safe_ingest_path(".wiki/../../../etc/passwd"));
        assert!(!is_safe_ingest_path("wiki/concepts/foo.md"));
        assert!(!is_safe_ingest_path("concepts/foo.md"));
    }

    #[test]
    fn test_parse_file_blocks_basic() {
        let text = r#"---FILE: .wiki/concepts/test.md---
# Test Concept
This is a test.
---END FILE---"#;

        let result = parse_file_blocks(text);
        assert_eq!(result.blocks.len(), 1);
        assert_eq!(result.blocks[0].path, ".wiki/concepts/test.md");
        assert!(result.blocks[0].content.contains("# Test Concept"));
        assert_eq!(result.warnings.len(), 0);
    }

    #[test]
    fn test_parse_file_blocks_crlf() {
        let text = "---FILE: .wiki/test.md---\r\nContent\r\n---END FILE---\r\n";
        let result = parse_file_blocks(text);
        assert_eq!(result.blocks.len(), 1);
        assert_eq!(result.blocks[0].path, ".wiki/test.md");
    }

    #[test]
    fn test_parse_file_blocks_unclosed() {
        let text = "---FILE: .wiki/test.md---\nContent without closing";
        let result = parse_file_blocks(text);
        assert_eq!(result.blocks.len(), 0);
        assert_eq!(result.warnings.len(), 1);
        assert!(result.warnings[0].contains("not closed"));
    }

    #[test]
    fn test_parse_file_blocks_empty_path() {
        let text = "---FILE: ---\nContent\n---END FILE---";
        let result = parse_file_blocks(text);
        assert_eq!(result.blocks.len(), 0);
        assert_eq!(result.warnings.len(), 1);
        assert!(result.warnings[0].contains("empty path"));
    }

    #[test]
    fn test_parse_file_blocks_unsafe_path() {
        let text = "---FILE: ../../../etc/passwd---\nContent\n---END FILE---";
        let result = parse_file_blocks(text);
        assert_eq!(result.blocks.len(), 0);
        assert_eq!(result.warnings.len(), 1);
        assert!(result.warnings[0].contains("unsafe path"));
    }

    #[test]
    fn test_parse_file_blocks_with_fence() {
        let text = r#"---FILE: .wiki/test.md---
# Test
```rust
fn main() {
    println!("---END FILE---");
}
```
---END FILE---"#;

        let result = parse_file_blocks(text);
        assert_eq!(result.blocks.len(), 1);
        assert!(result.blocks[0].content.contains("---END FILE---"));
    }
}
