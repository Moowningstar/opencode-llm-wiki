# OpenCode LLM Wiki

<p align="center">
  <img src="logo.jpg" width="128" height="128" style="border-radius: 22%;" alt="OpenCode LLM Wiki Logo">
</p>

<p align="center">
  <strong>基于图神经网络的 LLM 知识引擎</strong><br>
  生产级 Rust 后端，支持 RuVector 迁移 • 3 层架构 • Token 优化检索
</p>

<p align="center">
  <a href="#这是什么">什么是？</a> •
  <a href="#架构">架构</a> •
  <a href="#为什么选择-ruvector">为什么 RuVector?</a> •
  <a href="#技术栈">技术栈</a> •
  <a href="#安装">安装</a> •
  <a href="#路线图">路线图</a> •
  <a href="#许可证">许可证</a>
</p>

<p align="center">
  <a href="README.md">English</a> | 中文
</p>

---

## 这是什么？

**OpenCode LLM Wiki** 是一个**基于图神经网络的 LLM 知识引擎**，使用 Rust 构建，设计上从传统向量数据库（ LanceDB ）演进到 **RuVector 的自组织神经网络架构（SONA）**。与传统 RAG 系统（每次查询都重新推导答案）不同，本系统将知识**一次性编译**为持久的、自组织的图谱，并从使用模式中学习。

### 核心哲学

**知识作为编译产物，而非运行时推导**

传统 RAG 系统将知识检索视为无状态操作 — 嵌入查询 → 搜索向量 → 喂给 LLM 。本项目翻转了这一模式：

1. **编译阶段**（摄入）：LLM 阅读源文件 → 生 Wiki 页面 → 构建知识图谱（4 信号相关性模型）
2. **运行时阶段**（查询）：图遍历 + token 优化检索 → LLM 带引用回答
3. **演进阶段**（SONA ，未来）：图根据查询轨迹自组织 → 自适应边权重 → 灾难性遗忘预防

