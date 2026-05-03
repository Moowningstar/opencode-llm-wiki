# OpenCode LLM Wiki

<p align="center">
  <img src="logo.jpg" width="128" height="128" style="border-radius: 22%;" alt="OpenCode LLM Wiki Logo">
</p>

<p align="center">
  <strong>多接口知识引擎后端</strong><br>
  HTTP API • CLI 工具 • MCP 协议 • Wiki 文件系统 • 向量搜索 • 知识图谱
</p>

<p align="center">
  <a href="#这是什么">这是什么?</a> •
  <a href="#架构设计">架构设计</a> •
  <a href="#功能特性">功能特性</a> •
  <a href="#安装">安装</a> •
  <a href="#快速开始">快速开始</a> •
  <a href="#开源协议">开源协议</a>
</p>

<p align="center">
  <a href="README.md">English</a> | 中文
</p>

---

## 这是什么？

**OpenCode LLM Wiki** 是一个**知识引擎后端**，提供持久化、可查询的知识存储，支持多种访问接口。它维护一个结构化的 wiki，可以通过 HTTP API、CLI 工具或 AI 代理（通过 Model Context Protocol）访问。

### 核心理念

**持久化知识引擎，服务于 AI 代理和开发者**

这是一个知识引擎后端，为存储、索引和检索结构化知识提供多种访问方法。与会话结束后就遗忘一切的临时 RAG 系统不同，本项目提供：

1. **持久化 Wiki 存储**：`.wiki/pages/` 中的 Markdown 文件，元数据驱动的索引
2. **多种接口**：HTTP API、CLI 工具和 MCP 协议，适用于不同场景
3. **知识图谱**：自动提取链接和关系映射
4. **向量搜索**：基于 LanceDB 的语义搜索，查找相关内容
5. **跨会话记忆**：知识在会话、对话和工具之间持久化

### 使用场景

- **AI 代理记忆**：跨对话持久化的上下文（通过 MCP）
- **代码库文档**：可通过 API 或 CLI 查询的活文档
- **项目知识库**：存储决策、模式和部落知识
- **研究笔记**：通过语义搜索组织论文、文章和发现
- **个人 Wiki**：构建可通过多种接口访问的第二大脑

---

## 架构设计

### 三层知识引擎

这是一个**知识引擎后端**，具有多种接口，而不仅仅是一个 MCP 工具。

```
┌─────────────────────────────────────────────────────────────┐
│ 第 1 层：接口层（多种访问点）                                 │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐     │
│  │  HTTP API    │  │     CLI      │  │  MCP 服务器  │     │
│  │  (Axum)      │  │   (clap)     │  │  (Node.js)   │     │
│  │  端口 19828   │  │              │  │  stdio       │     │
│  └──────────────┘  └──────────────┘  └──────────────┘     │
│  • 请求验证和路由                                           │
│  • 响应格式化                                               │
│  • 无业务逻辑                                               │
└─────────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────────┐
│ 第 2 层：索引与检索（Rust 后端）                             │
│  ┌──────────────────────────────────────────────────────┐   │
│  │ Wiki 文件系统                                        │   │
│  │  • WikiFileSystem: .wiki/pages/ 管理                │   │
│  │  • IndexManager: index.json 元数据                  │   │
│  │  • GraphManager: graph.json 关系                    │   │
│  └──────────────────────────────────────────────────────┘   │
│  ┌──────────────────────────────────────────────────────┐   │
│  │ 搜索与检索                                           │   │
│  │  • 关键词搜索（分词）                                 │   │
│  │  • 语义搜索（向量嵌入）                               │   │
│  │  • 图遍历（链接提取）                                 │   │
│  └──────────────────────────────────────────────────────┘   │
│  ┌──────────────────────────────────────────────────────┐   │
│  │ 文档处理                                             │   │
│  │  • 导入流程（解析 → 分块 → 嵌入 → 存储）             │   │
│  │  • Markdown 分块（标题感知）                         │   │
│  │  • 元数据提取                                        │   │
│  └──────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────────┐
│ 第 3 层：存储层（可插拔后端）                                │
│  ┌──────────────────┐              ┌──────────────────┐    │
│  │  文件系统        │              │  向量数据库      │    │
│  │  .wiki/pages/    │              │  LanceDB         │    │
│  │  index.json      │              │  (→ RuVector)    │    │
│  │  graph.json      │              │                  │    │
│  └──────────────────┘              └──────────────────┘    │
│  • VectorStorage trait 抽象                                │
│  • 轻松切换后端（LanceDB → RuVector）                       │
└─────────────────────────────────────────────────────────────┘
```

**核心设计原则：**

- **多种接口**：HTTP API 用于编程访问，CLI 用于自动化，MCP 用于 AI 代理
- **元数据驱动**：index.json 管理页面元数据，graph.json 存储关系
- **可插拔存储**：VectorStorage trait 支持轻松迁移（LanceDB → RuVector）
- **清晰分层**：接口层无业务逻辑，存储层无检索逻辑

