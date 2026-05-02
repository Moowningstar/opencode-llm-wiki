import { invoke } from "@tauri-apps/api/core"
import { createTemplateConfig, saveLlmConfigFile, loadLlmConfigFile, exportToConfigFile } from "@/lib/llm-config-file"
import { useWikiStore } from "@/stores/wiki-store"

export async function generateConfigTemplate(projectPath: string): Promise<void> {
  const template = createTemplateConfig()
  const configPath = `${projectPath}/.llm-wiki/llm-config.json`
  
  await invoke("write_file", {
    path: configPath,
    contents: template,
  })
}

export async function exportCurrentConfig(projectPath: string): Promise<void> {
  const activePreset = useWikiStore.getState().activePresetId
  const providerConfigs = useWikiStore.getState().providerConfigs
  
  const config = exportToConfigFile(activePreset, providerConfigs)
  await saveLlmConfigFile(projectPath, config)
}

export async function reloadConfigFromFile(projectPath: string): Promise<boolean> {
  const fileConfig = await loadLlmConfigFile(projectPath)
  if (!fileConfig) {
    return false
  }
  
  const { mergeLlmConfigs } = await import("@/lib/llm-config-file")
  const currentActivePreset = useWikiStore.getState().activePresetId
  const currentProviderConfigs = useWikiStore.getState().providerConfigs
  const merged = mergeLlmConfigs(fileConfig, currentActivePreset, currentProviderConfigs)
  
  useWikiStore.getState().setProviderConfigs(merged.providerConfigs)
  if (merged.activePreset) {
    useWikiStore.getState().setActivePresetId(merged.activePreset)
    
    const { LLM_PRESETS } = await import("@/components/settings/llm-presets")
    const { resolveConfig } = await import("@/components/settings/preset-resolver")
    const preset = LLM_PRESETS.find((p) => p.id === merged.activePreset)
    if (preset) {
      const currentFallback = useWikiStore.getState().llmConfig
      const override = merged.providerConfigs[merged.activePreset]
      const resolved = resolveConfig(preset, override, currentFallback)
      useWikiStore.getState().setLlmConfig(resolved)
    }
  }
  
  return true
}
