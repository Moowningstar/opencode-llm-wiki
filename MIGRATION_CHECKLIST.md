# 代码迁移清单

> 跟踪重构进度

**开始时间**: 2026-05-02  
**预计完成**: 2026-05-10

---

## 📋 整理阶段

- [x] 创建 docs 目录结构
- [x] 移动文档到 docs/
- [x] 创建文档索引
- [x] 创建新目录结构
- [ ] 创建占位文件
- [ ] 更新 .gitignore
- [ ] 创建工作空间配置
- [ ] Git 提交

---

## 🦀 核心逻辑迁移 (src/lib/ → src-server/src/services/)

### 高优先级（核心功能）

- [ ] **llm-client.ts** → `src-server/src/services/llm_client.rs`
  - [ ] 基础结构
  - [ ] OpenAI 提供商
  - [ ] Anthropic 提供商
  - [ ] Google 提供商
  - [ ] 其他提供商
  - [ ] 流式响应处理

- [ ] **llm-providers.ts** → `src-server/src/services/llm_client/providers.rs`
  - [ ] 提供商抽象
  - [ ] 请求构建
  - [ ] 响应解析

- [ ] **ingest.ts** → `src-server/src/services/ingest_engine.rs`
  - [ ] 文档读取
  - [ ] 两步分析
  - [ ] Wiki 生成
  - [ ] 队列管理

- [ ] **ingest-parse.ts** → `src-server/src/services/ingest_engine/parser.rs`
  - [ ] FILE 块解析
  - [ ] 安全路径验证
  - [ ] 错误处理

- [ ] **search.ts** → `src-server/src/services/query_optimizer.rs`
  - [ ] 分词搜索
  - [ ] 向量搜索
  - [ ] 图扩展
  - [ ] 预算控制

- [ ] **graph-relevance.ts** → `src-server/src/services/graph_engine/relevance.rs`
  - [ ] 4-Signal 模型
  - [ ] 相关性计算

- [ ] **graph-insights.ts** → `src-server/src/services/graph_engine/insights.rs`
  - [ ] 惊喜连接
  - [ ] 知识缺口
  - [ ] 桥接节点

### 中优先级（辅助功能）

- [ ] **embedding.ts** → `src-server/src/services/embedding.rs`
- [ ] **context-budget.ts** → `src-server/src/services/query_optimizer/budget.rs`
- [ ] **wiki-graph.ts** → `src-server/src/services/graph_engine/builder.rs`
- [ ] **deep-research.ts** → `src-server/src/services/research.rs`
- [ ] **enrich-wikilinks.ts** → `src-server/src/services/wikilink_enricher.rs`
- [ ] **lint.ts** → `src-server/src/services/linter.rs`
- [ ] **sweep-reviews.ts** → `src-server/src/services/reviewer.rs`

### 低优先级（工具函数）

- [ ] **path-utils.ts** → `src-server/src/types/path.rs`
- [ ] **file-types.ts** → `src-server/src/types/file.rs`
- [ ] **detect-language.ts** → `src-server/src/services/language.rs`
- [ ] **text-chunker.ts** → `src-server/src/services/chunker.rs`

---

## 🌐 API 层实现 (src-server/src/api/)

- [ ] **routes.rs** - 路由定义
- [ ] **handlers/llm.rs** - LLM API
- [ ] **handlers/ingest.rs** - 导入 API
- [ ] **handlers/query.rs** - 查询 API
- [ ] **handlers/token.rs** - Token 缓存 API
- [ ] **middleware/cors.rs** - CORS 中间件
- [ ] **middleware/logging.rs** - 日志中间件

---

## 💾 存储层实现 (src-server/src/storage/)

- [ ] **ruvector.rs** - RuVector 适配器
- [ ] **sqlite.rs** - SQLite 适配器
- [ ] **filesystem.rs** - 文件系统适配器

---

## 🖥️ 桌面客户端更新 (src/ → src-desktop/)

### 移动文件

- [ ] 移动 `src-tauri/` → `src-desktop/src-tauri/`
- [ ] 移动 `src/` → `src-desktop/ui/`

### 更新代码

- [ ] **src-desktop/src-tauri/src/main.rs** - 启动 API 服务器
- [ ] **src-desktop/ui/src/api/client.ts** - API 客户端
- [ ] 更新所有组件的 API 调用
- [ ] 更新状态管理

---

## 📡 MCP 服务器更新 (mcp-server/ → src-mcp/)

- [ ] 移动 `mcp-server/` → `src-mcp/`
- [ ] **src/api-client.ts** - 调用核心 API
- [ ] 更新所有工具的 API 调用
- [ ] 测试 MCP 集成

---

## 🔌 Chrome 扩展更新 (extension/ → src-extension/)

- [ ] 移动 `extension/` → `src-extension/`
- [ ] 更新 API 调用
- [ ] 测试扩展功能

---

## 🧪 测试

### 单元测试

- [ ] src-server 单元测试
- [ ] src-desktop 单元测试
- [ ] src-mcp 单元测试

### 集成测试

- [ ] API 端点测试
- [ ] 端到端测试

### 性能测试

- [ ] 查询性能基准
- [ ] Token 缓存性能
- [ ] 内存占用测试

---

## 📦 构建与发布

- [ ] 构建脚本
- [ ] CI/CD 配置
- [ ] 发布流程
- [ ] 文档更新

---

## 📊 进度统计

- **整理阶段**: 40% (4/10)
- **核心迁移**: 5% (1/18) - Token 缓存服务完成
- **API 实现**: 0% (0/7)
- **存储实现**: 33% (1/3) - LanceDB 适配器完成，Token 缓存集成
- **客户端更新**: 0% (0/8)
- **测试**: 0% (0/6)
- **构建发布**: 0% (0/4)

**总体进度**: 10% (6/56)

---

## 🎯 Phase 2.2 完成 (2026-05-02)

✅ **Token 缓存服务实现**
- TokenCacheService (tiktoken-rs)
- QueryOptimizer (token 预算控制)
- LanceDB schema 扩展 (token_ids, token_count)
- 性能目标: 缓存命中率 100%, Token 消耗减少 70%, 查询延迟 500ms → 150ms

---

*最后更新: 2026-05-02*