### MCP 工具（10 个可用）

| 工具 | 用途 |
|------|------|
| `wiki_read` | 按路径读取单个 wiki 页面 |
| `wiki_list` | 列出所有 wiki 页面及元数据 |
| `wiki_search` | 跨 wiki 内容的关键词搜索 |
| `wiki_query_with_context` | 智能上下文注入（关键词 + 向量）|
| `wiki_get_graph` | 获取知识图谱（节点和边）|
| `wiki_graph_insights` | 分析图结构（孤立页面、桥接节点）|
| `wiki_deep_research` | 多跳推理与图遍历 |
| `wiki_get_index` | 获取内容目录（index.md）|
| `wiki_get_overview` | 获取全局摘要（overview.md）|
| `wiki_get_purpose` | 获取 wiki 目标和范围（purpose.md）|
| `wiki_ingest` | 将文档导入知识库 |

### API 端点（Rust 后端）

| 端点 | 方法 | 用途 |
|------|------|------|
| `/health` | GET | 健康检查 |
| `/api/pages` | POST | 列出所有页面 |
| `/api/pages/read` | POST | 读取页面内容 |
| `/api/search/keyword` | POST | 关键词搜索 |
| `/api/search/semantic` | POST | 向量语义搜索 |
| `/api/graph` | POST | 获取知识图谱 |
| `/api/graph/insights` | POST | 图分析 |
| `/api/research` | POST | 深度研究 |
| `/api/meta/index` | POST | 获取索引 |
| `/api/meta/overview` | POST | 获取概览 |
| `/api/meta/purpose` | POST | 获取目标 |
| `/api/ingest` | POST | 导入文档 |

---

## 功能特性

### 后端（生产就绪的 Rust）

- ✅ **三层架构** — 清晰分层：接口 → 服务 → 存储
- ✅ **VectorStorage Trait** — 存储后端抽象（当前 LanceDB，未来 RuVector）
- ✅ **Token 缓存层** — tiktoken-rs 预计算，减少 70% token，100% 缓存命中
- ✅ **Markdown 感知分块** — 标题路径保留，可配置重叠，智能合并
- ✅ **多提供商 LLM** — OpenAI、Anthropic、Google、Ollama、自定义端点
- ✅ **HTTP API + CLI** — Axum 服务器（端口 19828）+ 独立 CLI 工具
- ✅ **异步优先设计** — tokio 运行时，非阻塞 I/O，并发任务处理

### 前端（桌面客户端 - 待分离）

- **两步思维链导入** — LLM 先分析，再生成带源追溯的 wiki 页面
- **4 信号知识图谱** — 直接链接、源重叠、Adamic-Adar、类型亲和力
- **Louvain 社区检测** — 自动知识聚类发现，带内聚评分
- **图洞察** — 意外连接和知识缺口，一键深度研究
- **向量语义搜索** — 可选的基于嵌入的检索，支持任何 OpenAI 兼容端点
- **持久化导入队列** — 串行处理，崩溃恢复，取消、重试、进度可视化
- **深度研究** — LLM 优化的搜索主题，多查询网络搜索，自动导入结果
- **异步审查系统** — LLM 标记需要人工判断的项目，预定义操作，预生成查询
- **Chrome 网页剪藏器** — 一键网页捕获，自动导入

---

## 致谢与灵感

