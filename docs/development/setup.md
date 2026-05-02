# 🚀 OpenCode 集成配置指南

> 让 OpenCode IDE 通过 MCP 协议访问 LLM Wiki 的完整配置步骤

## 📋 前置条件

- ✅ Node.js 20+ 已安装
- ✅ Rust 工具链已安装
- ✅ 项目已克隆到本地

---

## 🎯 架构说明

```
OpenCode IDE
    ↓ MCP Protocol (stdio)
MCP Server (Node.js)
    ↓ HTTP API
Rust API Server (独立进程, 端口 19828)
    ↓ 核心服务
LLM Wiki 核心功能
```

**重要**：MCP Server 依赖 Rust API Server，两者必须同时运行！

---

## 🛠️ 步骤 1: 构建 Rust API 服务器

```bash
cd C:\Users\Moow\Projects\opencode-llm-wiki\src-tauri

# 构建 API 服务器
cargo build --release --bin llm-wiki-api-server
```

**构建产物位置**：
```
src-tauri\target\release\llm-wiki-api-server.exe
```

---

## 🚀 步骤 2: 启动 Rust API 服务器

### 方式 1: 直接运行（推荐）

```powershell
# 在项目根目录
.\src-tauri\target\release\llm-wiki-api-server.exe
```

### 方式 2: 使用 Cargo

```bash
cd src-tauri
cargo run --release --bin llm-wiki-api-server
```

### 验证服务器运行

打开新终端：

```bash
curl http://127.0.0.1:19828/health
```

**预期输出**：
```json
{"status":"ok","version":"0.0.1"}
```

如果看到这个输出，说明 API 服务器正常运行！

---

## 📦 步骤 3: 安装 MCP Server

```bash
cd C:\Users\Moow\Projects\opencode-llm-wiki\mcp-server

# 安装依赖
npm install

# 全局安装 CLI
npm install -g .
```

### 验证安装

```bash
llm-wiki --version
```

应该显示版本号 `0.0.1`。

---

## ⚙️ 步骤 4: 配置 OpenCode

### 方式 1: 项目级配置（推荐）

在项目根目录创建或编辑 `.mcp.json`：

```json
{
  "mcpServers": {
    "llm-wiki": {
      "command": "llm-wiki",
      "args": ["serve", "."],
      "env": {
        "LLM_WIKI_PROJECT": "${workspaceFolder}",
        "LLM_WIKI_API_URL": "http://127.0.0.1:19828"
      }
    }
  }
}
```

### 方式 2: 全局配置

编辑 OpenCode 配置文件：

**Windows**:
```
%APPDATA%\OpenCode\opencode_config.json
```

**macOS**:
```
~/Library/Application Support/OpenCode/opencode_config.json
```

**Linux**:
```
~/.config/opencode/opencode_config.json
```

添加配置：

```json
{
  "mcpServers": {
    "llm-wiki": {
      "command": "llm-wiki",
      "args": ["serve", "C:\\Users\\Moow\\Projects\\opencode-llm-wiki"],
      "env": {
        "LLM_WIKI_PROJECT": "C:\\Users\\Moow\\Projects\\opencode-llm-wiki",
        "LLM_WIKI_API_URL": "http://127.0.0.1:19828"
      }
    }
  }
}
```

**注意**：Windows 路径需要使用双反斜杠 `\\` 或正斜杠 `/`。

---

## 🎮 步骤 5: 启动完整系统

### 终端 1: 启动 Rust API 服务器

```bash
cd C:\Users\Moow\Projects\opencode-llm-wiki
.\src-tauri\target\release\llm-wiki-api-server.exe
```

**保持此终端运行！**

### 终端 2: 验证 API（可选）

```bash
curl http://127.0.0.1:19828/health
```

### 终端 3: 启动 OpenCode

```bash
# 打开项目
code C:\Users\Moow\Projects\opencode-llm-wiki
```

OpenCode 会自动读取 `.mcp.json` 并启动 MCP Server。

---

## ✅ 验证 MCP 连接

在 OpenCode 中：

1. 打开命令面板（Ctrl+Shift+P / Cmd+Shift+P）
2. 输入 "MCP"
3. 应该看到 `llm-wiki` 服务器已连接
4. 可用工具列表应该包含：
   - `wiki_read`
   - `wiki_list`
   - `wiki_search`
   - `wiki_query_with_context`
   - `wiki_get_graph`
   - `wiki_get_index`
   - `wiki_get_overview`
   - `wiki_get_purpose`
   - `wiki_graph_insights`
   - `wiki_deep_research`

---

## 🧪 测试 MCP 工具

在 OpenCode 聊天中尝试：

```
User: List all wiki pages
AI: [调用 wiki_list 工具]

User: Search for "transformer"
AI: [调用 wiki_search 工具]

User: Show me the knowledge graph
AI: [调用 wiki_get_graph 工具]
```

---

## 🐛 故障排查

### 问题 1: MCP Server 无法连接

**症状**：OpenCode 显示 "MCP server connection failed"

**原因**：Rust API 服务器未运行

**解决方案**：
1. 检查 API 服务器是否运行：`curl http://127.0.0.1:19828/health`
2. 如果没有响应，启动 API 服务器：`.\src-tauri\target\release\llm-wiki-api-server.exe`

