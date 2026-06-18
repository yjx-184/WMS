# TASK_BOARD — WMS MVP 任务看板（最终版）

> 生成日期：2026-06-10  
> 更新日期：2026-06-10（最终架构决议修订）  
> 总估算工时：约 54 小时（按单人全职约 2 周）  
> 组织形式：Epic → Feature → Task  
> 每个 Task 均独立可验收，粒度控制在 1~3 小时

---

## 依赖关系图

```
Epic 0 (基础设施)
  ├─► Epic 1 (商品管理) ──┐
  ├─► Epic 2 (仓库管理) ──┤
  └─► Epic 3 (库位管理) ──┤
                          ├─► Epic 4 (入库管理)
                          ├─► Epic 5 (出库管理)
                          └─► Epic 6 (库存查询)
                                   │
                                   ▼
                              Epic 7 (Testing)
                              Epic 8 (Seed Data)
```

> Epic 1/2/3 是主数据模块，互相独立可并行。Epic 4/5/6 依赖主数据全部完成。

---

## 数据库规模

**9 张表**（严禁新增）：

| # | 表名 | 说明 |
|---|------|------|
| 1 | `products` | 商品（无 category_id，无分类关联） |
| 2 | `warehouses` | 仓库 |
| 3 | `locations` | 库位 |
| 4 | `inventories` | 库存快照（无 locked_quantity） |
| 5 | `inbound_orders` | 入库单（status: draft/completed/cancelled） |
| 6 | `inbound_order_items` | 入库单明细 |
| 7 | `outbound_orders` | 出库单（status: draft/completed/cancelled） |
| 8 | `outbound_order_items` | 出库单明细 |
| 9 | `inventory_transactions` | 库存流水（change_type: inbound/outbound，后台记录，无前端页面） |

**已删除的表**：`product_categories`

---

## Epic 0：基础设施搭建

> 目标：可运行的后端服务 + 前端 SPA 骨架 + 数据库就绪  
> 产出：`cargo run` 启动 API 服务，`npm run dev` 启动前端，PostgreSQL 含 9 张表

## 当前执行状态

- 当前 Epic：Epic 5 — 出库管理
- 当前进度：4/7 Completed
- 最近通过 Review：TASK_BOARD.md / T5.1.4
- 已映射完成任务：T0.1.1、T0.1.2、T0.1.3、T0.2.1、T0.2.2、T0.3.1、T0.3.2、T0.3.3、T0.3.4、T0.3.5、T0.4.1、T0.4.2、T0.4.3、T1.1.1、T1.1.2、T1.2.1、T1.2.2、T2.1.1、T2.1.2、T2.2.1、T3.1.1、T3.1.2、T3.2.1、T4.1.1、T4.1.2、T4.1.3、T4.1.4、T4.2.1、T4.2.2、T4.2.3、T5.1.1、T5.1.2、T5.1.3、T5.1.4
- 下一任务：TASK_BOARD.md / T5.2.1 实现出库单列表页面
- 注意：T5.1.4 已通过 Review；TASK_BOARD.md / T5.2.1 依赖已满足，必须先生成派发包并经过 Review Gate，不得跳过进入后续任务

### Feature 0.1 — 项目脚手架

| ID | Status | Priority | Owner | Reviewer | 任务 | 工时 | 验收标准 | 依赖 |
|----|--------|----------|-------|----------|------|------|----------|------|
| T0.1.1 | Done | P0 | Unassigned | Codex | 初始化后端 Rust 项目 | 1h | `backend/` 目录含 Cargo.toml（Axum/Tokio/SQLx/Serde/tracing），`src/main.rs` 启动后监听 3000 端口，`/api/v1/health` 返回 `{"code":0}` | — |
| T0.1.2 | Done | P0 | Unassigned | Codex | 初始化前端 Vite + React 项目 | 1h | `frontend/` 目录含 package.json（React18/Vite5/Antd5/ReactRouter6/Axios/ReactQuery），`npm run dev` 正常启动 | — |
| T0.1.3 | Done | P0 | Unassigned | Codex | 配置开发环境与 docker-compose | 1h | 项目根 `docker-compose.yml` 含 PostgreSQL 16 服务，`.env` 含 `DATABASE_URL`，后端可连接数据库 | — |

### Feature 0.2 — 数据库迁移

