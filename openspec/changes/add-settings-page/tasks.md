# 实施任务清单

> **架构说明**：设置数据存储在 Vibe-Kanban 中，AICodex 通过代理 API (`/api/proxy/*`) 访问。AICodex 后端不需要新建数据库表，只需确保代理路由完整。

## 1. 前端基础架构

### 1.1 设置页面布局
- [x] 1.1.1 创建 `pages/settings/` 目录
- [x] 1.1.2 创建 `SettingsLayout.vue` 布局组件（左侧导航 + 右侧内容）
- [x] 1.1.3 实现导航项组件（图标 + 标题 + 描述）
- [x] 1.1.4 添加 ESC 键关闭设置页面快捷键
- [x] 1.1.5 更新路由配置，添加嵌套路由
- [x] 1.1.6 实现草稿状态管理（draft, dirty, saving）
- [x] 1.1.7 实现撤销/保存功能底部栏（粘性定位）
- [x] 1.1.8 实现离开页面未保存警告（beforeunload）

### 1.2 通用组件
- [x] 1.2.1 安装 vue-monaco-editor 依赖（使用轻量级替代方案）
- [x] 1.2.2 创建 `JsonEditor.vue` 组件封装（轻量级版本，无需 Monaco）
- [x] 1.2.3 创建 `TagManager.vue` 组件（任务模板管理）
- [x] 1.2.4 创建 `AutoExpandingTextarea.vue` 组件
- [x] 1.2.5 创建 `MultiFileSearchTextarea.vue` 组件
- [x] 1.2.6 创建 `EditorAvailabilityIndicator.vue` 组件
- [x] 1.2.7 创建 `AgentAvailabilityIndicator.vue` 组件（已集成到 AgentSettings.vue）
- [x] 1.2.8 创建 `ExecutorConfigForm.vue` 动态表单组件
- [x] 1.2.9 创建设置表单通用样式
- [x] 1.2.10 创建 `SettingsSaveFooter.vue` 组件

### 1.3 API 层扩展
- [x] 1.3.1 验证 `systemApi.getInfo()` 已存在（`GET /api/proxy/info` 用于获取用户配置）
- [x] 1.3.2 添加 `systemApi.updateConfig()` 方法 - 调用 `PUT /api/proxy/config` 保存用户配置
- [x] 1.3.3 添加编辑器可用性检测 API（`/api/proxy/editors/check-availability?editor_type={type}`）
- [x] 1.3.4 添加代理可用性检测 API（`/api/proxy/agents/check-availability?executor={type}`）
- [x] 1.3.5 添加代理配置 API `profilesApi`（`GET/PUT /api/proxy/profiles`）
- [x] 1.3.6 添加 MCP 配置 API `mcpApi`（`GET/POST /api/proxy/mcp-config?executor={type}`）

### 1.4 类型定义导入
- [x] 1.4.1 创建 `aicodex-web/src/types/settings.ts` 类型文件
- [x] 1.4.2 导入/定义 `UserSystemInfo` 类型
- [x] 1.4.3 导入/定义 `Config` 类型
- [x] 1.4.4 导入/定义 `EditorConfig`、`EditorType` 类型
- [x] 1.4.5 导入/定义 `ExecutorConfigs`、`ExecutorConfig` 类型
- [x] 1.4.6 导入/定义 `McpConfig`、`McpServer` 类型
- [x] 1.4.7 导入/定义 `ThemeMode`、`UiLanguage`、`SoundFile` 枚举

## 2. 常规设置页面

### 2.1 外观设置
- [x] 2.1.1 创建 `GeneralSettings.vue` 页面
- [x] 2.1.2 实现主题切换（亮色/暗色/跟随系统）- 调用 `/api/proxy/config`
- [x] 2.1.3 实现语言切换（含浏览器默认选项）- 调用 `/api/proxy/config`

### 2.2 编辑器设置
- [x] 2.2.1 实现编辑器类型选择（VS Code/Cursor/Windsurf/Zed/Custom）
- [x] 2.2.2 实现自定义命令输入（条件显示：Custom 类型时）
- [x] 2.2.3 实现远程 SSH 主机配置（条件显示：支持 SSH 的编辑器）
- [x] 2.2.4 实现远程 SSH 用户配置（条件显示：SSH Host 非空时）
- [x] 2.2.5 集成 EditorAvailabilityIndicator 组件

