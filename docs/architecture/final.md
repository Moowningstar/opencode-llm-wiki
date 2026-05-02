# OpenCode LLM Wiki 最终架构方案

> 单仓库 Monorepo，顶层目录清晰划分

**生成时间**: 2026-05-02  
**架构原则**: 单仓库、模块独立、职责清晰

---

## 📁 项目结构

```
opencode-llm-wiki/
├── README.md                    # 项目总览
├── LICENSE
├── Cargo.toml                   # Rust 工作空间
├── package.json                 # 前端依赖（可选）
│
├── docs/                        # 📚 文档
│   ├── README.md               # 文档索引
│   ├── api.md                  # API 文档
│   ├── cli.md                  # CLI 文档
│   ├── architecture.md         # 架构设计
│   └── development.md          # 开发指南
│
├── src-server/                  # 🦀 核心服务端 (Rust)
│   ├── Cargo.toml
│   ├── src/
│   │   ├── main.rs             # API 服务器入口
│   │   ├── cli.rs              # CLI 入口
│   │   ├── lib.rs              # 核心库
│   │   │
│   │   ├── api/                # API 层
│   │   │   ├── mod.rs
│   │   │   ├── routes.rs
│   │   │   └── handlers/
│   │   │       ├── llm.rs
│   │   │       ├── ingest.rs
│   │   │       ├── query.rs
│   │   │       └── token.rs
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
│   └── benches/                # 性能测试
│
├── src-desktop/                 # 🖥️ 桌面客户端 (Tauri + React)
│   ├── README.md
│   ├── package.json
│   ├── Cargo.toml              # Tauri 配置
│   │
│   ├── src-tauri/              # Tauri 后端
│   │   ├── Cargo.toml
│   │   ├── tauri.conf.json
│   │   ├── build.rs
│   │   └── src/
│   │       ├── main.rs         # 启动服务器 + UI
│   │       └── lib.rs
│   │
│   └── ui/                     # React 前端
│       ├── package.json
│       ├── vite.config.ts
│       ├── index.html
│       └── src/
│           ├── App.tsx
│           ├── api/            # API 客户端
│           │   └── client.ts
│           ├── components/     # UI 组件
│           ├── stores/         # 状态管理
│           └── types/          # 类型定义
│
├── src-mcp/                     # 📡 MCP 服务器 (Node.js)
│   ├── README.md
│   ├── package.json
│   ├── tsconfig.json
│   └── src/
│       ├── server.ts           # MCP 服务器入口
│       ├── tools/              # MCP 工具
│       │   ├── wiki-read.ts
│       │   ├── wiki-query.ts
│       │   └── wiki-ingest.ts
│       └── api-client.ts       # 调用核心 API
│
├── src-web/                     # 🌐 Web UI (React/Vue - 未来)
│   ├── README.md
│   ├── package.json
│   └── src/
│       ├── App.tsx
│       ├── api/
│       └── components/
│
├── src-extension/               # 🔌 Chrome 扩展
│   ├── README.md
│   ├── manifest.json
│   ├── package.json
│   └── src/
│       ├── popup/
│       ├── content/
│       └── background/
│
├── examples/                    # 示例项目
│   ├── basic-wiki/
│   └── research-notes/
│
└── scripts/                     # 构建/部署脚本
    ├── build-server.sh         # 构建服务端
    ├── build-desktop.sh        # 构建桌面客户端
    ├── build-mcp.sh            # 构建 MCP 服务器
    ├── build-all.sh            # 构建所有
    └── dev.sh                  # 开发模式
```

---

## 🦀 Rust 工作空间配置

### 根目录 Cargo.toml

