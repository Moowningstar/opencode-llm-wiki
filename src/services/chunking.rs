use anyhow::Result;
use regex::Regex;
use std::sync::OnceLock;
use tiktoken_rs::CoreBPE;

/// Configuration for markdown chunking behavior.
#[derive(Debug, Clone)]
pub struct ChunkingConfig {
    /// Target characters per chunk (soft limit).
    pub target_chars: usize,
    /// Maximum characters per chunk (hard limit for splittable content).
    pub max_chars: usize,
    /// Minimum characters per chunk (smaller chunks get merged).
    pub min_chars: usize,
    /// Characters of overlap between adjacent chunks.
    pub overlap_chars: usize,
}

impl Default for ChunkingConfig {
    fn default() -> Self {
        Self {
            target_chars: 1000,
            max_chars: 1500,
            min_chars: 200,
            overlap_chars: 200,
        }
    }
}

/// A single chunk of markdown content with metadata.
#[derive(Debug, Clone)]
pub struct Chunk {
    /// 0-based position in emission order.
    pub index: usize,
    /// The visible content of the chunk.
    pub text: String,
    /// Heading breadcrumb (e.g., "## Intro > ### Usage").
    pub heading_path: String,
    /// Character offset into the original input.
    pub char_start: usize,
    /// Character offset (exclusive) into the original input.
    pub char_end: usize,
    /// True if this chunk exceeds max_chars due to indivisible content.
    pub oversized: bool,
    /// Token IDs (computed if tokenizer provided).
    pub token_ids: Option<Vec<u32>>,
    /// Token count (computed if tokenizer provided).
    pub token_count: Option<u32>,
}

/// Service for chunking markdown documents into embedding-sized pieces.
pub struct ChunkingService {
    config: ChunkingConfig,
    tokenizer: Option<CoreBPE>,
}

impl ChunkingService {
    /// Create a new chunking service with default configuration.
    pub fn new() -> Self {
        Self {
            config: ChunkingConfig::default(),
            tokenizer: None,
        }
    }

    /// Create a new chunking service with custom configuration.
    pub fn with_config(config: ChunkingConfig) -> Self {
        Self {
            config,
            tokenizer: None,
        }
    }

    /// Enable token counting using the specified tokenizer.
    pub fn with_tokenizer(mut self, tokenizer: CoreBPE) -> Self {
        self.tokenizer = Some(tokenizer);
        self
    }

    /// Chunk a markdown document into embedding-sized pieces.
    pub fn chunk_markdown(&self, content: &str) -> Result<Vec<Chunk>> {
        let mut config = self.config.clone();

        if config.max_chars < config.target_chars {
            config.max_chars = config.target_chars;
        }
        if config.overlap_chars >= config.target_chars {
            config.overlap_chars = config.target_chars / 2;
        }

        let (body, body_offset) = strip_frontmatter(content);
        if body.trim().is_empty() {
            return Ok(Vec::new());
        }

        let sections = split_into_sections(&body, body_offset);

        let mut chunks = Vec::new();
        let mut running_index = 0;

        for section in sections {
            let section_chunks = self.chunk_section(&section, &config)?;
            for mut chunk in section_chunks {
                chunk.index = running_index;
                running_index += 1;
                chunks.push(chunk);
            }
        }

        Ok(chunks)
    }

    fn chunk_section(&self, section: &Section, config: &ChunkingConfig) -> Result<Vec<Chunk>> {
        let text = &section.text;
        let body_start = section.body_start;
        let heading_path = &section.heading_path;

        if text.len() <= config.target_chars {
            let mut chunk = Chunk {
                index: 0,
                text: text.clone(),
                heading_path: heading_path.clone(),
                char_start: body_start,
                char_end: body_start + text.len(),
                oversized: false,
                token_ids: None,
                token_count: None,
            };
            self.compute_tokens(&mut chunk)?;
            return Ok(vec![chunk]);
        }

        let atoms = tokenize_atoms(text);
        let pieces = split_atoms_to_pieces(&atoms, config);
        let sized = size_pieces(&pieces, config);
        let merged = merge_small(&sized, config);
        let with_overlap = apply_overlap(&merged, config);

        let mut chunks = Vec::new();
        for piece in with_overlap {
            let mut chunk = Chunk {
                index: 0,
                text: piece.text.clone(),
                heading_path: heading_path.clone(),
                char_start: body_start + piece.offset,
                char_end: body_start + piece.offset + piece.text.len(),
                oversized: piece.text.len() > config.max_chars,
                token_ids: None,
                token_count: None,
            };
            self.compute_tokens(&mut chunk)?;
            chunks.push(chunk);
        }

        Ok(chunks)
    }

