# OpenCode LLM Wiki

<p align="center">
  <img src="logo.jpg" width="128" height="128" style="border-radius: 22%;" alt="OpenCode LLM Wiki Logo">
</p>

<p align="center">
  <strong>知识引擎后端服务</strong><br>
  HTTP API • CLI 工具 • MCP 协议 • 向量搜索 • 知识图谱
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
3. **知识图谱**：自动提取链接和关系映射，支持 PageRank、Louvain 社区检测
4. **向量搜索**：基于 RuVector 的语义搜索，查找相关内容
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
│  │  • 图算法（PageRank、Louvain、中心性分析）           │   │
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
│ 第 3 层：存储层（RuVector）                                  │
│  ┌──────────────────┐              ┌──────────────────┐    │
│  │  文件系统        │              │  向量数据库      │    │
│  │  .wiki/pages/    │              │  RuVector        │    │
│  │  index.json      │              │  VectorDB +      │    │
│  │  graph.json      │              │  GraphDB         │    │
│  └──────────────────┘              └──────────────────┘    │
│  • VectorStorage trait 抽象                                │
│  • 向量搜索 + 图数据库统一存储                              │
└─────────────────────────────────────────────────────────────┘
```

**核心设计原则：**

- **多种接口**：HTTP API 用于编程访问，CLI 用于自动化，MCP 用于 AI 代理
- **元数据驱动**：index.json 管理页面元数据，graph.json 存储关系
- **RuVector 存储**：统一的向量数据库 + 图数据库后端
- **清晰分层**：接口层无业务逻辑，存储层无检索逻辑

### MCP 工具（11 个可用）

| 工具 | 用途 |
|------|------|
| `wiki_read` | 按路径读取单个 wiki 页面 |
| `wiki_list` | 列出所有 wiki 页面及元数据 |
| `wiki_search` | 跨 wiki 内容的关键词搜索 |
| `wiki_query_with_context` | 智能上下文注入（关键词 + 向量）|
| `wiki_get_graph` | 获取知识图谱（节点和边）|
| `wiki_graph_insights` | 分析图结构（孤立页面、桥接节点、统计信息）|
| `wiki_deep_research` | 多跳推理与图遍历 |
| `wiki_get_index` | 获取内容目录（index.md）|
| `wiki_get_overview` | 获取全局摘要（overview.md）|
| `wiki_get_purpose` | 获取 wiki 目标和范围（purpose.md）|
| `wiki_ingest` | 将文档导入知识库 |

### API 端点（16 个）

| 端点 | 方法 | 用途 |
|------|------|------|
| `/health` | GET | 健康检查 |
| `/api/pages` | POST | 列出所有页面 |
| `/api/pages/read` | POST | 读取页面内容 |
| `/api/search/keyword` | POST | 关键词搜索 |
| `/api/search/semantic` | POST | 向量语义搜索 |
| `/api/graph` | POST | 获取知识图谱 |
| `/api/graph/insights` | POST | 图洞察分析（孤立页面、桥接节点、统计）|
| `/api/research` | POST | 深度研究（语义搜索 + BFS 遍历）|
| `/api/meta/index` | POST | 获取索引 |
| `/api/meta/overview` | POST | 获取概览 |
| `/api/meta/purpose` | POST | 获取目标 |
| `/api/ingest` | POST | 导入文档 |

---

## 功能特性

### v1.0.1 核心功能

- ✅ **HTTP API 服务器** — 16 个端点，Axum + tokio 异步架构
- ✅ **MCP 服务器** — 11 个工具，支持 Claude Desktop 等 AI 代理
- ✅ **CLI 工具** — 命令行接口，支持 serve、ingest、query 等命令
- ✅ **RuVector 存储** — 统一的向量数据库 + 图数据库后端
- ✅ **知识图谱算法** — PageRank、Louvain 社区检测、中心性分析、最短路径
- ✅ **向量语义搜索** — 基于嵌入的相似度搜索
- ✅ **图洞察分析** — 孤立页面检测、桥接节点识别、图统计
- ✅ **深度研究** — 语义搜索 + BFS 图遍历，多跳推理
- ✅ **文档导入** — 支持 Markdown、PDF、DOCX 等格式
- ✅ **元数据管理** — index.json、graph.json、purpose.md

### 性能指标

- **查询延迟**：< 200ms（向量搜索）
- **索引速度**：< 5s（1000 个文档）
- **图算法**：< 100ms（PageRank、Louvain）
- **测试覆盖**：61/61 单元测试通过

---

## 致谢与灵感

**基础方法论**：[Andrej Karpathy 的 LLM Wiki 模式](https://gist.github.com/karpathy/442a6bf555914893e9891c11519de94f) — 核心三层架构（原始源 → Wiki → Schema）和增量编译理念。

**存储引擎**：[RuVector](https://github.com/ruvector/ruvector) — 高性能向量数据库 + 图数据库，支持 SONA（自组织神经架构）。

**我们构建的**：一个生产级 Rust 后端，实现 Karpathy 的模式，采用 RuVector 作为统一存储引擎，提供 HTTP API、CLI 和 MCP 多种接口。

---

## 技术栈

| 层级 | 技术 | 用途 |
|------|------|------|
| **存储抽象** | VectorStorage trait | 后端无关接口 |
| **存储引擎** | RuVector 2.2.0 | 向量数据库 + 图数据库 |
| **图算法** | 自定义 Rust | PageRank、Louvain、中心性分析 |
| **HTTP 服务器** | Axum + tokio | 异步 Rust Web 框架 |
| **CLI** | clap | 命令行接口 |
| **MCP 服务器** | Node.js + TypeScript | Model Context Protocol |
| **嵌入** | OpenAI 兼容 API | 任何 /v1/embeddings 端点 |

---

## 安装

### 从源码构建

```bash
# 前置要求：Rust 1.70+, Node.js 20+
git clone https://github.com/yourusername/opencode-llm-wiki.git
cd opencode-llm-wiki

