# Settings Page Specification

> **架构说明**：设置数据存储在 Vibe-Kanban 中，AICodex 通过代理 API (`/api/proxy/*`) 访问。

## ADDED Requirements

### Requirement: Settings Page Layout
系统 SHALL 提供多页面设置布局，包含左侧导航栏和右侧内容区域，支持草稿管理和撤销/保存功能。

#### Scenario: 访问设置页面
- **WHEN** 用户点击导航栏的设置图标
- **THEN** 系统显示设置页面，左侧显示设置分类导航，右侧显示默认的常规设置内容

#### Scenario: 导航切换
- **WHEN** 用户点击左侧导航中的某个分类
- **THEN** 右侧内容区域切换到对应的设置页面，URL 更新为对应路径

#### Scenario: ESC 键关闭
- **WHEN** 用户在设置页面按下 ESC 键
- **THEN** 设置页面关闭，返回之前的页面

#### Scenario: 草稿管理
- **WHEN** 用户修改设置但未保存
- **THEN** 系统显示"有未保存的更改"提示，并启用撤销和保存按钮

#### Scenario: 离开页面警告
- **WHEN** 用户有未保存的更改并尝试离开页面
- **THEN** 系统显示确认对话框，询问是否放弃更改

---

### Requirement: General Settings - Appearance
系统 SHALL 提供外观设置，允许用户配置主题和语言。数据通过 `/api/proxy/config` 保存到 Vibe-Kanban。

#### Scenario: 主题切换
- **WHEN** 用户选择主题选项（亮色/暗色/跟随系统）
- **THEN** 系统立即应用选中的主题

#### Scenario: 语言切换
- **WHEN** 用户选择界面语言
- **THEN** 系统界面切换到选中的语言

---

### Requirement: General Settings - Editor
系统 SHALL 提供编辑器设置，允许用户配置代码编辑器偏好。数据通过 `/api/proxy/config` 保存。

#### Scenario: 编辑器类型选择
- **WHEN** 用户选择编辑器类型（VS Code/Cursor/Windsurf/Zed/Custom）
- **THEN** 系统保存编辑器偏好，并显示编辑器可用性状态

#### Scenario: 自定义命令
- **WHEN** 用户选择 Custom 编辑器类型
- **THEN** 系统显示自定义命令输入框

#### Scenario: 远程 SSH 配置
- **WHEN** 用户配置远程 SSH 主机
- **THEN** 系统显示 SSH 用户输入框，保存远程 SSH 配置

#### Scenario: 编辑器可用性检测
- **WHEN** 用户选择一个编辑器
- **THEN** 系统调用 `/api/proxy/editors/check-availability?editor_type={type}` 显示该编辑器的可用性状态指示器

---

### Requirement: General Settings - Git
系统 SHALL 提供 Git 设置，允许用户配置分支命名规则。

#### Scenario: 分支前缀配置
- **WHEN** 用户设置分支前缀
- **THEN** 系统验证前缀格式并显示分支名称预览