| ID | Status | Priority | Owner | Reviewer | 任务 | 工时 | 验收标准 | 依赖 |
|----|--------|----------|-------|----------|------|------|----------|------|
| T0.2.1 | Done | P0 | Unassigned | Codex | 编写数据库迁移 SQL（9 张表） | 2h | `backend/migrations/` 生成迁移文件，仅含 9 张表。`products` 无 category_id；`inventories` 无 locked_quantity；`inbound_orders.status` 为 `draft/completed/cancelled`；`outbound_orders.status` 为 `draft/completed/cancelled`；`inventory_transactions.change_type` 为 `inbound/outbound`。无 product_categories 表 | T0.1.3 |
| T0.2.2 | Done | P0 | Unassigned | Codex | 执行迁移并验证 | 0.5h | `sqlx migrate run` 成功，psql 连接后 `\dt` 列出 9 张表，无多余表 | T0.2.1 |

### Feature 0.3 — 后端基础设施

| ID | Status | Priority | Owner | Reviewer | 任务 | 工时 | 验收标准 | 依赖 |
|----|--------|----------|-------|----------|------|------|----------|------|
| T0.3.1 | Done | P0 | Unassigned | Codex | 实现配置加载模块 | 1h | `src/config.rs` 从环境变量加载数据库 URL、服务端口等，启动时打印配置 | T0.1.1 |
| T0.3.2 | Done | P0 | Unassigned | Codex | 实现数据库连接池 | 1h | `src/db.rs` 使用 SQLx 创建 PgPool，注入 Axum State | T0.3.1 |
| T0.3.3 | Done | P0 | Unassigned | Codex | 实现统一错误类型与响应格式 | 2h | `src/error.rs` 含 `AppError` 枚举（Validation/NotFound/Conflict/BusinessRule/Internal），实现 `IntoResponse`；统一 JSON 响应 `{code, data, message}` | T0.1.1 |
| T0.3.4 | Done | P1 | Unassigned | Codex | 实现中间件（CORS / request_id / tracing） | 2h | CORS 允许前端跨域；每个请求自动生成 `X-Request-Id`；tracing 输出结构化日志 | T0.1.1 |
| T0.3.5 | Done | P0 | Unassigned | Codex | 实现路由注册骨架 | 1h | `src/router.rs` 挂载中间件层，注册各模块路由（此时路由返回 501），Axum State 注入连接池 | T0.3.2, T0.3.3, T0.3.4 |

### Feature 0.4 — 前端基础设施

| ID | Status | Priority | Owner | Reviewer | 任务 | 工时 | 验收标准 | 依赖 |
|----|--------|----------|-------|----------|------|------|----------|------|
| T0.4.1 | Done | P0 | Unassigned | Codex | 实现 AppLayout 全局布局 | 1.5h | 左侧可折叠菜单栏（商品管理/仓库管理/入库管理/出库管理/库存查询）+ 顶部 Header + 底部 Footer，Ant Design Layout 组件。菜单项不含分类管理、不含流水查询 | T0.1.2 |
| T0.4.2 | Done | P0 | Unassigned | Codex | 实现路由配置 | 1h | React Router 配置路由：`/products`、`/warehouses`、`/warehouses/:id/locations`、`/inbounds`、`/inbounds/new`、`/inbounds/:id`、`/outbounds`、`/outbounds/new`、`/outbounds/:id`、`/inventory`。默认重定向到 `/products`，404 页面。无分类、无流水页面路由 | T0.4.1 |
| T0.4.3 | Done | P1 | Unassigned | Codex | 实现 Axios 客户端封装 | 1h | `src/api/client.ts` 含 baseURL `/api/v1`、超时 10s、响应拦截器统一 toast 错误消息 | T0.1.2 |

---

## Epic 1：商品管理

> 目标：商品完整 CRUD，无分类  
> 产出：商品列表可在前端增删改查

### Feature 1.1 — 后端：商品

