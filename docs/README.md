# OpenCode LLM Wiki 文档

> v1.1.0 文档导航

---

## 📚 快速开始

- [项目总览](../README.md)
- [API 快速开始](api/quickstart.md)
- [开发环境设置](development/setup.md)

---

## 🚀 发布文档

- [v1.1.0 发布说明](RELEASE_v1.1.0.md)
- [v1.1.0 验证报告](V1.1.0_VERIFICATION.md)
- [RuVector Phase 1 完成报告](RUVECTOR_PHASE1_COMPLETE.md)
- [RuVector Phase 2 完成报告](RUVECTOR_PHASE2_COMPLETE.md)

---

## 📖 API 文档

- [API 服务器设计](api/server.md)
- [API 快速开始](api/quickstart.md)

---

## 🔧 使用指南

- [LLM 配置指南](guides/llm-config.md)
- [MCP 服务器指南](guides/mcp-server.md)
- [Windows 部署指南](guides/deployment-windows.md)

---

## 🏗️ 架构文档（历史参考）

以下文档描述的是旧版本架构（Tauri 桌面应用），仅供历史参考：

- [旧架构概览](architecture/overview.md) - ⚠️ 已过时
- [旧最终架构方案](architecture/final.md) - ⚠️ 已过时
- [知识图谱设计](architecture/knowledge-graph.md)
- [RuVector 集成方案](architecture/ruvector-integration.md)
- [RuVector 迁移路线图](architecture/ruvector-migration-roadmap.md)

---

## 📚 技术文档

- [LLM + RuVector 集成](LLM+RuVector.md)

---

## 📦 归档

- [Karpathy 原始文档](archive/karpathy-original.md)

---

## 当前架构（v1.1.0）

**核心组件**:
- Rust 后端服务器（HTTP API）
- MCP 服务器（Model Context Protocol）
- RuVector 存储引擎（向量数据库 + 图数据库）
- 知识图谱算法（PageRank、Louvain、中心性分析）

**不再包含**:
- ❌ Tauri 桌面客户端
- ❌ React 前端 UI
- ❌ Chrome 扩展
- ❌ Web UI

---

*最后更新: 2026-05-04*