**基础方法论**：[Andrej Karpathy 的 LLM Wiki 模式](https://gist.github.com/karpathy/442a6bf555914893e9891c11519de94f) — 核心三层架构（原始源 → Wiki → Schema）和增量编译理念。

**未来方向**：[RuVector SONA](https://github.com/ruvector/ruvector) — 自组织神经架构，用于自适应知识图谱，具有增量学习和防止灾难性遗忘能力。

**我们构建的**：一个生产级 Rust 后端，实现 Karpathy 的模式，采用图原生设计，为 RuVector 的 SONA 能力做准备，同时保持清晰的抽象层以实现存储后端灵活性。

---

## 技术栈

| 层级 | 技术 | 用途 |
|------|------|------|
| **存储抽象** | VectorStorage trait | 后端无关接口 |
| **当前存储** | LanceDB 0.4 | 嵌入式向量数据库（生产）|
| **未来存储** | RuVector + SONA | 图原生，自适应学习 |
| **Token 缓存** | tiktoken-rs | 预计算 token ID（节省 70%）|
| **分块** | 自定义 Rust | Markdown 标题感知分割 |
| **嵌入** | OpenAI 兼容 API | 任何 /v1/embeddings 端点 |
| **HTTP 服务器** | Axum + tokio | 异步 Rust Web 框架 |
| **CLI** | clap | 命令行接口 |
| **桌面** | Tauri v2（遗留）| 待分离为独立客户端 |
| **前端** | React 19 + TypeScript | UI 层（待解耦）|

---

## 安装

### 预构建二进制文件

从 [Releases](https://github.com/yourusername/opencode-llm-wiki/releases) 下载：
- **macOS**：`.dmg`（Apple Silicon + Intel）
- **Windows**：`.msi`
- **Linux**：`.deb` / `.AppImage`

### 从源码构建

```bash
# 前置要求：Node.js 20+, Rust 1.70+
git clone https://github.com/yourusername/opencode-llm-wiki.git
cd opencode-llm-wiki

# 安装依赖
npm install

# 构建 Rust 后端
cargo build --release

# 运行 API 服务器
cargo run --bin llm-wiki-server -- serve --port 19828

# 运行 CLI 工具
cargo run --bin llm-wiki -- --help
# 可用命令：
#   serve    启动 API 服务器
#   init     初始化新 wiki 项目
#   ingest   导入文档
#   query    查询知识库

# 开发模式（前端）
npm run dev
```

### Docker（即将推出）

```bash
docker run -p 19828:19828 -v ./data:/data opencode-llm-wiki
```

### Chrome 扩展

1. 打开 `chrome://extensions`
2. 启用"开发者模式"
3. 点击"加载已解压的扩展程序"
4. 选择 `extension/` 目录

## 快速开始

1. 启动应用 → 创建新项目（选择模板）
2. 进入**设置** → 配置 LLM 提供商（API 密钥 + 模型）
3. 进入**源文件** → 导入文档（PDF、DOCX、MD 等）
4. 查看**活动面板** — LLM 自动构建 wiki 页面
5. 使用**聊天**查询知识库
6. 浏览**知识图谱**查看连接
7. 检查**审查**中需要注意的项目
8. 定期运行 **Lint** 维护 wiki 健康

## 项目结构

### Wiki 项目结构
```
my-wiki/
├── purpose.md
├── schema.md
├── raw/
├── .raw/
├── .wiki/
│   ├── index.md            # 内容目录
│   ├── log.md              # 操作历史
│   ├── overview.md         # 全局摘要（自动更新）
│   ├── entities/           # 人物、组织、产品
│   ├── concepts/           # 理论、方法、技术
│   ├── sources/            # 源摘要
│   ├── queries/            # 保存的聊天答案 + 研究
│   ├── synthesis/          # 跨源分析
│   └── comparisons/        # 并排比较
├── .obsidian/              # Obsidian vault 配置（自动生成）
└── .llm-wiki/              # 应用配置、聊天历史、审查项目
```

### 代码库结构
```
opencode-llm-wiki/
├── src/                    # Rust 后端（三层架构）
│   ├── api/                # 第 1 层：HTTP API + 处理器
│   │   ├── state.rs        # AppState（依赖注入）
│   │   ├── handlers.rs     # 请求处理器
│   │   ├── routes.rs       # 路由定义
│   │   └── server.rs       # Axum 服务器
│   ├── services/           # 第 2 层：业务逻辑
│   │   ├── embedding.rs    # OpenAI 兼容嵌入 API
│   │   ├── chunking.rs     # Markdown 标题感知分割
│   │   ├── ingest.rs       # 编排（解析→分块→嵌入→存储）
│   │   ├── query.rs        # 搜索 + 上下文优化
│   │   ├── token_cache.rs  # tiktoken-rs 预计算
│   │   └── llm_client.rs   # 多提供商流式传输
│   ├── storage/            # 第 3 层：数据抽象
│   │   ├── traits.rs       # VectorStorage trait
│   │   └── lancedb_impl.rs # LanceDB 实现
│   ├── types/              # 共享类型
│   ├── utils/              # 工具
│   ├── main.rs             # API 服务器二进制（端口 19828）
│   └── cli.rs              # CLI 工具二进制
├── src-mcp/                # MCP 服务器（Node.js）
├── src-desktop/            # 桌面客户端（待分离）
│   ├── ui-new/             # React 前端
│   └── src-tauri-new/      # Tauri 包装器
├── src-legacy/             # 归档的 TypeScript 实现
├── extension/              # Chrome 扩展
└── docs/                   # 架构文档
    └── architecture/
        ├── 3-layer-refactoring-plan.md
        └── ruvector-migration-roadmap.md
```

**架构**：清晰的三层设计，基于 trait 的存储抽象。未来的后端迁移（例如 LanceDB → RuVector）只需实现 `VectorStorage` trait。

## Star History

<a href="https://www.star-history.com/?repos=yourusername%2Fopencode-llm-wiki&type=date&legend=top-left">
 <picture>
   <source media="(prefers-color-scheme: dark)" srcset="https://api.star-history.com/chart?repos=yourusername/opencode-llm-wiki&type=date&theme=dark&legend=top-left" />
   <source media="(prefers-color-scheme: light)" srcset="https://api.star-history.com/chart?repos=yourusername/opencode-llm-wiki&type=date&legend=top-left" />
   <img alt="Star History Chart" src="https://api.star-history.com/chart?repos=yourusername/opencode-llm-wiki&type=date&legend=top-left" />
 </picture>
</a>

## 开源协议

本项目采用 **GNU General Public License v3.0** 协议 — 详见 [LICENSE](LICENSE)。