| ID | Status | Priority | Owner | Reviewer | 任务 | 工时 | 验收标准 | 依赖 |
|----|--------|----------|-------|----------|------|------|----------|------|
| T1.1.1 | Done | P0 | Unassigned | Codex | 定义 Product Model/DTO + Repository | 1.5h | `model/product.rs` 含 `Product`（id/sku_code/name/unit/spec/barcode/status/created_at/updated_at，无 category_id）+ `ProductStatus` 枚举；`dto/` 含请求/响应 DTO；`repository/product_repo.rs` 含 `list`（keyword/status 过滤+分页）/`find_by_id`/`find_by_sku`/`insert`/`update`/`update_status` | T0.3.5 |
| T1.1.2 | Done | P0 | Unassigned | Codex | 实现 Product Service + Handler | 2h | Service 含 `list`/`get_by_id`/`create`（SKU 唯一性校验）/`update`/`toggle_status`；Handler 含 `GET /api/v1/products` 分页搜索、`GET /api/v1/products/{id}` 详情、`POST` 创建、`PUT /{id}` 更新、`PATCH /{id}/status` 状态切换。无分类相关端点 | T1.1.1 |

### Feature 1.2 — 前端：商品管理

| ID | Status | Priority | Owner | Reviewer | 任务 | 工时 | 验收标准 | 依赖 |
|----|--------|----------|-------|----------|------|------|----------|------|
| T1.2.1 | Done | P0 | Unassigned | Codex | 实现商品列表页面 | 2h | `pages/ProductList.tsx`：搜索栏（SKU/名称输入框 + 状态下拉）+ [+ 新增商品] 按钮 + Ant Table（SKU/名称/单位/状态/操作），支持分页。无分类列、无分类筛选 | T1.1.2, T0.4.2 |
| T1.2.2 | Done | P0 | Unassigned | Codex | 实现商品表单 Modal | 1.5h | Modal 含 SKU 编码（失焦异步校验唯一性）、名称、单位（Select）、规格、条形码。支持新增/编辑复用。Ant Form 校验。无分类选择器 | T1.2.1 |

---

## Epic 2：仓库管理

> 目标：仓库完整 CRUD  
> 产出：仓库列表可增删改查

### Feature 2.1 — 后端：仓库

| ID | Status | Priority | Owner | Reviewer | 任务 | 工时 | 验收标准 | 依赖 |
|----|--------|----------|-------|----------|------|------|----------|------|
| T2.1.1 | Done | P0 | Unassigned | Codex | 定义 Warehouse Model/DTO + Repository | 1.5h | `model/warehouse.rs` 含 `Warehouse`（id/code/name/address/status）+ `WarehouseStatus` 枚举；`repository/warehouse_repo.rs` 含 `list`/`find_by_id`/`find_by_code`/`insert`/`update`/`update_status` | T0.3.5 |
| T2.1.2 | Done | P0 | Unassigned | Codex | 实现 Warehouse Service + Handler | 2h | Service 含 CRUD + 状态切换；Handler 含 `GET /api/v1/warehouses` 列表、`GET /api/v1/warehouses/{id}` 详情、`POST/PUT/PATCH` 创建/更新/状态 | T2.1.1 |

### Feature 2.2 — 前端：仓库

| ID | Status | Priority | Owner | Reviewer | 任务 | 工时 | 验收标准 | 依赖 |
|----|--------|----------|-------|----------|------|------|----------|------|
| T2.2.1 | Done | P0 | Unassigned | Codex | 实现仓库管理页面 | 2h | `pages/WarehouseList.tsx`：搜索（名称/编码）+ 表格（编码/名称/地址/状态/操作）+ Modal 表单（编码/名称/地址）。操作列含编辑、启用/禁用、"库位"按钮跳转 | T2.1.2, T0.4.2 |

---

## Epic 3：库位管理

> 目标：按仓库维度管理库位  
> 产出：库位列表可按仓库筛选和 CRUD

### Feature 3.1 — 后端：库位

| ID | Status | Priority | Owner | Reviewer | 任务 | 工时 | 验收标准 | 依赖 |
|----|--------|----------|-------|----------|------|------|----------|------|
| T3.1.1 | Done | P0 | Unassigned | Codex | 定义 Location Model/DTO + Repository | 1.5h | `model/location.rs` 含 `Location`（id/warehouse_id/code/location_type/max_volume/max_weight/status，max_volume/max_weight DB 保留但 DTO 不序列化）+ `LocationType`/`LocationStatus` 枚举；`repository/location_repo.rs` 含 `list_by_warehouse`/`find_by_id`/`find_by_code`/`insert`/`update`/`update_status` | T2.1.1 |
| T3.1.2 | Done | P0 | Unassigned | Codex | 实现 Location Service + Handler | 2h | Service 含 CRUD（库位编码同仓库内唯一校验、库位归属校验）；Handler 含 `GET /api/v1/warehouses/{id}/locations` 列表、`POST/PUT/PATCH` 创建/更新/状态 | T3.1.1 |

