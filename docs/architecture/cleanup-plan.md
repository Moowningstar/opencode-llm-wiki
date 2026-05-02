# 代码整理计划

> 目标：清理现有代码，为重构做准备

**执行时间**: 预计 2-3 小时  
**原则**: 不破坏现有功能，只做组织和清理

---

## 📋 整理清单

### Phase 1: 文档整理（30 分钟）

#### 1.1 创建 docs 目录结构

```bash
mkdir -p docs/{api,cli,architecture,guides,development}
```

#### 1.2 移动现有文档

```bash
# 架构文档
mv ARCHITECTURE.md docs/architecture/overview.md
mv KNOWLEDGE_GRAPH_SUMMARY.md docs/architecture/knowledge-graph.md

# API 文档
mv RUST_API_SERVER.md docs/api/server.md
mv QUICKSTART_API.md docs/api/quickstart.md

# 集成文档
mv MCP_SERVER_GUIDE.md docs/guides/mcp-server.md
mv LLM_CONFIG_GUIDE.md docs/guides/llm-config.md

# 部署文档
mv DEPLOY_WINDOWS.md docs/guides/deployment-windows.md

# 开发文档
mv OPENCODE_SETUP.md docs/development/setup.md

# 集成方案
mv RUVECTOR_INTEGRATION_PLAN.md docs/architecture/ruvector-integration.md
mv PROJECT_RESTRUCTURE_PLAN.md docs/architecture/restructure-plan.md
mv SIMPLIFIED_ARCHITECTURE.md docs/architecture/simplified.md
mv FINAL_ARCHITECTURE.md docs/architecture/final.md

# 保留在根目录
# README.md
# README_CN.md
# LICENSE
# AGENTS.md (OpenCode 配置)
# CLAUDE.md (Claude 配置)
```

#### 1.3 创建文档索引

```bash
# docs/README.md
cat > docs/README.md << 'EOF'
# OpenCode LLM Wiki 文档

## 📚 文档导航

### 快速开始
- [README](../README.md) - 项目总览
- [安装指南](guides/installation.md)
- [快速开始](api/quickstart.md)

### 架构设计
- [架构概览](architecture/overview.md)
- [最终架构](architecture/final.md)
- [知识图谱](architecture/knowledge-graph.md)
- [RuVector 集成](architecture/ruvector-integration.md)

### API 文档
- [API 服务器](api/server.md)
- [API 快速开始](api/quickstart.md)

### 使用指南
- [LLM 配置](guides/llm-config.md)
- [MCP 服务器](guides/mcp-server.md)
- [Windows 部署](guides/deployment-windows.md)

### 开发文档
- [开发环境设置](development/setup.md)
- [贡献指南](development/contributing.md)

EOF
```

### Phase 2: 代码备份（10 分钟）

```bash
# 创建备份分支
git checkout -b backup-before-restructure
git add .
git commit -m "backup: 重构前的代码备份"
git push origin backup-before-restructure

# 回到主分支
git checkout main
```

### Phase 3: 清理根目录（20 分钟）

#### 3.1 移除过时文件

```bash
# 检查是否有过时的文件
ls -la | grep -E '\.(log|tmp|cache)$'

# 移除 llm-wiki.md (Karpathy 原始文档，已有链接)
rm llm-wiki.md

# 移除测试文件
rm test-mcp-connection.js
```

#### 3.2 更新 .gitignore

```bash
cat >> .gitignore << 'EOF'

# ========== 重构相关 ==========
# 旧代码备份
*.backup
*.old

# 临时文件
*.tmp
*.temp

# 构建产物
/src-server/target/
/src-desktop/target/
/src-desktop/ui/dist/

# 依赖
/src-server/Cargo.lock
/src-desktop/Cargo.lock
/src-mcp/node_modules/
/src-web/node_modules/
/src-extension/node_modules/

# IDE
.vscode/
.idea/

# 测试数据
/test-data/
/examples/*/node_modules/

EOF
```

### Phase 4: 整理现有代码（1 小时）

#### 4.1 标记待迁移的代码

在每个需要迁移的文件顶部添加注释：

**src/lib/llm-client.ts**:
```typescript
/**
 * ⚠️ TODO: 迁移到 src-server/src/services/llm_client.rs
 * 
 * 这个文件将在重构时迁移到 Rust 服务端
 * 迁移完成后，桌面客户端将通过 API 调用
 */
```

**src/lib/ingest.ts**:
```typescript
/**
 * ⚠️ TODO: 迁移到 src-server/src/services/ingest_engine.rs
 * 
 * 这个文件将在重构时迁移到 Rust 服务端
 */
```

#### 4.2 创建迁移清单

