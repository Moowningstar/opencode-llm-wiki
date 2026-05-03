# OpenCode LLM Wiki - MCP 集成完成

## ✅ 完成的工作

### 1. MCP 服务器迁移
- ✅ 将 `src-web` 的完整 MCP 服务器代码迁移到 `src-mcp/src/`
- ✅ 创建 `src-mcp/package.json` 配置
- ✅ 安装所有依赖（130 个包）

### 2. 配置文件更新
- ✅ 更新 `.mcp.json` 指向 `src-mcp/src/server.js`
- ✅ 更新 `opencode.jsonc` 添加 llm-wiki MCP 服务器配置
- ✅ 保留现有的 context7 远程 MCP 配置

### 3. 测试验证
- ✅ MCP 服务器成功启动（输出：`LLM Wiki MCP Server running`）
- ✅ 所有文件已暂存到 Git

## 📁 项目结构

```
opencode-llm-wiki/
├── src-mcp/                    # MCP 服务器（新增）
│   ├── src/
│   │   ├── server.js           # MCP 协议处理器
│   │   ├── cli.js              # CLI 工具
│   │   └── lib/                # 核心模块
│   │       ├── wiki-bridge.js      # Wiki 文件系统桥接
│   │       ├── database.js         # SQLite 数据库
│   │       ├── vector-cache.js     # 向量缓存
│   │       ├── context-manager.js  # 智能上下文注入
│   │       ├── semantic-search.js  # 语义搜索
│   │       ├── graph-analyzer.js   # 知识图谱分析
│   │       ├── indexer.js          # 全文索引
│   │       ├── keyword-detector.js # 关键词检测
│   │       └── file-watcher.js     # 文件监控
│   ├── package.json
│   └── README.md
├── opencode.jsonc              # OpenCode 配置（已更新）
└── .mcp.json                   # MCP 配置（已更新）
```

## 🛠️ MCP 工具列表

服务器暴露 **10 个工具**供 OpenCode 调用：

### 核心工具
1. **wiki_read** - 读取 Wiki 页面
2. **wiki_list** - 列出所有页面
3. **wiki_search** - 关键词搜索
4. **wiki_query_with_context** - 智能上下文注入（关键词 + 向量搜索）

### 图谱工具
5. **wiki_get_graph** - 获取知识图谱数据
6. **wiki_graph_insights** - 图谱结构分析（孤立页面、意外连接、桥接节点）
7. **wiki_deep_research** - 深度研究（图遍历 + 语义搜索 + 多跳推理）

### 元数据工具
8. **wiki_get_index** - 获取 index.md（内容目录）
9. **wiki_get_overview** - 获取 overview.md（全局摘要）
10. **wiki_get_purpose** - 获取 purpose.md（Wiki 目标和范围）

## 🚀 使用方式

### 在 OpenCode 中使用

1. **打开项目** - OpenCode 会自动读取 `opencode.jsonc`
2. **MCP 服务器启动** - 自动执行 `node src-mcp/src/server.js`
3. **工具可用** - 所有 10 个 MCP 工具立即可用

### 示例对话

```
你: "我的知识库里有关于 transformer 架构的内容吗？"

OpenCode 内部调用:
1. wiki_query_with_context(query="transformer architecture")
2. 接收相关 Wiki 页面和上下文
3. 基于知识库内容回答你的问题
```

## 📝 配置详情

### opencode.jsonc

```jsonc
{
  "$schema": "https://opencode.ai/config.json",
  
  "mcp": {
    // Context7 - 远程文档搜索
    "context7": {
      "type": "remote",
      "enabled": true,
      "url": "https://mcp.context7.com/mcp"
    },
    
    // LLM Wiki - 本地知识库
    "servers": {
      "llm-wiki": {
        "command": "node",
        "args": ["src-mcp/src/server.js"],
        "env": {
          "LLM_WIKI_PROJECT": "${workspaceFolder}"
        }
      }
    }
  }
}
```

### .mcp.json

```json
{
  "mcpServers": {
    "llm-wiki": {
      "command": "node",
      "args": ["src-mcp/src/server.js"],
      "env": {
        "LLM_WIKI_PROJECT": "${workspaceFolder}",
        "LLM_WIKI_API_URL": "http://127.0.0.1:19828"
      }
    }
  }
}
```

## 🔧 技术栈

- **协议**: Model Context Protocol (MCP) 1.0
- **传输**: stdio（标准输入/输出）
- **SDK**: @modelcontextprotocol/sdk ^1.0.4
- **数据库**: SQLite (better-sqlite3)
- **文件监控**: chokidar
- **CLI**: commander

## 📚 相关文档

- [MCP 服务器 README](src-mcp/README.md) - 详细使用指南
- [MCP 服务器架构](docs/guides/mcp-server.md) - 架构文档
- [项目主 README](README.md) - 项目总览

## ✨ 下一步

1. **重启 OpenCode** - 让配置生效
2. **测试工具** - 尝试调用 MCP 工具
3. **添加文档** - 将文件放入 `.raw/` 目录开始构建知识库

## 🎉 集成完成！

OpenCode LLM Wiki 现在已完全集成到 OpenCode 中，可以通过 MCP 协议访问你的个人知识库。

---

**完成时间**: 2026-05-03  
**集成方式**: MCP (Model Context Protocol)  
**服务器位置**: `src-mcp/src/server.js`