### Feature 3.2 — 前端：库位

| ID | Status | Priority | Owner | Reviewer | 任务 | 工时 | 验收标准 | 依赖 |
|----|--------|----------|-------|----------|------|------|----------|------|
| T3.2.1 | Done | P0 | Unassigned | Codex | 实现库位管理页面 | 1.5h | `pages/LocationList.tsx`：页面标题含仓库名称（Breadcrumb），搜索（编码+库位类型下拉）+ [+ 新增库位] + 表格（编码/类型/状态/操作）。不展示容积/承重列。Modal 表单含库位编码、类型 | T3.1.2, T0.4.2 |

---

## Epic 4：入库管理

> 目标：入库单 Draft → Completed 直达，完成时库存增加 + 流水记录  
> 产出：创建入库单 → 完成入库 → 库存和流水正确

### Feature 4.1 — 后端：入库单

| ID | Status | Priority | Owner | Reviewer | 任务 | 工时 | 验收标准 | 依赖 |
|----|--------|----------|-------|----------|------|------|----------|------|
| T4.1.1 | Done | P0 | Unassigned | Codex | 定义 InboundOrder / InboundOrderItem Model/DTO | 1.5h | `model/inbound.rs` 含 `InboundOrder`（id/order_no/warehouse_id/order_type/status/remark/created_at/completed_at）+ `InboundOrderItem`。status 枚举仅 `draft/completed/cancelled`；order_type 枚举 `purchase/return/manual`。`dto/` 含创建请求（含 items 数组）、更新请求、完成请求（含实收数量）、列表响应 | T0.3.5 |
| T4.1.2 | Done | P0 | Unassigned | Codex | 实现 InboundOrder Repository + Inventory Repository | 3h | `repository/inbound_repo.rs` 含 `list`/`find_by_id_with_items`/`insert_order`/`insert_items`/`update_order`/`delete_items`/`update_status`/`update_item_actual`；`repository/inventory_repo.rs` 含 `upsert`（INSERT ON CONFLICT DO UPDATE 累加 quantity）/`find_by_keys`/`decrease`（WHERE quantity >= ? 乐观校验）；`repository/transaction_repo.rs` 含 `insert` 流水（change_type: inbound/outbound） | T4.1.1, T1.1.1, T3.1.1 |
| T4.1.3 | Done | P0 | Unassigned | Codex | 实现 InventoryService + InboundService | 2.5h | `InventoryService`：`increase_stock()`（调 upsert + 写 inbound 流水）、`decrease_stock()`（调 decrease + 写 outbound 流水）。`InboundService`：`create()`（生成 order_no=`IN+YYYYMMDD+6位序列`）、`update()`（仅 draft）、`complete()`（校验 draft → 写 actual_quantity → 调 increase_stock → 更新 completed → 事务内）、`cancel()`（draft→cancelled；completed 取消需回滚库存） | T4.1.2 |
| T4.1.4 | Done | P0 | Unassigned | Codex | 实现 InboundOrder Handler 并注册路由 | 1.5h | `GET /api/v1/inbound-orders` 列表；`GET /api/v1/inbound-orders/{id}` 详情含明细；`POST` 创建；`PUT /{id}` 更新；`POST /{id}/complete` 完成；`POST /{id}/cancel` 取消。无 submit/approve 端点 | T4.1.3 |

### Feature 4.2 — 前端：入库单

