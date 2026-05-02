# MCP 服务器架构与使用指南

> OpenCode LLM Wiki MCP Server - 让 IDE 通过 Model Context Protocol 接入知识库系统

## 📋 目录

- [架构概览](#架构概览)
- [快速开始](#快速开始)
- [MCP 工具列表](#mcp-工具列表)
- [IDE 集成配置](#ide-集成配置)
- [服务器架构](#服务器架构)
- [开发指南](#开发指南)
- [故障排查](#故障排查)

---

## 架构概览

### 技术栈

**服务端**:
- **协议**: Model Context Protocol (MCP) 1.0
- **传输**: stdio (标准输入/输出)
- **SDK**: `@modelcontextprotocol/sdk` ^1.0.4
- **数据库**: SQLite (better-sqlite3)
- **向量搜索**: LanceDB (可选)
- **文件监控**: chokidar

**客户端**:
- Tauri v2 桌面应用 (Rust + React)
- 支持任何 MCP 兼容的 IDE (Claude Desktop, Cursor, Windsurf 等)

### 系统架构

```
┌─────────────────────────────────────────────────────────────┐
│                    MCP 客户端 (IDE)                          │
│  Claude Desktop / Cursor / Windsurf / OpenCode              │
└────────────────────┬────────────────────────────────────────┘
                     │ MCP Protocol (stdio)
                     │
┌────────────────────▼────────────────────────────────────────┐
│                  MCP 服务器 (Node.js)                        │
│  ┌──────────────────────────────────────────────────────┐   │
│  │  server.js - MCP 协议处理器                          │   │
│  │  - ListTools: 暴露 10 个工具                         │   │
│  │  - CallTool: 处理工具调用                            │   │
│  └──────────────────────────────────────────────────────┘   │
│                                                              │
│  ┌──────────────────────────────────────────────────────┐   │
│  │  核心模块                                             │   │
│  │  ├─ wiki-bridge.js      - Wiki 文件系统桥接          │   │
│  │  ├─ database.js         - SQLite 数据库管理          │   │
│  │  ├─ vector-cache.js     - 向量嵌入缓存               │   │
│  │  ├─ context-manager.js  - 智能上下文注入             │   │
│  │  ├─ semantic-search.js  - 语义搜索                   │   │
│  │  ├─ graph-analyzer.js   - 知识图谱分析               │   │
│  │  ├─ indexer.js          - 全文索引                   │   │
│  │  ├─ keyword-detector.js - 关键词检测                 │   │
│  │  └─ file-watcher.js     - 文件变更监控               │   │
│  └──────────────────────────────────────────────────────┘   │
└────────────────────┬────────────────────────────────────────┘
                     │ 文件系统访问
                     │
┌────────────────────▼────────────────────────────────────────┐
│                  Wiki 项目目录                               │
│  ├─ .raw/              - 待处理文件                         │
│  ├─ .wiki/             - Wiki 页面                          │
│  │   ├─ entities/      - 实体页面                          │
│  │   ├─ concepts/      - 概念页面                          │
│  │   ├─ sources/       - 来源页面                          │
│  │   ├─ index.md       - 内容目录                          │
│  │   └─ overview.md    - 全局摘要                          │
│  ├─ .llm-wiki/         - 内部数据                           │
│  │   ├─ wiki.db        - SQLite 数据库                     │
│  │   ├─ project.json   - 项目元数据                        │
│  │   └─ graph-cache.json - 知识图谱缓存                    │
│  ├─ purpose.md         - Wiki 目标和范围                    │
│  └─ schema.md          - 结构规则                           │
└─────────────────────────────────────────────────────────────┘
```

---

## 快速开始

### 1. 安装 MCP 服务器

#### 全局安装（推荐）

```bash
cd mcp-server
npm install -g .
```

安装后，`llm-wiki` 命令全局可用。

#### 本地开发

```bash
cd mcp-server
npm install
```

### 2. 初始化 Wiki 项目

```bash
# 创建通用知识库
llm-wiki init my-wiki

# 创建研究型知识库
llm-wiki init research-notes --template research

# 创建个人成长知识库
llm-wiki init personal-kb --template personal
```

**可用模板**:
- `general` - 通用知识库（默认）
- `research` - 学术研究
- `reading` - 阅读笔记
- `personal` - 个人成长
- `business` - 商业分析

### 3. 启动 MCP 服务器

```bash
# 服务当前目录
llm-wiki serve

# 服务指定目录
llm-wiki serve ~/my-wiki
```

服务器以 stdio 模式运行，等待 MCP 客户端连接。

### 4. 添加文档

将文件拖放到 `.raw/` 目录：

```bash
cp ~/Downloads/paper.pdf my-wiki/.raw/
cp ~/Documents/notes.md my-wiki/.raw/
```

桌面应用或 MCP 服务器会自动处理这些文件。

---

## MCP 工具列表

MCP 服务器暴露 **10 个工具**，供 IDE 调用：

### 1. `wiki_read`
**读取 Wiki 页面**

```typescript
{
  path: string,        // 页面路径（相对于 .wiki/ 目录）
  project?: string,    // 项目根路径（可选）
  scope?: string       // 范围："global" | "project:name" | null
}
```

**示例**:
```json
{
  "path": "entities/gpt-4.md",
  "scope": "global"
}
```

### 2. `wiki_list`
**列出所有 Wiki 页面**

```typescript
{
  project?: string,    // 项目根路径（可选）
  scope?: string       // 范围："global" | "project:name" | "all"
}
```

**返回**: 页面路径列表

### 3. `wiki_search`
**关键词搜索**

```typescript
{
  query: string,       // 搜索查询
  project?: string,
  scope?: string
}
```

**返回**: 匹配页面列表（带相关性评分）

### 4. `wiki_query_with_context`
**智能上下文注入查询**（推荐）

```typescript
{
  query: string,           // 用户查询
  max_tokens?: number,     // 最大上下文 token 数（默认 4000）
  project?: string,
  scope?: string
}
```

**功能**:
- 关键词检测
- 向量语义搜索（可选）
- 智能上下文注入
- Token 预算管理

**返回**:
```json
{
  "context": "注入的上下文内容",
  "pages_used": ["page1.md", "page2.md"],
  "tokens_used": 3500
}
```

### 5. `wiki_get_graph`
**获取知识图谱**

```typescript
{
  project?: string,
  scope?: string
}
```

**返回**:
```json
{
  "nodes": [
    { "id": "page1.md", "type": "entity", "title": "GPT-4" }
  ],
  "edges": [
    { "source": "page1.md", "target": "page2.md", "type": "related" }
  ]
}
```

### 6. `wiki_get_index`
**获取内容目录**

读取 `.wiki/index.md`（自动维护的内容目录）

### 7. `wiki_get_overview`
**获取全局摘要**

读取 `.wiki/overview.md`（LLM 自动生成的全局摘要）

### 8. `wiki_get_purpose`
**获取 Wiki 目标和范围**

读取 `purpose.md`（Wiki 的目标、关键问题和范围）

### 9. `wiki_graph_insights`
**知识图谱洞察分析**

```typescript
{
  project?: string,
  scope?: string,
  analysis_type?: "isolated" | "surprising" | "bridges" | "stats" | "all"
}
```

**分析类型**:
- `isolated` - 孤立页面（无连接）
- `surprising` - 意外连接（跨领域链接）
- `bridges` - 桥接节点（连接不同社区）
- `stats` - 图谱统计
- `all` - 全部分析（默认）

### 10. `wiki_deep_research`
**深度研究**（图遍历 + 语义搜索 + 多跳推理）

```typescript
{
  query: string,           // 研究查询或主题
  max_depth?: number,      // 最大图遍历深度（默认 3）
  max_results?: number,    // 最大页面数（默认 10）
  project?: string,
  scope?: string
}
```

**工作流程**:
1. 关键词搜索 + 语义搜索找到种子页面
2. 从种子页面开始图遍历（BFS）
3. 收集相关页面内容
4. 返回结构化研究结果

**返回**:
```json
{
  "query": "transformer architecture",
  "pages": [
    { "path": "concepts/attention.md", "content": "...", "depth": 0 },
    { "path": "entities/bert.md", "content": "...", "depth": 1 }
  ],
  "total_pages": 10,
  "max_depth_reached": 2
}
```

---

## IDE 集成配置

### Claude Desktop

编辑 `~/Library/Application Support/Claude/claude_desktop_config.json` (macOS) 或 `%APPDATA%\Claude\claude_desktop_config.json` (Windows):

```json
{
  "mcpServers": {
    "llm-wiki": {
      "command": "llm-wiki",
      "args": ["serve", "/path/to/your/wiki"],
      "env": {
        "LLM_WIKI_PROJECT": "/path/to/your/wiki"
      }
    }
  }
}
```

### Cursor / Windsurf

在项目根目录创建 `.mcp.json`:

```json
{
  "mcpServers": {
    "llm-wiki": {
      "command": "llm-wiki",
      "args": ["serve", "."],
      "env": {
        "LLM_WIKI_PROJECT": "${workspaceFolder}"
      }
    }
  }
}
```

### OpenCode

项目已包含 `.mcp.json`:

```json
{
  "mcpServers": {
    "llm-wiki": {
      "command": "node",
      "args": ["mcp-server/server.js"],
      "env": {
        "LLM_WIKI_PROJECT": "${HOME}/.opencode-wiki"
      }
    }
  }
}
```

---

## 服务器架构

### 核心模块详解

#### 1. `wiki-bridge.js` - Wiki 文件系统桥接

**职责**:
- 读取/写入 Wiki 页面
- 列出页面
- 搜索页面
- 获取知识图谱数据

**关键方法**:
```javascript
class WikiBridge {
  async readPage(path, scope)
  async listPages(scope)
  async searchPages(query, scope)
  async getGraphData(scope)
  async getIndex(scope)
  async getOverview(scope)
  async getPurpose()
}
```

#### 2. `database.js` - SQLite 数据库管理

**表结构**:
```sql
CREATE TABLE pages (
  id TEXT PRIMARY KEY,
  path TEXT UNIQUE,
  title TEXT,
  type TEXT,
  content TEXT,
  embedding BLOB,
  created_at TEXT,
  updated_at TEXT
);

CREATE TABLE links (
  source TEXT,
  target TEXT,
  type TEXT,
  PRIMARY KEY (source, target)
);

CREATE TABLE embeddings (
  page_id TEXT PRIMARY KEY,
  embedding BLOB,
  model TEXT,
  created_at TEXT
);
```

#### 3. `vector-cache.js` - 向量嵌入缓存

**功能**:
- 缓存页面向量嵌入
- 避免重复计算
- 支持增量更新

**方法**:
```javascript
class VectorCache {
  async get(pageId)
  async set(pageId, embedding, model)
  async delete(pageId)
  async clear()
}
```

#### 4. `context-manager.js` - 智能上下文注入

**策略**:
1. **关键词检测**: 从查询中提取关键词
2. **关键词搜索**: 基于关键词找到相关页面
3. **向量搜索**: 基于语义相似度找到相关页面（可选）
4. **相关性排序**: 4 信号相关性模型
5. **Token 预算管理**: 在 token 限制内注入最相关内容

**4 信号相关性模型**:
- 关键词匹配度
- 向量相似度
- 图结构中心性
- 页面类型权重

#### 5. `semantic-search.js` - 语义搜索

**功能**:
- 向量相似度搜索
- 余弦相似度计算
- Top-K 结果返回

**依赖**: LanceDB（可选，如果未安装则回退到关键词搜索）

#### 6. `graph-analyzer.js` - 知识图谱分析

**分析类型**:
- **孤立页面**: 无入链和出链的页面
- **意外连接**: 跨领域的链接（基于社区检测）
- **桥接节点**: 连接不同社区的关键页面
- **图谱统计**: 节点数、边数、平均度、连通分量

**算法**:
- Louvain 社区检测
- PageRank 中心性
- 连通分量分析

#### 7. `indexer.js` - 全文索引

**功能**:
- 倒排索引构建
- TF-IDF 计算
- BM25 排序

#### 8. `keyword-detector.js` - 关键词检测

**策略**:
- 停用词过滤
- 词频统计
- 实体识别（基于 Wiki 页面标题）

#### 9. `file-watcher.js` - 文件变更监控

**功能**:
- 监控 `.raw/` 目录
- 监控 `.wiki/` 目录
- 触发增量索引更新

**使用 chokidar**:
```javascript
const watcher = chokidar.watch('.raw/', {
  ignored: /(^|[\/\\])\../,
  persistent: true
});

watcher.on('add', path => {
  // 触发文档导入
});
```

---

## 开发指南

### 本地开发

```bash
cd mcp-server
npm install

# 运行服务器
node server.js

# 运行 CLI
node cli.js --help
```

### 调试模式

```bash
DEBUG=* llm-wiki serve my-wiki
```

### 添加新工具

1. **在 `server.js` 中注册工具**:

```javascript
server.setRequestHandler(ListToolsRequestSchema, async () => {
  return {
    tools: [
      // ... 现有工具
      {
        name: 'wiki_new_tool',
        description: '新工具描述',
        inputSchema: {
          type: 'object',
          properties: {
            param1: { type: 'string', description: '参数 1' },
          },
          required: ['param1'],
        },
      },
    ],
  };
});
```

2. **实现工具处理器**:

```javascript
server.setRequestHandler(CallToolRequestSchema, async (request) => {
  const { name, arguments: args } = request.params;

  switch (name) {
    case 'wiki_new_tool': {
      const result = await handleNewTool(args);
      return {
        content: [{ type: 'text', text: JSON.stringify({ success: true, result }) }],
      };
    }
    // ... 其他工具
  }
});
```

3. **在 `wiki-bridge.js` 中添加业务逻辑**:

```javascript
class WikiBridge {
  async handleNewTool(args) {
    // 实现业务逻辑
  }
}
```

### 测试

```bash
cd mcp-server
npm test
```

---

## 故障排查

### 问题 1: 服务器无法启动

**症状**: `llm-wiki serve` 命令失败

**解决方案**:
1. 检查 Node.js 版本（需要 >= 18）
2. 重新安装依赖: `npm install`
3. 检查项目目录是否存在
4. 查看错误日志

### 问题 2: IDE 无法连接到 MCP 服务器

**症状**: IDE 显示 "MCP server not found"

**解决方案**:
1. 确认 `llm-wiki` 命令全局可用: `which llm-wiki`
2. 检查 `.mcp.json` 配置路径是否正确
3. 重启 IDE
4. 查看 IDE 的 MCP 日志

### 问题 3: 向量搜索不工作

**症状**: `wiki_query_with_context` 只返回关键词搜索结果

**解决方案**:
1. 检查 LanceDB 是否安装: `npm list lancedb`
2. 检查嵌入生成是否成功（需要 OpenAI API 密钥或本地嵌入模型）
3. 查看服务器日志中的警告信息

### 问题 4: 知识图谱为空

**症状**: `wiki_get_graph` 返回空图

**解决方案**:
1. 确认 `.wiki/` 目录中有页面
2. 检查页面是否包含 `[[wikilink]]` 格式的链接
3. 运行 `llm-wiki scan` 触发重新索引

### 问题 5: 文件监控不工作

**症状**: 添加文件到 `.raw/` 后没有自动处理

**解决方案**:
1. 确认桌面应用或 MCP 服务器正在运行
2. 检查文件权限
3. 查看 `file-watcher.js` 日志

---

## 性能优化

### 1. 向量嵌入缓存

向量嵌入计算成本高，使用缓存避免重复计算：

```javascript
// 检查缓存
const cached = await vectorCache.get(pageId);
if (cached) {
  return cached;
}

// 计算并缓存
const embedding = await generateEmbedding(content);
await vectorCache.set(pageId, embedding, 'text-embedding-ada-002');
```

### 2. 增量索引

只索引变更的文件：

```javascript
const currentHash = sha256(content);
const cachedHash = await getIngestCache(filePath);

if (currentHash === cachedHash) {
  console.log('File unchanged, skipping');
  return;
}

await ingestFile(filePath);
await setIngestCache(filePath, currentHash);
```

### 3. Token 预算管理

智能选择最相关内容，避免超出 token 限制：

```javascript
const maxTokens = 4000;
let usedTokens = 0;
const selectedPages = [];

for (const page of rankedPages) {
  const pageTokens = estimateTokens(page.content);
  if (usedTokens + pageTokens > maxTokens) break;
  
  selectedPages.push(page);
  usedTokens += pageTokens;
}
```

### 4. 图遍历剪枝

限制遍历深度和广度，避免爆炸性增长：

```javascript
const maxDepth = 3;
const maxBranching = 3;

const outgoingLinks = graph.edges
  .filter(e => e.source === pagePath)
  .map(e => e.target)
  .slice(0, maxBranching); // 只遍历前 3 个链接
```

---

## 扩展阅读

- [Model Context Protocol 规范](https://modelcontextprotocol.io)
- [MCP SDK 文档](https://github.com/modelcontextprotocol/sdk)
- [LanceDB 文档](https://lancedb.github.io/lancedb/)
- [Chokidar 文档](https://github.com/paulmillr/chokidar)

---

## 许可证

GPL-3.0

---

*本文档由 GitNexus 知识图谱辅助生成*
