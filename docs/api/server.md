# Rust HTTP API 服务器架构文档

> OpenCode LLM Wiki - 独立的 Rust HTTP API 服务，支持 MCP 和桌面应用共同调用

## 📋 目录

- [架构概览](#架构概览)
- [快速开始](#快速开始)
- [API 端点](#api-端点)
- [MCP 集成](#mcp-集成)
- [开发指南](#开发指南)

---

## 架构概览

### 新架构设计

```
┌─────────────────────────────────────────────────────────┐
│  IDE (Claude Desktop/Cursor/Windsurf)                   │
│  通过 MCP 可以：                                         │
│  - 读取 Wiki 内容                                        │
│  - 触发文档导入                                          │
│  - 调用 LLM 分析                                         │
│  - 配置 LLM 提供商                                       │
└────────────────┬────────────────────────────────────────┘
                 │ MCP Protocol (stdio)
                 ↓
┌─────────────────────────────────────────────────────────┐
│  MCP Server (Node.js)                                   │
│  - 暴露 MCP 工具                                         │
│  - 调用 Rust HTTP API                                   │
└────────────────┬────────────────────────────────────────┘
                 │ HTTP (127.0.0.1:19828)
                 ↓
┌─────────────────────────────────────────────────────────┐
│  Rust HTTP API Server (独立进程)                        │
│  ┌──────────────────────────────────────────────────┐   │
│  │  API 端点                                         │   │
│  │  - GET  /health                                  │   │
│  │  - POST /api/llm/stream                          │   │
│  │  - POST /api/ingest                              │   │
│  │  - POST /api/config/get                          │   │
│  │  - POST /api/config/save                         │   │
│  └──────────────────────────────────────────────────┘   │
│                                                          │
│  ┌──────────────────────────────────────────────────┐   │
│  │  核心服务模块                                     │   │
│  │  - LLM 客户端 (多提供商支持)                     │   │
│  │  - 文档导入引擎                                   │   │
│  │  - Wiki 生成引擎                                  │   │
│  │  - 知识图谱构建                                   │   │
│  │  - 配置管理                                       │   │
│  └──────────────────────────────────────────────────┘   │
└────────────────┬────────────────────────────────────────┘
                 │ 共享调用
                 ↓
┌─────────────────────────────────────────────────────────┐
│  Tauri 桌面应用 (可选的 UI 前端)                         │
│  - 可视化界面                                            │
│  - 调用 Rust HTTP API                                   │
└─────────────────────────────────────────────────────────┘
```

### 技术栈

**API 服务器**:
- **框架**: Axum 0.7 (高性能异步 Web 框架)
- **运行时**: Tokio (异步运行时)
- **序列化**: Serde + serde_json
- **CORS**: tower-http
- **日志**: tracing + tracing-subscriber
- **HTTP 客户端**: reqwest (用于调用外部 LLM API)

**端口**: 默认 `19828`（可通过环境变量 `API_PORT` 配置）

---

## 快速开始

### 1. 构建 API 服务器

```bash
cd src-tauri

# 开发模式
cargo build

# 生产模式
cargo build --release --bin llm-wiki-api-server
```

### 2. 启动 API 服务器

```bash
# 使用默认端口 19828
cargo run --bin llm-wiki-api-server

# 自定义端口
API_PORT=8080 cargo run --bin llm-wiki-api-server

# 生产模式
./target/release/llm-wiki-api-server
```

### 3. 验证服务器运行

```bash
curl http://127.0.0.1:19828/health
```

**预期响应**:
```json
{
  "status": "ok",
  "version": "0.0.1"
}
```

---

## API 端点

### 1. 健康检查

**端点**: `GET /health`

**响应**:
```json
{
  "status": "ok",
  "version": "0.0.1"
}
```

---

### 2. LLM 流式聊天

**端点**: `POST /api/llm/stream`

**请求体**:
```json
{
  "config": {
    "provider": "openai",
    "model": "gpt-4",
    "api_key": "sk-...",
    "base_url": null,
    "max_tokens": 4096,
    "temperature": 0.7
  },
  "messages": [
    {
      "role": "system",
      "content": "You are a helpful assistant."
    },
    {
      "role": "user",
      "content": "Hello!"
    }
  ],
  "stream": true
}
```

**响应**: Server-Sent Events (SSE) 流

```
data: {"token":"Hello","done":false,"error":null}

data: {"token":" there","done":false,"error":null}

data: {"token":"!","done":false,"error":null}

data: {"token":null,"done":true,"error":null}
```

**支持的提供商**:
- `openai` - OpenAI (GPT-4, GPT-3.5)
- `anthropic` - Anthropic (Claude)
- `google` - Google (Gemini)
- `ollama` - Ollama (本地模型)
- `deepseek` - DeepSeek
- `groq` - Groq
- `custom` - 自定义 OpenAI 兼容端点

---

### 3. 文档导入

**端点**: `POST /api/ingest`

**请求体**:
```json
{
  "project_path": "/path/to/wiki-project",
  "file_path": "/path/to/document.pdf",
  "config": {
    "provider": "openai",
    "model": "gpt-4",
    "api_key": "sk-..."
  }
}
```

**响应**:
```json
{
  "success": true,
  "pages_created": [
    ".wiki/entities/example-entity.md",
    ".wiki/concepts/key-concept.md",
    ".wiki/sources/document-summary.md"
  ],
  "error": null
}
```

**工作流程**:
1. 读取文档内容（支持 PDF, DOCX, TXT, MD 等）
2. 使用 LLM 分析文档（两阶段 Chain-of-Thought）
3. 生成 Wiki 页面（实体、概念、来源）
4. 更新知识图谱
5. 返回创建的页面列表

---

### 4. 获取 LLM 配置

**端点**: `POST /api/config/get`

**请求体**:
```json
{
  "project_path": "/path/to/wiki-project"
}
```

**响应**:
```json
{
  "config": {
    "provider": "openai",
    "model": "gpt-4",
    "api_key": null,
    "base_url": null,
    "max_tokens": 4096,
    "temperature": 0.7
  },
  "error": null
}
```

**配置来源优先级**:
1. 项目配置文件 (`.llm-wiki/llm-config.json`)
2. 全局配置
3. 默认配置

---

### 5. 保存 LLM 配置

**端点**: `POST /api/config/save`

**请求体**:
```json
{
  "project_path": "/path/to/wiki-project",
  "config": {
    "provider": "openai",
    "model": "gpt-4",
    "api_key": "sk-...",
    "base_url": null,
    "max_tokens": 4096,
    "temperature": 0.7
  }
}
```

**响应**:
```json
{
  "success": true,
  "error": null
}
```

**保存位置**: `{project_path}/.llm-wiki/llm-config.json`

---

## MCP 集成

### 更新 MCP Server

MCP Server 现在通过 HTTP 调用 Rust API：

```javascript
// mcp-server/lib/core-api-client.js
import { CoreApiClient } from './core-api-client.js';

const apiClient = new CoreApiClient('http://127.0.0.1:19828');

// 调用 LLM
const stream = await apiClient.streamChat(config, messages);

// 导入文档
const result = await apiClient.ingestFile(projectPath, filePath, config);

// 获取配置
const config = await apiClient.getConfig(projectPath);

// 保存配置
await apiClient.saveConfig(projectPath, config);
```

### 新增 MCP 工具

在 `mcp-server/server.js` 中添加新工具：

```javascript
{
  name: 'wiki_llm_chat',
  description: 'Chat with LLM using project configuration',
  inputSchema: {
    type: 'object',
    properties: {
      messages: { type: 'array', description: 'Chat messages' },
      project: { type: 'string', description: 'Project path' },
    },
    required: ['messages'],
  },
}

{
  name: 'wiki_ingest_file',
  description: 'Ingest a document into the wiki',
  inputSchema: {
    type: 'object',
    properties: {
      file_path: { type: 'string', description: 'Path to document' },
      project: { type: 'string', description: 'Project path' },
    },
    required: ['file_path'],
  },
}

{
  name: 'wiki_configure_llm',
  description: 'Configure LLM provider and model',
  inputSchema: {
    type: 'object',
    properties: {
      provider: { type: 'string', description: 'LLM provider' },
      model: { type: 'string', description: 'Model name' },
      api_key: { type: 'string', description: 'API key' },
      project: { type: 'string', description: 'Project path' },
    },
    required: ['provider', 'model'],
  },
}
```

---

## 开发指南

### 项目结构

```
src-tauri/
├── src/
│   ├── api/
│   │   ├── mod.rs           # API 模块入口
│   │   ├── server.rs        # Axum 服务器启动
│   │   ├── routes.rs        # 路由定义
│   │   ├── handlers.rs      # 请求处理器
│   │   └── types.rs         # API 类型定义
│   ├── bin/
│   │   └── api-server.rs    # API 服务器二进制入口
│   ├── commands/            # Tauri 命令
│   ├── lib.rs               # 库入口
│   └── main.rs              # Tauri 应用入口
└── Cargo.toml
```

### 添加新的 API 端点

#### 1. 在 `types.rs` 中定义类型

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct NewFeatureRequest {
    pub param1: String,
    pub param2: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NewFeatureResponse {
    pub result: String,
    pub error: Option<String>,
}
```

#### 2. 在 `handlers.rs` 中实现处理器

```rust
pub async fn new_feature_handler(
    Json(payload): Json<NewFeatureRequest>,
) -> Result<Json<NewFeatureResponse>, (StatusCode, Json<ErrorResponse>)> {
    // 实现业务逻辑
    Ok(Json(NewFeatureResponse {
        result: "success".to_string(),
        error: None,
    }))
}
```

#### 3. 在 `routes.rs` 中注册路由

```rust
pub fn create_router() -> Router {
    Router::new()
        // ... 现有路由
        .route("/api/new-feature", post(handlers::new_feature_handler))
}
```

### 测试 API

```bash
# 健康检查
curl http://127.0.0.1:19828/health

# LLM 流式聊天
curl -X POST http://127.0.0.1:19828/api/llm/stream \
  -H "Content-Type: application/json" \
  -d '{
    "config": {
      "provider": "openai",
      "model": "gpt-4",
      "api_key": "sk-..."
    },
    "messages": [
      {"role": "user", "content": "Hello!"}
    ]
  }'

# 文档导入
curl -X POST http://127.0.0.1:19828/api/ingest \
  -H "Content-Type: application/json" \
  -d '{
    "project_path": "/path/to/project",
    "file_path": "/path/to/document.pdf",
    "config": {
      "provider": "openai",
      "model": "gpt-4",
      "api_key": "sk-..."
    }
  }'
```

### 日志配置

```bash
# 设置日志级别
RUST_LOG=debug cargo run --bin llm-wiki-api-server

# 只显示 API 相关日志
RUST_LOG=llm_wiki_lib::api=debug cargo run --bin llm-wiki-api-server
```

---

## 部署

### 开发环境

```bash
# 启动 API 服务器
cargo run --bin llm-wiki-api-server

# 启动 MCP 服务器
cd mcp-server
npm install
llm-wiki serve
```

### 生产环境

```bash
# 构建 API 服务器
cargo build --release --bin llm-wiki-api-server

# 运行
./target/release/llm-wiki-api-server

# 或使用 systemd (Linux)
sudo systemctl start llm-wiki-api
```

### Docker 部署

```dockerfile
FROM rust:1.75 as builder
WORKDIR /app
COPY src-tauri/ .
RUN cargo build --release --bin llm-wiki-api-server

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/llm-wiki-api-server /usr/local/bin/
EXPOSE 19828
CMD ["llm-wiki-api-server"]
```

---

## 安全考虑

### 1. API 密钥保护

- API 密钥不应硬编码在代码中
- 使用环境变量或配置文件
- 配置文件应添加到 `.gitignore`

### 2. CORS 配置

当前配置允许所有来源（开发模式）：

```rust
let cors = CorsLayer::new()
    .allow_origin(Any)
    .allow_methods(Any)
    .allow_headers(Any);
```

生产环境应限制来源：

```rust
let cors = CorsLayer::new()
    .allow_origin("http://localhost:1420".parse::<HeaderValue>().unwrap())
    .allow_methods([Method::GET, Method::POST])
    .allow_headers([CONTENT_TYPE]);
```

### 3. 路径安全

文档导入时验证路径，防止路径遍历攻击：

```rust
fn is_safe_path(path: &str) -> bool {
    !path.contains("..") && !path.starts_with("/")
}
```

---

## 故障排查

### 问题 1: 端口已被占用

**错误**: `Address already in use (os error 48)`

**解决方案**:
```bash
# 查找占用端口的进程
lsof -i :19828

# 杀死进程
kill -9 <PID>

# 或使用其他端口
API_PORT=8080 cargo run --bin llm-wiki-api-server
```

### 问题 2: MCP Server 无法连接到 API

**错误**: `ECONNREFUSED 127.0.0.1:19828`

**解决方案**:
1. 确认 API 服务器正在运行
2. 检查端口配置是否一致
3. 检查防火墙设置

### 问题 3: LLM API 调用失败

**错误**: `API error: Unauthorized`

**解决方案**:
1. 检查 API 密钥是否正确
2. 检查提供商配置
3. 查看 API 服务器日志

---

## 性能优化

### 1. 连接池

使用 reqwest 的连接池复用 HTTP 连接：

```rust
use reqwest::Client;

lazy_static! {
    static ref HTTP_CLIENT: Client = Client::builder()
        .pool_max_idle_per_host(10)
        .build()
        .unwrap();
}
```

### 2. 异步处理

所有 I/O 操作使用异步：

```rust
#[tokio::main]
async fn main() {
    // 异步处理
}
```

### 3. 流式响应

使用 SSE 流式返回 LLM 响应，减少延迟。

---

## 许可证

GPL-3.0

---

*本文档由 Sisyphus 生成*