```bash
cat > MIGRATION_CHECKLIST.md << 'EOF'
# 代码迁移清单

## 核心逻辑 (src/lib/ → src-server/src/services/)

### 高优先级（核心功能）
- [ ] llm-client.ts → llm_client.rs
- [ ] llm-providers.ts → llm_client/providers.rs
- [ ] ingest.ts → ingest_engine.rs
- [ ] ingest-parse.ts → ingest_engine/parser.rs
- [ ] search.ts → query_optimizer.rs
- [ ] graph-relevance.ts → graph_engine/relevance.rs
- [ ] graph-insights.ts → graph_engine/insights.rs

### 中优先级（辅助功能）
- [ ] embedding.ts → services/embedding.rs
- [ ] token-cache.ts → services/token_cache.rs
- [ ] context-budget.ts → query_optimizer/budget.rs
- [ ] wiki-graph.ts → graph_engine/builder.rs

### 低优先级（工具函数）
- [ ] path-utils.ts → types/path.rs
- [ ] file-types.ts → types/file.rs
- [ ] detect-language.ts → services/language.rs

## UI 组件 (src/components/ → src-desktop/ui/src/components/)

- [ ] chat/ → 保持不变
- [ ] editor/ → 保持不变
- [ ] graph/ → 保持不变
- [ ] layout/ → 保持不变
- [ ] settings/ → 更新 API 调用

## 状态管理 (src/stores/ → src-desktop/ui/src/stores/)

- [ ] wiki-store.ts → 保持不变
- [ ] chat-store.ts → 保持不变
- [ ] activity-store.ts → 保持不变
- [ ] review-store.ts → 保持不变

## Tauri 命令 (src-tauri/src/commands/ → src-desktop/src-tauri/src/)

- [ ] fs.rs → 保持不变（文件系统操作）
- [ ] project.rs → 保持不变（项目管理）
- [ ] vectorstore.rs → 移除（改用 API）

## MCP 服务器 (mcp-server/ → src-mcp/)

- [ ] server.js → server.ts
- [ ] lib/ → src/
- [ ] 更新 API 调用逻辑

## Chrome 扩展 (extension/ → src-extension/)

- [ ] 移动所有文件
- [ ] 更新 API 调用

EOF
```

### Phase 5: 创建新目录结构（20 分钟）

```bash
# 创建服务端目录
mkdir -p src-server/src/{api,services,storage,types}
mkdir -p src-server/src/api/handlers
mkdir -p src-server/tests
mkdir -p src-server/benches

# 创建桌面客户端目录
mkdir -p src-desktop/src-tauri/src
mkdir -p src-desktop/ui/src/{api,components,stores,types}

# 创建 MCP 服务器目录
mkdir -p src-mcp/src/{tools,types}

# 创建 Web UI 目录
mkdir -p src-web/src

# 创建 Chrome 扩展目录
mkdir -p src-extension/src

# 创建脚本目录
mkdir -p scripts

# 创建示例目录
mkdir -p examples/{basic-wiki,research-notes}
```

### Phase 6: 创建占位文件（20 分钟）

#### 6.1 根目录 Cargo.toml

```bash
cat > Cargo.toml << 'EOF'
[workspace]
members = [
    "src-server",
    # "src-desktop/src-tauri",  # 暂时注释，等迁移时启用
]

[workspace.package]
version = "0.1.0"
edition = "2021"
license = "GPL-3.0"
authors = ["OpenCode Team"]

[workspace.dependencies]
tokio = { version = "1", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
anyhow = "1.0"
thiserror = "1.0"
tracing = "0.1"
tracing-subscriber = "0.3"
axum = "0.7"
tower-http = { version = "0.5", features = ["cors", "trace"] }
clap = { version = "4", features = ["derive"] }
ruvector-core = "2.1"
ruvector-graph = "2.1"
ruvector-gnn = "2.1"
tiktoken-rs = "0.5"
EOF
```

#### 6.2 src-server/Cargo.toml

```bash
cat > src-server/Cargo.toml << 'EOF'
[package]
name = "llm-wiki-server"
version.workspace = true
edition.workspace = true
license.workspace = true

[[bin]]
name = "llm-wiki-server"
path = "src/main.rs"

[[bin]]
name = "llm-wiki"
path = "src/cli.rs"

[dependencies]
tokio.workspace = true
serde.workspace = true
serde_json.workspace = true
anyhow.workspace = true
thiserror.workspace = true
tracing.workspace = true
tracing-subscriber.workspace = true
axum.workspace = true
tower-http.workspace = true
clap.workspace = true
ruvector-core.workspace = true
ruvector-graph.workspace = true
ruvector-gnn.workspace = true
tiktoken-rs.workspace = true

chrono = "0.4"
uuid = { version = "1.0", features = ["v4", "serde"] }

[dev-dependencies]
criterion = "0.5"
EOF
```

#### 6.3 src-server/src/main.rs (占位)

