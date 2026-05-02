# LLM 配置文件使用指南

## 概述

从 v0.0.2 开始，OpenCode LLM Wiki 支持通过 JSON/JSONC 配置文件管理 LLM 提供商配置。这使得配置管理更加灵活，支持：

- 版本控制（Git）
- 团队共享配置
- 批量配置多个提供商
- 支持注释（JSONC 格式）

## 配置文件位置

```
your-project/
└── .llm-wiki/
    └── llm-config.json
```

## 配置文件格式

### 基本结构

```jsonc
{
  // 当前激活的提供商 ID
  "activePreset": "deepseek",
  
  // 各提供商的配置
  "providers": {
    "deepseek": {
      "apiKey": "sk-xxx",
      "model": "deepseek-chat",
      "baseUrl": "https://api.deepseek.com/v1",
      "maxContextSize": 64000
    }
  }
}
```

### 完整示例

```jsonc
{
  "activePreset": "deepseek",
  
  "providers": {
    // DeepSeek 配置
    "deepseek": {
      "apiKey": "sk-your-deepseek-api-key",
      "model": "deepseek-chat",
      "baseUrl": "https://api.deepseek.com/v1",
      "maxContextSize": 64000
    },
    
    // Ollama 本地配置
    "ollama-local": {
      "baseUrl": "http://localhost:11434",
      "model": "qwen3:32b",
      "maxContextSize": 32768
    },
    
    // 自定义 OpenAI 兼容端点
    "custom": {
      "apiKey": "your-api-key",
      "model": "gpt-4o",
      "baseUrl": "https://your-proxy.com/v1",
      "apiMode": "chat_completions",
      "maxContextSize": 128000
    },
    
    // 自定义 Anthropic 兼容端点
    "custom-anthropic": {
      "apiKey": "your-api-key",
      "model": "claude-sonnet-4-6",
      "baseUrl": "https://your-proxy.com/v1/messages",
      "apiMode": "anthropic_messages",
      "maxContextSize": 200000
    },
    
    // Groq 配置
    "groq": {
      "apiKey": "gsk_xxx",
      "model": "llama-3.3-70b-versatile",
      "baseUrl": "https://api.groq.com/openai/v1",
      "maxContextSize": 128000
    },
    
    // xAI Grok 配置
    "xai": {
      "apiKey": "xai-xxx",
      "model": "grok-3",
      "baseUrl": "https://api.x.ai/v1",
      "maxContextSize": 131072
    }
  }
}
```

## 配置字段说明

### 顶层字段

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `activePreset` | string | 否 | 当前激活的提供商 ID |
| `providers` | object | 否 | 各提供商的配置对象 |

### 提供商配置字段

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `apiKey` | string | 否 | API 密钥（Ollama 本地不需要） |
| `model` | string | 否 | 模型名称 |
| `baseUrl` | string | 否 | API 端点 URL |
| `apiMode` | string | 否 | API 协议：`chat_completions` 或 `anthropic_messages` |
| `maxContextSize` | number | 否 | 最大上下文窗口大小（字符数） |

## 可用的提供商 ID

### 官方提供商
- `anthropic` - Anthropic Claude
- `openai` - OpenAI GPT
- `google` - Google Gemini
- `claude-code-cli` - Claude Code CLI（本地）

### 第三方提供商
- `deepseek` - DeepSeek
- `groq` - Groq
- `xai` - xAI Grok
- `nvidia-nim` - NVIDIA NIM
- `kimi` - Kimi (Moonshot)
- `kimi-cn` - Kimi (Moonshot 中国)
- `zhipu` - 智谱 GLM
- `minimax-global` - MiniMax (Global)
- `minimax-cn` - MiniMax (中国)
- `bailian-coding` - 阿里百炼 Coding Plan
- `xiaomi-mimo` - 小米 MiMo
- `volcengine-ark` - 火山引擎 Ark
- `ollama-local` - Ollama (本地)
- `ollama-cloud` - Ollama Cloud
- `custom` - 自定义端点

## 使用方法

