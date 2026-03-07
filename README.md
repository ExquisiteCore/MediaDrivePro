# MediaDrive Pro

基于对象存储的私有云网盘系统，提供 Web UI、WebDAV、HTTP API、文件分享等功能。

## 功能

- **Web UI** — 现代化 Web 界面，文件浏览/上传/预览/管理
- **文件管理** — 文件上传/下载、目录结构、重命名/移动/删除
- **分片上传** — 大文件自动分片，断点续传，并行上传
- **文件预览** — 图片/视频/音频/PDF/文本 在线预览
- **文件分享** — 生成分享链接，支持密码保护、过期时间、下载次数限制
- **WebDAV** — 挂载为本地磁盘（Windows/macOS/Linux/移动端）
- **API Token** — 创建独立 Token 用于 WebDAV 或第三方集成
- **管理面板** — 管理员查看所有用户及存储使用

## 技术栈

| 层 | 技术 |
|---|---|
| 后端 | Rust + Axum + Tokio |
| ORM | SeaORM |
| 数据库 | SQLite（默认）/ PostgreSQL |
| 对象存储 | OpenDAL（本地文件系统 / S3 / MinIO / OSS / COS） |
| 前端 | React 19 + TypeScript + Vite + TailwindCSS v4 |
| 状态管理 | Zustand |
| 路由 | React Router v7 |
| 包管理 | pnpm |

## 快速开始

### 前置要求

- Rust 1.85+
- Node.js 20+
- pnpm

### 构建前端

```bash
cd web
pnpm install
pnpm build
```

### 启动服务

```bash
cargo run
```

默认使用 SQLite，无需安装任何数据库，启动后自动建表。

服务监听 `http://localhost:8080`，前端自动从 `web/dist/` 提供服务。

### 前端开发模式

```bash
# 终端 1：启动后端
cargo run

# 终端 2：启动前端开发服务器（带热更新）
cd web
pnpm dev
```

前端开发服务器监听 `http://localhost:5173`，API 请求自动代理到后端。

### 配置

编辑 `config.toml`：

```toml
[server]
host = "0.0.0.0"
port = 8080

[database]
url = "sqlite:./data/mediadrive.db?mode=rwc"  # 或 postgres://user:pass@localhost/mediadrive
max_connections = 5
auto_migrate = true

[storage]
backend = "fs"  # fs | s3

[storage.fs]
root = "./uploads"

[storage.s3]
bucket = "mediadrive"
region = "us-east-1"
endpoint = ""
access_key_id = ""
secret_access_key = ""

[auth]
jwt_secret = "change-me-in-production"
access_token_ttl_secs = 1800

[webdav]
enabled = true
prefix = "/dav"
```

支持环境变量覆盖（`MDP_` 前缀）：

```bash
MDP_DATABASE__URL=postgres://... MDP_AUTH__JWT_SECRET=your-secret cargo run
```

## Web UI

Web 界面包含以下页面：

| 页面 | 路径 | 说明 |
|---|---|---|
| 登录 | `/login` | 用户名 + 密码登录 |
| 注册 | `/register` | 创建新账号 |
| 文件浏览器 | `/files` | 文件/文件夹管理主页面 |
| 分享管理 | `/shares` | 查看和管理所有分享链接 |
| Token 管理 | `/tokens` | 创建和管理 API Token |
| 用户设置 | `/settings` | 个人信息和存储空间用量 |
| 管理员面板 | `/admin` | 用户列表（仅管理员） |
| 公开分享 | `/s/:token` | 访客查看/下载分享文件 |

### 文件浏览器功能

- 文件夹导航（面包屑路径）
- 拖拽上传（大文件自动分片，显示进度条）
- 右键上下文菜单（预览、下载、重命名、移动、分享、删除）
- 文件搜索和排序（名称/大小/日期）
- 在线预览（图片、视频、音频、PDF、文本/代码）

## API

所有接口以 `/api/v1/` 为前缀，需 Bearer Token 认证（除注册/登录外）。

### 认证

```
POST /api/v1/auth/register   — 注册
POST /api/v1/auth/login      — 登录
GET  /api/v1/auth/me         — 当前用户信息
```

### 文件