| ID | Status | Priority | Owner | Reviewer | 任务 | 工时 | 验收标准 | 依赖 |
|----|--------|----------|-------|----------|------|------|----------|------|
| T4.2.1 | Done | P0 | Unassigned | Codex | 实现入库单列表页面 | 2h | `pages/InboundList.tsx`：搜索（单号/仓库/状态/日期范围）+ [+ 新增入库单] + 表格（单号/仓库/类型/状态彩色标签/时间/操作），操作列按状态显示"编辑"(draft)/"查看"/"完成"(draft) | T4.1.4, T0.4.2 |
| T4.2.2 | Done | P0 | Unassigned | Codex | 实现入库单表单（新增/编辑） | 3h | `pages/InboundForm.tsx`：基本信息区（入库类型 Select / 目标仓库 Select / 备注）+ 明细表格（商品选择器搜索 / 库位级联选择仅显示目标仓库库位 / 计划数量 / 删除行）+ [+ 添加明细] + [保存草稿]。无分类筛选 | T4.2.1, T1.2.2, T3.2.1 |
| T4.2.3 | Done | P0 | Unassigned | Codex | 实现入库单详情与完成操作 | 1.5h | `pages/InboundDetail.tsx`：基本信息（只读）+ 明细表格（计划数量/实收数量）+ [完成入库](Modal 输入每行实收数量+二次确认) / [取消] / [返回] | T4.2.2 |

---

## Epic 5：出库管理

> 目标：出库单 Draft → Completed 直达，完成时校验库存 + 扣减 + 流水  
> 产出：创建出库单 → 完成出库 → 库存减少和流水正确

### Feature 5.1 — 后端：出库单

| ID | Status | Priority | Owner | Reviewer | 任务 | 工时 | 验收标准 | 依赖 |
|----|--------|----------|-------|----------|------|------|----------|------|
| T5.1.1 | Done | P0 | Unassigned | Codex | 定义 OutboundOrder / OutboundOrderItem Model/DTO | 1.5h | `model/outbound.rs` 含 `OutboundOrder`/`OutboundOrderItem`。status 枚举仅 `draft/completed/cancelled`；order_type 枚举 `sales/manual/scrap`。`dto/` 含创建、更新、完成请求，列表响应 | T4.1.1 |
| T5.1.2 | Done | P0 | Unassigned | Codex | 实现 OutboundOrder Repository | 2h | `repository/outbound_repo.rs` 含 `list`/`find_by_id_with_items`/`insert_order`/`insert_items`/`update_order`/`delete_items`/`update_status`/`update_item_actual` | T5.1.1 |
| T5.1.3 | Done | P0 | Unassigned | Codex | 实现 OutboundService | 2h | `complete()`：校验 draft → 遍历明细调 `decrease_stock()` → 库存不足返回 422+缺货商品列表 → 写 actual_quantity → 更新 completed → 事务内；`cancel()`：draft→cancelled；completed 取消需回滚库存。严禁 lock/unlock/approve | T5.1.2, T4.1.3 |
| T5.1.4 | Done | P0 | Unassigned | Codex | 实现 OutboundOrder Handler 并注册路由 | 1.5h | `GET /api/v1/outbound-orders` 列表；`GET /api/v1/outbound-orders/{id}` 详情；`POST` 创建；`PUT /{id}` 更新；`POST /{id}/complete` 完成；`POST /{id}/cancel` 取消。无 submit/approve 端点 | T5.1.3 |

### Feature 5.2 — 前端：出库单

| ID | Status | Priority | Owner | Reviewer | 任务 | 工时 | 验收标准 | 依赖 |
|----|--------|----------|-------|----------|------|------|----------|------|
| T5.2.1 | Todo | P0 | Unassigned | Codex | 实现出库单列表页面 | 2h | `pages/OutboundList.tsx`：与入库列表对称（出库单号/出库类型）。操作列按状态显示"编辑"(draft)/"查看"/"完成"(draft) | T5.1.4, T0.4.2 |
| T5.2.2 | Todo | P0 | Unassigned | Codex | 实现出库单表单（含可用库存显示） | 2.5h | `pages/OutboundForm.tsx`：与入库表单对称，明细表格增加"可用库存"列（选择商品和库位后实时查询库存 quantity），出库数量>库存时红色警告 | T5.2.1, T4.2.2 |
| T5.2.3 | Todo | P0 | Unassigned | Codex | 实现出库单详情与完成操作 | 1.5h | `pages/OutboundDetail.tsx`：基本信息+明细表格+[完成出库](Modal 输入实发数量+库存不足提示) / [取消] / [返回] | T5.2.2 |

---

## Epic 6：库存查询

> 目标：库存查询（前端页面）+ 库存流水（仅后端记录，无前端页面）  
> 产出：库存看板可按商品/仓库/库位查询

### Feature 6.1 — 后端