```toml
[workspace]
members = [
    "src-server",
    "src-desktop/src-tauri",
]

[workspace.package]
version = "0.1.0"
edition = "2021"
license = "GPL-3.0"
authors = ["OpenCode Team"]

[workspace.dependencies]
# 共享依赖
tokio = { version = "1", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
anyhow = "1.0"
thiserror = "1.0"
tracing = "0.1"
tracing-subscriber = "0.3"

# RuVector
ruvector-core = "2.1"
ruvector-graph = "2.1"
ruvector-gnn = "2.1"

# Token 处理
tiktoken-rs = "0.5"

# Web 框架
axum = "0.7"
tower-http = { version = "0.5", features = ["cors", "trace"] }

# CLI
clap = { version = "4", features = ["derive"] }

# Tauri
tauri = { version = "2", features = [] }
tauri-build = { version = "2", features = [] }
```

---

## 🦀 核心服务端 (src-server/)

### src-server/Cargo.toml

```toml
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
# 工作空间依赖
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

# 额外依赖
chrono = "0.4"
uuid = { version = "1.0", features = ["v4", "serde"] }

[dev-dependencies]
criterion = "0.5"
```

### src-server/src/main.rs

```rust
//! OpenCode LLM Wiki - API 服务器
//! 
//! 启动方式:
//! ```bash
//! llm-wiki-server --port 19828
//! ```

use axum::{Router, routing::get};
use clap::Parser;
use tower_http::cors::CorsLayer;
use tracing_subscriber;

mod api;
mod services;
mod storage;
mod types;

#[derive(Parser)]
#[command(name = "llm-wiki-server")]
#[command(about = "OpenCode LLM Wiki API Server")]
struct Args {
    #[arg(short, long, default_value = "127.0.0.1")]
    host: String,
    
    #[arg(short, long, default_value = "19828")]
    port: u16,
    
    #[arg(short, long)]
    config: Option<String>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_target(false)
        .compact()
        .init();
    
    let args = Args::parse();
    
    // 读取配置
    let config = if let Some(config_path) = args.config {
        types::Config::from_file(&config_path)?
    } else {
        types::Config::default()
    };
    
    // 初始化存储
    let storage = storage::init(&config).await?;
    
    // 初始化服务
    let app_state = services::AppState::new(storage, config).await?;
    
    // 构建路由
    let app = Router::new()
        .route("/health", get(api::health))
        .nest("/api/llm", api::llm::routes())
        .nest("/api/ingest", api::ingest::routes())
        .nest("/api/query", api::query::routes())
        .nest("/api/token", api::token::routes())
        .layer(CorsLayer::permissive())
        .with_state(app_state);
    
    // 启动服务器
    let addr = format!("{}:{}", args.host, args.port);
    tracing::info!("🚀 OpenCode LLM Wiki Server");
    tracing::info!("   Listening on http://{}", addr);
    tracing::info!("   API docs: http://{}/docs", addr);
    
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;
    
    Ok(())
}
```

### src-server/src/cli.rs

```rust
//! OpenCode LLM Wiki - CLI 工具
//! 
//! 使用示例:
//! ```bash
//! # 启动服务器
//! llm-wiki serve --port 19828
//! 
//! # 初始化项目
//! llm-wiki init my-wiki --template research
//! 
//! # 导入文档
//! llm-wiki ingest document.pdf --project my-wiki
//! 
//! # 查询
//! llm-wiki query "transformer architecture" --project my-wiki
//! ```

use clap::{Parser, Subcommand};
use anyhow::Result;

#[derive(Parser)]
#[command(name = "llm-wiki")]
#[command(about = "OpenCode LLM Wiki CLI", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the API server
    Serve {
        #[arg(short, long, default_value = "127.0.0.1")]
        host: String,
        
        #[arg(short, long, default_value = "19828")]
        port: u16,
    },
    
    /// Initialize a new wiki project
    Init {
        /// Project path
        path: String,
        
        /// Template (research, reading, personal, business, general)
        #[arg(short, long, default_value = "general")]
        template: String,
    },
    
    /// Ingest a document
    Ingest {
        /// File path
        file: String,
        
        /// Project path
        #[arg(short, long)]
        project: Option<String>,
    },
    
    /// Query the wiki
    Query {
        /// Query string
        query: String,
        
        /// Project path
        #[arg(short, long)]
        project: Option<String>,
        
        /// Max tokens
        #[arg(short, long, default_value = "4000")]
        max_tokens: usize,
    },
    
    /// Manage the wiki
    #[command(subcommand)]
    Manage(ManageCommands),
}

