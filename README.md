# MediaDrive Pro

基于对象存储的私有云网盘系统，提供 Web UI、WebDAV、HTTP API、图床、视频转码播放、媒体识别、同步观影室等功能。

## 功能

- **Web UI** — 现代化 Web 界面，文件浏览/上传/预览/管理
- **文件管理** — 文件上传/下载、目录结构、重命名/移动/删除
- **分片上传** — 大文件自动分片，断点续传，并行上传
- **文件预览** — 图片/视频/音频/PDF/文本 在线预览
- **文件分享** — 生成分享链接，支持密码保护、过期时间、下载次数限制
- **图床** — 图片上传自动压缩为 WebP + 生成缩略图，返回 URL/Markdown 链接，支持防盗链，兼容 PicGo
- **视频转码** — FFmpeg 后台转码为 HLS，支持 480p/720p/1080p 多档位，自动重试
- **HLS 播放** — HLS.js 流媒体播放，支持字幕轨道，Safari 原生兼容
- **媒体识别** — 智能文件名解析（剧集/电影/动漫）+ TMDB 刮削（海报/简介/年份）
- **同步观影室** — 多人实时同步观影，WebSocket 房间、播放同步 + 延迟补偿、聊天、弹幕
- **WebDAV** — 挂载为本地磁盘（Windows/macOS/Linux/移动端）
- **API Token** — 创建独立 Token 用于 WebDAV、图床或第三方集成
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

### Docker 部署（推荐）

```bash
docker pull ghcr.io/exquisitecore/mediadrivepro:1.2
```

```bash
docker run -d \
  --name mediadrivepro \
  -p 8080:8080 \
  -v mdp-data:/app/data \
  -v mdp-uploads:/app/uploads \
  ghcr.io/exquisitecore/mediadrivepro:1.2
```

启动后访问 `http://localhost:8080`。

**自定义配置**：挂载 `config.toml` 并通过环境变量覆盖：

```bash
docker run -d \
  --name mediadrivepro \
  -p 8080:8080 \
  -v mdp-data:/app/data \
  -v mdp-uploads:/app/uploads \
  -v ./config.toml:/app/config.toml \
  -e MDP_AUTH__JWT_SECRET=your-secret \
  ghcr.io/exquisitecore/mediadrivepro:1.2
```

**Docker Compose**：

```yaml
services:
  mediadrivepro:
    image: ghcr.io/exquisitecore/mediadrivepro:1.2
    ports:
      - "8080:8080"
    volumes:
      - mdp-data:/app/data
      - mdp-uploads:/app/uploads
      # - ./config.toml:/app/config.toml
    environment:
      - MDP_AUTH__JWT_SECRET=change-me-in-production
    restart: unless-stopped

volumes:
  mdp-data:
  mdp-uploads:
```

```bash
docker compose up -d
```

### 从源码构建

**前置要求：**

