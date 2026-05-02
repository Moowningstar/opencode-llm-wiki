# OpenCode LLM Wiki 项目重构方案

> 目标：解耦架构、清晰职责、支持未来扩展

**生成时间**: 2026-05-02  
**当前问题**: 桌面端与核心逻辑耦合、文档混乱、职责不清

---

## 📊 当前结构问题分析

### 问题 1: 文档混乱（14 个 MD 文件在根目录）

```
根目录/
├── AGENTS.md                    # OpenCode 配置
├── ARCHITECTURE.md              # 架构文档
├── CLAUDE.md                    # Claude 配置
├── DEPLOY_WINDOWS.md            # 部署指南
├── KNOWLEDGE_GRAPH_SUMMARY.md   # 知识图谱总结
├── LLM_CONFIG_GUIDE.md          # LLM 配置指南
├── llm-wiki.md                  # Karpathy 原始文档
├── MCP_SERVER_GUIDE.md          # MCP 服务器指南
├── OPENCODE_SETUP.md            # OpenCode 设置
├── QUICKSTART_API.md            # API 快速开始
├── README.md                    # 主文档
├── README_CN.md                 # 中文文档
├── RUST_API_SERVER.md           # Rust API 文档
└── RUVECTOR_INTEGRATION_PLAN.md # RuVector 集成方案
```

**问题**:
- 文档分散，难以查找
- 职责不清（部署、配置、架构混在一起）
- 缺乏文档索引

### 问题 2: 代码职责不清

**前端 (src/)**:
```
src/
├── lib/                         # 核心业务逻辑（165 个 TS 文件）
│   ├── llm-client.ts           # ❌ 应该在后端
│   ├── ingest.ts               # ❌ 应该在后端
│   ├── embedding.ts            # ❌ 应该在后端
│   ├── graph-relevance.ts      # ❌ 应该在后端
│   └── ...
├── components/                  # ✅ UI 组件（正确）
└── stores/                      # ✅ 状态管理（正确）
```

**后端 (src-tauri/src/)**:
```
src-tauri/src/
├── commands/                    # Tauri 命令（文件系统操作）
│   ├── fs.rs                   # ✅ 文件操作
│   ├── project.rs              # ✅ 项目管理
│   └── vectorstore.rs          # ✅ 向量存储
├── api/                         # ❌ API 服务器（未完成）
│   ├── handlers.rs
│   ├── routes.rs
│   └── server.rs
└── lib.rs                       # Tauri 应用入口
```

**问题**:
- **核心逻辑在前端** (TypeScript)，应该在后端 (Rust)
- **API 服务器未完成**，无法独立运行
- **MCP Server 直接调用前端代码**，耦合严重

### 问题 3: 三个独立模块缺乏统一管理

```
项目根目录/
├── src/                         # 前端（Tauri 桌面应用）
├── src-tauri/                   # 后端（Tauri + Rust）
└── mcp-server/                  # MCP 服务器（Node.js）
```

**问题**:
- 三个模块各自为政
- 共享代码重复
- 难以统一测试和部署

---

## 🎯 重构目标

### 1. 清晰的三层架构

```
┌─────────────────────────────────────────────────────────┐
│  客户端层 (Clients)                                      │
│  ├─ Desktop App (Tauri + React)                         │
│  ├─ MCP Server (Node.js)                                │
│  └─ CLI Tool (Rust)                                     │
└────────────────┬────────────────────────────────────────┘
                 │ HTTP REST API
                 ↓
┌─────────────────────────────────────────────────────────┐
│  核心服务层 (Core Services - Rust)                      │
│  ├─ LLM Client                                          │
│  ├─ Ingest Engine                                       │
│  ├─ Query Optimizer                                     │
│  ├─ Token Cache                                         │
│  └─ Graph Engine                                        │
└────────────────┬────────────────────────────────────────┘
                 │
                 ↓
┌─────────────────────────────────────────────────────────┐
│  存储层 (Storage)                                        │
│  ├─ RuVector (Vector + Graph)                          │
│  ├─ SQLite (Metadata)                                   │
│  └─ File System (Wiki Pages)                           │
└─────────────────────────────────────────────────────────┘
```

### 2. 文档结构化

