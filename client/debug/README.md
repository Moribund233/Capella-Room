# Seredeli Debug

Seredeli Room 调试工具端，提供多用户测试、WebSocket测试、API测试等开发调试功能。基于 SeredeliUI 框架构建，采用配置驱动的设计理念。

## 文档目录

1. [框架介绍](./docs/introduction.md) - 了解框架的设计理念与核心特性
2. [快速开始](./docs/quickstart.md) - 5分钟上手框架
3. [页面开发指南](./docs/page-development.md) - 学习如何开发新页面
4. [配置系统](./docs/configuration.md) - 掌握配置驱动的UI系统
5. [QuickBar 开发](./docs/quickbar-development.md) - 自定义快捷按钮
6. [接入真实后端](./docs/backend-integration.md) - 从模拟数据切换到真实API

## 核心特性

- **配置驱动** - 通过简单的配置对象定义侧边栏、Dock栏、快捷按钮等UI元素
- **三端通用** - 内置移动端、平板端、桌面端三端适配
- **主题系统** - 支持浅色/深色主题切换
- **图表集成** - 内置 ECharts 图表组件，支持数据可视化
- **模块化架构** - 清晰的代码结构，易于扩展和维护
- **类型安全** - 完整的 TypeScript 类型支持

## 技术栈

- Vue 3 + Composition API
- TypeScript
- Naive UI（组件库）
- Lucide Vue Next（图标库）
- ECharts（图表库）
- Pinia（状态管理）
- Vue Router（路由管理）

## 快速开始

```bash
# 安装依赖
npm install

# 启动开发服务器
npm run dev
```

访问 http://localhost:3000 查看应用。

## 项目结构

```
SeredeliUI/
├── src/
│   ├── api/           # API 接口层
│   ├── components/    # 组件目录
│   │   ├── common/    # 通用组件（QuickBar, DockBar, Chart等）
│   │   ├── layout/    # 布局组件（Header, Sidebar, Footer）
│   │   └── ...        # 业务组件
│   ├── composables/   # 组合式函数
│   ├── config/        # 配置文件
│   ├── router/        # 路由配置
│   ├── store/         # Pinia Store
│   ├── types/         # 类型定义
│   └── views/         # 页面视图
├── docs/              # 文档目录
└── ...
```

## 快速链接

- [查看示例页面](./src/views/ExampleView.vue)
- [查看配置示例](./src/config/ui.ts)
- [查看类型定义](./src/types/types.ts)

## 许可证

MIT
