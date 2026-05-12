# Capella Room 文档中心

> 本文档中心包含 Capella Room 项目的所有技术文档

---

## 文档目录

### 📋 项目规划

| 文档 | 说明 |
|------|------|
| [v1/roadmap.md](v1/roadmap.md) | **v1 版本开发路线图** - 项目开发阶段规划与进度追踪 |
| [v1/code-review-01.md](v1/code-review-01.md) | **第一次代码审查报告** - 代码质量问题及修复记录 |
| [v1/backend-refinement.md](v1/backend-refinement.md) | **后端细节修复清单** - API 响应优化与用户体验改进 |

### 🔌 API 文档

| 文档 | 说明 |
|------|------|
| [api/README.md](api/README.md) | API 文档入口 |
| [api/v1/http/](api/v1/http/) | HTTP REST API 文档 |
| [api/v1/websocket/](api/v1/websocket/) | WebSocket 实时通信协议文档 |

### 🏗️ 架构文档

| 文档 | 说明 |
|------|------|
| [architecture/README.md](architecture/README.md) | 系统架构设计文档 - 分布式架构、数据库优化、安全加固 |

### 🧪 测试报告

| 文档 | 说明 |
|------|------|
| [test-reports/test-summary.md](test-reports/test-summary.md) | 测试总览与统计 |
| [test-reports/e2e-test-report.md](test-reports/e2e-test-report.md) | 端到端测试报告 |
| [test-reports/stress-test-report.md](test-reports/stress-test-report.md) | 压力测试报告 |

---

## 快速导航

### 开发者指南

1. **新成员入门**: 先阅读 [v1/roadmap.md](v1/roadmap.md) 了解项目整体规划
2. **API 开发**: 参考 [api/](api/) 目录下的接口文档
3. **架构设计**: 查看 [architecture/README.md](architecture/README.md) 了解系统架构

### 维护者指南

1. **代码审查**: 参考 [v1/code-review-01.md](v1/code-review-01.md) 了解历史问题及修复方案
2. **后端优化**: 查看 [v1/backend-refinement.md](v1/backend-refinement.md) 了解待优化的细节
3. **测试情况**: 查看 [test-reports/](test-reports/) 目录了解测试覆盖情况

---

## 文档规范

### 文件命名

- 使用小写字母和连字符（kebab-case）
- 版本文档放在 `v{n}/` 目录下，如 `v1/roadmap.md`
- 代码审查报告使用 `code-review-{序号}.md` 格式

### 版本管理

- `v1/` 目录存放第一版项目的规划文档
- 后续版本创建 `v2/`、`v3/` 等目录
- 历史版本文档保留，便于追溯

---

*最后更新: 2026-04-29*