### 方法 1：通过 UI 生成模板

1. 打开 **Settings（设置）**
2. 找到 **LLM Provider** 部分
3. 点击 **Generate Template** 按钮
4. 编辑生成的 `.llm-wiki/llm-config.json` 文件

### 方法 2：导出当前配置

1. 在 UI 中配置好你的提供商
2. 点击 **Export Current** 按钮
3. 当前配置会保存到 `.llm-wiki/llm-config.json`

### 方法 3：手动创建

直接在项目目录创建 `.llm-wiki/llm-config.json` 文件。

## 配置优先级

配置文件的优先级 **高于** UI 配置：

1. **文件配置**（最高优先级）
2. UI 配置

当文件配置存在时，会自动合并到 UI 配置中，文件中的值会覆盖 UI 中的值。

## 重新加载配置

修改配置文件后，有两种方式重新加载：

### 方法 1：通过 UI 重新加载

1. 打开 **Settings**
2. 点击 **Reload from File** 按钮

### 方法 2：重新打开项目

关闭并重新打开项目，配置会自动加载。

## 注释支持

配置文件支持 JSONC 格式（带注释的 JSON）：

```jsonc
{
  // 这是单行注释
  "activePreset": "deepseek",
  
  /*
   * 这是多行注释
   * 可以写多行
   */
  "providers": {
    "deepseek": {
      "apiKey": "sk-xxx"  // 行尾注释也支持
    }
  }
}
```

## 常见场景

### 场景 1：使用代理访问 OpenAI

```jsonc
{
  "activePreset": "custom",
  "providers": {
    "custom": {
      "apiKey": "sk-your-openai-key",
      "model": "gpt-4o",
      "baseUrl": "https://your-proxy.com/v1",
      "apiMode": "chat_completions",
      "maxContextSize": 128000
    }
  }
}
```

### 场景 2：本地 Ollama + 远程 DeepSeek

```jsonc
{
  "activePreset": "ollama-local",
  "providers": {
    "ollama-local": {
      "baseUrl": "http://localhost:11434",
      "model": "qwen3:32b",
      "maxContextSize": 32768
    },
    "deepseek": {
      "apiKey": "sk-xxx",
      "model": "deepseek-chat",
      "baseUrl": "https://api.deepseek.com/v1",
      "maxContextSize": 64000
    }
  }
}
```

### 场景 3：团队共享配置（不包含密钥）

```jsonc
{
  "activePreset": "deepseek",
  "providers": {
    "deepseek": {
      // API Key 留空，每个人在 UI 中单独配置
      "model": "deepseek-chat",
      "baseUrl": "https://api.deepseek.com/v1",
      "maxContextSize": 64000
    }
  }
}
```

然后在 `.gitignore` 中添加：
```
.llm-wiki/llm-config.local.json
```

## 安全建议

⚠️ **不要将包含 API Key 的配置文件提交到公共仓库！**

### 推荐做法

1. **使用环境变量**（未来版本支持）
2. **分离敏感配置**：
   - `llm-config.json` - 提交到 Git（不含密钥）
   - `llm-config.local.json` - 本地使用（含密钥，加入 .gitignore）
3. **团队共享**：只共享端点和模型配置，API Key 由每个人单独在 UI 中配置

## 故障排查

### 配置文件不生效

1. 检查文件路径是否正确：`.llm-wiki/llm-config.json`
2. 检查 JSON 格式是否有效（可以用 JSON 验证工具）
3. 点击 **Reload from File** 按钮手动重新加载
4. 查看浏览器控制台是否有错误信息

### JSON 解析错误

- 确保所有字符串用双引号 `"` 包裹
- 确保对象和数组的最后一项后面没有多余的逗号
- 如果使用注释，确保注释格式正确

### 配置被 UI 覆盖

配置文件的优先级高于 UI，但如果你在 UI 中修改并保存，会更新内存中的配置。要恢复文件配置，点击 **Reload from File**。

## 更新日志

- **v0.0.2** - 首次支持配置文件功能