| ID | Status | Priority | Owner | Reviewer | 任务 | 工时 | 验收标准 | 依赖 |
|----|--------|----------|-------|----------|------|------|----------|------|
| T6.1.1 | Todo | P0 | Unassigned | Codex | 实现 StockQueryService | 1.5h | `service/stock_query_service.rs` 含 `query_inventory`（多条件过滤+分页，JOIN products/warehouses/locations 查名称，available=quantity）、`query_transactions`（仅后端内部使用，按条件过滤+分页。change_type 仅 inbound/outbound） | T4.1.2 |
| T6.1.2 | Todo | P0 | Unassigned | Codex | 实现 Inventory Handler 并注册路由 | 1.5h | `GET /api/v1/inventory` 库存查询（product_id/warehouse_id/location_id/keyword/page）；`GET /api/v1/inventory-transactions` 流水查询（仅后端/测试用，无前端页面） | T6.1.1 |

### Feature 6.2 — 前端

| ID | Status | Priority | Owner | Reviewer | 任务 | 工时 | 验收标准 | 依赖 |
|----|--------|----------|-------|----------|------|------|----------|------|
| T6.2.1 | Todo | P1 | Unassigned | Codex | 实现库存查询页面 | 1.5h | `pages/InventoryQuery.tsx`：搜索栏（商品选择器/仓库下拉/库位下拉）+ 表格（商品名称/SKU/仓库/库位/库存数量/更新时间）。表格只读。无 locked_quantity/available_quantity 列 | T6.1.2, T0.4.2 |

> 库存流水（inventory_transactions）仅在后台记录，不做前端页面。流水查询 API 保留供测试和审计使用。

---

## Epic 7：Testing

> 目标：核心流程自动化测试覆盖  
> 产出：4 个集成/单元测试全部通过

| ID | Status | Priority | Owner | Reviewer | 任务 | 工时 | 验收标准 | 依赖 |
|----|--------|----------|-------|----------|------|------|----------|------|
| T7.1.1 | Todo | P1 | Unassigned | Codex | Product API 集成测试 | 2h | 创建商品 → 查询列表含该商品 → 按 ID 查询 → 更新 → SKU 重复创建返回 409 → 禁用后列表过滤正确。使用 `#[tokio::test]` + 测试数据库 | T1.1.2 |
| T7.1.2 | Todo | P0 | Unassigned | Codex | Inventory Service 单元测试 | 2h | `increase_stock()`：插入新库存 → 累加已有库存 → quantity 正确 → 流水记录正确。`decrease_stock()`：正常扣减 → 扣减到 0 → 扣减超过库存返回 Err → 事务回滚后 quantity 不变。流水表 insert 验证 | T4.1.3 |
| T7.1.3 | Todo | P0 | Unassigned | Codex | Inbound Workflow 集成测试 | 2h | 创建入库单（2 个明细）→ complete 填实收数量 → 验证库存 quantity 增加 → 验证流水表插入 inbound 记录（before/after 正确）→ 入库单 status=completed → 取消已完成入库单 → 库存回滚 → 流水插入 outbound 回滚记录 | T4.1.4 |
| T7.1.4 | Todo | P0 | Unassigned | Codex | Outbound Workflow 集成测试 | 2h | 先入库建立库存 → 创建出库单 → complete（数量≤库存）→ 库存扣减正确 → 流水 outbound 记录 → 再创建出库单（数量>库存）→ complete 返回 422 → 库存未变化（事务回滚）→ 取消已完成出库单 → 库存回滚 | T5.1.4 |

---

## Epic 8：Seed Data

> 目标：新环境启动后无需人工录入即可立即测试核心流程  
> 产出：数据库初始化后自动写入示例数据

| ID | Status | Priority | Owner | Reviewer | 任务 | 工时 | 验收标准 | 依赖 |
|----|--------|----------|-------|----------|------|------|----------|------|
| T8.1.1 | Todo | P1 | Unassigned | Codex | 编写 Seed Data SQL 脚本 | 1.5h | `backend/migrations/seed.sql` 含：5 个示例商品（sku_code/name/unit）、1 个默认仓库（主仓库）、4 个默认库位（A-01-01/A-01-02/RECV-01/SHIP-01）。无分类数据 | T0.2.2 |
| T8.1.2 | Todo | P1 | Unassigned | Codex | 实现 Seed 运行入口 | 1h | `cargo run -- --seed` 或 `POST /api/v1/seed` 一键写入种子数据；启动检测到空数据库自动执行 seed；seed 幂等 | T8.1.1 |
| T8.1.3 | Todo | P2 | Unassigned | Codex | 验证 Seed 数据可用性 | 0.5h | Seed 后：商品列表显示 5 个商品、仓库列表显示 1 个仓库、库位列表显示 4 个库位。创建入库单时可选择这些预置数据 | T8.1.2, T4.2.2 |

