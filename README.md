# MediaDrivePro

基于对象存储的网盘系统，同时提供 WebDAV、HTTP API、图床、多人同步观影等媒体能力。

## 功能

- **网盘** — 文件上传/下载/管理，目录结构，分享链接
- **WebDAV** — 挂载为本地磁盘（Windows/macOS/Linux/移动端）
- **图床** — 上传图片自动压缩转 WebP，返回 CDN 链接
- **同步观影** — 多人实时同步播放室，聊天弹幕
- **媒体库** — 视频转码 HLS，TMDB 刮削，字幕支持

## 技术栈

| 层 | 技术 |
|---|---|
| 后端 | Rust + Axum + Tokio |
| ORM | SeaORM |
| 数据库 | SQLite（默认）/ PostgreSQL |
| 对象存储 | OpenDAL（本地文件系统 / S3 / MinIO / OSS / COS） |
| 前端 | React + TailwindCSS |

## 快速开始

### 前置要求

- Rust 1.75+

### 运行

```bash
git clone https://github.com/yourname/MediaDrivePro.git
cd MediaDrivePro
cargo run
```

默认使用 SQLite，无需安装任何数据库，启动后自动建表。

服务监听 `http://localhost:8080`。

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
backend = "fs"  # fs | s3 | minio

[storage.fs]
root = "./uploads"

[auth]
jwt_secret = "change-me-in-production"
access_token_ttl_secs = 1800
```

支持环境变量覆盖（`MDP_` 前缀）：

```bash
MDP_DATABASE__URL=postgres://... cargo run
```

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
DELETE /api/v1/files/:id          — 删除文件
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

### API Token

```
POST   /api/v1/tokens      — 创建 API Token（返回明文，仅此一次）
GET    /api/v1/tokens       — 列出所有 Token
DELETE /api/v1/tokens/:id   — 删除 Token
```

### 分片上传

```
POST   /api/v1/files/multipart/init                  — 初始化分片上传
PUT    /api/v1/files/multipart/:upload_id/:part_num   — 上传分片
POST   /api/v1/files/multipart/:upload_id/complete    — 完成上传（合并分片）
DELETE /api/v1/files/multipart/:upload_id             — 取消上传
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
├── src/main.rs           # 入口
├── config.toml           # 配置文件
├── migration/            # 数据库迁移
└── crates/
    ├── common/           # 配置、错误、响应类型
    ├── auth/             # JWT、密码哈希、认证中间件
    ├── storage/          # OpenDAL 存储封装
    ├── core/             # 业务逻辑（用户/文件/目录/Token/分片上传 Service）
    ├── api/              # HTTP 路由和处理器
    └── webdav/           # WebDAV 协议实现（dav-server + Basic Auth）
```

## 路线图

- [x] V0.1 — 基础骨架（用户认证 + 文件上传下载 + 目录管理）
- [x] V0.2 — 网盘完善（重命名/移动、搜索排序、分享、存储配额）
- [x] V0.3 — WebDAV + 分片上传 + API Token
- [ ] V1.0 — Web UI + Docker 部署
- [ ] V1.1 — 图床
- [ ] V2.0 — 视频播放 + 转码 + 媒体库
- [ ] V3.0 — 同步观影室

## License

MIT