# 构建 Rust 后端
cargo build --release

# 运行 API 服务器
cargo run --bin llm-wiki-server -- --port 19828

# 运行 CLI 工具
cargo run --bin llm-wiki -- --help
# 可用命令：
#   serve    启动 API 服务器
#   ingest   导入文档
#   query    查询知识库
#   list     列出所有页面
#   stats    显示统计信息

# 安装 MCP 服务器
cd src-mcp
npm install
npm start
```

### Docker（即将推出）

```bash
docker run -p 19828:19828 -v ./data:/data opencode-llm-wiki
```

---

## 快速开始

### 1. 启动服务器

```bash
cargo run --bin llm-wiki-server -- --port 19828
```

### 2. 导入文档

```bash
# 使用 CLI
cargo run --bin llm-wiki -- ingest document.md

# 或使用 API
curl -X POST http://localhost:19828/api/ingest \
  -H "Content-Type: application/json" \
  -d '{"path": "document.md"}'
```

### 3. 查询知识库

```bash
# 使用 CLI
cargo run --bin llm-wiki -- query "transformer architecture"

# 或使用 API
curl -X POST http://localhost:19828/api/search/semantic \
  -H "Content-Type: application/json" \
  -d '{"query": "transformer architecture", "limit": 5}'
```

### 4. 使用 MCP（Claude Desktop）

在 Claude Desktop 配置文件中添加：

```json
{
  "mcpServers": {
    "llm-wiki": {
      "command": "node",
      "args": ["/path/to/opencode-llm-wiki/src-mcp/src/server.js"]
    }
  }
}
```

---

## 项目结构

### Wiki 项目结构
```
.wiki/
├── pages/              # Wiki 页面（Markdown）
├── _meta/
│   ├── index.json      # 页面元数据索引
│   ├── graph.json      # 知识图谱（节点和边）
│   └── purpose.md      # Wiki 目标和范围
└── .ruvector/          # RuVector 数据库文件
```

### 代码库结构
```
opencode-llm-wiki/
├── src/                    # Rust 后端（三层架构）
│   ├── api/                # 第 1 层：HTTP API + 处理器
│   │   ├── state.rs        # AppState（依赖注入）
│   │   ├── handlers.rs     # 请求处理器
│   │   └── routes.rs       # 路由定义
│   ├── wiki/               # 第 2 层：业务逻辑
│   │   ├── filesystem.rs   # Wiki 文件系统管理
│   │   ├── ingest.rs       # 文档导入流程
│   │   ├── query.rs        # 搜索和检索
│   │   ├── graph_algorithms.rs  # PageRank、Louvain 等
│   │   └── cypher.rs       # Cypher 查询引擎
│   ├── storage/            # 第 3 层：数据抽象
│   │   ├── traits.rs       # VectorStorage trait
│   │   └── ruvector_impl.rs # RuVector 实现
│   ├── types/              # 共享类型
│   ├── main.rs             # API 服务器入口
│   └── cli.rs              # CLI 工具入口
├── src-mcp/                # MCP 服务器（Node.js）
├── benches/                # 性能测试
├── tests/                  # 集成测试
└── docs/                   # 文档
    ├── RELEASE_v1.0.1.md
    ├── V1.0.1_VERIFICATION.md
    └── RUVECTOR_PHASE2_COMPLETE.md
```

---

## 开发

### 运行测试

```bash
# 运行所有测试
cargo test

# 运行特定测试
cargo test --test graph_algorithms

# 运行性能测试
cargo bench
```

### 代码检查

```bash
# 格式化代码
cargo fmt

# 代码检查
cargo clippy

# 构建文档
cargo doc --open
```

---

## 开源协议

本项目采用 **GNU General Public License v3.0** 协议 — 详见 [LICENSE](LICENSE)。

---

## Star History

<a href="https://www.star-history.com/?repos=yourusername%2Fopencode-llm-wiki&type=date&legend=top-left">
 <picture>
   <source media="(prefers-color-scheme: dark)" srcset="https://api.star-history.com/chart?repos=yourusername/opencode-llm-wiki&type=date&theme=dark&legend=top-left" />
   <source media="(prefers-color-scheme: light)" srcset="https://api.star-history.com/chart?repos=yourusername/opencode-llm-wiki&type=date&legend=top-left" />
   <img alt="Star History Chart" src="https://api.star-history.com/chart?repos=yourusername/opencode-llm-wiki&type=date&legend=top-left" />
 </picture>
</a>
