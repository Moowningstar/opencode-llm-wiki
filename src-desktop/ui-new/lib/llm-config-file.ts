/**
 * LLM configuration file loader and manager.
 * 
 * Supports loading LLM provider configurations from JSON/JSONC files:
 * - Project-level: `{project}/.llm-wiki/llm-config.json`
 * - Supports JSONC (JSON with comments)
 * - Auto-merges with UI settings (file config takes precedence)
 * 
 * Example config file:
 * ```jsonc
 * {
 *   // Active provider preset ID
 *   "activePreset": "deepseek",
 *   
 *   // Provider-specific configurations
 *   "providers": {
 *     "deepseek": {
 *       "apiKey": "sk-xxx",
 *       "model": "deepseek-chat",
 *       "baseUrl": "https://api.deepseek.com/v1",
 *       "maxContextSize": 64000
 *     },
 *     "ollama-local": {
 *       "baseUrl": "http://localhost:11434",
 *       "model": "qwen3:32b",
 *       "maxContextSize": 32768
 *     },
 *     "custom": {
 *       "apiKey": "your-api-key",
 *       "model": "gpt-4o",
 *       "baseUrl": "https://your-proxy.com/v1",
 *       "apiMode": "chat_completions",
 *       "maxContextSize": 128000
 *     }
 *   }
 * }
 * ```
 */

import { readFile, writeFile, fileExists } from "@/commands/fs"
import { normalizePath } from "@/lib/path-utils"
import type { ProviderOverride, ProviderConfigs } from "@/stores/wiki-store"

export interface LlmConfigFile {
  /** Active provider preset ID (e.g., "deepseek", "ollama-local", "custom") */
  activePreset?: string
  /** Provider-specific configurations */
  providers?: Record<string, ProviderOverride>
}

/**
 * Get the path to the LLM config file for a project.
 */
export function getLlmConfigPath(projectPath: string): string {
  return `${normalizePath(projectPath)}/.llm-wiki/llm-config.json`
}

/**
 * Strip JSONC comments from a string to make it valid JSON.
 * Handles:
 * - Single-line comments: // comment
 * - Multi-line comments: /* comment *\/
 */
function stripJsonComments(jsonc: string): string {
  // Remove single-line comments
  let result = jsonc.replace(/\/\/.*$/gm, "")
  
  // Remove multi-line comments
  result = result.replace(/\/\*[\s\S]*?\*\//g, "")
  
  return result
}

/**
 * Load LLM configuration from file.
 * Returns null if file doesn't exist or is invalid.
 */
export async function loadLlmConfigFile(projectPath: string): Promise<LlmConfigFile | null> {
  const configPath = getLlmConfigPath(projectPath)
  
  try {
    const exists = await fileExists(configPath)
    if (!exists) {
      return null
    }
    
    const content = await readFile(configPath)
    const stripped = stripJsonComments(content)
    const config = JSON.parse(stripped) as LlmConfigFile
    
    return config
  } catch (error) {
    console.error(`Failed to load LLM config from ${configPath}:`, error)
    return null
  }
}

/**
 * Save LLM configuration to file.
 */
export async function saveLlmConfigFile(
  projectPath: string,
  config: LlmConfigFile
): Promise<void> {
  const configPath = getLlmConfigPath(projectPath)
  
  try {
    const content = JSON.stringify(config, null, 2)
    await writeFile(configPath, content)
  } catch (error) {
    console.error(`Failed to save LLM config to ${configPath}:`, error)
    throw error
  }
}

/**
 * Merge file-based config with UI-based config.
 * File config takes precedence over UI config.
 */
export function mergeLlmConfigs(
  fileConfig: LlmConfigFile | null,
  uiActivePreset: string | null,
  uiProviderConfigs: ProviderConfigs
): {
  activePreset: string | null
  providerConfigs: ProviderConfigs
} {
  if (!fileConfig) {
    return {
      activePreset: uiActivePreset,
      providerConfigs: uiProviderConfigs,
    }
  }
  
  // File config takes precedence
  const activePreset = fileConfig.activePreset ?? uiActivePreset
  
  // Merge provider configs (file overrides UI)
  const providerConfigs: ProviderConfigs = { ...uiProviderConfigs }
  
  if (fileConfig.providers) {
    for (const [presetId, override] of Object.entries(fileConfig.providers)) {
      providerConfigs[presetId] = {
        ...(providerConfigs[presetId] ?? {}),
        ...override,
      }
    }
  }
  
  return {
    activePreset,
    providerConfigs,
  }
}

/**
 * Export current UI config to file format.
 * Useful for generating a template config file from UI settings.
 */
export function exportToConfigFile(
  activePreset: string | null,
  providerConfigs: ProviderConfigs
): LlmConfigFile {
  return {
    activePreset: activePreset ?? undefined,
    providers: providerConfigs,
  }
}

/**
 * Create a template config file with examples.
 */
export function createTemplateConfig(): string {
  return `{
  // Active provider preset ID
  // Available presets: anthropic, openai, google, deepseek, groq, xai, 
  // kimi, zhipu, minimax-global, ollama-local, custom, etc.
  "activePreset": "deepseek",
  
  // Provider-specific configurations
  "providers": {
    // DeepSeek example
    "deepseek": {
      "apiKey": "sk-your-deepseek-api-key",
      "model": "deepseek-chat",
      "baseUrl": "https://api.deepseek.com/v1",
      "maxContextSize": 64000
    },
    
    // Ollama local example
    "ollama-local": {
      "baseUrl": "http://localhost:11434",
      "model": "qwen3:32b",
      "maxContextSize": 32768
    },
    
    // Custom provider example (OpenAI-compatible)
    "custom": {
      "apiKey": "your-api-key",
      "model": "gpt-4o",
      "baseUrl": "https://your-proxy.com/v1",
      "apiMode": "chat_completions",
      "maxContextSize": 128000
    },
    
    // Anthropic-compatible custom provider
    "custom-anthropic": {
      "apiKey": "your-api-key",
      "model": "claude-sonnet-4-6",
      "baseUrl": "https://your-proxy.com/v1/messages",
      "apiMode": "anthropic_messages",
      "maxContextSize": 200000
    }
  }
}
`
}