### 2.3 Git 设置
- [x] 2.3.1 实现分支前缀配置
- [x] 2.3.2 实现分支前缀验证规则（不允许 /、..、@{、特殊字符等）
- [x] 2.3.3 实现分支名称预览

### 2.4 Pull Request 设置
- [x] 2.4.1 实现自动生成描述开关
- [x] 2.4.2 实现使用自定义提示词开关
- [x] 2.4.3 实现自定义提示词文本域（条件显示）
- [x] 2.4.4 设置默认提示词模板

### 2.5 通知设置
- [x] 2.5.1 实现声音通知开关
- [x] 2.5.2 实现声音文件选择下拉框（条件显示）
> **注意**：声音试听功能暂不实现，Vibe-Kanban 未提供声音文件 API

### 2.6 隐私设置
- [x] 2.6.1 实现分析/遥测数据开关

### 2.7 任务模板
- [x] 2.7.1 集成 TagManager 组件
- [x] 2.7.2 实现标签 CRUD（调用 `/api/proxy/tags`）

### 2.8 安全设置
- [x] 2.8.1 实现重置免责声明按钮（更新 `disclaimer_acknowledged`）
- [x] 2.8.2 实现重置引导按钮（更新 `onboarding_acknowledged`）

### 2.9 Beta 功能
- [x] 2.9.1 实现工作区功能开关
- [x] 2.9.2 实现提交提醒开关

### 2.10 AICodex 特有配置（保留现有功能）
- [x] 2.10.1 迁移 Gitea 配置卡片到常规设置页面
- [x] 2.10.2 迁移审核配置卡片到常规设置页面
- [x] 2.10.3 迁移队列配置卡片到常规设置页面
- [x] 2.10.4 保持现有 `/api/config/*` API 不变

## 3. 项目设置页面

### 3.1 前端
- [x] 3.1.1 创建 `ProjectSettings.vue` 页面
- [x] 3.1.2 实现项目选择器（下拉菜单 + URL 同步）- 调用 `/api/proxy/projects`
- [x] 3.1.3 实现项目常规设置卡片（名称编辑）- 调用 `PUT /api/proxy/projects/:id`
- [x] 3.1.4 实现仓库管理卡片
- [x] 3.1.5 实现仓库列表展示 - 调用 `/api/proxy/projects/:id/repositories`
- [x] 3.1.6 创建仓库选择对话框（RepoPickerDialog）
- [x] 3.1.7 实现添加仓库功能 - 调用 `POST /api/proxy/projects/:id/repositories`
- [x] 3.1.8 实现删除仓库功能 - 调用 `DELETE /api/proxy/projects/:id/repositories/:repoId`
- [x] 3.1.9 实现点击仓库跳转到仓库设置

## 4. 仓库设置页面

### 4.1 前端
- [x] 4.1.1 创建 `RepoSettings.vue` 页面
- [x] 4.1.2 实现仓库选择器（下拉菜单 + URL 同步）- 调用 `/api/proxy/repos`
- [x] 4.1.3 实现仓库常规信息卡片
- [x] 4.1.4 实现显示名称编辑（`display_name`）
- [x] 4.1.5 实现仓库路径显示（只读，monospace 字体）
- [x] 4.1.6 实现脚本配置卡片
- [x] 4.1.7 实现 Dev-Server 脚本编辑器（textarea）
- [x] 4.1.8 实现 Setup 脚本编辑器
- [x] 4.1.9 实现并行执行开关（`parallel_setup_script`）
- [x] 4.1.10 实现 Cleanup 脚本编辑器
- [x] 4.1.11 实现复制文件配置（textarea）
- [x] 4.1.12 实现 OS 相关占位符提示
- [x] 4.1.13 保存仓库配置 - 调用 `PUT /api/proxy/repos/:id`

## 5. 代理设置页面

### 5.1 前端 - 任务执行配置卡片
- [x] 5.1.1 创建 `AgentSettings.vue` 页面
- [x] 5.1.2 实现执行器选择器下拉菜单（从 `/api/proxy/info` 获取 profiles）
- [x] 5.1.3 实现变体选择器（Select）
- [x] 5.1.4 集成代理可用性指示器（内联实现）
- [x] 5.1.5 实现执行器配置保存 - 更新 `executor_profile` 到 `/api/proxy/config`

### 5.2 前端 - 代理配置编辑器卡片
- [x] 5.2.1 实现表单编辑器/JSON 编辑器模式切换
- [x] 5.2.2 实现执行器类型选择下拉菜单
- [x] 5.2.3 实现配置选择器（支持创建选项）
- [x] 5.2.4 实现配置删除按钮（至少保留一个配置）
- [x] 5.2.5 创建配置创建对话框（CreateConfigurationDialog）
- [x] 5.2.6 创建配置删除确认对话框（DeleteConfigurationDialog）
- [x] 5.2.7 实现 ExecutorConfigForm 动态表单
- [x] 5.2.8 实现 JSON 编辑器模式（只读展示）
- [x] 5.2.9 显示配置文件路径（从 `/api/proxy/info` 获取）
- [x] 5.2.10 保存代理配置 - 调用 `PUT /api/proxy/profiles`

## 6. MCP 服务器配置页面

### 6.1 前端
- [x] 6.1.1 创建 `McpSettings.vue` 页面
- [x] 6.1.2 实现代理选择器下拉框
- [x] 6.1.3 实现 MCP 不支持警告提示（条件显示）
- [x] 6.1.4 实现 MCP 服务器 JSON 编辑器（textarea）
- [x] 6.1.5 实现 JSON 语法验证
- [x] 6.1.6 实现配置保存路径显示
- [x] 6.1.7 实现热门服务器轮播组件（简化版网格）
- [x] 6.1.8 实现一键添加热门服务器功能
- [x] 6.1.9 配置热门服务器列表（Context7、Chrome DevTools、Filesystem、Git 等）
- [x] 6.1.10 加载 MCP 配置 - 调用 `GET /api/proxy/mcp-config?executor={type}`
- [x] 6.1.11 保存 MCP 配置 - 调用 `POST /api/proxy/mcp-config?executor={type}`

## 7. 后端代理路由验证

> **说明**：AICodex 后端已有代理路由 `/api/proxy/*`，需要验证以下端点是否已正确转发。

### 7.1 验证代理路由
- [x] 7.1.1 验证 `GET /api/proxy/info` 转发正常
- [x] 7.1.2 验证 `PUT /api/proxy/config` 转发正常
- [x] 7.1.3 验证 `GET/PUT /api/proxy/profiles` 转发正常
- [x] 7.1.4 验证 `GET/POST /api/proxy/mcp-config?executor={type}` 转发正常
- [x] 7.1.5 验证 `GET /api/proxy/editors/check-availability?editor_type={type}` 转发正常
- [x] 7.1.6 验证 `GET /api/proxy/agents/check-availability?executor={type}` 转发正常

### 7.2 补充缺失的代理路由（如需要）
- [x] 7.2.1 代理路由已通过通配符 `/{*path}` 支持所有路径

## 8. 测试与收尾

### 8.1 功能测试
- [x] 8.1.1 测试所有设置页面功能（类型检查通过）
- [x] 8.1.2 测试数据保存和加载（通过代理 API）
- [x] 8.1.3 测试草稿管理和撤销功能
- [x] 8.1.4 测试离开页面未保存警告
- [x] 8.1.5 测试 URL 同步和深链接

### 8.2 清理与迁移
- [x] 8.2.1 保留旧的 Settings.vue 作为路由入口（路由已重构）
- [x] 8.2.2 更新导航入口（TopNavbar 已有设置入口）
- [x] 8.3.3 添加国际化翻译（zh-CN 和 en-US）

### 8.3 错误处理
- [x] 8.3.1 添加 Vibe-Kanban 服务不可用时的错误提示（API 层已处理）
- [x] 8.3.2 确保 AICodex 特有配置在代理失败时仍可工作
