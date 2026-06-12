# WMS 仓库管理系统

WMS 是一个基于 Rust、React 和 PostgreSQL 的仓库管理系统 MVP，面向中小型仓库的基础库存管理场景。

## 功能概览

当前已完成基础设施能力：

- Rust 后端服务骨架
- PostgreSQL 16 本地开发环境
- SQLx 数据库迁移
- 统一后端响应格式
- 请求追踪、请求 ID、CORS 中间件
- React + Vite 前端应用骨架
- Ant Design 全局布局
- React Router SPA 路由
- Axios API 客户端封装

计划实现的业务模块：

- 商品管理
- 仓库管理
- 库位管理
- 入库管理
- 出库管理
- 库存查询

## 技术栈

后端：

- Rust
- Axum
- Tokio
- SQLx
- Serde
- tracing
- tower-http
- dotenvy

前端：

- React 18
- Vite 5
- TypeScript
- Ant Design 5
- React Router 6
- Axios
- TanStack Query

数据库：

- PostgreSQL 16
- SQLx migrations

## 项目结构

```text
.
├── backend/             # Rust API 服务
├── frontend/            # React 前端应用
├── docker-compose.yml   # 本地 PostgreSQL 服务
├── README.md
└── .gitignore
```

## 数据库设计

MVP 阶段数据库包含 9 张表：

- `products`
- `warehouses`
- `locations`
- `inventories`
- `inbound_orders`
- `inbound_order_items`
- `outbound_orders`
- `outbound_order_items`
- `inventory_transactions`

MVP 阶段不包含：

- 商品分类
- 用户认证
- RBAC 权限
- 审批流
- 锁定库存数量
- 库存流水前端页面

## 本地运行

### 1. 启动 PostgreSQL

```bash
docker compose up -d
```

PostgreSQL 默认运行在：

```text
localhost:5432
```

本地默认配置：

```text
POSTGRES_USER=wms
POSTGRES_PASSWORD=wms123
POSTGRES_DB=wms
```

### 2. 配置后端环境变量

创建 `backend/.env`：

```env
DATABASE_URL=postgres://wms:wms123@localhost:5432/wms
SERVER_PORT=3000
```

### 3. 执行数据库迁移

在 `backend/` 目录执行：

```bash
sqlx migrate run
```

### 4. 启动后端

```bash
cargo run --manifest-path backend/Cargo.toml
```

健康检查：

```bash
curl http://localhost:3000/api/v1/health
```

预期响应：

```json
{"code":0,"data":null,"message":"ok"}
```

### 5. 启动前端

```bash
npm --prefix frontend install
npm --prefix frontend run dev
```

前端开发服务默认地址：

```text
http://localhost:5173
```

## 构建

后端：

```bash
cargo build --manifest-path backend/Cargo.toml
```

前端：

```bash
npm --prefix frontend run build
```

## API 状态

当前后端基础设施已完成，业务路由已预留，但具体业务 API 会按模块逐步实现。

已实现：

- `GET /api/v1/health`

已预留的业务路由模块：

- 商品
- 仓库
- 库位
- 入库单
- 出库单
- 库存

## 开发状态

当前项目已完成基础设施搭建，正在进入业务模块开发阶段。

## License

暂未指定许可证。