```
POST   /api/v1/files              — 上传文件（multipart/form-data）
GET    /api/v1/files              — 文件列表（?folder_id=&page=&per_page=&search=&sort=&order=）
GET    /api/v1/files/:id          — 文件详情
PUT    /api/v1/files/:id          — 重命名/移动文件
GET    /api/v1/files/:id/download — 下载文件
GET    /api/v1/files/:id/preview  — 预览文件（浏览器内联展示）
DELETE /api/v1/files/:id          — 删除文件
```

### 分片上传

```
POST   /api/v1/files/multipart/init                  — 初始化分片上传
PUT    /api/v1/files/multipart/:upload_id/:part_num   — 上传分片
POST   /api/v1/files/multipart/:upload_id/complete    — 完成上传（合并分片）
DELETE /api/v1/files/multipart/:upload_id             — 取消上传
```

### 目录

```
POST   /api/v1/folders              — 创建目录
GET    /api/v1/folders              — 根目录内容
GET    /api/v1/folders/:id          — 目录详情
PUT    /api/v1/folders/:id          — 重命名/移动目录
GET    /api/v1/folders/:id/children — 子目录和文件
DELETE /api/v1/folders/:id          — 删除目录
```

### 分享

```
POST   /api/v1/shares         — 创建分享（可设密码、过期时间、下载次数）
GET    /api/v1/shares         — 列出我的分享
DELETE /api/v1/shares/:id     — 取消分享
GET    /s/:token              — 公开访问分享（无需认证）
POST   /s/:token/verify       — 验证提取码
GET    /s/:token/download     — 通过分享下载文件
```

### API Token

```
POST   /api/v1/tokens       — 创建 API Token（返回明文，仅此一次）
GET    /api/v1/tokens        — 列出所有 Token
DELETE /api/v1/tokens/:id    — 删除 Token
```

### 管理员

```
GET    /api/v1/admin/users   — 列出所有用户（仅管理员）
```

### WebDAV

通过 WebDAV 协议挂载为本地磁盘，使用 API Token 进行 HTTP Basic Auth 认证。

```
挂载地址: http://localhost:8080/dav/
用户名:   <注册的用户名>
密码:     <API Token 明文>
```

支持操作：PROPFIND、GET、PUT、MKCOL、DELETE、MOVE、COPY、OPTIONS

### 使用示例

```bash
# 注册
curl -X POST http://localhost:8080/api/v1/auth/register \
  -H "Content-Type: application/json" \
  -d '{"username":"demo","email":"demo@example.com","password":"123456"}'

# 登录
curl -X POST http://localhost:8080/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username":"demo","password":"123456"}'

# 上传文件（用登录返回的 token）
curl -X POST http://localhost:8080/api/v1/files \
  -H "Authorization: Bearer <token>" \
  -F "file=@photo.jpg"

# 文件列表
curl http://localhost:8080/api/v1/files \
  -H "Authorization: Bearer <token>"
```

## 项目结构

```
MediaDrivePro/
├── src/main.rs               # 入口（服务启动、WebDAV 挂载、SPA 静态文件）
├── config.toml               # 配置文件
├── migration/                 # 数据库迁移
├── crates/
│   ├── common/               # 配置、错误类型、响应格式
│   ├── auth/                 # JWT、密码哈希、认证中间件
│   ├── storage/              # OpenDAL 存储封装
│   ├── core/                 # 业务逻辑（用户/文件/目录/分享/Token/分片上传）
│   ├── api/                  # HTTP 路由和处理器
│   └── webdav/               # WebDAV 协议实现（dav-server + Basic Auth）
└── web/                      # 前端（React + Vite + TypeScript）
    ├── src/
    │   ├── api/              # API 客户端封装
    │   ├── components/       # UI 组件
    │   ├── pages/            # 页面
    │   ├── store/            # Zustand 状态管理
    │   └── lib/              # 工具函数
    ├── package.json
    └── vite.config.ts
```

## 路线图

- [x] V0.1 — 基础骨架（用户认证 + 文件上传下载 + 目录管理）
- [x] V0.2 — 网盘完善（重命名/移动、搜索排序、分享、存储配额）
- [x] V0.3 — WebDAV + 分片上传 + API Token
- [x] V1.0 — Web UI
- [ ] V1.1 — 图床
- [ ] V2.0 — 视频播放 + 转码 + 媒体库
- [ ] V3.0 — 同步观影室

## License

MIT