    fn compute_tokens(&self, chunk: &mut Chunk) -> Result<()> {
        if let Some(tokenizer) = &self.tokenizer {
            let tokens = tokenizer
                .encode_with_special_tokens(&chunk.text)
                .into_iter()
                .map(|t| t as u32)
                .collect::<Vec<_>>();
            chunk.token_count = Some(tokens.len() as u32);
            chunk.token_ids = Some(tokens);
        }
        Ok(())
    }
}

impl Default for ChunkingService {
    fn default() -> Self {
        Self::new()
    }
}

fn strip_frontmatter(content: &str) -> (String, usize) {
    if !content.starts_with("---\n") && !content.starts_with("---\r\n") {
        return (content.to_string(), 0);
    }

    let rest = &content[4..];
    let close_pattern = regex::Regex::new(r"(^|\n)---\s*(\n|$)").unwrap();
    
    if let Some(m) = close_pattern.find(rest) {
        let body_offset = 4 + m.end();
        return (content[body_offset..].to_string(), body_offset);
    }

    (content.to_string(), 0)
}

#[derive(Debug, Clone)]
struct Section {
    text: String,
    body_start: usize,
    heading_path: String,
}

fn split_into_sections(body: &str, body_offset: usize) -> Vec<Section> {
    let lines: Vec<&str> = body.split('\n').collect();
    let mut sections = Vec::new();

    let mut headings: std::collections::HashMap<usize, String> = std::collections::HashMap::new();
    let mut current_lines = Vec::new();
    let mut current_start = body_offset;
    let mut current_heading_path = String::new();
    let mut in_fence = false;
    let mut fence_marker = String::new();
    let mut char_cursor = body_offset;

    let flush = |sections: &mut Vec<Section>, lines: &Vec<&str>, start: usize, path: &str| {
        let text = lines.join("\n");
        if !text.trim().is_empty() {
            sections.push(Section {
                text,
                body_start: start,
                heading_path: path.to_string(),
            });
        }
    };

    for (i, line) in lines.iter().enumerate() {
        let line_len = line.len() + if i < lines.len() - 1 { 1 } else { 0 };

        if let Some(fence_match) = regex::Regex::new(r"^(`{3,}|~{3,})").unwrap().find(line) {
            if !in_fence {
                in_fence = true;
                fence_marker = fence_match.as_str().chars().next().unwrap().to_string();
            } else if line.starts_with(&fence_marker) && line.trim() == fence_marker {
                in_fence = false;
            }
            current_lines.push(*line);
            char_cursor += line_len;
            continue;
        }

        if !in_fence {
            if let Some(caps) = regex::Regex::new(r"^(#{1,6})\s+(.+?)\s*$").unwrap().captures(line) {
                flush(&mut sections, &current_lines, current_start, &current_heading_path);

                let level = caps.get(1).unwrap().as_str().len();
                let title = caps.get(2).unwrap().as_str().trim();
                headings.insert(level, title.to_string());

                for lvl in (level + 1)..=6 {
                    headings.remove(&lvl);
                }

                let mut path_parts = Vec::new();
                for lvl in 1..=6 {
                    if let Some(h) = headings.get(&lvl) {
                        path_parts.push(format!("{} {}", "#".repeat(lvl), h));
                    }
                }

                current_lines = vec![*line];
                current_start = char_cursor;
                current_heading_path = path_parts.join(" > ");
                char_cursor += line_len;
                continue;
            }
        }

        current_lines.push(*line);
        char_cursor += line_len;
    }

    flush(&mut sections, &current_lines, current_start, &current_heading_path);
    sections
}

#[derive(Debug, Clone)]
struct Atom {
    text: String,
    offset: usize,
    indivisible: bool,
    #[allow(dead_code)]
    kind: AtomKind,
}

#[derive(Debug, Clone, Copy)]
enum AtomKind {
    Code,
    Table,
    Paragraph,
    Blank,
}