#[derive(Subcommand)]
enum ManageCommands {
    /// List all pages
    List {
        #[arg(short, long)]
        project: Option<String>,
    },
    
    /// Show wiki statistics
    Stats {
        #[arg(short, long)]
        project: Option<String>,
    },
    
    /// Rebuild index
    Reindex {
        #[arg(short, long)]
        project: Option<String>,
    },
    
    /// Clean cache
    Clean {
        #[arg(short, long)]
        project: Option<String>,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Serve { host, port } => {
            // 启动服务器（复用 main.rs 逻辑）
            println!("🚀 Starting server on {}:{}...", host, port);
            // TODO: 调用 main.rs 的启动逻辑
        }
        
        Commands::Init { path, template } => {
            println!("📁 Initializing wiki at: {}", path);
            println!("   Template: {}", template);
            // TODO: 实现初始化逻辑
        }
        
        Commands::Ingest { file, project } => {
            println!("📄 Ingesting file: {}", file);
            if let Some(proj) = project {
                println!("   Project: {}", proj);
            }
            // TODO: 实现导入逻辑
        }
        
        Commands::Query { query, project, max_tokens } => {
            println!("🔍 Query: {}", query);
            if let Some(proj) = project {
                println!("   Project: {}", proj);
            }
            println!("   Max tokens: {}", max_tokens);
            // TODO: 实现查询逻辑
        }
        
        Commands::Manage(cmd) => {
            match cmd {
                ManageCommands::List { project } => {
                    println!("📋 Listing pages...");
                    if let Some(proj) = project {
                        println!("   Project: {}", proj);
                    }
                    // TODO: 实现列表逻辑
                }
                
                ManageCommands::Stats { project } => {
                    println!("📊 Wiki statistics:");
                    if let Some(proj) = project {
                        println!("   Project: {}", proj);
                    }
                    // TODO: 实现统计逻辑
                }
                
                ManageCommands::Reindex { project } => {
                    println!("🔄 Rebuilding index...");
                    if let Some(proj) = project {
                        println!("   Project: {}", proj);
                    }
                    // TODO: 实现重建索引逻辑
                }
                
                ManageCommands::Clean { project } => {
                    println!("🧹 Cleaning cache...");
                    if let Some(proj) = project {
                        println!("   Project: {}", proj);
                    }
                    // TODO: 实现清理逻辑
                }
            }
        }
    }
    
    Ok(())
}
```

---

## 🖥️ 桌面客户端 (src-desktop/)

### src-desktop/src-tauri/src/main.rs

```rust
//! OpenCode LLM Wiki - 桌面客户端
//! 
//! 职责:
//! 1. 启动嵌入式 API 服务器
//! 2. 提供 UI 界面

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

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
    let url = format!("http://localhost:19828{}", endpoint);
    
    let client = reqwest::Client::new();
    let request = match method.as_str() {
        "GET" => client.get(&url),
        "POST" => {
            let req = client.post(&url);
            if let Some(b) = body {
                req.header("Content-Type", "application/json").body(b)
            } else {
                req
            }
        }
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
            let server_path = if cfg!(debug_assertions) {
                // 开发模式：使用 cargo run
                "cargo"
            } else {
                // 生产模式：使用打包的二进制
                "llm-wiki-server"
            };
            
            let mut cmd = if cfg!(debug_assertions) {
                let mut c = Command::new(server_path);
                c.args(&["run", "--bin", "llm-wiki-server", "--", "--port", "19828"]);
                c
            } else {
                let mut c = Command::new(server_path);
                c.args(&["--port", "19828"]);
                c
            };
            
            let api_server = cmd
                .spawn()
                .expect("Failed to start API server");
            
            app.manage(ApiServer(Mutex::new(Some(api_server))));
            
            tracing::info!("✅ API server started on port 19828");
            
            Ok(())
        })
        .on_window_event(|event| {
            if let tauri::WindowEvent::Destroyed = event.event() {
                // 关闭 API 服务器
                if let Some(api_server) = event.window()
                    .state::<ApiServer>()
                    .0
                    .lock()
                    .unwrap()
                    .as_mut() 
                {
                    let _ = api_server.kill();
                    tracing::info!("🛑 API server stopped");
                }
            }
        })
        .invoke_handler(tauri::generate_handler![api_request])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

