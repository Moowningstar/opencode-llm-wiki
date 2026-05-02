# OpenCode LLM Wiki 简化架构方案

> 核心理念：服务端 + CLI 为核心，桌面客户端独立打包

**生成时间**: 2026-05-02  
**架构原则**: 服务端优先、客户端可选、模块独立

---

## 🎯 核心架构

```
┌─────────────────────────────────────────────────────────┐
│  核心项目 (opencode-llm-wiki-core)                      │
│  ├─ HTTP API 服务器 (Rust)                              │
│  └─ CLI 工具 (Rust)                                     │
└────────────────┬────────────────────────────────────────┘
                 │ HTTP REST API
                 ↓
┌─────────────────────────────────────────────────────────┐
│  可选客户端 (独立仓库/独立打包)                         │
│  ├─ Desktop App (opencode-llm-wiki-desktop)            │
│  ├─ MCP Server (opencode-llm-wiki-mcp)                 │
│  ├─ Web UI (opencode-llm-wiki-web)                     │
│  └─ Chrome Extension (opencode-llm-wiki-extension)     │
└─────────────────────────────────────────────────────────┘
```

---

## 📁 新项目结构

### 方案 A: 单仓库 (Monorepo)

```
opencode-llm-wiki/
├── README.md                    # 项目总览
├── LICENSE
├── Cargo.toml                   # Rust 工作空间
│
├── docs/                        # 📚 文档
│   ├── README.md
│   ├── api/                    # API 文档
│   ├── cli/                    # CLI 文档
│   └── development/            # 开发文档
│
├── core/                        # 🦀 核心服务端 (Rust)
│   ├── Cargo.toml
│   ├── src/
│   │   ├── main.rs             # API 服务器入口
│   │   ├── cli.rs              # CLI 入口
│   │   ├── lib.rs              # 核心库
│   │   │
│   │   ├── api/                # API 层
│   │   │   ├── mod.rs
│   │   │   ├── routes.rs
│   │   │   ├── handlers/
│   │   │   │   ├── llm.rs
│   │   │   │   ├── ingest.rs
│   │   │   │   ├── query.rs
│   │   │   │   └── token.rs
│   │   │   └── middleware/
│   │   │
│   │   ├── services/           # 业务逻辑层
│   │   │   ├── mod.rs
│   │   │   ├── llm_client.rs
│   │   │   ├── ingest_engine.rs
│   │   │   ├── query_optimizer.rs
│   │   │   ├── token_cache.rs
│   │   │   └── graph_engine.rs
│   │   │
│   │   ├── storage/            # 存储层
│   │   │   ├── mod.rs
│   │   │   ├── ruvector.rs
│   │   │   ├── sqlite.rs
│   │   │   └── filesystem.rs
│   │   │
│   │   └── types/              # 类型定义
│   │       ├── mod.rs
│   │       ├── config.rs
│   │       ├── wiki.rs
│   │       └── api.rs
│   │
│   ├── tests/                  # 测试
│   │   ├── integration/
│   │   └── unit/
│   │
│   └── benches/                # 性能测试
│
├── clients/                     # 🖥️ 客户端（独立打包）
│   │
│   ├── desktop/                # 桌面客户端
│   │   ├── README.md
│   │   ├── package.json
│   │   ├── Cargo.toml          # Tauri 配置
│   │   ├── src-tauri/          # Tauri 后端
│   │   │   └── src/
│   │   │       └── main.rs     # 仅启动 API 服务器 + UI
│   │   └── ui/                 # React 前端
│   │       ├── src/
│   │       │   ├── api/        # API 客户端
│   │       │   ├── components/
│   │       │   ├── stores/
│   │       │   └── App.tsx
│   │       └── package.json
│   │
│   ├── mcp-server/             # MCP 服务器
│   │   ├── README.md
│   │   ├── package.json
│   │   └── src/
│   │       ├── server.ts
│   │       ├── tools/
│   │       └── api-client.ts   # 调用核心 API
│   │
│   ├── web-ui/                 # Web UI（未来）
│   │   └── README.md
│   │
│   └── chrome-extension/       # Chrome 扩展
│       ├── manifest.json
│       └── src/
│
├── examples/                    # 示例项目
│   ├── basic-wiki/
│   └── research-notes/
│
└── scripts/                     # 构建/部署脚本
    ├── build-core.sh           # 构建核心
    ├── build-desktop.sh        # 构建桌面客户端
    ├── build-all.sh            # 构建所有
    └── release.sh              # 发布脚本
```

### 方案 B: 多仓库 (Multirepo) - 推荐