### 问题 2: `llm-wiki` 命令未找到

**症状**：OpenCode 报错 "command not found: llm-wiki"

**解决方案**：
```bash
cd mcp-server
npm install -g .

# 验证
llm-wiki --version
```

### 问题 3: 端口 19828 被占用

**症状**：API 服务器启动失败，提示 "Address already in use"

**解决方案**：

**Windows**:
```powershell
# 查找占用端口的进程
netstat -ano | findstr :19828

# 杀死进程（替换 <PID>）
taskkill /PID <PID> /F
```

**macOS/Linux**:
```bash
# 查找并杀死进程
lsof -ti:19828 | xargs kill -9
```

或者使用自定义端口：
```bash
API_PORT=8080 .\src-tauri\target\release\llm-wiki-api-server.exe
```

然后更新 `.mcp.json` 中的 `LLM_WIKI_API_URL`。

### 问题 4: MCP Server 日志在哪里？

**查看 MCP Server 日志**：

OpenCode 通常会将 MCP Server 的 stderr 输出到开发者工具：

1. 打开 OpenCode 开发者工具（Help → Toggle Developer Tools）
2. 查看 Console 标签
3. 搜索 "llm-wiki" 或 "MCP"

### 问题 5: API 服务器返回 404

**症状**：MCP 工具调用失败，API 返回 404

**原因**：API 端点路径不匹配

**解决方案**：
检查 `mcp-server/lib/core-api-client.js` 中的端点路径是否与 Rust API 服务器一致。

---

## 📊 系统状态检查清单

运行此清单确保所有组件正常：

```bash
# 1. 检查 Rust API 服务器
curl http://127.0.0.1:19828/health
# ✅ 预期: {"status":"ok","version":"0.0.1"}

# 2. 检查 llm-wiki CLI
llm-wiki --version
# ✅ 预期: 0.0.1

# 3. 检查 .mcp.json 存在
ls .mcp.json
# ✅ 预期: 文件存在

# 4. 检查 OpenCode MCP 配置
# 打开 OpenCode → 命令面板 → "MCP: Show Servers"
# ✅ 预期: llm-wiki 显示为 "Connected"
```

---

## 🎯 完整启动脚本

### Windows (PowerShell)

创建 `start-llm-wiki.ps1`：

```powershell
# 启动 Rust API 服务器
Start-Process powershell -ArgumentList "-NoExit", "-Command", "cd '$PSScriptRoot\src-tauri'; cargo run --release --bin llm-wiki-api-server"

# 等待 API 服务器启动
Start-Sleep -Seconds 5

# 验证 API 服务器
$response = Invoke-WebRequest -Uri "http://127.0.0.1:19828/health" -UseBasicParsing
Write-Host "API Server Status: $($response.Content)"

# 启动 OpenCode
code .
```

运行：
```powershell
.\start-llm-wiki.ps1
```

### macOS/Linux (Bash)

创建 `start-llm-wiki.sh`：

```bash
#!/bin/bash

# 启动 Rust API 服务器（后台）
cd src-tauri
cargo run --release --bin llm-wiki-api-server &
API_PID=$!

# 等待 API 服务器启动
sleep 5

# 验证 API 服务器
curl http://127.0.0.1:19828/health

# 启动 OpenCode
code .

# 保存 PID 以便后续关闭
echo $API_PID > .api-server.pid
echo "API Server PID: $API_PID"
```

运行：
```bash
chmod +x start-llm-wiki.sh
./start-llm-wiki.sh
```

停止：
```bash
kill $(cat .api-server.pid)
```

---

## 📚 相关文档

- **[QUICKSTART_API.md](./QUICKSTART_API.md)** - API 服务器快速开始
- **[RUST_API_SERVER.md](./RUST_API_SERVER.md)** - Rust API 完整文档
- **[MCP_SERVER_GUIDE.md](./MCP_SERVER_GUIDE.md)** - MCP 服务器指南
- **[mcp-server/README.md](./mcp-server/README.md)** - MCP CLI 使用说明

---

## 🎉 成功标志

当一切正常时，你应该看到：

1. ✅ Rust API 服务器运行在 `http://127.0.0.1:19828`
2. ✅ `curl http://127.0.0.1:19828/health` 返回 `{"status":"ok"}`
3. ✅ OpenCode 显示 `llm-wiki` MCP 服务器已连接
4. ✅ 可以在 OpenCode 中调用 `wiki_*` 工具
5. ✅ MCP 工具返回正确的数据

---

## 💡 提示

### 开发模式

如果你在开发 API 服务器，使用 `cargo watch` 自动重新编译：

```bash
# 安装 cargo-watch
cargo install cargo-watch

# 自动重新编译并运行
cd src-tauri
cargo watch -x 'run --bin llm-wiki-api-server'
```

### 生产部署

对于生产环境，建议：

1. 使用 systemd (Linux) 或 Windows Service 管理 API 服务器
2. 配置日志轮转
3. 设置健康检查和自动重启
4. 使用反向代理（nginx/caddy）处理 HTTPS

---

*本指南由 Sisyphus 生成 - 2026-04-27*
