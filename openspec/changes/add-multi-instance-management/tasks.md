# Tasks: Multi-Instance Vibe-Kanban Management

## 1. 数据库迁移

- [x] 1.1 创建 `users` 表迁移文件
- [x] 1.2 创建 `user_sessions` 表迁移文件
- [x] 1.3 创建 `vibe_instances` 表迁移文件
- [x] 1.4 创建 `user_instance_assignments` 表迁移文件（多对多关系）
- [x] 1.5 创建 `instance_ai_agents` 表迁移文件
- [x] 1.6 创建 `instance_usage_stats` 表迁移文件
- [x] 1.7 添加默认管理员用户迁移（通过 create-admin 工具实现）
- [x] 1.8 测试迁移和回滚

## 2. 用户管理服务 (Rust)

- [x] 2.1 创建 `UserManager` 结构体
  - 用户 CRUD 操作
  - 密码哈希（Argon2id）
- [x] 2.2 实现用户注册逻辑
- [x] 2.3 实现用户认证（登录/登出）
  - 登录返回用户信息、实例列表、当前实例
- [x] 2.4 实现会话管理
  - JWT Token 生成和验证
  - 会话过期处理
- [x] 2.5 实现用户-实例分配（多对多）
  - 分配用户到多个实例
  - 取消分配
  - 检查实例用户数限制
- [x] 2.6 实现实例切换
  - 验证用户有权访问目标实例
  - 更新 current_instance_id
  - 自动启动实例
- [x] 2.7 实现认证中间件
- [ ] 2.8 单元测试

## 3. 实例管理服务 (Rust)

- [x] 3.1 创建 `InstanceManager` 结构体
  - 实例 CRUD 操作
  - 进程生命周期管理
  - 端口分配器
- [x] 3.2 实现实例启动逻辑
  - 环境变量注入
  - 数据目录创建
  - 进程启动和监控
- [x] 3.3 实现实例停止逻辑
  - 优雅关闭
  - 强制终止超时处理
- [x] 3.4 实现健康检查
  - HTTP 健康端点探测
  - 自动重启策略
- [x] 3.5 实现实例状态持久化
  - 重启后恢复运行中实例
- [x] 3.6 实现获取实例用户列表
- [ ] 3.7 单元测试

## 4. AI 智能体隔离服务

- [x] 4.1 创建 `AgentConfigManager` 结构体
- [x] 4.2 实现 API Key 加密存储
- [x] 4.3 实现 Claude Code 配置生成
- [x] 4.4 实现 Codex CLI 配置生成
- [x] 4.5 实现 Gemini CLI 配置生成
- [x] 4.6 实现 OpenCode 配置生成
- [x] 4.7 实现配置热更新
- [ ] 4.8 单元测试

## 5. API 端点 - 用户认证 (Rust)

- [x] 5.1 实现 `POST /api/auth/register` 用户注册
- [x] 5.2 实现 `POST /api/auth/login` 用户登录（返回实例列表和当前实例）
- [x] 5.3 实现 `POST /api/auth/logout` 用户登出
- [x] 5.4 实现 `GET /api/auth/me` 获取当前用户（含实例列表）
- [x] 5.5 实现 `PUT /api/auth/password` 修改密码

## 6. API 端点 - 用户管理 (Rust)

- [x] 6.1 实现 `GET /api/users` 列出用户（管理员）
- [x] 6.2 实现 `POST /api/users` 创建用户（管理员）
- [x] 6.3 实现 `GET /api/users/{id}` 获取用户详情（含分配的实例列表）
- [x] 6.4 实现 `PUT /api/users/{id}` 更新用户信息
- [x] 6.5 实现 `DELETE /api/users/{id}` 删除用户
- [x] 6.6 实现 `GET /api/users/{id}/instances` 获取用户分配的实例
- [x] 6.7 实现 `POST /api/users/{id}/instances` 分配用户到实例（支持多实例）
- [x] 6.8 实现 `DELETE /api/users/{id}/instances/{instance_id}` 取消实例分配
- [x] 6.9 实现 `PUT /api/users/{id}/activate` 激活/停用用户

## 7. API 端点 - 实例管理 (Rust)

- [x] 7.1 实现 `POST /api/instances` 创建实例
- [x] 7.2 实现 `GET /api/instances` 列出实例
- [x] 7.3 实现 `GET /api/instances/{id}` 获取详情（含用户列表）
- [x] 7.4 实现 `PUT /api/instances/{id}` 更新配置
- [x] 7.5 实现 `DELETE /api/instances/{id}` 删除实例
- [x] 7.6 实现 `POST /api/instances/{id}/start` 启动
- [x] 7.7 实现 `POST /api/instances/{id}/stop` 停止
- [x] 7.8 实现 `POST /api/instances/{id}/restart` 重启
- [x] 7.9 实现 `GET /api/instances/{id}/health` 健康检查
- [x] 7.10 实现 `GET /api/instances/{id}/users` 获取实例用户
- [ ] 7.11 实现 `GET /api/instances/{id}/logs` 获取实例日志
- [ ] 7.12 实现 `GET /api/instances/{id}/stats` 获取实例统计
- [x] 7.13 实现 AI 智能体配置 API
- [ ] 7.14 API 集成测试

