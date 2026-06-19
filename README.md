# WMS 仓库管理系统

一个面向中小型仓库的 MVP 仓储管理系统，覆盖商品、仓库、库位、入库、出库与库存流水的基础闭环。

## 已实现功能

- 商品管理：新增、查询、编辑、启用/禁用、SKU 唯一性校验。
- 仓库与库位管理：维护仓库、普通库位、收货库位和发货库位。
- 入库流程：创建入库单、维护明细、完成入库、取消已完成单据并回滚库存。
- 出库流程：创建出库单、维护明细、完成出库、库存不足拦截、取消已完成单据并回滚库存。
- 库存查询：按商品、仓库、库位查看现存量与库存变动记录。
- 种子数据：空数据库首次启动时自动初始化 5 个商品、1 个仓库和 4 个库位；也可手动重复执行。

## 技术栈

| 层级 | 技术 |
| --- | --- |
| 后端 | Rust、Axum、SQLx、PostgreSQL |
| 前端 | React、TypeScript、Vite、Ant Design |
| 数据库 | PostgreSQL 16 |
| 本地基础设施 | Docker Compose |

## 快速开始

### 运行环境

安装以下工具后即可在 Windows、macOS 或 Linux 上运行：

- Docker Desktop（含 Docker Compose）
- Rust stable 工具链（建议通过 `rustup` 安装）
- Node.js 18 或更高版本（建议 Node.js 20 LTS）和 npm
- 可选：Nginx，用于以一个地址同时提供前端页面和后端 API

### 1. 启动 PostgreSQL

在项目根目录运行：

```bash
docker compose up -d
docker compose ps
```

默认开发数据库监听 `localhost:5432`，测试数据库监听 `localhost:5433`。首次启动会创建对应的 Docker 数据卷。

### 2. 配置并启动后端

后端从 `backend/.env` 读取配置。本地默认配置如下：

```dotenv
DATABASE_URL=postgres://wms:wms123@localhost:5432/wms
SERVER_PORT=3000
```

启动后端：

```bash
cd backend
cargo run
```

服务启动时会自动执行待执行的数据库迁移；当商品、仓库和库位三张主数据表均为空时，会自动导入种子数据。默认 API 地址为 `http://localhost:3000`。

用健康检查确认服务已就绪：

```bash
curl http://localhost:3000/api/v1/health
```

在 Windows PowerShell 中，也可以使用：

```powershell
curl.exe http://localhost:3000/api/v1/health
```

### 3. 启动前端

另开一个终端，在项目根目录运行：

```bash
cd frontend
npm install
npm run dev
```

Vite 默认提供 `http://localhost:5173`。前端 API 地址使用相对路径 `/api/v1`；因此浏览器访问前端时，需要由反向代理把 `/api/` 转发到后端，或者在本地 Vite 配置中增加同等的开发代理。下一节提供无需改动业务代码的 Nginx 配置。

## 通过 Nginx 访问完整系统

前端构建产物和 API 使用同一个域名时，页面可以直接访问后端接口，也避免浏览器跨域问题。

### 1. 构建前端

```bash
cd frontend
npm install
npm run build
```

构建产物位于 `frontend/dist`。

### 2. 启动生产模式后端

```bash
cd backend
cargo run --release
```

### 3. 配置 Nginx

将以下站点配置中的 `root` 改为本机 `frontend/dist` 的绝对路径，然后重载 Nginx：

```nginx
server {
    listen 80;
    server_name _;

    root /absolute/path/to/WMS/frontend/dist;
    index index.html;

    location /api/ {
        proxy_pass http://127.0.0.1:3000;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }

    location / {
        try_files $uri $uri/ /index.html;
    }
}
```

随后通过 `http://localhost`（或服务器域名）访问系统。

## 种子数据

空数据库启动后会自动初始化以下可直接用于演示和联调的数据：

| 类型 | 数据 |
| --- | --- |
| 商品 | `SKU-0001` 至 `SKU-0005`，共 5 条 |
| 仓库 | `WH-MAIN`，共 1 条 |
| 库位 | `A-01-01`、`A-01-02`、`RECV-01`、`SHIP-01`，共 4 条 |

也可以在后端目录手动执行种子数据导入：

```bash
cargo run -- --seed
```

该命令可重复运行。种子 SQL 使用唯一约束冲突忽略策略，不会重复插入现有的种子记录。

## 推荐业务操作顺序

1. 确认商品、仓库和库位已存在；新数据库可直接使用种子数据。
2. 创建入库单，填写商品、目标库位和实收数量，然后完成入库。
3. 在库存查询中确认入库库存与流水。
4. 创建出库单并完成出库；系统会在库存不足时拒绝完成操作。
5. 对已完成的入库单或出库单执行取消时，系统会生成反向库存流水并回滚对应库存。

## API 概览

所有接口使用 `/api/v1` 前缀。

| 领域 | 主要路径 |
| --- | --- |
| 健康检查 | `GET /health` |
| 商品 | `/products`、`/products/{id}` |
| 仓库与库位 | `/warehouses`、`/warehouses/{id}/locations`、`/locations/{id}` |
| 入库 | `/inbound-orders`、`/inbound-orders/{id}/complete`、`/inbound-orders/{id}/cancel` |
| 出库 | `/outbound-orders`、`/outbound-orders/{id}/complete`、`/outbound-orders/{id}/cancel` |
| 库存 | `/inventories`、`/inventory-transactions` |

前端页面已覆盖主数据、入库单、出库单与库存查询等日常操作。

## 验证与测试

启动 Docker 数据库服务后，在后端目录执行：

```bash
cargo fmt --check
cargo test
```

集成测试使用 `TEST_DATABASE_URL` 指向独立的测试数据库，默认对应 Docker Compose 的 `localhost:5433/wms_test`。测试会自动迁移测试库并使用隔离数据执行。

在前端目录执行构建验证：

```bash
npm run build
```

## 配置说明

| 配置项 | 默认值 | 说明 |
| --- | --- | --- |
| `DATABASE_URL` | `postgres://wms:wms123@localhost:5432/wms` | 后端 PostgreSQL 连接地址 |
| `SERVER_PORT` | `3000` | 后端 HTTP 监听端口 |
| `TEST_DATABASE_URL` | `postgres://wms:wms123@localhost:5433/wms_test` | 集成测试数据库连接地址 |

部署到共享环境时，应替换默认数据库密码，限制数据库端口暴露范围，并通过 HTTPS 终止 TLS。当前 MVP 未提供登录、用户身份认证和权限控制；对外部署前应由网关或后续业务模块补足访问控制。

## 常见问题

### 后端无法连接数据库

先确认 `docker compose ps` 中 `postgres` 状态为运行中，再核对 `backend/.env` 的 `DATABASE_URL` 是否使用了正确的端口、用户名、密码和数据库名。

### 页面能够打开，但列表请求失败

确认后端运行在 `3000` 端口，并确认浏览器访问的站点已将 `/api/` 反向代理到 `http://127.0.0.1:3000`。直接运行 Vite 开发服务器不会自动转发 API 请求。

### 需要重新开始本地数据

停止数据库服务并删除 Docker 数据卷后重新启动：

```bash
docker compose down -v
docker compose up -d
```

这会删除本地开发和测试数据库中的全部数据；下次后端启动时会重新执行迁移并初始化种子数据。