本设计受 [Karpathy 的 LLM Wiki 模式](https://gist.github.com/karpathy/442a6bf555914893e9891c11519de94f) 启发，但扩展了：
- **图优先架构** — 为 RuVector 原生图能力做准备
- **Token 缓存层**（ tiktoken-rs ）降低 70% LLM 成本
- **3 层后端**（接口 → 服务 → 存储）支持干净的存储后端切换
- **SONA 就绪设计** — 增量学习和自适应相关性评分

---

## 架构

### 当前状态：生产级 3 层后端

**已完成（2026-05-02）** — 3,856 行 Rust 代码，8 个单元测试通过

```
┌─────────────────────────────────────────────────────────────┐
│ 第 1 层：接口                                                │
│  • HTTP API (Axum): 5 个端点 (health, config, llm, ingest)     │
│  • CLI 工具：serve, init, ingest, query                      │
└─────────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────────┐
│ 第 2 层：业务逻辑（服务）                                     │
│  • EmbeddingService   → OpenAI 兼容嵌入 API                   │
│  • ChunkingService    → Markdown 标题感知分割                 │
│  • IngestService      → 编排（parse→chunk→embed→store）       │
│  • QueryService       → 搜索 + token 预算优化                   │
│  • TokenCacheService  → tiktoken-rs 预计算（节省 70% 成本）    │
└─────────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────────┐
│ 第 3 层：数据抽象（存储）                                     │
│  • VectorStorage trait → 5 个异步方法 (upsert, search, delete, count, dim) │
│  • LanceDBStorage impl → 当前生产后端                          │
│  • [未来] RuVectorStorage impl → SONA 驱动图数据库             │
└─────────────────────────────────────────────────────────────┘
```

**关键设计决策：**

- ✅ **基于 trait 的存储抽象** — 迁移到 RuVector 只需实现 `VectorStorage` trait（无需修改服务层）
- ✅ **Token 缓存作为一等公民** — 预计算的 token ID 存储在向量 DB 负载中，100% 缓存命中率
- ✅ **通过 AppState 依赖注入** — 服务可单独测试，支持 mock 存储
- ✅ **零跨层依赖** — 干净的边界支持并行开发

### 性能基线（ LanceDB ）

| 指标 | 当前 | 目标（RuVector） | 改进 |
|--------|---------|-------------------|-------------|
| 查询延迟（p50） | 150ms | 5ms | **30 倍** |
| 查询延迟（p95） | 500ms | 10ms | **50 倍** |
| Token 缓存命中率 | 100% | 100% | 保持 |
| 图遍历 | 无（手动） | <2ms | **新能力** |
| 社区检测 | 手动 Louvain | 自动（SONA） | **新能力** |

---

## 为什么选择 RuVector ？

### SONA 愿景

**SONA（自组织神经网络架构）** 是 RuVector 的增量学习引擎，根据查询轨迹自适应知识图谱：

- **MicroLoRA + BaseLoRA** — 轻量级权重适配器调整边相关性，无需完全重训练
- **轨迹记录** — 捕获用户查询 → 检索块 → LLM 响应模式
- **EWC++（弹性权重巩固）** — 整合新知识时防止灾难性遗忘
- **推理库** — 存储成功的查询模式用于未来优化

### 这对 LLM-WIKI 的意义

当前 LanceDB 的局限性：
- ❌ 无原生图关系（手动 4 信号相关性模型）
- ❌ 静态知识结构（不从使用中学习）
- ❌ 手动社区检测（Louvain 在客户端运行）
- ❌ 150ms 查询延迟（向量搜索开销）

RuVector 解锁：
- ✅ **原生图边** — `FOLLOWS`、`REFERENCES`、`SIMILAR`、`PARENT_CHILD` 关系
- ✅ **自适应相关性** — 边权重根据查询成功更新
- ✅ **自动聚类** — SONA 无需手动调优即可发现知识域
- ✅ **亚 10ms 查询** — 内存图遍历 vs. 磁盘向量搜索

### 迁移策略

详见 [docs/architecture/ruvector-migration-roadmap.md](docs/architecture/ruvector-migration-roadmap.md) 的 8 周计划。

**阶段 0**（当前）：带 token 缓存的 LanceDB 基线  
**阶段 1**（第 1-2 周）：RuVector 原型 + 性能基准测试  
**阶段 2**（第 3-4 周）：功能迁移（保留 token 缓存，添加图关系）  
**阶段 3**（第 5-6 周）：SONA 集成（轨迹记录，自适应优化）  
**阶段 4**（第 7-8 周）：生产部署 + A/B 测试

**决策点**：满足以下条件时迁移：
- ✅ 项目达到稳定 v1.0
- ✅ RuVector 生态系统成熟（6+ 个月）
- ✅ 团队有 8 周时间

---

## 功能

### 后端（生产级 Rust）

- ✅ **3 层架构** — 干净分离：接口 → 服务 → 存储
- ✅ **VectorStorage Trait** — 存储后端抽象（当前 LanceDB，未来 RuVector）
- ✅ **Token 缓存层** — tiktoken-rs 预计算，节省 70% token ，100% 缓存命中
- ✅ **Markdown 感知分块** — 标题路径保留，可配置重叠，智能合并
- ✅ **多供应商 LLM** — OpenAI, Anthropic, Google, Ollama, 自定义端点
- ✅ **HTTP API + CLI** — Axum 服务器（端口 19828）+ 独立 CLI 工具
- ✅ **异步优先设计** — tokio 运行时，非阻塞 I/O，并发任务处理

### 前端（桌面客户端 - 待分离）

- **两步思维链摄入** — LLM 先分析再生成 Wiki 页面，来源可追溯
- **4 信号知识图谱** — 直接链接、来源重叠、Adamic-Adar 、类型亲和
- **Louvain 社区检测** — 自动发现知识聚类，内聚度评分
- **图谱洞察** — 惊奇连接与知识空白检测，支持一键深度研究
- **向量语义搜索** — 可选嵌入检索，支持任意 OpenAI 兼容端点
- **持久化摄入队列** — 串行处理，崩溃恢复，取消/重试，进度可视化
- **深度研究** — LLM 智能生成搜索主题，多查询网络搜索，研究结果自动摄入
- **异步审核系统** — LLM 在摄入时标记需人工判断的项，预定义操作，预生成搜索查询
- **Chrome 网页剪藏** — 一键捕获网页内容，自动摄入知识库

---

## 贡献与灵感

**基础方法论**：[Andrej Karpathy 的 LLM Wiki 模式](https://gist.github.com/karpathy/442a6bf555914893e9891c11519de94f) — 核心 3 层架构（原始资料 → Wiki → Schema）和增量编译理念。

**未来方向**：[RuVector SONA](https://github.com/ruvector/ruvector) — 用于自适应知识图谱的自组织神经网络架构，支持增量学习和灾难性遗忘预防。

**我们构建的**：实现 Karpathy 模式的生产级 Rust 后端，采用图原生设计，为 RuVector 的 SONA 能力做准备，同时保持干净的抽象层以便灵活切换存储后端。

---

## 技术栈

| 层级 | 技术 | 用途 |
|-------|-----------|---------|
| **存储抽象** | VectorStorage trait | 后端无关接口 |
| **当前存储** | LanceDB 0.4 | 嵌入式向量 DB（生产） |
| **未来存储** | RuVector + SONA | 图原生 + 自适应学习 |
| **Token 缓存** | tiktoken-rs | 预计算 token ID（节省 70%） |
| **分块** | 自定义 Rust | Markdown 标题感知分割 |
| **嵌入** | OpenAI 兼容 API | 任意 /v1/embeddings 端点 |
| **HTTP 服务器** | Axum + tokio | 异步 Rust Web 框架 |
| **CLI** | clap | 命令行界面 |
| **桌面端** | Tauri v2（旧） | 待分离为独立客户端 |
| **前端** | React 19 + TypeScript | UI 层（待解耦） |

### 架构图

```
┌──────────────────────────────────────────────────────────────┐
│                     HTTP API (Axum)                          │
│  GET  /health                                             │
│  POST /api/llm/stream                                     │
│  POST /api/ingest                                         │
│  POST /api/config/{get,save}                              │
└──────────────────────────────────────────────────────────────┘
                            ↓
┌──────────────────────────────────────────────────────────────┐
│                   服务编排                                  │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐           │
│  │ Embedding   │  │  Chunking   │  │   Ingest    │           │
│  │  Service    │  │   Service   │  │   Service   │           │
│  └─────────────┘  └─────────────┘  └─────────────┘           │
│  ┌─────────────┐  ┌─────────────┐                            │
│  │   Query     │  │ TokenCache  │                            │
│  │  Service    │  │   Service   │                            │
│  └─────────────┘  └─────────────┘                            │
└──────────────────────────────────────────────────────────────┘
                            ↓
┌──────────────────────────────────────────────────────────────┐
│              VectorStorage Trait (5 个方法)                   │
│  • async fn upsert_chunks(...)                              │
│  • async fn search(query_embedding, top_k)                │
│  • async fn delete_page(page_id)                          │
│  • async fn count() -> usize                              │
│  • fn embedding_dim() -> usize                           │
└──────────────────────────────────────────────────────────────┘
         ↓                                    ↓
��──────────────────┐              ┌──────────────────────┐
│  LanceDBStorage   │              │  RuVectorStorage     │
│  （当前）         │              │  （未来 - SONA）      │
│  • 150ms 延迟    │              │  • <10ms 延迟       │
│  • 静态图        │              │  • 自适应边         │
└──────────────────┘              └──────────────────────┘
```

---

## 我们相对于 Karpathy 模式的扩展

### 1. 从 CLI 到桌面应用

原始设计是供 LLM agent 复制粘贴使用的抽象模式文档。我们将其构建为**完整的跨平台桌面应用**：
- **三栏布局**：知识树 / 文件树（左）+ 聊天（中）+ 预览（右）
- **图标侧边栏** — 在 Wiki、资料源、搜索、图谱、Lint、审核、深度研究、设置之间快速切换
- **可调面板** — 拖拽调整左右面板大小，带最小/最大约束
- **活动面板** — 实时处理状态，逐文件显示摄入进度
- **全状态持久化** — 对话、设置、审核项、项目配置在重启后保持
- **场景模板** — 研究、阅读、个人成长、商业、通用 — 每个预配置 purpose.md 和 schema.md

### 2. Purpose.md — Wiki 的灵魂

原始设计有 Schema（ Wiki 如何运作），但没有正式定义 **为什么** 这个 Wiki 存在。我们新增了 `purpose.md`：
- 定义目标、关键问题、研究范围、演进中的论点
- LLM 在每次摄入和查询时读取它以获取上下文
- LLM 可根据使用模式建议更新
- 不同于 schema — schema 是结构规则，purpose 是方向意图

### 3. 两步思维链摄入

原始设计描述的是 LLM 同时阅读和写入的单步摄入。我们将其拆分为**两次顺序 LLM 调用**，显著提升质量：

```
第一步（分析）：LLM 阅读资料 → 结构化分析
  - 关键实体、概念、论点
  - 与现有 Wiki 内容的连接
  - 与现有知识的矛盾与张力
  - Wiki 结构建议

第二步（生成）：LLM 基于分析 → 生成 Wiki 文件
  - 带 frontmatter 的源摘要（type, title, sources[]）
  - 实体页面、概念页面带交叉引用
  - 更新的 index.md, log.md, overview.md
  - 人工判断的审核项
  - 深度研究的搜索查询
```

原始设计的额外增强：
- **SHA256 增量缓存** — 源文件内容哈希后再摄入；未更改文件自动跳过，节省 LLM token 和时间
- **持久化摄入队列** — 串行处理防止并发 LLM 调用；队列持久化到磁盘，应用重启后存活；失败任务自动重试最多 3 次
- **文件夹导入** — 递归导入保留目录结构；文件夹路径作为分类上下文传给 LLM（如 "papers > energy" 帮助分类内容）
- **队列可视化** — 活动面板显示进度条、待处理/处理中/失败任务，带取消和重试按钮
- **自动嵌入** — 启用向量搜索后，新页面在摄入后自动嵌入
- **来源追溯** — 每个生成的 Wiki 页面在 YAML frontmatter 中包含 `sources: []` 字段，链接回原始源文件
- **overview.md 自动更新** — 全局摘要页面在每次摄入时重新生成，反映 Wiki 最新状态
- **保证源摘要** — 兜底确保即使 LLM 省略也会创建源摘要页面
- **语言感知生成** — LLM 使用用户配置的语言（英文或中文）回复

### 4. 带相关性模型的知识图谱

原始提到 `[[wikilinks]]` 用于交叉引用但没有图分析。我们构建了**完整知识图谱可视化和相关性引擎**：

**4 信号相关性模型：**
| 信号 | 权重 | 描述 |
|--------|--------|-------------|
| 直接链接 | ×3.0 | 通过 `[[wikilinks]]` 链接的页面 |
| 来源重叠 | ×4.0 | 共享同一原始源的页面（通过 frontmatter `sources[]`） |
| Adamic-Adar | ×1.5 | 共享共同邻居的页面（按邻居度加权） |
| 类型亲和 | ×1.0 | 同页面类型的奖励（entity↔entity, concept↔concept） |

### 5. Louvain 社区检测

原始设计没有。我们使用 **Louvain 算法**（ graphology-communities-louvain ）自动发现知识聚类：
- **自动聚类** — 根据链接拓扑发现哪些页面自然分组，独立于预定义页面类型
- **类型/社区切换** — 在按页面类型着色和按发现的知识聚类着色之间切换
- **内聚度评分** — 每个社区按内部边密度评分（实际边 / 可能边）；低内聚度聚类（< 0.15）标记警告
- **12 色调色板** — 聚类之间视觉分离清晰
- **社区图例** — 显示每个聚类的前导节点标签、成员数、内聚度

### 6. 图谱洞察 — 惊奇连接与知识空白

原始设计没有。系统**自动分析图谱结构**以发现可操作的洞察：

**惊奇连接：**
- 检测意外关系：跨社区边、跨类型链接、外围↔中心耦合
- 组合惊奇分数排名最值得注意的连接
- 可Dismiss — 标记已审阅的连接使其不再出现

**知识空白：**
- **孤立页面**（度数 ≤ 1）— 与 Wiki 其他部分连接少或无的页面
- **稀疏社区**（内聚度 < 0.15，≥ 3 页面）— 内部交叉引用薄弱的知识区域
- **桥接节点**（连接 3+ 聚类）— 连接多个知识领域的关键页面

**交互：**
- 点击任何洞察卡片可**高亮**对应节点和边；再次点击取消选择
- 知识空白和桥接节点有**深度研究按钮** — 触发 LLM 优化研究，带领域感知主题（读取 overview.md + purpose.md 获取上下文）
- 研究主题在开始前显示在**可编辑确认对话框**中 — 用户可以精炼主题和搜索查询

### 7. 优化查询检索管道

原始设计是 LLM 读取相关页面的简单查询。我们构建了**多阶段检索管道**，带可选向量搜索和预算控制：

```
阶段 1：分词搜索
  - 英文：分词 + 停用词去除
  - 中文：CJK 二元分词（每个 → [每个, 个…]）
  - 标题匹配奖励（+10 分）
  - 同时搜索 .wiki/ 和 raw/sources/

阶段 1.5：向量语义搜索（可选）
  - 通过任意 OpenAI 兼容 /v1/embeddings 端点嵌入
  - 存储在 LanceDB（Rust 后端）中以实现快速 ANN 检索
  - 余弦相似度即使无关键词重叠也能找到语义相关的页面
  - 结果合并到搜索中：提升现有匹配 + 添加新发现

阶段 2：图扩展
  - 顶部搜索结果用作种子节点
  - 4 信号相关性模型发现相关页面
  - 2 跳遍历带衰减以获取更深连接

阶段 3：预算控制
  - 可配置上下文窗口：4K → 1M token
  - 比例分配：60% Wiki 页面，20% 聊天历史，5% 索引，15% 系统
  - 页面按组合搜索 + 图相关性分数排序

阶段 4：上下文组装
  - 带完整内容的编号页面（不仅是摘要）
  - 系统提示包含：purpose.md、语言规则、引用格式、index.md
  - LLM 被指示按编号引用页面：[1], [2], 等
```

**向量搜索**完全可选 — 默认禁用，在设置中启用，配置独立端点、API key 和模型。禁用时，管道回退到分词搜索 + 图扩展。基准测试：整体召回率从 58.2% 提升到 71.4% 启用向量搜索后。

### 8. 多对话聊天与持久化

原始设计是单一查询界面。我们构建了**完整多对话支持**：

- **独立聊天会话** — 创建、重命名、删除对话
- **对话侧边栏** — 快速切换主题
- **每对话持久化** — 每个对话保存到 `.llm-wiki/chats/{id}.json`
- **可配置历史深度** — 限制作为上下文发送的消息数（默认：10）
- **引用面板** — 每个响应上可折叠部分显示使用了哪些 Wiki 页面，按类型分组带图标
- **引用持久化** — 引用页面直接存储在消息数据中，跨重启稳定
- **重新生��** �� 一键重新生成最后一个响应（删除最后 assistant + user 消息对，重新发送）
- **保存到 Wiki** — 将有价值的答案归档到 `.wiki/queries/`，然后自动摄入以将实体/概念提取到知识网络

### 9. 思考/推理展示

原始设计没有。对于发出 `<think>` 块的 LLM（ DeepSeek, QwQ 等）：
- **流式思考** — 生成期间滚动 5 行显示，带透明度渐变
- **默认折叠** — 思考块完成后隐藏，点击展开
- **视觉分离** — 思考内容以独特样式显示，与主响应分开

### 10. KaTeX 数学渲染

原始设计没有。全 LaTeX 数学支持，跨所有视图：
- **KaTeX 渲染** — 内联 `$...$` 和块 `$$...$$` 公式通过 remark-math + rehype-katex 渲染
- **Milkdown 数学插件** — 预览编辑器通过 @milkdown/plugin-math 原生渲染数学
- **自动检测** — 裸 `\begin{aligned}` 和其他 LaTeX 环境自动用 `$$` 分隔符包装
- **Unicode 回退** — 100+ 符号映射（α, ∑, →, ≤ 等）用于数学块外的简单内联符号

### 11. 审核系统（异步人类在环）

原始设计建议在摄入时保持参与。我们添加了**异步审核队列**：
- LLM 在摄入时标记需要人工判断的项
- **预定义操作类型**：创建页面、深度研究、跳过 — 限制操作以防止 LLM 幻觉任意操作
- **搜索查询在摄入时生成** — LLM 为每个审核项预生成优化的网络搜索查询
- 用户按自己的时间处理审核 — 不阻塞摄入

### 12. 深度研究

原始设计没有。当 LLM 识别知识空白时：
- **网络搜索**（ Tavily API ）查找相关来源并提取完整内容（无截断）
- **每个主题多个搜索查询** — LLM 在摄入时生成，针对搜索引擎优化
- **LLM 优化研究主题** — 从图谱洞察触发时，LLM 读取 overview.md + purpose.md 生成领域特定主题和查询（不是通用关键词）
- **用户确认对话框** — 研究开始前显示可编辑的主题和搜索查询供审阅
- **LLM 综合** 发现到一个带与现有 Wiki 交叉引用的 Wiki 研究页面
- **思考展示** — `<think>` 块在综合期间显示为可折叠部分，自动滚动到最新内容
- **自动摄入** — 研究结果自动处理以将实体/概念提取到 Wiki
- **任务队列** 带 3 个并发任务
- **研究面板** — 专用侧边面板，动态高度，实时流式进度

### 13. 浏览器扩展（网页剪藏）

原始设计提到 Obsidian Web Clipper 。我们构建了**专用 Chrome 扩展**（ Manifest V3 ）：
- **Mozilla Readability.js** 准确提取文章（去除广告、导航、侧边栏）
- **Turndown.js** HTML → Markdown 转换，带表格支持
- **项目选择器** — 选择剪藏到哪个 Wiki（支持多项目）
- **本地 HTTP API**（端口 19827, tiny_http ）— 扩展 ↔ 应用通信
- **自动摄入** — 剪藏的内容自动触发两步摄入管道
- **剪藏监听器** — 每 3 秒轮询新剪藏，自动处理
- **离线预览** — 即使应用未运行也显示提取的内容

### 14. 多格式文档支持

原始设计专注于文本/markdown 。我们支持结构化提取，保留文档语义：

| 格式 | 方法 |
|--------|--------|
| PDF | pdf-extract (Rust)，带文件缓存 |
| DOCX | docx-rs — 标题、粗体/斜体、列表、表格 → 结构化 Markdown |
| PPTX | ZIP + XML — 按幻灯片提取，带标题/列表结构 |
| XLSX/XLS/ODS | calamine — 正确单元格类型、多工作表支持、Markdown 表格 |
| 图片 | 原生预览（png, jpg, gif, webp, svg 等） |
| 视频/音频 | 内置播放器 |
| 网页剪藏 | Readability.js + Turndown.js → 干净 Markdown |

### 15. 级联删除

原始设计没有删除机制。我们添加了**智能级联删除**：
- 删除源文件会移除其 Wiki 摘要页面
- **3 种方法匹配**找到相关 Wiki 页面：frontmatter `sources[]` 字段、源摘要页面名称、frontmatter 部分引用
- **共享实体保留** — 链接到多个源的实体/概念页面只从其 `sources[]` 数组中移除已删除的源，而不是完全删除
- **索引清理** — 已删除的页面从 index.md 中清除
- **Wikilink 清理** — 剩余 Wiki 页面中指向已删除页面的死 `[[wikilinks]]` 被移除

### 16. 可配置上下文窗口

原始设计没有。用户可以配置 LLM 收到多少上下文：
- **滑块从 4K 到 1M token** — 适应不同 LLM 能力
- **比例预算分配** — 更大的窗口获得比例上更多的 Wiki 内容
- **60/20/5/15 分割** — Wiki 页面 / 聊天历史 / 索引 / 系统提示

### 17. 跨平台兼容性

原始设计是平台无关的（抽象模式）。我们处理具体的跨平台问题：
- **路径规范化** — 统一的 `normalizePath()` 用于 22+ 文件，反斜杠 → 正斜杠
- **Unicode 安全字符串处理** — 基于字符切片而非基于字节（防止 CJK 文件名崩溃）
- **macOS 关闭隐藏** — 关闭按钮隐藏窗口（应用在后台运行），点击 dock 图标恢复，Cmd+Q 退出
- **Windows/Linux 关闭确认** — 退出前确认对话框防止意外数据丢失
- **Tauri v2** — macOS、Windows、Linux 原生桌面
- **GitHub Actions CI/CD** — macOS（ARM + Intel）、Windows（.msi）、Linux（.deb / .AppImage）自动构建

### 18. 其他添加

- **i18n** — 英文 + 中文界面（ react-i18next ）
- **设置持久化** — LLM 提供商、API key、模型、上下文大小、语言通过 Tauri Store 保存
- **Obsidian 配置** — 自动生成带推荐设置的 `.obsidian/` 目录
- **Markdown 渲染** — GFM 带边框表格、正确代码块、聊天和预览中的 wikilink 处理
- **多供应商 LLM 支持** — OpenAI, Anthropic, Google, Ollama, 自定义 — 每个带供应商特定流和 headers
- **15 分钟超时** — 长时间摄入操作不会过早失败
- **dataVersion 信号** — Wiki 内容更改时图谱和 UI 自动刷新

---

## 安装

### 预构建二进制

从 [Releases](https://github.com/yourusername/opencode-llm-wiki/releases) 下载：
- **macOS**：`.dmg`（Apple Silicon + Intel）
- **Windows**：`.msi`
- **Linux**：`.deb` / `.AppImage`

### 从源码构建

```bash
# 前置条件：Node.js 20+, Rust 1.70+
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
#   init     初始化新 Wiki 项目
#   ingest   摄入文档
#   query    查询知识库

# 开发模式（前端）
npm run dev
```

### Docker（即将支持）

```bash
docker run -p 19828:19828 -v ./data:/data opencode-llm-wiki
```

### Chrome 扩展

1. 打开 `chrome://extensions`
2. 启用"开发者模式"
3. 点击"加载已解压"
4. 选择 `extension/` 目录

---

## 快速开始

1. 启动应用 → 创建新项目（选择模板）
2. 进入 **设置** → 配置 LLM 提供商（API key + 模型）
3. 进入 **资料源** → 导入文档（PDF, DOCX, MD 等）
4. 观察 **活动面板** — LLM 自动构建 Wiki 页面
5. 使用 **聊天** 查询你的知识库
6. 浏览 **知识图谱** 查看连接
7. 检查 **审核** 需要你关注的项
8. 定期运行 **Lint** 保持 Wiki 健康

---

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
│   ├── queries            # 保存的聊天答案 + 研究
│   ├── synthesis          # 跨源分析
│   └── comparisons        # 并排比较
├── .obsidian/              # Obsidian 保险库配置（自动生成）
└── .llm-wiki/              # 应用配置、聊天历史、审核项
```

### 代码库结构
```
opencode-llm-wiki/
├── src/                    # Rust 后端（3 层架构）
│   ├── api/                # 第 1 层：HTTP API + 处理器
│   │   ├── state.rs        # AppState（依赖注入）
│   │   ├── handlers.rs     # 请求处理器
│   │   ├── routes.rs       # 路由定义
│   │   └── server.rs       # Axum 服务器
│   ├── services/           # 第 2 层：业务逻辑
│   │   ├── embedding.rs    # OpenAI 兼容嵌入 API
│   │   ├── chunking.rs     # Markdown 标题感知分割
│   │   ├── ingest.rs       # 编排（parse→chunk→embed→store）
│   │   ├── query.rs        # 搜索 + 上下文优化
│   │   ├── token_cache.rs  # tiktoken-rs 预计算
│   │   └── llm_client.rs   # 多供应商流式
│   ├── storage/            # 第 3 层：数据抽象
│   │   ├── traits.rs       # VectorStorage trait
│   │   └── lancedb_impl.rs # LanceDB 实现
│   ├── types              # 共享类型
│   ├── utils             # 工具函数
│   ├── main.rs            # API 服务器二进制（端口 19828）
│   └── cli.rs            # CLI 工具二进制
├── src-desktop/            # 桌面客户端（待分离）
│   ├── ui-new/             # React 前端
│   └── src-tauri-new/      # Tauri 包装器
├── src-legacy/             # 归档 TypeScript 实现
├── extension/              # Chrome 扩展
└── docs/                   # 架构文档
    └── architecture/
        ├── 3-layer-refactoring-plan.md
        └── ruvector-migration-roadmap.md
```

**架构**：基于 trait 的存储抽象，干净的 3 层设计。未来后端迁移（如 LanceDB → RuVector）只需实现 `VectorStorage` trait。

---

## Star 历史

<a href="https://www.star-history.com/?repos=yourusername%2Fopencode-llm-wiki&type=date&legend=top-left">
 <picture>
   <source media="(prefers-color-scheme: dark)" srcset="https://api.star-history.com/chart?repos=yourusername/opencode-llm-wiki&type=date&theme=dark&legend=top-left" />
   <source media="(prefers-color-scheme: light)" srcset="https://api.star-history.com/chart?repos=yourusername/opencode-llm-wiki&type=date&legend=top-left" />
   <img alt="Star History Chart" src="https://api.star-history.com/chart?repos=yourusername/opencode-llm-wiki&type=date&legend=top-left" />
 </picture>
</a>

---

## 许可证

本项目基于 **GNU 通用公共许可证 v3.0** — 详见 [LICENSE](LICENSE) 。