```
# 核心仓库（必需）
opencode-llm-wiki-core/
├── README.md
├── Cargo.toml
├── src/
│   ├── main.rs                 # API 服务器 + CLI
│   ├── api/
│   ├── services/
│   ├── storage/
│   └── types/
└── docs/

# 桌面客户端（可选）
opencode-llm-wiki-desktop/
├── README.md
├── package.json
├── Cargo.toml
├── src-tauri/
└── ui/

# MCP 服务器（可选）
opencode-llm-wiki-mcp/
├── README.md
├── package.json
└── src/

# Web UI（可选）
opencode-llm-wiki-web/
├── README.md
├── package.json
└── src/

# Chrome 扩展（可选）
opencode-llm-wiki-extension/
├── README.md
├── manifest.json
└── src/
```

---

## 🦀 核心服务端设计

### Cargo.toml

```toml
[package]
name = "opencode-llm-wiki-core"
version = "0.1.0"
edition = "2021"
license = "GPL-3.0"

[[bin]]
name = "llm-wiki-server"
path = "src/main.rs"

[[bin]]
name = "llm-wiki"
path = "src/cli.rs"

[dependencies]
# Web 框架
axum = "0.7"
tower-http = { version = "0.5", features = ["cors", "trace"] }
tokio = { version = "1", features = ["full"] }

# RuVector
ruvector-core = "2.1"
ruvector-graph = "2.1"
ruvector-gnn = "2.1"

# Token 处理
tiktoken-rs = "0.5"

# 序列化
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# CLI
clap = { version = "4", features = ["derive"] }

# 日志
tracing = "0.1"
tracing-subscriber = "0.3"

# 错误处理
anyhow = "1.0"
thiserror = "1.0"

[dev-dependencies]
criterion = "0.5"
```

### 核心入口

**src/main.rs** (API 服务器):
```rust
use axum::{Router, routing::get};
use tower_http::cors::CorsLayer;
use tracing_subscriber;

mod api;
mod services;
mod storage;
mod types;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 初始化日志
    tracing_subscriber::fmt::init();
    
    // 读取配置
    let config = load_config()?;
    
    // 初始化存储
    let storage = storage::init(&config).await?;
    
    // 初始化服务
    let services = services::init(storage, &config).await?;
    
    // 构建路由
    let app = Router::new()
        .route("/health", get(api::health))
        .nest("/api/llm", api::llm::routes())
        .nest("/api/ingest", api::ingest::routes())
        .nest("/api/query", api::query::routes())
        .nest("/api/token", api::token::routes())
        .layer(CorsLayer::permissive())
        .with_state(services);
    
    // 启动服务器
    let addr = format!("{}:{}", config.host, config.port);
    tracing::info!("🚀 Server listening on {}", addr);
    
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;
    
    Ok(())
}
```

**src/cli.rs** (CLI 工具):
```rust
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "llm-wiki")]
#[command(about = "OpenCode LLM Wiki CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the API server
    Serve {
        #[arg(short, long, default_value = "19828")]
        port: u16,
    },
    
    /// Initialize a new wiki project
    Init {
        path: String,
        #[arg(short, long)]
        template: Option<String>,
    },
    
    /// Ingest a document
    Ingest {
        file: String,
        #[arg(short, long)]
        project: Option<String>,
    },
    
    /// Query the wiki
    Query {
        query: String,
        #[arg(short, long)]
        project: Option<String>,
    },
    
    /// Manage the wiki
    #[command(subcommand)]
    Manage(ManageCommands),
}

#[derive(Subcommand)]
enum ManageCommands {
    /// List all pages
    List,
    /// Show wiki statistics
    Stats,
    /// Rebuild index
    Reindex,
    /// Clean cache
    Clean,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Serve { port } => {
            // 启动 API 服务器
            serve(port).await?;
        }
        Commands::Init { path, template } => {
            // 初始化项目
            init_project(&path, template.as_deref()).await?;
        }
        Commands::Ingest { file, project } => {
            // 导入文档
            ingest_file(&file, project.as_deref()).await?;
        }
        Commands::Query { query, project } => {
            // 查询
            query_wiki(&query, project.as_deref()).await?;
        }
        Commands::Manage(cmd) => {
            match cmd {
                ManageCommands::List => list_pages().await?,
                ManageCommands::Stats => show_stats().await?,
                ManageCommands::Reindex => reindex().await?,
                ManageCommands::Clean => clean_cache().await?,
            }
        }
    }
    
    Ok(())
}
```

---

## 🖥️ 桌面客户端设计

### 职责