#### Scenario: 分支前缀验证失败
- **WHEN** 用户输入包含非法字符的分支前缀（/、..、@{、特殊字符等）
- **THEN** 系统显示错误提示，禁用保存按钮

---

### Requirement: General Settings - Pull Request
系统 SHALL 提供 Pull Request 设置，允许用户配置 PR 描述自动生成。

#### Scenario: 启用自动描述
- **WHEN** 用户启用 PR 自动描述功能
- **THEN** 系统在创建 PR 时自动生成描述

#### Scenario: 自定义提示词
- **WHEN** 用户启用自定义提示词并编辑内容
- **THEN** 系统使用自定义提示词生成 PR 描述

---

### Requirement: General Settings - Notifications
系统 SHALL 提供通知设置，允许用户配置声音通知。

> **注意**：声音试听功能暂不实现，Vibe-Kanban 未提供声音文件 API。

#### Scenario: 声音通知
- **WHEN** 用户启用声音通知
- **THEN** 系统显示声音文件选择下拉框

#### Scenario: 选择声音文件
- **WHEN** 用户从下拉框选择一个声音文件
- **THEN** 系统保存声音文件偏好（通过 `/api/proxy/config`）

---

### Requirement: General Settings - Privacy
系统 SHALL 提供隐私设置，允许用户控制数据收集。

#### Scenario: 分析数据开关
- **WHEN** 用户切换分析/遥测数据开关
- **THEN** 系统保存偏好，启用或禁用数据收集

---

### Requirement: General Settings - Task Templates
系统 SHALL 提供任务模板管理，允许用户创建和管理标签。数据通过 `/api/proxy/tags` 操作。

#### Scenario: 创建标签
- **WHEN** 用户在任务模板区域添加新标签
- **THEN** 系统调用 `POST /api/proxy/tags` 创建标签并更新列表

#### Scenario: 编辑标签
- **WHEN** 用户编辑标签名称或内容
- **THEN** 系统调用 `PUT /api/proxy/tags/:id` 更新标签信息

#### Scenario: 删除标签
- **WHEN** 用户删除标签
- **THEN** 系统调用 `DELETE /api/proxy/tags/:id` 移除标签并更新列表

---

### Requirement: General Settings - Safety
系统 SHALL 提供安全设置，允许用户重置免责声明和引导状态。

#### Scenario: 重置免责声明
- **WHEN** 用户点击重置免责声明按钮
- **THEN** 系统更新 `disclaimer_acknowledged` 为 false，下次启动时显示免责声明

#### Scenario: 重置引导
- **WHEN** 用户点击重置引导按钮
- **THEN** 系统更新 `onboarding_acknowledged` 为 false，下次启动时显示引导流程

---

### Requirement: General Settings - Beta Features
系统 SHALL 提供 Beta 功能设置，允许用户启用实验性功能。

#### Scenario: 工作区功能
- **WHEN** 用户启用工作区功能
- **THEN** 系统启用多工作区支持

#### Scenario: 提交提醒
- **WHEN** 用户启用提交提醒
- **THEN** 系统在长时间未提交时显示提醒

---

### Requirement: General Settings - AICodex Configuration
系统 SHALL 保留 AICodex 特有的配置（Gitea、审核、队列），使用现有的 `/api/config/*` API。

#### Scenario: Gitea 配置
- **WHEN** 用户配置 Gitea API URL、Token、Webhook Secret
- **THEN** 系统调用 `PUT /api/config/gitea` 保存配置

#### Scenario: 审核配置
- **WHEN** 用户配置默认代理、自动启动、超时时间
- **THEN** 系统调用 `PUT /api/config/review` 保存配置

#### Scenario: 队列配置
- **WHEN** 用户配置最大并发、重试次数、重试延迟
- **THEN** 系统调用 `PUT /api/config/queue` 保存配置

---

### Requirement: Project Settings
系统 SHALL 提供项目设置页面，允许用户管理代码审核项目及其关联的仓库。数据通过 `/api/proxy/projects/*` 操作。

#### Scenario: 项目选择器
- **WHEN** 用户访问项目设置页面
- **THEN** 系统调用 `/api/proxy/projects` 显示项目选择下拉菜单，URL 同步当前选中的项目 ID

#### Scenario: 查看项目列表
- **WHEN** 用户打开项目选择下拉菜单
- **THEN** 系统显示所有已配置的项目列表

#### Scenario: 创建项目
- **WHEN** 用户点击"新建项目"并填写信息提交
- **THEN** 系统调用 `POST /api/proxy/projects` 创建项目并自动选中

#### Scenario: 编辑项目名称
- **WHEN** 用户在项目常规设置卡片中修改名称
- **THEN** 系统调用 `PUT /api/proxy/projects/:id` 更新项目名称

#### Scenario: 删除项目
- **WHEN** 用户确认删除项目
- **THEN** 系统调用 `DELETE /api/proxy/projects/:id` 删除项目及其关联关系

#### Scenario: 查看项目仓库
- **WHEN** 用户选中一个项目
- **THEN** 系统调用 `/api/proxy/projects/:id/repositories` 在仓库管理卡片中显示该项目关联的仓库列表

#### Scenario: 添加仓库到项目
- **WHEN** 用户点击添加仓库按钮并选择仓库
- **THEN** 系统调用 `POST /api/proxy/projects/:id/repositories` 将仓库关联到项目并刷新列表

#### Scenario: 从项目移除仓库
- **WHEN** 用户点击仓库的删除按钮
- **THEN** 系统调用 `DELETE /api/proxy/projects/:id/repositories/:repoId` 移除关联并刷新列表

#### Scenario: 跳转到仓库设置
- **WHEN** 用户点击仓库列表中的某个仓库
- **THEN** 系统跳转到仓库设置页面并选中该仓库

---

### Requirement: Repository Settings
系统 SHALL 提供仓库设置页面，允许用户配置仓库信息和脚本。数据通过 `/api/proxy/repos/*` 操作。

#### Scenario: 仓库选择器
- **WHEN** 用户访问仓库设置页面
- **THEN** 系统调用 `/api/proxy/repos` 显示仓库选择下拉菜单，URL 同步当前选中的仓库 ID

#### Scenario: 编辑显示名称
- **WHEN** 用户修改仓库显示名称
- **THEN** 系统更新显示名称（`display_name`）

#### Scenario: 查看仓库路径
- **WHEN** 用户选中一个仓库
- **THEN** 系统以只读方式显示仓库路径（monospace 字体）

#### Scenario: 配置 Dev-Server 脚本
- **WHEN** 用户编辑 Dev-Server 脚本
- **THEN** 脚本内容保存，需要开发服务器时执行此脚本

#### Scenario: 配置 Setup 脚本
- **WHEN** 用户编辑 Setup 脚本
- **THEN** 脚本内容保存，代码审核前执行此脚本

#### Scenario: 启用并行执行
- **WHEN** 用户启用 Setup 脚本的并行执行开关
- **THEN** 系统允许多个 Setup 命令并行执行（`parallel_setup_script`）

#### Scenario: 配置 Cleanup 脚本
- **WHEN** 用户编辑 Cleanup 脚本
- **THEN** 脚本内容保存，代码审核后执行此脚本

#### Scenario: 配置复制文件
- **WHEN** 用户在复制文件文本区输入文件模式
- **THEN** 系统支持通配符搜索和多文件选择

#### Scenario: 保存仓库配置
- **WHEN** 用户保存仓库配置
- **THEN** 系统调用 `PUT /api/proxy/repos/:id` 保存所有更改

---

### Requirement: Agent Settings - Task Execution
系统 SHALL 提供任务执行配置，允许用户选择当前使用的执行器。

#### Scenario: 选择执行器
- **WHEN** 用户在执行器下拉菜单中选择一个执行器
- **THEN** 系统更新当前使用的执行器（`executor_profile`）

#### Scenario: 选择变体
- **WHEN** 用户选择执行器的变体配置
- **THEN** 系统更新执行器变体

#### Scenario: 查看代理可用性
- **WHEN** 用户选择一个执行器
- **THEN** 系统调用 `/api/proxy/agents/check-availability?executor={type}` 显示该代理的可用性状态指示器

---

### Requirement: Agent Settings - Configuration Editor
系统 SHALL 提供代理配置编辑器，支持表单和 JSON 双模式编辑。数据通过 `/api/proxy/profiles` 操作。

#### Scenario: 切换编辑模式
- **WHEN** 用户切换"使用表单编辑器"开关
- **THEN** 系统在表单编辑器和 JSON 编辑器之间切换

#### Scenario: 选择执行器类型
- **WHEN** 用户在执行器类型下拉菜单中选择
- **THEN** 系统加载该类型的配置列表

#### Scenario: 选择配置
- **WHEN** 用户在配置下拉菜单中选择一个配置
- **THEN** 系统加载并显示该配置内容

#### Scenario: 创建配置
- **WHEN** 用户点击创建配置并填写名称
- **THEN** 系统创建新配置（可选从现有配置克隆）

#### Scenario: 删除配置
- **WHEN** 用户删除一个非唯一配置
- **THEN** 系统删除配置并切换到其他配置

#### Scenario: 保护最后配置
- **WHEN** 用户尝试删除某类型的最后一个配置
- **THEN** 系统禁用删除按钮，提示必须至少保留一个配置

#### Scenario: 表单编辑
- **WHEN** 用户在 ExecutorConfigForm 中修改字段
- **THEN** 系统实时更新配置

#### Scenario: JSON 编辑
- **WHEN** 用户在 JSON 编辑器中修改配置
- **THEN** 系统验证 JSON 格式并更新配置

#### Scenario: 显示配置路径
- **WHEN** 用户查看代理配置
- **THEN** 系统显示配置文件的保存路径（从 `/api/proxy/info` 获取）

#### Scenario: 保存代理配置
- **WHEN** 用户保存代理配置
- **THEN** 系统调用 `PUT /api/proxy/profiles` 保存配置

---

### Requirement: MCP Server Configuration
系统 SHALL 提供 MCP 服务器配置页面，允许用户为代理配置模型上下文协议服务器。数据通过 `/api/proxy/mcp-config` 操作。

#### Scenario: 选择代理
- **WHEN** 用户在 MCP 配置页面选择一个代理
- **THEN** 系统调用 `GET /api/proxy/mcp-config?executor={type}` 加载并显示该代理的 MCP 配置

#### Scenario: MCP 不支持警告
- **WHEN** 用户选择的执行器不支持 MCP
- **THEN** 系统显示警告提示框

#### Scenario: 编辑 MCP 配置
- **WHEN** 用户在 JSON 编辑器中修改 MCP 服务器配置
- **THEN** 系统实时验证 JSON 语法

#### Scenario: 保存 MCP 配置
- **WHEN** 用户保存 MCP 配置
- **THEN** 系统调用 `POST /api/proxy/mcp-config?executor={type}` 保存配置并显示保存路径提示

#### Scenario: 浏览热门服务器
- **WHEN** 用户查看热门服务器轮播
- **THEN** 系统显示预配置的 MCP 服务器卡片

#### Scenario: 添加热门服务器
- **WHEN** 用户点击热门服务器卡片
- **THEN** 系统将该服务器配置添加到 JSON 编辑器

---

### Requirement: Settings API - Proxy
系统 SHALL 通过代理 API 访问 Vibe-Kanban 的设置数据。

#### Scenario: 用户配置 API
- **WHEN** 客户端发送 `GET /api/proxy/info`
- **THEN** 系统返回用户系统信息（包含配置、profiles、capabilities）

#### Scenario: 更新用户配置 API
- **WHEN** 客户端发送 `PUT /api/proxy/config`
- **THEN** 系统更新用户配置到 Vibe-Kanban

#### Scenario: 标签 API
- **WHEN** 客户端发送 `GET/POST/PUT/DELETE /api/proxy/tags`
- **THEN** 系统执行相应的标签操作

#### Scenario: 项目 API
- **WHEN** 客户端发送 `GET/POST/PUT/DELETE /api/proxy/projects`
- **THEN** 系统执行相应的项目操作

#### Scenario: 项目仓库 API
- **WHEN** 客户端发送 `GET/POST/DELETE /api/proxy/projects/:id/repositories`
- **THEN** 系统管理项目与仓库的关联关系

#### Scenario: 仓库 API
- **WHEN** 客户端发送 `GET/PUT /api/proxy/repos`
- **THEN** 系统获取或更新仓库配置

#### Scenario: 代理配置 API
- **WHEN** 客户端发送 `GET/PUT /api/proxy/profiles`
- **THEN** 系统获取或保存代理 profiles

#### Scenario: 代理可用性 API
- **WHEN** 客户端发送 `GET /api/proxy/agents/check-availability?executor={type}`
- **THEN** 系统返回指定代理类型的可用性状态

#### Scenario: MCP 配置 API
- **WHEN** 客户端发送 `GET/POST /api/proxy/mcp-config?executor={type}`
- **THEN** 系统获取或更新对应执行器的 MCP 配置

---

### Requirement: Settings API - AICodex Native
系统 SHALL 保留 AICodex 特有的配置 API。

#### Scenario: 获取所有配置
- **WHEN** 客户端发送 `GET /api/config`
- **THEN** 系统返回 Gitea、审核、队列配置

#### Scenario: 更新 Gitea 配置
- **WHEN** 客户端发送 `PUT /api/config/gitea`
- **THEN** 系统更新 Gitea 配置

#### Scenario: 测试 Gitea 连接
- **WHEN** 客户端发送 `POST /api/config/gitea/test`
- **THEN** 系统测试 Gitea 连接并返回结果

#### Scenario: 更新审核配置
- **WHEN** 客户端发送 `PUT /api/config/review`
- **THEN** 系统更新审核配置

#### Scenario: 更新队列配置
- **WHEN** 客户端发送 `PUT /api/config/queue`
- **THEN** 系统更新队列配置