### src-desktop/ui/src/api/client.ts

```typescript
/**
 * 统一的 API 客户端
 * 
 * 在 Tauri 环境中使用 Tauri 命令
 * 在浏览器环境中直接调用 HTTP API
 */

class ApiClient {
  private baseUrl = 'http://localhost:19828';

  private async request<T>(
    endpoint: string,
    method: 'GET' | 'POST' = 'GET',
    body?: any
  ): Promise<T> {
    // 在 Tauri 环境中
    if (window.__TAURI__) {
      const response = await window.__TAURI__.invoke('api_request', {
        endpoint,
        method,
        body: body ? JSON.stringify(body) : undefined,
      });
      return JSON.parse(response);
    }
    
    // 在浏览器环境中
    const response = await fetch(`${this.baseUrl}${endpoint}`, {
      method,
      headers: body ? { 'Content-Type': 'application/json' } : {},
      body: body ? JSON.stringify(body) : undefined,
    });
    
    if (!response.ok) {
      throw new Error(`API error: ${response.statusText}`);
    }
    
    return response.json();
  }

  // ========== LLM API ==========
  
  async streamChat(config: LlmConfig, messages: ChatMessage[]) {
    return this.request('/api/llm/stream', 'POST', { config, messages });
  }

  // ========== Ingest API ==========
  
  async ingestFile(projectPath: string, filePath: string) {
    return this.request('/api/ingest', 'POST', { 
      project_path: projectPath, 
      file_path: filePath 
    });
  }

  async getIngestStatus(taskId: string) {
    return this.request(`/api/ingest/status/${taskId}`);
  }

  // ========== Query API ==========
  
  async query(query: string, options?: QueryOptions) {
    return this.request<QueryResult>('/api/query', 'POST', { 
      query, 
      ...options 
    });
  }

  // ========== Token Cache API ==========
  
  async refreshTokenCache(projectPath: string) {
    return this.request('/api/token/cache/refresh', 'POST', { 
      project_path: projectPath 
    });
  }

  async getTokenCacheStats(projectPath: string) {
    return this.request(`/api/token/cache/stats?project=${projectPath}`);
  }

  // ========== Health Check ==========
  
  async health() {
    return this.request('/health');
  }
}

export const apiClient = new ApiClient();

// ========== Types ==========

export interface LlmConfig {
  provider: string;
  model: string;
  api_key: string;
  max_tokens?: number;
}

export interface ChatMessage {
  role: 'system' | 'user' | 'assistant';
  content: string;
}

export interface QueryOptions {
  project_path?: string;
  max_tokens?: number;
  use_token_cache?: boolean;
}

export interface QueryResult {
  context: string;
  pages_used: string[];
  tokens_used: number;
  cache_hit_rate: number;
  gnn_enhanced: boolean;
}
```

---

## 📡 MCP 服务器 (src-mcp/)

### src-mcp/src/api-client.ts