**桌面客户端只做两件事**:
1. **启动核心 API 服务器**（嵌入式）
2. **提供 UI 界面**（调用 API）

### src-tauri/src/main.rs

```rust
use tauri::Manager;
use std::process::{Command, Child};
use std::sync::Mutex;

struct ApiServer(Mutex<Option<Child>>);

#[tauri::command]
async fn api_request(
    endpoint: String,
    method: String,
    body: Option<String>,
) -> Result<String, String> {
    // 调用本地 API 服务器
    let url = format!("http://localhost:19828{}", endpoint);
    
    let client = reqwest::Client::new();
    let request = match method.as_str() {
        "GET" => client.get(&url),
        "POST" => client.post(&url).body(body.unwrap_or_default()),
        _ => return Err("Unsupported method".to_string()),
    };
    
    let response = request
        .send()
        .await
        .map_err(|e| e.to_string())?;
    
    let text = response.text().await.map_err(|e| e.to_string())?;
    Ok(text)
}

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            // 启动嵌入式 API 服务器
            let api_server = Command::new("llm-wiki-server")
                .arg("--port")
                .arg("19828")
                .spawn()
                .expect("Failed to start API server");
            
            app.manage(ApiServer(Mutex::new(Some(api_server))));
            
            Ok(())
        })
        .on_window_event(|event| {
            if let tauri::WindowEvent::Destroyed = event.event() {
                // 关闭 API 服务器
                if let Some(api_server) = event.window().state::<ApiServer>().0.lock().unwrap().as_mut() {
                    let _ = api_server.kill();
                }
            }
        })
        .invoke_handler(tauri::generate_handler![api_request])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

### ui/src/api/client.ts

```typescript
// 统一的 API 客户端
class ApiClient {
  private baseUrl = 'http://localhost:19828';

  async request<T>(
    endpoint: string,
    method: 'GET' | 'POST' = 'GET',
    body?: any
  ): Promise<T> {
    // 在 Tauri 环境中，使用 Tauri 命令
    if (window.__TAURI__) {
      const response = await window.__TAURI__.invoke('api_request', {
        endpoint,
        method,
        body: body ? JSON.stringify(body) : undefined,
      });
      return JSON.parse(response);
    }
    
    // 在浏览器环境中，直接调用 API
    const response = await fetch(`${this.baseUrl}${endpoint}`, {
      method,
      headers: { 'Content-Type': 'application/json' },
      body: body ? JSON.stringify(body) : undefined,
    });
    
    return response.json();
  }

  // LLM API
  async streamChat(config: LlmConfig, messages: ChatMessage[]) {
    return this.request('/api/llm/stream', 'POST', { config, messages });
  }

  // Ingest API
  async ingestFile(projectPath: string, filePath: string) {
    return this.request('/api/ingest', 'POST', { projectPath, filePath });
  }

  // Query API
  async query(query: string, maxTokens?: number) {
    return this.request('/api/query', 'POST', { query, maxTokens });
  }

  // Token Cache API
  async refreshTokenCache(projectPath: string) {
    return this.request('/api/token/cache/refresh', 'POST', { projectPath });
  }
}

export const apiClient = new ApiClient();
```

---

## 📦 构建与发布

### 构建脚本

**scripts/build-core.sh**:
```bash
#!/bin/bash
set -e

echo "🦀 Building core server..."
cd core
cargo build --release --bin llm-wiki-server
cargo build --release --bin llm-wiki

echo "✅ Core built successfully!"
echo "   Server: target/release/llm-wiki-server"
echo "   CLI: target/release/llm-wiki"
```

**scripts/build-desktop.sh**:
```bash
#!/bin/bash
set -e

echo "🖥️ Building desktop client..."

# 1. 构建核心（嵌入到桌面客户端）
./scripts/build-core.sh

# 2. 复制核心二进制到桌面客户端
cp core/target/release/llm-wiki-server clients/desktop/src-tauri/binaries/

# 3. 构建桌面客户端
cd clients/desktop
npm install
npm run tauri build

echo "✅ Desktop client built successfully!"
```

**scripts/build-all.sh**:
```bash
#!/bin/bash
set -e

echo "🚀 Building all components..."

# 1. 核心
./scripts/build-core.sh

# 2. 桌面客户端
./scripts/build-desktop.sh

# 3. MCP 服务器
echo "📡 Building MCP server..."
cd clients/mcp-server
npm install
npm run build

# 4. Chrome 扩展
echo "🌐 Building Chrome extension..."
cd clients/chrome-extension
npm install
npm run build