- Rust 1.85+
- Node.js 20+
- pnpm
- FFmpeg + ffprobe（视频转码需要，[下载](https://ffmpeg.org/download.html)）

```bash
# 构建前端
cd web && pnpm install && pnpm build && cd ..

# 启动服务
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
log_level = "info,sqlx::query=warn"  # 日志等级（RUST_LOG 语法）

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

[image]
max_upload_size = 20971520   # 20MB
compression_quality = 80     # WebP 压缩质量 (1-100)
cdn_base_url = ""            # 留空则使用服务器自身地址
allowed_referers = []        # 防盗链白名单，留空则不限制

[video]
ffmpeg_path = "ffmpeg"       # FFmpeg 可执行文件路径
ffprobe_path = "ffprobe"     # ffprobe 可执行文件路径
max_concurrent = 1           # 最大并行转码数
default_profiles = ["720p"]  # 默认转码档位
poll_interval_secs = 5       # 任务轮询间隔（秒）

[tmdb]
api_key = ""                 # TMDB API Key（留空则禁用媒体刮削）
language = "zh-CN"           # TMDB 搜索语言
```

支持环境变量覆盖（`MDP_` 前缀）：

```bash
MDP_DATABASE__URL=postgres://... MDP_AUTH__JWT_SECRET=your-secret cargo run
```

日志等级也可用 `RUST_LOG` 环境变量覆盖（优先级高于 config.toml）：

```bash
# 调试 SQL 查询
RUST_LOG=info,sqlx::query=info cargo run

# 只看警告和错误
RUST_LOG=warn cargo run

# 调试全部
RUST_LOG=debug cargo run
```

## Web UI

Web 界面包含以下页面：

| 页面 | 路径 | 说明 |
|---|---|---|
| 登录 | `/login` | 用户名 + 密码登录 |
| 注册 | `/register` | 创建新账号（多步骤向导 + 头像设置） |
| 文件浏览器 | `/files` | 文件/文件夹管理主页面 |
| 分享管理 | `/shares` | 查看和管理所有分享链接 |
| 图床 | `/images` | 图片上传/管理，一键复制 URL/Markdown |
| 观影室 | `/rooms` | 创建/加入同步观影房间 |
| 观影室播放 | `/rooms/:id` | 同步播放 + 聊天 + 弹幕 |
| Token 管理 | `/tokens` | 创建和管理 API Token |
| 用户设置 | `/settings` | 个人信息、头像和存储空间用量 |
| 管理员面板 | `/admin` | 用户列表（仅管理员） |
| 公开分享 | `/s/:token` | 访客查看/下载分享文件 |

### 文件浏览器功能

- 文件夹导航（面包屑路径）
- 拖拽上传（大文件自动分片，显示进度条）
- 右键上下文菜单（预览、下载、重命名、移动、分享、删除）
- 文件搜索和排序（名称/大小/日期）
- 在线预览（图片、视频、音频、PDF、文本/代码）

## API

所有接口以 `/api/v1/` 为前缀，需 Bearer Token 认证（除注册/登录外）。也支持 `X-API-Token: username:token` 头认证（用于 PicGo 等客户端）。

### 认证

```
POST /api/v1/auth/register   — 注册
POST /api/v1/auth/login      — 登录
GET  /api/v1/auth/me         — 当前用户信息
POST /api/v1/auth/avatar     — 上传头像
GET  /api/v1/users/:id/avatar — 获取用户头像
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

### 图床

```
POST   /api/v1/images       — 上传图片（自动压缩为 WebP + 生成缩略图）
GET    /api/v1/images        — 列出已上传图片
DELETE /api/v1/images/:id    — 删除图片
GET    /img/:hash.webp       — 公开访问图片（可配置防盗链）
GET    /img/thumb/:hash.webp — 缩略图
```

上传响应包含 `url`、`thumb_url`、`markdown` 字段，方便直接使用。

### 视频转码

```
POST /api/v1/transcode            — 创建转码任务（{ file_id, profile? }）
GET  /api/v1/transcode/:id        — 查询转码任务状态
GET  /api/v1/transcode?file_id=   — 列出文件的所有转码任务
```

转码档位：`480p`（1Mbps）、`720p`（2.5Mbps）、`1080p`（5Mbps），默认 `720p`。

### HLS 流媒体

```
GET /api/v1/files/:file_id/stream/:path — HLS 流（m3u8 播放列表 + ts 分片 + vtt 字幕）
```

转码完成后，前端自动通过 HLS.js 播放。支持的输入格式：mp4、webm、ogg、mkv、avi、mov、flv、mpeg。

### 媒体识别

```
GET  /api/v1/media/:file_id       — 获取媒体信息
POST /api/v1/media/:file_id/scan  — 手动触发媒体识别（文件名解析 + TMDB 刮削）
```

文件名解析支持常见格式：
- 剧集：`Title.S01E03.1080p.BluRay.mkv`
- 动漫：`[字幕组] 标题 - 03 (1080p).mkv`
- 电影：`Movie.Name.2024.720p.mkv`

### 同步观影室

```
POST   /api/v1/rooms              — 创建房间（{ name, max_members? }）
GET    /api/v1/rooms              — 列出我的房间
GET    /api/v1/rooms/:id          — 房间详情（含成员列表）
DELETE /api/v1/rooms/:id          — 关闭房间（仅房主）
POST   /api/v1/rooms/join         — 通过邀请码加入（{ invite_code }）
POST   /api/v1/rooms/:id/play     — 设置播放文件（仅房主，{ file_id }）
GET    /api/v1/rooms/:id/members  — 成员列表
WS     /api/v1/rooms/:id/ws?token=— WebSocket 实时同步
```

WebSocket 消息：
- **客户端→服务端**：`play`、`pause`、`seek { time }`、`chat { content }`、`danmaku { content, color?, position? }`、`ping`
- **服务端→客户端**：`sync { status, time, file_id, server_time }`、`member_join`、`member_leave`、`chat`、`danmaku`、`pong`、`error`

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

# 上传图片到图床（使用 JWT）
curl -X POST http://localhost:8080/api/v1/images \
  -H "Authorization: Bearer <token>" \
  -F "image=@photo.jpg"

# 上传图片到图床（使用 API Token，适用于 PicGo 等客户端）
curl -X POST http://localhost:8080/api/v1/images \
  -H "X-API-Token: demo:<api_token>" \
  -F "image=@photo.jpg"

# 创建视频转码任务
curl -X POST http://localhost:8080/api/v1/transcode \
  -H "Authorization: Bearer <token>" \
  -H "Content-Type: application/json" \
  -d '{"file_id": "<uuid>", "profile": "720p"}'

# 查询转码状态
curl http://localhost:8080/api/v1/transcode/<task_id> \
  -H "Authorization: Bearer <token>"

# 触发媒体识别
curl -X POST http://localhost:8080/api/v1/media/<file_id>/scan \
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
│   ├── core/                 # 业务逻辑（用户/文件/目录/分享/Token/图床/转码/媒体识别/观影室）
│   ├── api/                  # HTTP 路由和处理器（含 WebSocket）
│   └── webdav/               # WebDAV 协议实现（dav-server + Basic Auth）
└── web/                      # 前端（React + Vite + TypeScript）
    ├── src/
    │   ├── api/              # API 客户端封装
    │   ├── components/       # UI 组件（含弹幕、聊天、同步播放器）
    │   ├── pages/            # 页面
    │   ├── hooks/            # 自定义 Hooks（WebSocket 等）
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
- [x] V1.1 — 图床（WebP 压缩 + 缩略图 + 防盗链 + PicGo 兼容）
- [x] V1.2 — 视频转码（FFmpeg HLS）+ 流媒体播放（HLS.js）+ 媒体识别（TMDB 刮削）
- [x] V1.3 — 同步观影室（WebSocket 房间、播放同步 + 延迟补偿、聊天、弹幕）

## License

MIT
