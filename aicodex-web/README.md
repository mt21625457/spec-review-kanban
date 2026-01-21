# aicodex-web - 多智能体编排系统

aicodex-web 是一个现代化的 Vue 3 前端应用，用于管理和编排多智能体任务。

## 技术栈

- **框架**: Vue 3 + Composition API
- **构建工具**: Vite 7
- **样式**: Tailwind CSS 3
- **状态管理**: Pinia + Vue Query
- **路由**: Vue Router 4
- **国际化**: vue-i18n
- **拖拽**: vue-draggable-plus
- **类型检查**: TypeScript 5.7

## 快速开始

### 安装依赖

```bash
pnpm install
```

### 开发服务器

```bash
pnpm run dev
```

访问 http://localhost:5173

### 构建生产版本

```bash
pnpm run build
```

### 类型检查

```bash
pnpm run type-check
```

## 测试

### 单元测试 (Vitest)

```bash
# 开发模式（watch）
pnpm run test

# 单次运行
pnpm run test:run

# 带覆盖率
pnpm run test:coverage
```

### E2E 测试 (Playwright)

```bash
# 运行 E2E 测试
pnpm run test:e2e

# UI 模式
pnpm run test:e2e:ui
```

## 项目结构

```
src/
├── components/        # Vue 组件
│   ├── diff/         # 差异查看器
│   ├── git/          # Git 操作组件
│   ├── layout/       # 布局组件
│   ├── projects/     # 项目管理组件
│   ├── tasks/        # 任务看板组件
│   ├── ui/           # 基础 UI 组件
│   └── workspaces/   # 工作区组件
├── composables/      # Vue Composables (Hooks)
├── i18n/             # 国际化配置
├── lib/              # 工具库
├── pages/            # 页面组件
├── router/           # 路由配置
├── stores/           # Pinia Stores
├── styles/           # 全局样式
└── types/            # TypeScript 类型定义
```

## 功能特性

### 导航布局
- 顶部导航栏设计
- 响应式移动端适配
- 暗色/亮色主题切换
- 中英文语言切换

### 任务管理
- 项目列表与选择
- 看板视图任务管理
- 拖拽排序与状态变更
- 任务创建、编辑、删除
- 任务筛选与搜索

### 代码审核
- 审核列表展示
- 审核详情与事件时间线
- 文件差异查看

### 工作区管理
- 工作区列表
- Git 推送与 PR 创建

## 环境变量

开发环境默认配置在 `vite.config.ts` 中：

- `VITE_API_BASE_URL`: API 服务器地址（默认代理到 http://localhost:8765）

## 性能

- 路由级别代码分割
- 组件懒加载
- Bundle 大小: < 150 KB (gzip)
- 首屏加载: < 2s (3G)

## License

MIT