echo "✅ All components built successfully!"
```

### 发布策略

**核心服务端**:
```bash
# 发布到 crates.io
cd core
cargo publish

# 发布到 GitHub Releases
gh release create v0.1.0 \
  target/release/llm-wiki-server \
  target/release/llm-wiki
```

**桌面客户端**:
```bash
# 独立发布
cd clients/desktop
npm run tauri build

# 发布到 GitHub Releases
gh release create desktop-v0.1.0 \
  src-tauri/target/release/bundle/dmg/*.dmg \
  src-tauri/target/release/bundle/msi/*.msi
```

---

## 🔄 迁移步骤

### Phase 1: 创建核心服务端（3-5 天）

```bash
# 1. 创建核心目录
mkdir -p core/src/{api,services,storage,types}

# 2. 迁移核心逻辑
# src/lib/llm-client.ts → core/src/services/llm_client.rs
# src/lib/ingest.ts → core/src/services/ingest_engine.rs
# src/lib/search.ts → core/src/services/query_optimizer.rs
# ... 依次迁移

# 3. 实现 API 端点
# core/src/api/handlers/llm.rs
# core/src/api/handlers/ingest.rs
# core/src/api/handlers/query.rs

# 4. 实现 CLI
# core/src/cli.rs

# 5. 测试
cd core
cargo test
cargo run --bin llm-wiki-server
cargo run --bin llm-wiki -- --help
```

### Phase 2: 重构桌面客户端（2-3 天）

```bash
# 1. 创建客户端目录
mkdir -p clients/desktop

# 2. 移动现有代码
mv src-tauri clients/desktop/
mv src clients/desktop/ui/

# 3. 简化 Tauri 后端（只启动 API 服务器）
# clients/desktop/src-tauri/src/main.rs

# 4. 更新前端（调用 API）
# clients/desktop/ui/src/api/client.ts

# 5. 测试
cd clients/desktop
npm run tauri dev
```

### Phase 3: 更新其他客户端（1-2 天）

```bash
# 1. 移动 MCP 服务器
mv mcp-server clients/mcp-server

# 2. 更新 API 调用
# clients/mcp-server/src/api-client.ts

# 3. 移动 Chrome 扩展
mv extension clients/chrome-extension

# 4. 测试
cd clients/mcp-server
npm test

cd clients/chrome-extension
npm run build
```

### Phase 4: 文档与发布（1 天）

```bash
# 1. 更新文档
# README.md
# docs/api/
# docs/cli/

# 2. 构建所有组件
./scripts/build-all.sh

# 3. 发布
./scripts/release.sh
```

---

## 📊 架构对比

| 维度 | 旧架构 | 新架构 |
|------|--------|--------|
| **核心定位** | 桌面应用 | 服务端 + CLI |
| **客户端** | 紧耦合 | 独立打包 |
| **部署方式** | 仅桌面 | 服务器/桌面/CLI |
| **扩展性** | 低 | 高 |
| **维护成本** | 高 | 低 |
| **代码复用** | 低 | 高 |

---

## 🎯 预期收益

### 1. 灵活部署

- **服务器模式**: `llm-wiki-server --port 19828`
- **CLI 模式**: `llm-wiki query "transformer architecture"`
- **桌面模式**: 双击启动，自动启动服务器
- **Docker 模式**: `docker run -p 19828:19828 llm-wiki-core`

### 2. 客户端多样化

- **桌面客户端**: Windows/macOS/Linux
- **Web UI**: 浏览器访问
- **MCP Server**: IDE 集成
- **CLI**: 脚本自动化
- **移动端**: 未来可能

### 3. 开发效率

- **核心团队**: 专注服务端 (Rust)
- **UI 团队**: 专注客户端 (React/Vue/...)
- **独立发布**: 核心和客户端独立版本
- **并行开发**: 多个客户端同时开发

---

## 🚀 推荐方案

**推荐使用方案 B（多仓库）**:

**理由**:
1. **职责清晰**: 核心和客户端完全分离
2. **独立发布**: 各自独立版本管理
3. **团队协作**: 不同团队维护不同仓库
4. **用户选择**: 用户只需安装需要的部分

**仓库结构**:
```
opencode-llm-wiki-core        # 核心（必需）
opencode-llm-wiki-desktop     # 桌面客户端（可选）
opencode-llm-wiki-mcp         # MCP 服务器（可选）
opencode-llm-wiki-web         # Web UI（可选）
opencode-llm-wiki-extension   # Chrome 扩展（可选）
```

---

*文档生成时间: 2026-05-02*  
*作者: Sisyphus (OpenCode AI Agent)*