```
docs/
├── README.md                    # 项目总览 + 快速开始
├── architecture/                # 架构文档
│   ├── overview.md             # 架构概览
│   ├── core-services.md        # 核心服务设计
│   ├── storage-layer.md        # 存储层设计
│   └── api-design.md           # API 设计
├── guides/                      # 使用指南
│   ├── installation.md         # 安装指南
│   ├── configuration.md        # 配置指南
│   ├── llm-providers.md        # LLM 提供商配置
│   └── deployment.md           # 部署指南
├── development/                 # 开发文档
│   ├── setup.md                # 开发环境设置
│   ├── contributing.md         # 贡献指南
│   ├── testing.md              # 测试指南
│   └── api-reference.md        # API 参考
├── integrations/                # 集成文档
│   ├── mcp-server.md           # MCP 服务器
│   ├── chrome-extension.md     # Chrome 扩展
│   └── ruvector.md             # RuVector 集成
└── migration/                   # 迁移指南
    ├── from-v1.md              # 从 v1 迁移
    └── lancedb-to-ruvector.md  # LanceDB → RuVector
```

### 3. 代码模块化

```
opencode-llm-wiki/
├── crates/                      # Rust 工作空间
│   ├── core/                   # 核心库（可独立使用）
│   │   ├── llm-client/         # LLM 客户端
│   │   ├── ingest-engine/      # 导入引擎
│   │   ├── query-optimizer/    # 查询优化器
│   │   ├── token-cache/        # Token 缓存
│   │   └── graph-engine/       # 图引擎
│   ├── api-server/             # HTTP API 服务器
│   ├── desktop-app/            # Tauri 桌面应用
│   └── cli/                    # CLI 工具
├── clients/                     # 客户端
│   ├── mcp-server/             # MCP 服务器
│   ├── web-ui/                 # Web UI（未来）
│   └── chrome-extension/       # Chrome 扩展
├── docs/                        # 文档（结构化）
├── examples/                    # 示例项目
└── scripts/                     # 构建/部署脚本
```

---

## 📁 新项目结构

### 完整目录树

```
opencode-llm-wiki/
├── README.md                    # 项目总览
├── LICENSE
├── Cargo.toml                   # Rust 工作空间配置
├── package.json                 # 前端依赖
│
├── docs/                        # 📚 文档（结构化）
│   ├── README.md
│   ├── architecture/
│   ├── guides/
│   ├── development/
│   ├── integrations/
│   └── migration/
│
├── crates/                      # 🦀 Rust 工作空间
│   │
│   ├── core/                   # 核心库（可独立使用）
│   │   ├── llm-client/         # LLM 客户端
│   │   │   ├── Cargo.toml
│   │   │   ├── src/
│   │   │   │   ├── lib.rs
│   │   │   │   ├── providers/  # OpenAI, Anthropic, etc.
│   │   │   │   ├── streaming.rs
│   │   │   │   └── types.rs
│   │   │   └── tests/
│   │   │
│   │   ├── ingest-engine/      # 导入引擎
│   │   │   ├── Cargo.toml
│   │   │   ├── src/
│   │   │   │   ├── lib.rs
│   │   │   │   ├── parser.rs   # 文档解析
│   │   │   │   ├── analyzer.rs # LLM 分析
│   │   │   │   ├── generator.rs # Wiki 生成
│   │   │   │   └── queue.rs    # 导入队列
│   │   │   └── tests/
│   │   │
│   │   ├── query-optimizer/    # 查询优化器
│   │   │   ├── Cargo.toml
│   │   │   ├── src/
│   │   │   │   ├── lib.rs
│   │   │   │   ├── search.rs   # 向量搜索
│   │   │   │   ├── graph.rs    # 图扩展
│   │   │   │   └── budget.rs   # Token 预算
│   │   │   └── tests/
│   │   │
│   │   ├── token-cache/        # Token 缓存
│   │   │   ├── Cargo.toml
│   │   │   ├── src/
│   │   │   │   ├── lib.rs
│   │   │   │   ├── tokenizer.rs
│   │   │   │   └── cache.rs
│   │   │   └── tests/
│   │   │
│   │   └── graph-engine/       # 图引擎
│   │       ├── Cargo.toml
│   │       ├── src/
│   │       │   ├── lib.rs
│   │       │   ├── relevance.rs
│   │       │   ├── insights.rs
│   │       │   └── community.rs
│   │       └── tests/
│   │
│   ├── storage/                # 存储抽象层
│   │   ├── ruvector-store/     # RuVector 适配器
│   │   ├── sqlite-store/       # SQLite 适配器
│   │   └── file-store/         # 文件系统适配器
│   │
│   ├── api-server/             # HTTP API 服务器
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── main.rs
│   │   │   ├── routes/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── llm.rs
│   │   │   │   ├── ingest.rs
│   │   │   │   ├── query.rs
│   │   │   │   └── token.rs
│   │   │   ├── handlers/
│   │   │   ├── middleware/
│   │   │   └── types.rs
│   │   └── tests/
│   │
│   ├── desktop-app/            # Tauri 桌面应用
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── main.rs
│   │   │   ├── commands/       # Tauri 命令
│   │   │   └── lib.rs
│   │   ├── ui/                 # React 前端
│   │   │   ├── src/
│   │   │   │   ├── components/
│   │   │   │   ├── stores/
│   │   │   │   └── App.tsx
│   │   │   └── package.json
│   │   └── icons/
│   │
│   └── cli/                    # CLI 工具
│       ├── Cargo.toml
│       ├── src/
│       │   ├── main.rs
│       │   └── commands/
│       └── tests/
│
├── clients/                     # 客户端
│   ├── mcp-server/             # MCP 服务器
│   │   ├── package.json
│   │   ├── src/
│   │   │   ├── server.ts
│   │   │   ├── tools/
│   │   │   └── api-client.ts   # 调用 API 服务器
│   │   └── README.md
│   │
│   ├── chrome-extension/       # Chrome 扩展
│   │   ├── manifest.json
│   │   ├── src/
│   │   └── README.md
│   │
│   └── web-ui/                 # Web UI（未来）
│       └── README.md
│
├── examples/                    # 示例项目
│   ├── basic-wiki/
│   ├── research-notes/
│   └── personal-kb/
│
├── scripts/                     # 构建/部署脚本
│   ├── build.sh
│   ├── deploy.sh
│   ├── migrate-lancedb.sh
│   └── setup-dev.sh
│
└── tests/                       # 集成测试
    ├── integration/
    └── e2e/
```