## 7.5 API 端点 - 用户实例切换

- [x] 7.5.1 实现 `GET /api/my-instances` 获取当前用户的实例列表
- [x] 7.5.2 实现 `GET /api/my-instances/current` 获取当前选中的实例
- [x] 7.5.3 实现 `PUT /api/my-instances/current` 切换当前实例 ← 核心功能
- [x] 7.5.4 实现 `GET /api/my-instances/current/health` 当前实例健康检查

## 8. 代理路由修改

- [x] 8.1 修改 `proxy.rs` 基于用户 current_instance_id 路由
- [x] 8.2 添加认证中间件到代理路由
- [x] 8.3 实现自动启动实例逻辑
- [x] 8.4 验证用户有权访问当前实例
- [ ] 8.5 代理集成测试

## 9. 前端 - 用户认证 UI (Vue)

- [x] 9.1 创建 `LoginPage.vue` 登录页面
- [ ] 9.2 创建 `RegisterPage.vue` 注册页面（可选）
- [x] 9.3 实现认证状态管理 (`useAuth` composable)
- [x] 9.4 实现登录/登出逻辑（登录后获取实例列表）
- [x] 9.5 添加路由守卫（未登录跳转登录页）
- [x] 9.6 更新 `UserMenu.vue` 显示用户信息

## 9.5 前端 - 实例切换 UI (Vue)

- [x] 9.5.1 创建 `InstanceSelector.vue` 实例选择器组件
  - 下拉选择当前实例
  - 显示实例状态（运行中/已停止）
  - 切换时自动启动实例
- [x] 9.5.2 将 `InstanceSelector` 添加到顶部导航栏
- [x] 9.5.3 实现实例切换逻辑 (`useInstanceSwitcher` composable)
- [x] 9.5.4 切换实例后刷新当前页面数据

## 10. 前端 - 用户管理 UI (Vue) - 管理员

- [x] 10.1 创建 `UsersPage.vue` 用户列表页面
- [x] 10.2 创建 `UserCard.vue` 用户卡片组件
- [x] 10.3 创建 `CreateUserDialog.vue` 创建用户对话框
- [x] 10.4 创建 `AssignInstancesDialog.vue` 分配实例对话框（支持多选）
- [x] 10.5 实现用户激活/停用功能
- [x] 10.6 显示用户分配的所有实例

## 11. 前端 - 实例管理 UI (Vue) - 管理员

- [x] 11.1 创建 `InstancesPage.vue` 实例列表页面
- [x] 11.2 创建 `InstanceCard.vue` 实例卡片组件
- [x] 11.3 创建 `CreateInstanceDialog.vue` 创建对话框
- [x] 11.4 创建 `InstanceDetailPage.vue` 实例详情页（含用户列表）
- [ ] 11.5 实现实例状态实时更新（WebSocket 或轮询）

## 12. 前端 - AI 智能体配置 UI - 管理员

- [x] 12.1 创建 `AgentConfigPanel.vue` 智能体配置面板
- [x] 12.2 实现 Claude Code 配置表单
- [x] 12.3 实现 Codex CLI 配置表单
- [x] 12.4 实现 Gemini CLI 配置表单
- [x] 12.5 实现 OpenCode 配置表单
- [x] 12.6 实现连接测试功能
- [x] 12.7 实现配置保存和验证

## 13. 前端 - Composables & API

- [x] 13.1 添加 `useAuth` composable（含实例列表）
- [x] 13.2 添加 `useUsers` composable
- [x] 13.3 添加 `useInstances` composable
- [x] 13.4 添加 `useAgentConfig` composable
- [x] 13.5 添加 `useMyInstances` composable（用户实例切换）
- [x] 13.6 更新 `api.ts` 添加认证、用户管理和实例切换 API
- [x] 13.7 创建 `authStore` Pinia store（含当前实例）

## 14. 路由和导航

- [x] 14.1 添加 `/login` 路由
- [x] 14.2 添加 `/admin/users` 路由
- [x] 14.3 添加 `/admin/instances` 路由
- [x] 14.4 添加 `/admin/instances/:id` 路由
- [x] 14.5 更新导航菜单（区分管理员和普通用户）
- [x] 14.6 实现路由权限控制

## 15. 文档和测试

- [x] 15.1 更新 README 文档
- [x] 15.2 添加用户管理操作指南
- [x] 15.3 添加实例管理操作指南
- [x] 15.4 添加 AI 智能体配置指南
- [ ] 15.5 端到端测试
- [ ] 15.6 性能测试（多用户多实例场景）

## 16. 迁移和部署

- [x] 16.1 创建初始管理员创建脚本
- [x] 16.2 更新部署脚本
- [x] 16.3 更新 Docker Compose（如适用）
- [ ] 16.4 验证生产环境部署