fn tokenize_atoms(text: &str) -> Vec<Atom> {
    let lines: Vec<&str> = text.split('\n').collect();
    let mut atoms = Vec::new();
    let mut cursor = 0;
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i];

        if let Some(fence_match) = regex::Regex::new(r"^(`{3,}|~{3,})").unwrap().find(line) {
            let marker = fence_match.as_str();
            let start = cursor;
            let mut body_lines = vec![line];
            let mut j = i + 1;
            cursor += line.len() + 1;

            while j < lines.len() {
                body_lines.push(lines[j]);
                cursor += lines[j].len() + 1;
                if lines[j].starts_with(marker) && lines[j].trim() == marker {
                    j += 1;
                    break;
                }
                j += 1;
            }

            let content = body_lines.join("\n");
            atoms.push(Atom {
                text: content,
                offset: start,
                indivisible: true,
                kind: AtomKind::Code,
            });
            i = j;
            continue;
        }

        if line.starts_with('|') {
            let mut j = i;
            while j < lines.len() && lines[j].starts_with('|') {
                j += 1;
            }
            if j - i >= 2 {
                let start = cursor;
                let body_lines = &lines[i..j];
                let content = body_lines.join("\n");
                cursor += content.len() + if j < lines.len() { 1 } else { 0 };
                atoms.push(Atom {
                    text: content,
                    offset: start,
                    indivisible: true,
                    kind: AtomKind::Table,
                });
                i = j;
                continue;
            }
        }

        if line.trim().is_empty() {
            atoms.push(Atom {
                text: String::new(),
                offset: cursor,
                indivisible: false,
                kind: AtomKind::Blank,
            });
            cursor += line.len() + 1;
            i += 1;
            continue;
        }

        let start = cursor;
        let mut body_lines = Vec::new();
        while i < lines.len()
            && !lines[i].trim().is_empty()
            && !lines[i].starts_with('|')
            && !regex::Regex::new(r"^(`{3,}|~{3,})").unwrap().is_match(lines[i])
        {
            body_lines.push(lines[i]);
            cursor += lines[i].len() + 1;
            i += 1;
        }
        let content = body_lines.join("\n");
        atoms.push(Atom {
            text: content,
            offset: start,
            indivisible: false,
            kind: AtomKind::Paragraph,
        });
    }

    atoms
        .into_iter()
        .filter(|a| !matches!(a.kind, AtomKind::Blank) || !a.text.is_empty())
        .collect()
}

#[derive(Debug, Clone)]
struct Piece {
    text: String,
    offset: usize,
}

fn split_atoms_to_pieces(atoms: &[Atom], config: &ChunkingConfig) -> Vec<Piece> {
    let mut pieces = Vec::new();
    for atom in atoms {
        if atom.indivisible {
            pieces.push(Piece {
                text: atom.text.clone(),
                offset: atom.offset,
            });
            continue;
        }
        if matches!(atom.kind, AtomKind::Blank) {
            continue;
        }
        if atom.text.len() <= config.target_chars {
            pieces.push(Piece {
                text: atom.text.clone(),
                offset: atom.offset,
            });
            continue;
        }
        pieces.extend(recursive_split(&atom.text, atom.offset, config.target_chars));
    }
    pieces
}

fn recursive_split(text: &str, base_offset: usize, target_chars: usize) -> Vec<Piece> {
    static PARA_SEP: OnceLock<Regex> = OnceLock::new();
    let para_sep = PARA_SEP.get_or_init(|| Regex::new(r"(\n{2,})").unwrap());

    let para_pieces = split_keeping_sep(text, para_sep);
    let mut out = Vec::new();
    let mut cursor = base_offset;

    for chunk in para_pieces {
        if chunk.is_empty() {
            continue;
        }
        if chunk.len() <= target_chars {
            out.push(Piece {
                text: chunk.clone(),
                offset: cursor,
            });
            cursor += chunk.len();
            continue;
        }

        let splitters: Vec<(&str, Regex)> = vec![
            ("lines", Regex::new(r"(\n+)").unwrap()),
            ("sentences", Regex::new(r"([。！？!?；;]+\s*|(?:\.\s+))").unwrap()),
            ("spaces", Regex::new(r"(\s+)").unwrap()),
        ];

        let mut split_success = false;
        for (_, splitter) in &splitters {
            let subs = split_keeping_sep(&chunk, splitter);
            if subs.iter().all(|s| s.len() <= target_chars) && subs.len() > 1 {
                let mut sub_cursor = cursor;
                for s in subs {
                    if !s.is_empty() {
                        out.push(Piece {
                            text: s.clone(),
                            offset: sub_cursor,
                        });
                        sub_cursor += s.len();
                    }
                }
                cursor += chunk.len();
                split_success = true;
                break;
            }
        }

        if !split_success {
            let mut slice_cursor = cursor;
            for i in (0..chunk.len()).step_by(target_chars) {
                let end = std::cmp::min(i + target_chars, chunk.len());
                let piece = &chunk[i..end];
                out.push(Piece {
                    text: piece.to_string(),
                    offset: slice_cursor,
                });
                slice_cursor += piece.len();
            }
            cursor += chunk.len();
        }
    }

    out
}