```bash
cat > src-server/src/main.rs << 'EOF'
//! OpenCode LLM Wiki - API 服务器
//! 
//! ⚠️ 占位文件 - 等待迁移

fn main() {
    println!("🚧 OpenCode LLM Wiki Server");
    println!("   This is a placeholder. Migration in progress...");
}
EOF
```

#### 6.4 src-server/src/cli.rs (占位)

```bash
cat > src-server/src/cli.rs << 'EOF'
//! OpenCode LLM Wiki - CLI 工具
//! 
//! ⚠️ 占位文件 - 等待迁移

fn main() {
    println!("🚧 OpenCode LLM Wiki CLI");
    println!("   This is a placeholder. Migration in progress...");
}
EOF
```

#### 6.5 构建脚本

```bash
cat > scripts/build-server.sh << 'EOF'
#!/bin/bash
set -e

echo "🦀 Building server..."
cd src-server
cargo build --release --bin llm-wiki-server
cargo build --release --bin llm-wiki

echo "✅ Server built successfully!"
EOF

chmod +x scripts/build-server.sh
```

### Phase 7: 更新 README（10 分钟）

```bash
cat > README.md << 'EOF'
# OpenCode LLM Wiki

> ⚠️ **项目重构中** - 正在将架构从桌面应用优先改为服务端优先

<p align="center">
  <img src="logo.jpg" width="128" height="128" style="border-radius: 22%;" alt="OpenCode LLM Wiki Logo">
</p>

<p align="center">
  <strong>A personal knowledge base that builds itself.</strong><br>
  LLM reads your documents, builds a structured wiki, and keeps it current.
</p>

## 🚧 重构进度

我们正在重构项目架构，将核心逻辑从 TypeScript 迁移到 Rust，实现服务端优先的设计。

**新架构**:
```
opencode-llm-wiki/
├── src-server/          # 🦀 核心服务端 (Rust)
├── src-desktop/         # 🖥️ 桌面客户端 (Tauri + React)
├── src-mcp/             # 📡 MCP 服务器 (Node.js)
├── src-web/             # 🌐 Web UI (未来)
└── src-extension/       # 🔌 Chrome 扩展
```

**进度**:
- [x] 架构设计完成
- [x] 文档整理完成
- [x] 目录结构创建
- [ ] 核心逻辑迁移（进行中）
- [ ] 桌面客户端更新
- [ ] MCP 服务器更新
- [ ] 测试与发布

## 📚 文档

- [完整文档](docs/README.md)
- [最终架构](docs/architecture/final.md)
- [迁移清单](MIGRATION_CHECKLIST.md)

## 🚀 快速开始（旧版本）

当前版本仍然可用，但建议等待重构完成后使用新版本。

```bash
# 安装依赖
npm install

# 开发模式
npm run tauri dev

# 构建
npm run tauri build
```

## 📖 原始 README

原始的完整 README 已移至 [docs/archive/README-v1.md](docs/archive/README-v1.md)

## 📝 License

GPL-3.0

---

*重构进行中 - 预计完成时间: 2026-05-10*
EOF
```

### Phase 8: Git 提交（10 分钟）

```bash
# 添加所有更改
git add .

# 提交
git commit -m "chore: 代码整理 - 为重构做准备

- 移动文档到 docs/ 目录
- 创建新的目录结构
- 添加迁移清单
- 更新 README
- 创建占位文件
"

# 推送
git push origin main
```

---

## ✅ 整理完成检查清单

- [ ] 文档已移动到 docs/ 目录
- [ ] 创建了文档索引 (docs/README.md)
- [ ] 创建了备份分支
- [ ] 清理了根目录
- [ ] 更新了 .gitignore
- [ ] 标记了待迁移的代码
- [ ] 创建了迁移清单 (MIGRATION_CHECKLIST.md)
- [ ] 创建了新目录结构
- [ ] 创建了占位文件
- [ ] 更新了 README.md
- [ ] Git 提交并推送

---

## 📊 整理前后对比

### 根目录文件数量

| 类型 | 整理前 | 整理后 |
|------|--------|--------|
| MD 文件 | 14 个 | 4 个 (README, README_CN, LICENSE, AGENTS) |
| 配置文件 | 10+ 个 | 10+ 个 (保持不变) |
| 目录 | 8 个 | 12 个 (新增 docs, src-server, src-mcp 等) |

### 文档组织

| 维度 | 整理前 | 整理后 |
|------|--------|--------|
| 文档位置 | 根目录 | docs/ 目录 |
| 文档分类 | 无 | 5 个分类 |
| 查找难度 | 高 | 低 |
| 维护成本 | 高 | 低 |

---

## 🎯 下一步

整理完成后，我们可以开始真正的重构：

1. **Phase 1**: 迁移核心逻辑到 Rust
2. **Phase 2**: 更新桌面客户端
3. **Phase 3**: 更新 MCP 服务器
4. **Phase 4**: 测试与发布

---

*整理计划生成时间: 2026-05-02*  
*预计执行时间: 2-3 小时*