---

## 工时汇总

| Epic | Feature 数 | Task 数 | 估算工时 |
|------|-----------|---------|----------|
| Epic 0 — 基础设施 | 4 | 13 | 15.5 h |
| Epic 1 — 商品管理 | 2 | 4 | 7.0 h |
| Epic 2 — 仓库管理 | 2 | 3 | 5.5 h |
| Epic 3 — 库位管理 | 2 | 3 | 5.0 h |
| Epic 4 — 入库管理 | 2 | 7 | 15.0 h |
| Epic 5 — 出库管理 | 2 | 7 | 12.5 h |
| Epic 6 — 库存查询 | 2 | 3 | 4.5 h |
| Epic 7 — Testing | 1 | 4 | 8.0 h |
| Epic 8 — Seed Data | 1 | 3 | 3.0 h |
| **合计** | **18** | **47** | **~54 h** |

---

## MVP 开发顺序（2 周）

```
Day 1-2:  Epic 0 — 基础设施搭建                    (15.5h)
Day 3-4:  Epic 1+2+3 — 主数据并行开发               (17.5h)
Day 5-7:  Epic 4 — 入库管理 + T7.1.2 单测          (17.0h)
Day 8:    Epic 8 — Seed Data                       (3.0h)
Day 9-10: Epic 5 — 出库管理 + T7.1.4 测试          (14.5h)
Day 11:   Epic 6 — 库存查询 + T7.1.1/T7.1.3 补测   (10.5h)
Day 12:   联调 + Bug 修复 buffer                    (—)
```

### 里程碑

| 里程碑 | 完成条件 | 预计 |
|--------|----------|------|
| M0 | 后端启动 + 前端启动 + 9 张表迁移成功 | Day 2 |
| M1 | 商品/仓库/库位全部可 CRUD | Day 4 |
| M2 | 入库流程跑通 + 库存正确增加 + 流水正确 + 单测通过 | Day 7 |
| M3 | 出库流程跑通 + 库存不足正确拒绝 + 事务回滚 + 测试通过 | Day 10 |
| M4 | 库存查询可用 + 全部 4 个测试通过 + Seed 就绪 → **MVP 发布** | Day 12 |

---

## 状态机速查

### 入库单

```
Draft ──→ Completed (库存增加 + 流水记录)
  │
  └──────→ Cancelled (completed 取消需回滚库存)
```

### 出库单

```
Draft ──→ Completed (库存扣减 + 流水记录，库存不足→422)
  │
  └──────→ Cancelled (completed 取消需回滚库存)
```

---

## 成功演示流程

```
1. Seed 一键初始化
2. 创建商品（或使用 Seed 商品）
3. 创建仓库（或使用 Seed 仓库）
4. 创建库位（或使用 Seed 库位）
5. 创建入库单 → 完成入库 → 库存显示增加
6. 创建出库单 → 完成出库 → 库存显示减少
7. 库存查询页验证最终库存正确
8. 超量出库 → 返回错误提示
```

---

## 已删除的模块与功能

| 删除项 | 处理 |
|--------|------|
| `product_categories` 表 | 完全移除 |
| 商品分类 CRUD 接口 | 删除 |
| 分类管理前端页面 | 删除 |
| 分类 Tree/Cascader 组件 | 删除 |
| `products.category_id` 列 | 删除 |
| `/api/v1/product-categories` | 删除 |
| `POST .../submit` 端点 | 删除 |
| `POST .../approve` 端点 | 删除 |
| `inventories.locked_quantity` 列 | 删除 |
| `lock_stock()` / `unlock_stock()` | 删除 |
| `pending` / `approved` 状态值 | 删除 |
| 库存流水前端页面 | 删除 |
| 库存流水分页/筛选 UI | 删除 |
| JWT / 认证系统 | 不实现 |
| 软删除 | 不实现 |