fn split_keeping_sep(text: &str, sep: &Regex) -> Vec<String> {
    let mut out = Vec::new();
    let mut last = 0;

    for m in sep.find_iter(text) {
        let end = m.end();
        out.push(text[last..end].to_string());
        last = end;
    }

    if last < text.len() {
        out.push(text[last..].to_string());
    }

    out.into_iter().filter(|s| !s.is_empty()).collect()
}

fn size_pieces(pieces: &[Piece], config: &ChunkingConfig) -> Vec<Piece> {
    let mut out = Vec::new();
    let mut buf = String::new();
    let mut buf_offset: Option<usize> = None;

    for p in pieces {
        if p.text.is_empty() {
            continue;
        }

        if p.text.len() > config.target_chars {
            if !buf.is_empty() {
                if let Some(offset) = buf_offset {
                    out.push(Piece {
                        text: buf.clone(),
                        offset,
                    });
                }
            }
            out.push(p.clone());
            buf.clear();
            buf_offset = None;
            continue;
        }

        if buf.len() + p.text.len() > config.target_chars && !buf.is_empty() {
            if let Some(offset) = buf_offset {
                out.push(Piece {
                    text: buf.clone(),
                    offset,
                });
            }
            buf = p.text.clone();
            buf_offset = Some(p.offset);
            continue;
        }

        if buf.is_empty() {
            buf_offset = Some(p.offset);
        }
        buf.push_str(&p.text);
    }

    if !buf.is_empty() {
        if let Some(offset) = buf_offset {
            out.push(Piece { text: buf, offset });
        }
    }

    out
}

fn merge_small(pieces: &[Piece], config: &ChunkingConfig) -> Vec<Piece> {
    if pieces.len() < 2 {
        return pieces.to_vec();
    }

    let mut out: Vec<Piece> = Vec::new();
    for p in pieces {
        if let Some(last) = out.last_mut() {
            if last.text.len() < config.min_chars
                && last.text.len() + p.text.len() <= config.max_chars
            {
                last.text.push_str(&p.text);
                continue;
            }
        }
        out.push(p.clone());
    }

    out
}

fn apply_overlap(pieces: &[Piece], config: &ChunkingConfig) -> Vec<Piece> {
    if config.overlap_chars == 0 || pieces.len() < 2 {
        return pieces.to_vec();
    }

    let mut out = vec![pieces[0].clone()];
    for i in 1..pieces.len() {
        let prev = &pieces[i - 1];
        let curr = &pieces[i];
        let tail_start = prev.text.len().saturating_sub(config.overlap_chars);
        let tail_src = &prev.text[tail_start..];
        let snapped = snap_overlap_head(tail_src);
        out.push(Piece {
            text: format!("{}{}", snapped, curr.text),
            offset: curr.offset - snapped.len(),
        });
    }

    out
}

fn snap_overlap_head(tail: &str) -> String {
    static SENT_PATTERN: OnceLock<Regex> = OnceLock::new();
    let sent_pattern = SENT_PATTERN.get_or_init(|| Regex::new(r"[。！？!?.;；][\s]*").unwrap());

    if let Some(m) = sent_pattern.find(tail) {
        let after = m.end();
        if after > 0 && after < tail.len() {
            return tail[after..].to_string();
        }
    }

    static WS_PATTERN: OnceLock<Regex> = OnceLock::new();
    let ws_pattern = WS_PATTERN.get_or_init(|| Regex::new(r"\s").unwrap());

    if let Some(m) = ws_pattern.find(tail) {
        if m.start() < tail.len() - 1 {
            return tail[m.start() + 1..].to_string();
        }
    }

    tail.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_frontmatter() {
        let content = "---\ntitle: Test\n---\n# Content";
        let (body, offset) = strip_frontmatter(content);
        assert_eq!(body, "# Content");
        assert!(offset > 0);
    }

    #[test]
    fn test_chunk_small_document() {
        let service = ChunkingService::new();
        let content = "# Small Doc\nThis is a small document.";
        let chunks = service.chunk_markdown(content).unwrap();
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].heading_path, "# Small Doc");
    }

    #[test]
    fn test_chunk_with_code_block() {
        let service = ChunkingService::new();
        let content = r#"# Code Example
```rust
fn main() {
    println!("Hello");
}
```
Some text after."#;
        let chunks = service.chunk_markdown(content).unwrap();
        assert!(!chunks.is_empty());
        assert!(chunks[0].text.contains("```rust"));
    }

    #[test]
    fn test_chunk_with_table() {
        let service = ChunkingService::new();
        let content = r#"# Table Example
| Col1 | Col2 |
|------|------|
| A    | B    |
| C    | D    |"#;
        let chunks = service.chunk_markdown(content).unwrap();
        assert!(!chunks.is_empty());
        assert!(chunks[0].text.contains("| Col1 | Col2 |"));
    }
}