---

## 🔄 迁移步骤

### Phase 1: 文档重组（1 天）

**目标**: 清理根目录，结构化文档

```bash
# 1. 创建 docs 目录结构
mkdir -p docs/{architecture,guides,development,integrations,migration}

# 2. 移动现有文档
mv ARCHITECTURE.md docs/architecture/overview.md
mv RUST_API_SERVER.md docs/architecture/api-design.md
mv RUVECTOR_INTEGRATION_PLAN.md docs/integrations/ruvector.md
mv MCP_SERVER_GUIDE.md docs/integrations/mcp-server.md
mv LLM_CONFIG_GUIDE.md docs/guides/llm-providers.md
mv DEPLOY_WINDOWS.md docs/guides/deployment.md
mv OPENCODE_SETUP.md docs/development/setup.md
mv QUICKSTART_API.md docs/guides/quickstart.md

# 3. 创建新文档
# docs/README.md - 文档索引
# docs/architecture/core-services.md - 核心服务设计
# docs/guides/installation.md - 安装指南
# docs/development/contributing.md - 贡献指南

# 4. 删除过时文档
rm llm-wiki.md  # Karpathy 原始文档（保留链接即可）
rm CLAUDE.md    # 合并到 AGENTS.md
rm KNOWLEDGE_GRAPH_SUMMARY.md  # 合并到架构文档
```

### Phase 2: 代码重组（3-5 天）

**目标**: 创建 Rust 工作空间，解耦核心逻辑

#### 2.1 创建工作空间

```bash
# 1. 创建根 Cargo.toml
cat > Cargo.toml << 'EOF'
[workspace]
members = [
    "crates/core/llm-client",
    "crates/core/ingest-engine",
    "crates/core/query-optimizer",
    "crates/core/token-cache",
    "crates/core/graph-engine",
    "crates/storage/ruvector-store",
    "crates/storage/sqlite-store",
    "crates/storage/file-store",
    "crates/api-server",
    "crates/desktop-app",
    "crates/cli",
]

[workspace.package]
version = "0.1.0"
edition = "2021"
license = "GPL-3.0"

[workspace.dependencies]
# 共享依赖
tokio = { version = "1", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
anyhow = "1.0"
tracing = "0.1"
EOF
```

#### 2.2 迁移核心逻辑

**从 TypeScript 迁移到 Rust**:

```bash
# 1. 创建 llm-client crate
cargo new --lib crates/core/llm-client

# 2. 迁移 src/lib/llm-client.ts → crates/core/llm-client/src/lib.rs
# 3. 迁移 src/lib/llm-providers.ts → crates/core/llm-client/src/providers/

# 4. 创建 ingest-engine crate
cargo new --lib crates/core/ingest-engine

# 5. 迁移 src/lib/ingest.ts → crates/core/ingest-engine/src/lib.rs
# 6. 迁移 src/lib/ingest-parse.ts → crates/core/ingest-engine/src/parser.rs

# ... 依次迁移其他模块
```

#### 2.3 创建 API 服务器