```typescript
/**
 * MCP Server API 客户端
 * 调用核心 API 服务器
 */

export class CoreApiClient {
  private baseUrl: string;

  constructor(baseUrl = 'http://127.0.0.1:19828') {
    this.baseUrl = baseUrl;
  }

  async query(query: string, projectPath?: string, maxTokens = 4000) {
    const response = await fetch(`${this.baseUrl}/api/query`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        query,
        project_path: projectPath || process.env.LLM_WIKI_PROJECT,
        max_tokens: maxTokens,
        use_token_cache: true,
      }),
    });

    if (!response.ok) {
      throw new Error(`API error: ${response.statusText}`);
    }

    return response.json();
  }

  async ingestFile(filePath: string, projectPath?: string) {
    const response = await fetch(`${this.baseUrl}/api/ingest`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        file_path: filePath,
        project_path: projectPath || process.env.LLM_WIKI_PROJECT,
      }),
    });

    if (!response.ok) {
      throw new Error(`API error: ${response.statusText}`);
    }

    return response.json();
  }

  async health() {
    const response = await fetch(`${this.baseUrl}/health`);
    return response.json();
  }
}
```

---

## 🔧 构建脚本

### scripts/build-server.sh

```bash
#!/bin/bash
set -e

echo "🦀 Building server..."
cd src-server
cargo build --release --bin llm-wiki-server
cargo build --release --bin llm-wiki

echo "✅ Server built successfully!"
echo "   API Server: target/release/llm-wiki-server"
echo "   CLI: target/release/llm-wiki"
```

### scripts/build-desktop.sh

```bash
#!/bin/bash
set -e

echo "🖥️ Building desktop client..."

# 1. 构建服务端
./scripts/build-server.sh

# 2. 复制服务端二进制到桌面客户端
mkdir -p src-desktop/src-tauri/binaries
cp src-server/target/release/llm-wiki-server src-desktop/src-tauri/binaries/

# 3. 构建桌面客户端
cd src-desktop
npm install
npm run tauri build

echo "✅ Desktop client built successfully!"
```

### scripts/dev.sh

```bash
#!/bin/bash
set -e

echo "🚀 Starting development mode..."

# 启动服务端（后台）
cd src-server
cargo run --bin llm-wiki-server -- --port 19828 &
SERVER_PID=$!

# 等待服务端启动
sleep 2

# 启动桌面客户端
cd ../src-desktop
npm run tauri dev

# 清理
kill $SERVER_PID
```

---

## 📊 目录对比

| 旧结构 | 新结构 | 说明 |
|--------|--------|------|
| `src/` | `src-desktop/ui/` | 前端代码 |
| `src-tauri/` | `src-desktop/src-tauri/` | Tauri 后端 |
| `src/lib/` | `src-server/src/services/` | 核心逻辑 |
| `mcp-server/` | `src-mcp/` | MCP 服务器 |
| `extension/` | `src-extension/` | Chrome 扩展 |
| 14 个 MD 文件 | `docs/` | 文档 |

---

## 🚀 迁移步骤

### Phase 1: 创建新结构（1 天）

```bash
# 1. 创建目录
mkdir -p src-server/src/{api,services,storage,types}
mkdir -p src-desktop/{src-tauri,ui}
mkdir -p src-mcp/src
mkdir -p src-web/src
mkdir -p src-extension/src
mkdir -p docs scripts

# 2. 移动文档
mv *.md docs/  # 除了 README.md 和 LICENSE

# 3. 创建工作空间配置
# 编辑根目录 Cargo.toml
```

### Phase 2: 迁移服务端（3-5 天）

```bash
# 迁移核心逻辑
# src/lib/*.ts → src-server/src/services/*.rs
```

### Phase 3: 迁移客户端（2-3 天）

```bash
# 移动桌面客户端
mv src-tauri src-desktop/
mv src src-desktop/ui/

# 移动 MCP 服务器
mv mcp-server/* src-mcp/

# 移动 Chrome 扩展
mv extension/* src-extension/
```

---

*文档生成时间: 2026-05-02*  
*作者: Sisyphus (OpenCode AI Agent)*
