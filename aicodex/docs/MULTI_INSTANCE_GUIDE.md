# AICodex 多实例管理指南

## 概述

AICodex 支持管理多个 vibe-kanban 实例，每个实例可以独立运行并服务于不同的用户或团队。本指南介绍如何配置和使用多实例管理功能。

## 目录

- [快速开始](#快速开始)
- [用户管理](#用户管理)
- [实例管理](#实例管理)
- [AI 智能体配置](#ai-智能体配置)
- [部署指南](#部署指南)
- [API 参考](#api-参考)

## 快速开始

### 1. 环境准备

确保已安装以下依赖：
- Rust 1.75+
- Node.js 18+
- SQLite 3
- vibe-kanban 可执行文件

### 2. 配置环境变量

```bash
# 复制示例配置
cp docker/.env.aicodex.example docker/.env.aicodex

# 编辑配置文件
vim docker/.env.aicodex
```

重要配置项：

```bash
# vibe-kanban 可执行文件路径
VIBE_KANBAN_BINARY=/path/to/vibe-kanban

# 实例数据目录
VIBE_INSTANCES_DIR=/data/vibe-instances

# JWT 密钥（生产环境必须更改）
JWT_SECRET=your-secure-jwt-secret

# 配置加密密钥（用于加密 API Key）
CONFIG_ENCRYPTION_KEY=your-base64-encoded-32-bytes-key
```

### 3. 运行数据库迁移

```bash
cd aicodex
sqlx migrate run
```

### 4. 创建初始管理员

```bash
cargo run --bin create-admin -- \
  --username admin \
  --password your-secure-password \
  --email admin@example.com \
  --display-name "系统管理员"
```

### 5. 启动服务

```bash
# 开发模式
cargo run

# 或使用 Docker
docker-compose -f docker/docker-compose.aicodex.yml up -d
```

## 用户管理

### 用户角色

系统支持两种用户角色：

| 角色 | 权限 |
|------|------|
| `admin` | 完全管理权限，可管理用户、实例和配置 |
| `user` | 普通用户，只能访问分配的实例 |

### 管理用户

通过管理界面 (`/admin/users`) 可以：

1. **创建用户** - 点击"创建用户"按钮
2. **编辑用户** - 修改用户名、邮箱、显示名称
3. **启用/停用用户** - 临时禁止用户登录
4. **分配实例** - 将用户分配到一个或多个实例
5. **删除用户** - 永久删除用户账号

### 用户实例分配

每个用户可以被分配到多个实例：

- 用户只能访问已分配的实例
- 用户可以在已分配的实例之间切换
- 切换实例时，如果实例未运行，系统会自动启动

## 实例管理

### 创建实例

通过管理界面 (`/admin/instances`) 创建新实例：

1. 点击"创建实例"按钮
2. 填写实例信息：
   - **名称** - 实例显示名称
   - **描述** - 可选的描述信息
   - **端口** - 实例监听端口（推荐 18100-18199）
   - **最大用户数** - 可选的用户数限制
   - **自动启动** - 系统启动时是否自动启动实例

### 实例生命周期

| 状态 | 说明 |
|------|------|
| `stopped` | 已停止，未运行 |
| `starting` | 正在启动中 |
| `running` | 正在运行 |
| `stopping` | 正在停止中 |
| `error` | 发生错误 |

### 实例操作

- **启动** - 启动已停止的实例
- **停止** - 优雅停止运行中的实例
- **重启** - 重新启动实例
- **健康检查** - 检查实例运行状态

### 实例数据隔离

每个实例的数据存储在独立的目录中：

```
$VIBE_INSTANCES_DIR/
├── instance-uuid-1/
│   ├── data/
│   └── config/
├── instance-uuid-2/
│   ├── data/
│   └── config/
└── ...
```

## AI 智能体配置

每个实例可以独立配置 AI 智能体，支持的智能体类型：

### Claude Code

Anthropic Claude 代码智能体配置：

```json
{
  "is_enabled": true,
  "api_key": "sk-ant-...",
  "rate_limit_rpm": 60,
  "config_json": {
    "model": "claude-3-sonnet-20240229"
  }
}
```

### Codex CLI

OpenAI Codex 智能体配置：

```json
{
  "is_enabled": true,
  "api_key": "sk-...",
  "rate_limit_rpm": 60,
  "config_json": {
    "model": "gpt-4"
  }
}
```

### Gemini CLI

Google Gemini 智能体配置：

```json
{
  "is_enabled": true,
  "api_key": "...",
  "rate_limit_rpm": 60,
  "config_json": {
    "model": "gemini-pro"
  }
}
```

### OpenCode

开源代码智能体配置：

```json
{
  "is_enabled": true,
  "api_key": "...",
  "rate_limit_rpm": 60,
  "config_json": {}
}
```

### 配置说明

| 字段 | 类型 | 说明 |
|------|------|------|
| `is_enabled` | boolean | 是否启用此智能体 |
| `api_key` | string | API 密钥（加密存储） |
| `rate_limit_rpm` | number | 速率限制（请求/分钟） |
| `config_json` | object | 额外配置参数 |

## 部署指南

### Docker 部署

推荐使用 Docker Compose 部署：

```bash
# 设置环境变量
export JWT_SECRET=$(openssl rand -base64 32)
export CONFIG_ENCRYPTION_KEY=$(openssl rand -base64 32)

# 启动服务
docker-compose -f docker/docker-compose.aicodex.yml up -d

# 创建初始管理员
docker-compose exec aicodex /app/create-admin \
  --username admin \
  --password your-password
```

### 手动部署

1. 构建后端：
```bash
cd aicodex
cargo build --release
```

2. 构建前端：
```bash
cd aicodex-web
pnpm install
pnpm build
```

3. 配置 Nginx 反向代理（参考 `docker/nginx.aicodex.conf`）

4. 启动服务：
```bash
./target/release/aicodex
```

### 生产环境注意事项

1. **JWT 密钥** - 必须使用强随机密钥
2. **配置加密密钥** - 必须使用强随机密钥
3. **HTTPS** - 生产环境必须启用 HTTPS
4. **数据备份** - 定期备份数据库和实例数据
5. **日志监控** - 配置日志收集和监控告警

## API 参考

### 认证 API

| 端点 | 方法 | 说明 |
|------|------|------|
| `/api/auth/login` | POST | 用户登录 |
| `/api/auth/logout` | POST | 用户登出 |
| `/api/auth/me` | GET | 获取当前用户信息 |
| `/api/auth/password` | PUT | 修改密码 |

### 用户实例 API

| 端点 | 方法 | 说明 |
|------|------|------|
| `/api/my-instances` | GET | 获取用户可用实例列表 |
| `/api/my-instances/current` | GET | 获取当前实例 |
| `/api/my-instances/current` | PUT | 切换当前实例 |
| `/api/my-instances/current/health` | GET | 当前实例健康检查 |

### 用户管理 API（管理员）

| 端点 | 方法 | 说明 |
|------|------|------|
| `/api/users` | GET | 列出所有用户 |
| `/api/users` | POST | 创建用户 |
| `/api/users/{id}` | GET | 获取用户详情 |
| `/api/users/{id}` | PUT | 更新用户 |
| `/api/users/{id}` | DELETE | 删除用户 |
| `/api/users/{id}/activate` | PUT | 启用/停用用户 |
| `/api/users/{id}/instances` | GET | 获取用户实例 |
| `/api/users/{id}/instances` | POST | 分配实例 |
| `/api/users/{id}/instances/{instance_id}` | DELETE | 取消分配 |

### 实例管理 API（管理员）

| 端点 | 方法 | 说明 |
|------|------|------|
| `/api/instances` | GET | 列出所有实例 |
| `/api/instances` | POST | 创建实例 |
| `/api/instances/{id}` | GET | 获取实例详情 |
| `/api/instances/{id}` | PUT | 更新实例 |
| `/api/instances/{id}` | DELETE | 删除实例 |
| `/api/instances/{id}/start` | POST | 启动实例 |
| `/api/instances/{id}/stop` | POST | 停止实例 |
| `/api/instances/{id}/restart` | POST | 重启实例 |
| `/api/instances/{id}/health` | GET | 健康检查 |
| `/api/instances/{id}/users` | GET | 获取实例用户 |
| `/api/instances/{id}/agents/{type}` | PUT | 配置智能体 |
| `/api/instances/{id}/agents/{type}/test` | POST | 测试智能体连接 |

## 故障排除

### 实例无法启动

1. 检查端口是否被占用
2. 检查 vibe-kanban 可执行文件路径
3. 检查实例数据目录权限
4. 查看系统日志

### 用户无法登录

1. 检查用户是否被停用
2. 检查密码是否正确
3. 检查 JWT 密钥配置

### 实例健康检查失败

1. 检查实例是否正在运行
2. 检查网络连接
3. 检查实例日志

## 许可证

MIT License