```bash
# 1. 创建 api-server crate
cargo new --bin crates/api-server

# 2. 实现 API 端点
# crates/api-server/src/routes/llm.rs
# crates/api-server/src/routes/ingest.rs
# crates/api-server/src/routes/query.rs
# crates/api-server/src/routes/token.rs

# 3. 测试 API 服务器
cargo run --bin api-server
curl http://localhost:19828/health
```

#### 2.4 更新桌面应用

```bash
# 1. 移动 src-tauri → crates/desktop-app
mv src-tauri crates/desktop-app

# 2. 移动 src → crates/desktop-app/ui
mv src crates/desktop-app/ui

# 3. 更新 Tauri 配置
# crates/desktop-app/tauri.conf.json
# 更新路径引用

# 4. 桌面应用调用 API 服务器（而非直接调用核心逻辑）
```

#### 2.5 更新 MCP Server

```bash
# 1. 移动 mcp-server → clients/mcp-server
mv mcp-server clients/mcp-server

# 2. 更新 API 客户端
# clients/mcp-server/src/api-client.ts
# 调用 HTTP API 而非直接调用 TS 代码

# 3. 测试 MCP Server
cd clients/mcp-server
npm install
npm test
```

### Phase 3: 测试与验证（2-3 天）

```bash
# 1. 单元测试
cargo test --workspace

# 2. 集成测试
cargo test --test integration

# 3. E2E 测试
npm run test:e2e

# 4. 性能基准测试
cargo bench
```

### Phase 4: 文档更新（1 天）

```bash
# 1. 更新 README.md
# 2. 生成 API 文档
cargo doc --workspace --no-deps --open

# 3. 更新所有指南
# docs/guides/installation.md
# docs/guides/configuration.md
# docs/development/setup.md
```

---

## 📊 重构前后对比

### 文档组织

| 维度 | 重构前 | 重构后 |
|------|--------|--------|
| 根目录 MD 文件 | 14 个 | 2 个（README + LICENSE） |
| 文档结构 | 扁平 | 分类（5 个目录） |
| 查找难度 | 高 | 低 |
| 维护成本 | 高 | 低 |

### 代码组织

| 维度 | 重构前 | 重构后 |
|------|--------|--------|
| 核心逻辑位置 | 前端 (TS) | 后端 (Rust) |
| 模块耦合度 | 高 | 低 |
| 可独立部署 | 否 | 是 |
| 代码复用 | 低 | 高 |
| 测试覆盖 | 部分 | 完整 |

### 架构清晰度

| 维度 | 重构前 | 重构后 |
|------|--------|--------|
| 层次划分 | 模糊 | 清晰（3 层） |
| 职责分离 | 差 | 好 |
| 扩展性 | 低 | 高 |
| 维护性 | 差 | 好 |

---

## 🎯 预期收益

### 1. 开发效率提升

- **模块化开发**: 团队可以并行开发不同模块
- **代码复用**: 核心库可以被多个客户端使用
- **测试效率**: 单元测试 + 集成测试 + E2E 测试分离

### 2. 部署灵活性

- **独立部署**: API 服务器可以独立部署
- **多客户端**: 桌面应用、MCP Server、CLI、Web UI 共享同一后端
- **水平扩展**: API 服务器可以多实例部署

### 3. 维护成本降低

- **文档清晰**: 结构化文档，易于查找和更新
- **职责明确**: 每个模块职责单一，易于维护
- **版本管理**: Rust 工作空间统一版本管理

### 4. 性能优化

- **Rust 核心**: 核心逻辑用 Rust 实现，性能更好
- **并发处理**: Tokio 异步运行时，高并发支持
- **内存安全**: Rust 编译器保证内存安全

---

## 🚀 实施建议

### 优先级

**P0 (立即执行)**:
1. 文档重组（1 天）
2. 创建 Rust 工作空间（1 天）

**P1 (短期执行)**:
3. 迁移核心逻辑到 Rust（3-5 天）
4. 完善 API 服务器（2-3 天）

**P2 (中期执行)**:
5. 更新桌面应用（2 天）
6. 更新 MCP Server（1 天）
7. 完整测试（2-3 天）

### 风险控制

1. **保留旧代码**: 在新结构稳定前，保留 `src/` 和 `src-tauri/` 作为备份
2. **渐进迁移**: 一次迁移一个模块，逐步验证
3. **自动化测试**: 每次迁移后运行完整测试套件
4. **文档同步**: 代码迁移的同时更新文档

---

## 📝 下一步行动

1. **评审本方案** - 确认重构方向
2. **创建 GitHub Issue** - 跟踪重构进度
3. **开始 Phase 1** - 文档重组
4. **并行开发** - 创建 Rust 工作空间

---

*文档生成时间: 2026-05-02*  
*作者: Sisyphus (OpenCode AI Agent)*
