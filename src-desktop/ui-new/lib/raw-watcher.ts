import { useWikiStore } from "@/stores/wiki-store"
import { enqueueBatch } from "./ingest-queue"
import { listDirectory, readFile } from "@/commands/fs"
import { checkIngestCache } from "./ingest-cache"
import { normalizePath } from "@/lib/path-utils"
import type { FileNode } from "@/types/wiki"

const POLL_INTERVAL = 5000
let intervalId: ReturnType<typeof setInterval> | null = null

const SUPPORTED_EXTENSIONS = new Set([
  "md", "mdx", "txt", "pdf", "docx", "pptx", "xlsx", "xls",
  "csv", "json", "html", "htm", "rtf", "xml", "yaml", "yml"
])

/**
 * Recursively collect all file paths from a FileNode tree
 */
function collectFilePaths(node: FileNode, result: string[] = []): string[] {
  if (!node.is_dir) {
    result.push(node.path)
  } else if (node.children) {
    for (const child of node.children) {
      collectFilePaths(child, result)
    }
  }
  return result
}

/**
 * Extract folder context from file path relative to .raw/ directory.
 * Example: "/project/.raw/papers/AI/attention.pdf" → "papers > AI"
 */
function extractFolderContext(filePath: string, rawDir: string): string {
  const normFile = normalizePath(filePath)
  const normRaw = normalizePath(rawDir)
  
  const relPath = normFile.replace(normRaw + "/", "")
  const parts = relPath.split("/")
  parts.pop()
  
  return parts.length > 0 ? parts.join(" > ") : ""
}

/**
 * Check if file extension is supported for auto-ingest
 */
function isSupportedFile(filePath: string): boolean {
  const ext = filePath.split(".").pop()?.toLowerCase() ?? ""
  return SUPPORTED_EXTENSIONS.has(ext)
}

/**
 * Start polling the .raw/ directory for new files.
 * When new files are detected (not in ingest cache), triggers batch auto-ingest.
 */
export function startRawWatcher() {
  if (intervalId) return

  intervalId = setInterval(async () => {
    try {
      const store = useWikiStore.getState()
      const project = store.project
      const llmConfig = store.llmConfig

      if (!project) return
      
      const hasLlm =
        !!llmConfig.apiKey ||
        llmConfig.provider === "ollama" ||
        llmConfig.provider === "custom"
      
      if (!hasLlm) return

      const projectPath = normalizePath(project.path)
      const rawDir = `${projectPath}/.raw`

      let tree: FileNode[]
      try {
        tree = await listDirectory(rawDir)
      } catch {
        return
      }

      const allFiles: string[] = []
      for (const node of tree) {
        collectFilePaths(node, allFiles)
      }

      const supportedFiles = allFiles.filter(isSupportedFile)

      if (supportedFiles.length === 0) return

      const newFiles: Array<{ sourcePath: string; folderContext: string }> = []
      
      for (const filePath of supportedFiles) {
        try {
          const content = await readFile(filePath)
          const isCached = await checkIngestCache(projectPath, filePath, content)
          
          if (!isCached) {
            const folderContext = extractFolderContext(filePath, rawDir)
            newFiles.push({ sourcePath: filePath, folderContext })
          }
        } catch {
          continue
        }
      }

      if (newFiles.length > 0) {
        console.log(`[Raw Watcher] Detected ${newFiles.length} new file(s) in .raw/`)
        await enqueueBatch(project.id, newFiles)
      }
    } catch (err) {
      console.error("[Raw Watcher] Error during scan:", err)
    }
  }, POLL_INTERVAL)
}

/**
 * Stop the .raw/ directory watcher
 */
export function stopRawWatcher() {
  if (intervalId) {
    clearInterval(intervalId)
    intervalId = null
  }
}
