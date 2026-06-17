# Epic 5：出库管理 — 实施方案

> 版本：v1.0
> 角色：Tech Lead / Codex
> 目标读者：Reasonix / Codex Reviewer / GPT 架构讨论 / 后续交接
> 预计总工时：12.5h
> 依据：`_ai/tasks/TASK_BOARD.md`、`_ai/tasks/Epic4-Implementation-Plan.md`、`_ai/openspec/specs/003-database-design.md`、`_ai/openspec/specs/004-api-design.md`、`_ai/openspec/specs/005-ui-design.md`

---

## 1. Epic Overview

Epic 5 的目标是完成出库管理模块，使 WMS MVP 具备从创建出库单到完成出库、库存扣减、库存流水记录的完整业务闭环。

Epic 5 完成后，系统将获得：

- 创建出库单的能力
- 编辑 draft 出库单的能力
- 查询出库单列表的能力
- 查看出库单详情的能力
- 完成出库并写入实发数量的能力
- 完成出库前校验库存是否充足的能力
- 完成出库后自动扣减库存的能力
- 完成出库后自动写入 outbound 库存流水的能力
- 取消 draft 出库单的能力
- 取消 completed 出库单并回滚库存的能力
- 前端出库单列表、表单、详情与完成操作闭环

Epic 5 建立库存减少方向的核心流程：

- Product 表示“出什么商品”
- Warehouse 表示“从哪个仓库出”
- Location 表示“从哪个库位出”
- OutboundOrder 表示“出库业务单据”
- Inventory 表示“当前库存快照”
- InventoryTransaction 表示“库存变化审计记录”

Epic 5 严格对应 `TASK_BOARD.md` 中七个任务：

- T5.1.1 定义 OutboundOrder / OutboundOrderItem Model/DTO
- T5.1.2 实现 OutboundOrder Repository
- T5.1.3 实现 OutboundService
- T5.1.4 实现 OutboundOrder Handler 并注册路由
- T5.2.1 实现出库单列表页面
- T5.2.2 实现出库单表单（含可用库存显示）
- T5.2.3 实现出库单详情与完成操作

Epic 5 不实现库存查询页面，不实现库存流水前端页面，不实现审批流，不实现 submit / approve，不修改数据库 schema。

---

## 2. Domain Design

### OutboundOrder

`OutboundOrder` 是出库单聚合根，映射 `outbound_orders` 表。

字段职责：

| 字段 | 职责 |
|------|------|
| `id` | 出库单主键 |
| `order_no` | 出库单业务编号，后端生成 |
| `warehouse_id` | 来源仓库 ID |
| `order_type` | 出库类型 |
| `status` | 出库单状态 |
| `remark` | 备注 |
| `completed_at` | 完成时间 |
| `created_at` | 创建时间 |
| `updated_at` | 更新时间 |

状态仅允许：

- `draft`
- `completed`
- `cancelled`

出库类型仅允许：

- `sales`
- `manual`
- `scrap`

业务含义：

- 创建后默认为 `draft`
- `draft` 可编辑、可完成、可取消
- `completed` 表示库存已经扣减
- `cancelled` 表示单据已取消
- 不存在 `pending`
- 不存在 `approved`
- 不存在 submit / approve 流程

### OutboundOrderItem

`OutboundOrderItem` 是出库单明细，映射 `outbound_order_items` 表。

字段职责：

| 字段 | 职责 |
|------|------|
| `id` | 明细主键 |
| `order_id` | 所属出库单 ID |
| `product_id` | 出库商品 ID |
| `location_id` | 来源库位 ID |
| `planned_qty` | 计划出库数量 |
| `actual_qty` | 实发数量 |
| `created_at` | 创建时间 |

业务含义：

- 一个出库单包含多条明细
- 每条明细指定商品、来源库位、计划数量
- `planned_qty` 创建和编辑时填写，必须大于 0
- `actual_qty` 完成出库时填写
- 完成前 `actual_qty` 可为空
- 完成后 `actual_qty` 用于库存扣减

### Inventory

Epic 5 复用 Epic 4 已建立的 `Inventory` 设计。

库存粒度固定为：

```text
product_id + warehouse_id + location_id
```

业务含义：

- 出库完成时扣减 `quantity`
- 扣减后库存不得小于 0
- 库存不足时拒绝完成出库
- 无 `locked_quantity`
- 无库存锁定逻辑

### InventoryTransaction

Epic 5 复用 Epic 4 已建立的 `InventoryTransaction` 设计。

`change_type` 仅允许：

- `inbound`
- `outbound`

Epic 5 中：

- 完成出库扣减库存时写入 `outbound` 流水
- 取消 completed 出库单并回滚库存时，会产生库存增加流水，可使用 `inbound` 表示库存增加方向
- 不实现 `lock` / `unlock` / `adjust`
- 不删除历史流水

---

## 3. Business Rules

### 出库单创建规则

- 创建请求必须包含 `warehouse_id`、`order_type`、`items`
- `order_type` 仅允许 `sales` / `manual` / `scrap`
- `items` 至少包含一条明细
- 每条明细必须包含 `product_id`、`location_id`、`planned_qty`
- `planned_qty` 必须大于 0
- `warehouse_id` 必须指向已存在仓库
- `product_id` 必须指向已存在商品
- `location_id` 必须指向已存在库位
- 明细库位必须属于出库单来源仓库
- 后端生成 `order_no`
- `order_no` 格式为 `OUT + YYYYMMDD + 6位序列`
- 创建后状态为 `draft`
- 创建时不扣减库存
- 创建时不写库存流水
- 创建时不锁定库存

### 出库单更新规则

- 仅 `draft` 状态允许更新
- `completed` 不允许更新
- `cancelled` 不允许更新
- 更新采用全量替换 items
- 更新时仍需校验仓库、商品、库位、库位归属和计划数量
- 更新时不扣减库存
- 更新时不写库存流水
- 更新时不修改 completed_at
- 更新时不锁定库存

### 出库单完成规则

- 仅 `draft` 状态允许完成
- 完成请求必须包含每条明细的实发数量
- 每个 `item_id` 必须属于当前出库单
- `actual_qty` 必须大于或等于 0
- 允许 `actual_qty` 与 `planned_qty` 不一致
- 完成前必须校验对应库存是否充足
- 库存不足时返回业务错误，并且不得产生任何部分扣减
- 完成后写入每条明细的 `actual_qty`
- 以 `actual_qty` 扣减库存
- 每条产生库存变化的明细必须写入库存流水
- 完成后订单状态变为 `completed`
- 完成后写入 `completed_at`
- 完成操作必须在数据库事务内完成

complete 必须在数据库事务内完成：

1. 校验订单状态为 `draft`
2. 校验完成请求覆盖所有明细，且无重复 item_id
3. 校验每个 item_id 属于当前出库单
4. 遍历明细并按 `actual_qty` 校验库存充足
5. 扣减库存
6. 写入 `inventory_transactions`
7. 写入明细 `actual_qty`
8. 更新订单为 `completed`

任一步失败，整个事务必须回滚。

### 出库单取消规则

- `draft` 出库单取消：直接更新状态为 `cancelled`
- `completed` 出库单取消：必须回滚库存
- `cancelled` 出库单不可重复取消
- 取消 completed 出库单时，应根据已写入的 `actual_qty` 增加对应库存
- 回滚库存和写入流水必须在数据库事务内完成
- 回滚后订单状态更新为 `cancelled`
- 取消 completed 出库单不得删除原出库流水，只能追加新的库存变化流水

### 库存扣减规则

- 库存以 `product_id + warehouse_id + location_id` 为唯一粒度
- 出库完成时按 `actual_qty` 扣减库存
- 库存记录不存在视为库存不足
- 库存记录存在但 `quantity < actual_qty` 视为库存不足
- 扣减必须保证 `quantity >= 0`
- `quantity_before` 和 `quantity_after` 必须准确用于流水记录
- 所有数量字段必须使用 Decimal / NUMERIC 精确类型
- 不得使用 `f64`

### 流水记录规则

- 库存发生变化时必须写入流水
- 完成出库扣减库存时，`change_type = outbound`
- 取消 completed 出库单回滚库存时，`change_type = inbound`
- 流水必须记录：
  - product_id
  - warehouse_id
  - location_id
  - change_type
  - quantity
  - quantity_before
  - quantity_after
  - reference_type
  - reference_id
- `reference_type` 应能区分出库完成与出库取消回滚场景
- 流水只追加，不修改，不删除

### 事务边界

必须使用事务的场景：

- 创建出库单并插入多条明细
- 更新出库单并全量替换明细
- 完成出库
- 取消 completed 出库单并回滚库存

可以不使用事务的场景：

- 出库单列表查询
- 出库单详情查询
- 单纯取消 draft 出库单，如实现上为了统一也可使用事务

核心原则：

- 订单状态、明细实发数量、库存快照、库存流水必须保持一致
- 库存变更和流水写入不得分离提交
- complete / completed cancel 中任一环节失败必须全部回滚

---

## 4. API Design

Epic 5 涉及 6 个出库 API。

所有 API 使用 `/api/v1` 前缀，响应遵循统一格式：

```json
{
  "code": 0,
  "data": {},
  "message": "ok"
}
```

列表响应格式：

```json
{
  "code": 0,
  "data": {
    "items": [],
    "total": 0,
    "page": 1,
    "page_size": 20
  },
  "message": "ok"
}
```

### 4.1 查询出库单列表

```text
GET /api/v1/outbound-orders
```

Query：

| 参数 | 必填 | 说明 |
|------|------|------|
| `keyword` | 否 | 单号搜索 |
| `warehouse_id` | 否 | 仓库筛选 |
| `status` | 否 | `draft` / `completed` / `cancelled` |
| `start_date` | 否 | 创建时间开始 |
| `end_date` | 否 | 创建时间结束 |
| `page` | 否 | 默认 1 |
| `page_size` | 否 | 默认 20 |

Response data：

```json
{
  "items": [
    {
      "id": "uuid",
      "order_no": "OUT20260617000001",
      "warehouse_id": "uuid",
      "warehouse_name": "主仓库",
      "order_type": "sales",
      "status": "draft",
      "remark": "客户销售出库",
      "created_at": "2026-06-17T10:00:00Z",
      "completed_at": null
    }
  ],
  "total": 1,
  "page": 1,
  "page_size": 20
}
```

错误场景：

| 场景 | HTTP | 说明 |
|------|------|------|
| status 非法 | 400 | 参数校验失败 |
| warehouse_id 非法 | 400 | UUID 解析失败 |
| DB 查询失败 | 500 | Internal |

### 4.2 查询出库单详情

```text
GET /api/v1/outbound-orders/{id}
```

Response data：

```json
{
  "id": "uuid",
  "order_no": "OUT20260617000001",
  "warehouse_id": "uuid",
  "warehouse_name": "主仓库",
  "order_type": "sales",
  "status": "draft",
  "remark": "客户销售出库",
  "created_at": "2026-06-17T10:00:00Z",
  "completed_at": null,
  "items": [
    {
      "id": "uuid",
      "product_id": "uuid",
      "product_name": "iPhone 15",
      "sku_code": "SKU-001",
      "location_id": "uuid",
      "location_code": "A-01-01",
      "planned_qty": "10.0000",
      "actual_qty": null
    }
  ]
}
```

错误场景：

| 场景 | HTTP | 说明 |
|------|------|------|
| id 非法 | 400 | UUID 解析失败 |
| 出库单不存在 | 404 | NotFound |
| DB 查询失败 | 500 | Internal |

### 4.3 创建出库单

```text
POST /api/v1/outbound-orders
```

Request：

```json
{
  "warehouse_id": "uuid",
  "order_type": "sales",
  "remark": "客户销售出库",
  "items": [
    {
      "product_id": "uuid",
      "location_id": "uuid",
      "planned_qty": "10.0000"
    }
  ]
}
```

Response：

- 201 Created 或 200 OK，按项目现有 Handler 风格保持一致
- data 返回创建后的出库单详情或基础响应

错误场景：

| 场景 | HTTP | 说明 |
|------|------|------|
| warehouse_id 不存在 | 404 | 仓库不存在 |
| product_id 不存在 | 404 | 商品不存在 |
| location_id 不存在 | 404 | 库位不存在 |
| 库位不属于来源仓库 | 422 | 业务规则错误 |
| items 为空 | 400 | 参数校验失败 |
| planned_qty <= 0 | 400 | 参数校验失败 |
| order_type 非法 | 400 | 参数校验失败 |
| order_no 冲突 | 409 | 极端并发下单号冲突 |
| DB 写入失败 | 500 | Internal |

### 4.4 更新出库单

```text
PUT /api/v1/outbound-orders/{id}
```

Request：

```json
{
  "warehouse_id": "uuid",
  "order_type": "manual",
  "remark": "调整备注",
  "items": [
    {
      "product_id": "uuid",
      "location_id": "uuid",
      "planned_qty": "8.0000"
    }
  ]
}
```

Response：

- data 返回更新后的出库单详情或基础响应

错误场景：

| 场景 | HTTP | 说明 |
|------|------|------|
| 出库单不存在 | 404 | NotFound |
| 当前状态不是 draft | 422 | 仅 draft 可更新 |
| warehouse_id 不存在 | 404 | 仓库不存在 |
| product_id 不存在 | 404 | 商品不存在 |
| location_id 不存在 | 404 | 库位不存在 |
| 库位不属于来源仓库 | 422 | 业务规则错误 |
| items 为空 | 400 | 参数校验失败 |
| planned_qty <= 0 | 400 | 参数校验失败 |
| DB 写入失败 | 500 | Internal |

### 4.5 完成出库

```text
POST /api/v1/outbound-orders/{id}/complete
```

Request：

```json
{
  "items": [
    {
      "item_id": "uuid",
      "actual_qty": "10.0000"
    }
  ]
}
```

Response：

- data 返回完成后的出库单详情或基础响应

错误场景：

| 场景 | HTTP | 说明 |
|------|------|------|
| 出库单不存在 | 404 | NotFound |
| 当前状态不是 draft | 422 | 仅 draft 可完成 |
| item_id 不属于当前出库单 | 422 | 明细不匹配 |
| 缺少某条明细实发数量 | 400 | 参数校验失败 |
| actual_qty < 0 | 400 | 参数校验失败 |
| 库存不足 | 422 | 业务规则错误 |
| 库存扣减失败 | 500 | Internal |
| 流水写入失败 | 500 | Internal |

事务要求：

- 校验 draft
- 校验完成请求覆盖所有明细
- 扣减库存
- 写入库存流水
- 写入 actual_qty
- 更新订单 completed
- 任一步失败整体回滚

### 4.6 取消出库单

```text
POST /api/v1/outbound-orders/{id}/cancel
```

Request：

- 无请求体，或空 JSON 对象

Response：

- data 返回取消后的出库单详情或基础响应

错误场景：

| 场景 | HTTP | 说明 |
|------|------|------|
| 出库单不存在 | 404 | NotFound |
| 已 cancelled | 422 | 不可重复取消 |
| completed 回滚库存失败 | 500 | Internal |
| 流水写入失败 | 500 | Internal |

### 禁止 API

Epic 5 明确禁止实现：

```text
POST /api/v1/outbound-orders/{id}/submit
POST /api/v1/outbound-orders/{id}/approve
```

也不得新增任何审批、审核、提交、锁定、解锁相关端点。

---

## 5. Backend Implementation Plan

### T5.1.1 Model / DTO 设计

目标是定义出库单相关 Model 和 DTO，不实现 Repository、Service、Handler、Router。

预计涉及文件：

| 文件 | 职责 |
|------|------|
| `backend/src/model/mod.rs` | 导出 outbound model |
| `backend/src/model/outbound.rs` | 定义 OutboundOrder、OutboundOrderItem、枚举 |
| `backend/src/dto/mod.rs` | 导出 outbound DTO |
| `backend/src/dto/outbound.rs` | 定义出库请求、查询、响应 DTO |

Model 应包括：

- `OutboundOrder`
- `OutboundOrderItem`
- `OutboundOrderStatus`
- `OutboundOrderType`

枚举规则：

- `OutboundOrderStatus` 仅 `draft` / `completed` / `cancelled`
- `OutboundOrderType` 仅 `sales` / `manual` / `scrap`

DTO 应包括：

- 出库单列表查询 DTO
- 创建出库单请求 DTO
- 更新出库单请求 DTO
- 出库单明细请求 DTO
- 完成出库请求 DTO
- 完成出库明细 DTO
- 出库单列表响应 DTO
- 出库单详情响应 DTO
- 出库单明细响应 DTO

DTO 不应包含：

- submit
- approve
- pending
- approved
- locked_quantity
- 库存锁定字段
- 库存查询页面字段

数量字段必须使用 Decimal 类型，不得使用 `f64`。

### T5.1.2 Repository 设计

目标是实现出库单 Repository。

预计涉及文件：

| 文件 | 职责 |
|------|------|
| `backend/src/repository/mod.rs` | 导出 outbound repository |
| `backend/src/repository/outbound_repo.rs` | 出库单与明细 SQL |

`outbound_repo.rs` 应包含：

- `list`
- `find_by_id_with_items`
- `insert_order`
- `insert_items`
- `update_order`
- `delete_items`
- `update_status`
- `update_item_actual`

Repository 查询关联名称以支持列表/详情响应：

- warehouse_name
- product_name
- sku_code
- location_code

Repository 不得承载业务规则。

Repository 不得：

- 决定订单状态流转
- 判断是否允许 complete / cancel
- 判断库存是否足够
- 实现库存业务编排
- 构造 HTTP 响应
- 调用 Service
- 写前端逻辑
- 修改 InventoryRepository / TransactionRepository 的业务语义
- 实现库存查询 API

### T5.1.3 Service 设计

目标是实现出库业务编排服务，复用 Epic 4 已完成的库存服务能力。

预计涉及文件：

| 文件 | 职责 |
|------|------|
| `backend/src/service/mod.rs` | 导出 outbound service |
| `backend/src/service/outbound_service.rs` | 出库流程编排 |

`OutboundService` 应包含：

- `list()`
- `get_by_id()`
- `create()`
- `update()`
- `complete()`
- `cancel()`

职责：

- 生成 `order_no = OUT + YYYYMMDD + 6位序列`
- 校验仓库、商品、库位存在性
- 校验库位属于来源仓库
- 校验订单状态
- 控制数据库事务
- 调用 OutboundRepository
- 调用 InventoryService
- 处理库存不足业务错误
- 处理 completed cancel 的库存回滚

Service 不得：

- 解析 Axum Path / Query / Json
- 构造 Axum Response
- 写 SQL 字符串
- 写前端 UI
- 修改数据库 schema
- 引入库存锁定
- 引入审批流

### T5.1.4 Handler 与 Router 接入设计

目标是将出库单 API 接入真实 Handler 和 Router。

预计涉及文件：

| 文件 | 职责 |
|------|------|
| `backend/src/handler/mod.rs` | 导出 outbound handler |
| `backend/src/handler/outbound_handler.rs` | HTTP 请求解析与响应 |
| `backend/src/router.rs` | 将 outbound-orders 路由从 stub 替换为真实 Handler |

Handler 应实现：

- `GET /api/v1/outbound-orders`
- `GET /api/v1/outbound-orders/{id}`
- `POST /api/v1/outbound-orders`
- `PUT /api/v1/outbound-orders/{id}`
- `POST /api/v1/outbound-orders/{id}/complete`
- `POST /api/v1/outbound-orders/{id}/cancel`

Handler 职责：

- 解析 Query / Path / Json
- 调用 OutboundService
- 返回统一响应格式
- 映射已有 AppError

Router 接入规则：

- 仅替换出库相关 stub
- Product / Warehouse / Location / Inbound 路由保持现状可用
- Inventory 路由继续遵循当前任务板状态，不因 Epic 5 提前实现 Epic 6
- 不新增 submit / approve / lock / unlock 路由

---

## 6. Frontend Implementation Plan

### T5.2.1 OutboundList 页面设计

页面：`frontend/src/pages/OutboundList.tsx`
路由：`/outbounds`

页面目标：

- 查询出库单列表
- 按条件筛选
- 新增出库单入口
- draft 单据可编辑、完成、取消
- completed / cancelled 单据可查看

搜索区域：

- 单号输入框
- 仓库下拉
- 状态下拉
- 日期范围
- 查询按钮
- `+ 新增出库单` 按钮

表格列：

- 单号
- 仓库
- 类型
- 状态彩色标签
- 创建时间
- 完成时间或时间列
- 操作

操作规则：

| 状态 | 操作 |
|------|------|
| draft | 编辑 / 完成 / 取消 |
| completed | 查看 |
| cancelled | 查看 |

API client：

- `frontend/src/api/outbound.ts`
- 必须复用 `frontend/src/api/client.ts`

类型文件：

- `frontend/src/types/outbound.ts`

仓库选择：

- 仓库下拉可复用现有 Warehouse API
- 不新增仓库管理功能

### T5.2.2 OutboundForm 页面设计

页面：`frontend/src/pages/OutboundForm.tsx`
路由：

- `/outbounds/new`
- `/outbounds/:id` 在编辑模式下如现有路由区分不足，应遵循项目当前路由设计，不新增 TASK_BOARD 外路由

页面目标：

- 新增出库单
- 编辑 draft 出库单
- 保存草稿
- 选择商品和库位后显示可用库存
- 出库数量超过库存时明确提示

基本信息区：

- 出库类型 Select
- 来源仓库 Select
- 备注 TextArea

出库类型选项：

- sales
- manual
- scrap

明细表格：

- 商品选择器
- 来源库位选择器
- 可用库存
- 计划数量
- 操作

交互规则：

- 商品选择器支持搜索，显示 SKU + 商品名称
- 来源库位仅显示所选来源仓库下的库位
- 选择商品和库位后实时查询或获取对应库存 quantity
- 可用库存列只用于当前出库表单
- 计划数量大于可用库存时显示红色警告
- 保存 draft 时仍由后端做最终校验
- 不在前端实现完整库存查询页面

### T5.2.3 OutboundDetail 页面设计

页面：`frontend/src/pages/OutboundDetail.tsx`
路由：`/outbounds/:id`

页面目标：

- 查看出库单基本信息
- 查看出库单明细
- draft 状态完成出库
- draft / completed 状态取消出库
- completed / cancelled 只读查看

详情区：

- 出库单号
- 状态标签
- 出库类型
- 仓库
- 备注
- 创建时间
- 完成时间

明细表格：

- 商品
- SKU
- 来源库位
- 计划数量
- 实发数量

完成出库 Modal：

- 每行输入实发数量
- 可展示库存不足提示
- 完成前二次确认
- 调用 `POST /api/v1/outbound-orders/{id}/complete`
- 成功后刷新详情

取消操作：

- 取消前确认
- 调用 `POST /api/v1/outbound-orders/{id}/cancel`
- 成功后刷新详情

---

## 7. Repository Boundary

Repository 层只负责 SQL 与数据访问。

允许：

- 参数化查询
- 分页查询
- 条件筛选
- JOIN 主数据表补充展示字段
- 插入订单主表
- 插入订单明细
- 更新订单主表基础字段
- 删除并重建订单明细
- 更新订单状态
- 更新明细 actual_qty
- 将 SQLx 错误转换为项目错误类型

禁止：

- 判断订单是否可更新
- 判断订单是否可完成
- 判断订单是否可取消
- 判断库存是否足够
- 调用 InventoryService
- 调用 OutboundService
- 构造 HTTP Response
- 读取或修改前端文件
- 修改数据库 schema
- 新增表或枚举值
- 引入 locked_quantity

Repository 写方法必须能被 Service 纳入事务。

如果项目现有 Repository 已采用可同时接受 `&PgPool` 与事务执行器的模式，OutboundRepository 必须沿用同一模式。

---

## 8. Service Boundary

Service 层负责业务规则、状态流转与事务编排。

允许：

- 校验仓库、商品、库位存在
- 校验库位归属来源仓库
- 校验 items 非空
- 校验数量合法
- 生成出库单号
- 校验 draft / completed / cancelled 状态流转
- 调用 Repository
- 调用 InventoryService
- 处理库存不足业务错误
- 控制事务
- 组织返回 DTO

禁止：

- 解析 HTTP Path / Query / Json
- 构造 Axum Response
- 直接承载前端交互逻辑
- 修改数据库 schema
- 引入审批状态
- 引入库存锁定
- 实现 Epic 6 库存查询服务
- 实现测试任务
- 实现 Seed 任务

Service 必须保证：

- complete 是单事务
- completed cancel 是单事务
- 库存快照与库存流水同事务提交
- 库存不足时无部分扣减
- 状态更新与库存变化一致

---

## 9. Transaction Design

### 创建出库单事务

事务步骤：

1. 校验请求基本字段
2. 校验仓库存在
3. 校验每条明细商品存在
4. 校验每条明细库位存在且属于来源仓库
5. 生成订单号
6. 插入 `outbound_orders`
7. 插入 `outbound_order_items`
8. 提交事务

创建时不得：

- 扣减库存
- 写库存流水
- 锁定库存

### 更新出库单事务

事务步骤：

1. 查询出库单并校验状态为 `draft`
2. 校验请求基本字段
3. 校验仓库、商品、库位和库位归属
4. 更新 `outbound_orders`
5. 删除原明细
6. 插入新明细
7. 提交事务

更新时不得：

- 扣减库存
- 写库存流水
- 修改 completed_at

### 完成出库事务

事务步骤：

1. 查询出库单和明细
2. 校验订单状态为 `draft`
3. 校验完成请求覆盖所有明细
4. 校验请求中无重复 item_id
5. 校验每个 item_id 属于当前出库单
6. 校验 actual_qty 合法
7. 对每条明细调用库存扣减能力
8. 库存扣减时写 outbound 流水
9. 写入每条明细 actual_qty
10. 更新订单状态为 `completed`
11. 写入 completed_at
12. 提交事务

如果任一明细库存不足：

- 返回业务错误
- 回滚整个事务
- 不得更新任何明细 actual_qty
- 不得更新订单状态
- 不得保留任何部分库存扣减
- 不得写入部分流水

### 取消出库单事务

draft 取消：

1. 查询出库单
2. 校验状态为 `draft`
3. 更新状态为 `cancelled`
4. 提交事务

completed 取消：

1. 查询出库单和明细
2. 校验状态为 `completed`
3. 对每条明细按 actual_qty 增加库存
4. 增加库存时写 inbound 流水
5. 更新订单状态为 `cancelled`
6. 提交事务

cancelled 取消：

- 返回业务错误
- 不做任何库存操作

---

## 10. Frontend UX Rules

### 通用规则

- 使用 Ant Design
- 使用 TanStack Query 管理服务端状态
- 使用现有 Axios client
- 使用现有 AppLayout 与 Router 风格
- 页面信息密度与入库模块保持一致
- 所有关键操作有 toast 反馈
- 完成出库和取消操作需要二次确认

### 出库列表 UX

- 默认展示出库单列表
- 搜索条件保持轻量
- 状态使用彩色 Tag
- draft 显示编辑、完成、取消
- completed 显示查看，可按后端能力提供取消入口
- cancelled 只读查看
- 不展示审批按钮
- 不展示库存流水入口

### 出库表单 UX

- 表单与入库表单保持对称
- 出库类型使用 Select
- 来源仓库变化时清空不匹配库位
- 商品选择器支持搜索
- 库位选择器限定来源仓库
- 可用库存显示为当前商品 + 仓库 + 库位的 quantity
- 数量超过可用库存时使用红色提示
- 前端提示不能替代后端库存校验

### 出库详情 UX

- 基本信息只读展示
- 明细只读展示
- draft 状态可完成出库
- 完成出库 Modal 输入每行实发数量
- 完成前二次确认，明确提示库存将扣减
- 库存不足时展示后端业务错误
- completed / cancelled 页面保持只读

---

## 11. Out Of Scope

Epic 5 明确不包含：

- 修改数据库 schema
- 新增表
- 新增枚举值
- 商品分类
- `products.category_id`
- `inventories.locked_quantity`
- 库存锁定
- `lock_stock()`
- `unlock_stock()`
- `pending`
- `approved`
- submit 流程
- approve 流程
- JWT / RBAC / 认证系统
- 库存查询页面
- 库存流水前端页面
- Seed Data
- 自动化测试任务
- 入库业务重构
- 主数据业务重构
- 仓库调拨
- 批次 / 效期 / 序列号

---

## 12. Task Dispatch Boundaries

### T5.1.1 定义 OutboundOrder / OutboundOrderItem Model/DTO

目标：

定义出库单 Model 与 DTO。

允许修改文件：

- `backend/src/model/mod.rs`
- `backend/src/model/outbound.rs`
- `backend/src/dto/mod.rs`
- `backend/src/dto/outbound.rs`
- `backend/src/main.rs`（仅必要模块声明，如当前项目模式需要）

禁止修改文件：

- `_ai/tasks/TASK_BOARD.md`
- `_ai/openspec/specs/*`
- `AGENTS.md`
- `backend/migrations/*`
- `backend/src/repository/*`
- `backend/src/service/*`
- `backend/src/handler/*`
- `backend/src/router.rs`
- `frontend/*`

禁止越界内容：

- 不实现 Repository
- 不实现 Service
- 不实现 Handler
- 不注册路由
- 不实现前端
- 不实现库存扣减逻辑
- 不实现流水写入逻辑
- 不修改数据库 schema

### T5.1.2 实现 OutboundOrder Repository

目标：

实现出库单数据访问能力。

允许修改文件：

- `backend/src/repository/mod.rs`
- `backend/src/repository/outbound_repo.rs`
- `backend/src/model/outbound.rs`（仅 Repository 接入所需小修）
- `backend/src/dto/outbound.rs`（仅 Repository 响应适配小修）
- `backend/src/main.rs`（仅必要模块声明，如当前项目模式需要）

禁止修改文件：

- `_ai/tasks/TASK_BOARD.md`
- `_ai/openspec/specs/*`
- `AGENTS.md`
- `backend/migrations/*`
- `backend/src/service/*`
- `backend/src/handler/*`
- `backend/src/router.rs`
- `frontend/*`

禁止越界内容：

- 不实现 Service
- 不实现 Handler
- 不注册路由
- 不实现前端
- 不修改库存服务语义
- 不实现库存查询 API
- 不修改数据库 schema

### T5.1.3 实现 OutboundService

目标：

实现出库单业务编排服务。

允许修改文件：

- `backend/src/service/mod.rs`
- `backend/src/service/outbound_service.rs`
- `backend/src/repository/outbound_repo.rs`（仅 Service 接入所需小修）
- `backend/src/dto/outbound.rs`（仅 Service 返回适配小修）
- `backend/src/error.rs`（仅复用现有错误不足时的小范围补充，需谨慎）
- `backend/src/main.rs`（仅必要模块声明，如当前项目模式需要）

禁止修改文件：

- `_ai/tasks/TASK_BOARD.md`
- `_ai/openspec/specs/*`
- `AGENTS.md`
- `backend/migrations/*`
- `backend/src/handler/*`
- `backend/src/router.rs`
- `frontend/*`

禁止越界内容：

- 不实现 Handler
- 不注册路由
- 不实现前端
- 不实现库存查询页面
- 不引入审批流
- 不引入库存锁定
- 不修改数据库 schema

### T5.1.4 实现 OutboundOrder Handler 并注册路由

目标：

实现出库单 HTTP Handler，并将出库单 API 接入真实路由。

允许修改文件：

- `backend/src/handler/mod.rs`
- `backend/src/handler/outbound_handler.rs`
- `backend/src/router.rs`
- `backend/src/service/outbound_service.rs`（仅 Handler 接入所需小修）
- `backend/src/dto/outbound.rs`（仅请求/响应接入小修）
- `backend/src/main.rs`（仅必要模块声明，如当前项目模式需要）

禁止修改文件：

- `_ai/tasks/TASK_BOARD.md`
- `_ai/openspec/specs/*`
- `AGENTS.md`
- `backend/migrations/*`
- `frontend/*`
- 库存查询 Handler
- Seed / Testing 相关文件

禁止越界内容：

- 不实现前端页面
- 不实现库存查询 API
- 不新增认证
- 不新增审批
- 不修改数据库 schema

### T5.2.1 实现出库单列表页面

目标：

实现 `/outbounds` 出库单列表页面。

允许修改文件：

- `frontend/src/api/outbound.ts`
- `frontend/src/types/outbound.ts`
- `frontend/src/pages/OutboundList.tsx`
- `frontend/src/router/index.tsx`
- `frontend/src/api/warehouse.ts`（仅仓库下拉复用所需）
- `frontend/src/types/warehouse.ts`（仅类型复用小修）
- `frontend/src/components/*`（仅当前列表专用组件时）

禁止修改文件：

- `_ai/tasks/TASK_BOARD.md`
- `_ai/openspec/specs/*`
- `AGENTS.md`
- `backend/*`
- `frontend/src/pages/InventoryQuery.tsx`
- 与当前任务无关的主数据页面

禁止越界内容：

- 不实现出库单完整表单
- 不实现出库单详情完成 Modal
- 不实现库存查询页面
- 不修改后端
- 不新增审批按钮

### T5.2.2 实现出库单表单（含可用库存显示）

目标：

实现出库单新增和 draft 编辑表单，并展示当前明细可用库存。

允许修改文件：

- `frontend/src/pages/OutboundForm.tsx`
- `frontend/src/api/outbound.ts`
- `frontend/src/types/outbound.ts`
- `frontend/src/router/index.tsx`
- `frontend/src/api/product.ts`（仅商品选择复用所需）
- `frontend/src/api/warehouse.ts`（仅仓库选择复用所需）
- `frontend/src/api/location.ts`（仅库位选择复用所需）
- `frontend/src/api/inventory.ts`（仅查询当前商品+仓库+库位 quantity 所需）
- `frontend/src/types/product.ts`
- `frontend/src/types/warehouse.ts`
- `frontend/src/types/location.ts`
- `frontend/src/types/inventory.ts`（仅最小库存 quantity 类型所需）
- `frontend/src/components/*`（仅当前表单专用组件时）

禁止修改文件：

- `_ai/tasks/TASK_BOARD.md`
- `_ai/openspec/specs/*`
- `AGENTS.md`
- `backend/*`
- 完整库存查询页面
- 与当前任务无关的主数据业务逻辑

禁止越界内容：

- 不实现 InventoryQuery 页面
- 不实现库存流水页面
- 不修改后端
- 不新增审批流
- 不实现库存锁定

### T5.2.3 实现出库单详情与完成操作

目标：

实现出库单详情页面，支持查看基本信息、查看明细、完成出库、取消和返回。

允许修改文件：

- `frontend/src/pages/OutboundDetail.tsx`
- `frontend/src/api/outbound.ts`
- `frontend/src/types/outbound.ts`
- `frontend/src/pages/OutboundList.tsx`（仅完成 / 查看跳转接入小修）
- `frontend/src/router/index.tsx`
- `frontend/src/components/*`（仅当前详情或完成 Modal 专用组件时）

禁止修改文件：

- `_ai/tasks/TASK_BOARD.md`
- `_ai/openspec/specs/*`
- `AGENTS.md`
- `backend/*`
- 库存查询页面
- 库存流水页面
- 与当前任务无关的主数据页面

禁止越界内容：

- 不实现库存查询页面
- 不实现库存流水页面
- 不修改后端
- 不新增审批流
- 不新增 submit / approve UI

---

## 13. Review Checklist

### T5.1.1 Review 重点

- Model 是否对齐 `outbound_orders` / `outbound_order_items`
- status 是否仅 `draft` / `completed` / `cancelled`
- order_type 是否仅 `sales` / `manual` / `scrap`
- DTO 是否覆盖创建、更新、完成、列表、详情
- 数量字段是否使用 Decimal
- 是否未实现 Repository / Service / Handler / Router
- 是否未修改 migrations / frontend / TASK_BOARD / specs
- 是否无 submit / approve / pending / approved / locked_quantity

### T5.1.2 Review 重点

- `outbound_repo.rs` 方法是否齐全
- list 是否支持 keyword / warehouse / status / date range / pagination
- 详情是否包含商品、SKU、仓库、库位展示字段
- SQL 是否参数化
- 写方法是否支持事务执行上下文
- Repository 是否未承载业务状态流转
- Repository 是否未判断库存是否足够
- 是否未实现 Service / Handler / Frontend

### T5.1.3 Review 重点

- `OutboundService` 方法是否齐全
- create 是否生成正确单号
- create / update 是否校验仓库、商品、库位和库位归属
- update 是否仅允许 draft
- complete 是否完整事务
- complete 是否按 actual_qty 扣减库存并写 outbound 流水
- 库存不足是否返回业务错误并回滚
- cancel 是否支持 draft 和 completed
- completed cancel 是否按 actual_qty 回滚库存并写流水
- 是否复用 InventoryService
- 是否未解析 HTTP
- 是否未实现 Handler / Router / Frontend
- 是否未引入审批流或库存锁定

### T5.1.4 Review 重点

- 6 个出库 API 是否全部接入
- Handler 是否只做请求解析和响应封装
- 是否复用 OutboundService
- 响应格式是否统一
- 错误码语义是否符合现有 AppError
- Router 是否仅替换 outbound-orders stub
- submit / approve / lock / unlock API 是否不存在
- Inventory 是否未提前接入 Epic 6 真实业务

### T5.2.1 Review 重点

- `/outbounds` 是否为真实列表页
- 搜索条件是否包含单号、仓库、状态、日期范围
- 表格列是否符合 UI 设计
- 操作列是否按状态显示
- 新增入口是否可用
- 分页是否可用
- API client 是否复用 `client.ts`
- 是否未实现库存查询页面
- 是否无审批 UI

### T5.2.2 Review 重点

- 表单基本信息是否完整
- 明细行是否包含商品、库位、可用库存、计划数量
- 库位是否限定来源仓库
- 商品选择是否复用现有商品 API
- 可用库存显示是否只服务当前出库表单
- 超量是否有红色警告
- 保存草稿是否调用正确 API
- 编辑 draft 是否可回填
- 非 draft 是否不可编辑
- 是否未实现完整库存查询页面
- 是否未修改后端

### T5.2.3 Review 重点

- 详情基本信息和明细是否完整
- draft 是否可完成出库
- 完成 Modal 是否逐行输入实发数量
- 完成前是否二次确认
- complete API 调用是否正确
- 库存不足错误是否清楚展示
- 完成后是否刷新状态
- 取消操作是否有确认
- completed / cancelled 是否只读
- 是否无提交 / 审批按钮
- 是否未实现库存流水页面或库存查询页面

---

## 14. Key Risks

| 风险 | 表现 | 缓解方案 |
|------|------|----------|
| 库存不足后部分扣减 | 多明细前几条扣减成功，后续失败后未回滚 | complete 必须单事务；Review 检查事务边界 |
| 库存与流水不一致 | 库存变化成功但未写流水，或流水 before/after 错误 | 复用 InventoryService，库存与流水同事务 |
| completed 取消回滚错误 | 回滚数量使用 planned_qty 或重复回滚 | cancel completed 必须使用 actual_qty，且状态校验不可重复取消 |
| actual_qty 与 planned_qty 混淆 | 完成出库时按 planned_qty 扣减库存 | complete 必须明确使用 actual_qty |
| 库存方向错误 | 出库完成写 inbound，取消回滚写 outbound | 完成出库使用 outbound；取消回滚使用 inbound |
| Repository 承载业务 | Repo 判断状态流转或库存是否足够 | Repository Boundary Review 明确拒绝 |
| 提前引入库存锁定 | 出现 locked_quantity / lock_stock / unlock_stock | 全文搜索并 Reject |
| 误引入审批流 | 出现 pending / approved / submit / approve | 全文搜索并 Reject |
| 前端出库表单扩展成库存查询页 | InventoryQuery 或流水页面被真实实现 | T5.2.2 Scope Check 禁止 |
| 越界修改入库流程 | 为复用代码改坏 Epic4 已完成能力 | 除必要共享类型小修外不得改入库业务 |
| Decimal 精度错误 | 使用 f64 处理 NUMERIC | Model / DTO / Service Review 检查 Decimal |
| 单号并发冲突 | 同日序列生成重复 | Service 生成时应考虑数据库唯一约束和冲突处理 |
| 库位归属漏校验 | 出库单仓库与明细库位仓库不一致 | Service 创建、更新、完成前校验 |

---

## 15. Epic Completion Criteria

Epic 5 完成条件：

- T5.1.1 Review Approve
- T5.1.2 Review Approve
- T5.1.3 Review Approve
- T5.1.4 Review Approve
- T5.2.1 Review Approve
- T5.2.2 Review Approve
- T5.2.3 Review Approve
- 出库单 Model / DTO 可用
- 出库单 Repository 可用
- OutboundService 可用
- 出库单 Handler 和 Router 可用
- `GET /api/v1/outbound-orders` 可查询列表
- `GET /api/v1/outbound-orders/{id}` 可查询详情
- `POST /api/v1/outbound-orders` 可创建出库单
- `PUT /api/v1/outbound-orders/{id}` 可编辑 draft 出库单
- `POST /api/v1/outbound-orders/{id}/complete` 可完成出库
- `POST /api/v1/outbound-orders/{id}/cancel` 可取消出库单
- 完成出库前能校验库存
- 库存不足能拒绝完成并回滚
- 完成出库后库存扣减
- 完成出库后库存流水写入
- completed 出库单取消后库存可回滚
- 出库列表页面可用
- 出库新增 / 编辑表单可用
- 出库详情页面可用
- 完成出库 Modal 可用
- 取消操作可用
- 无审批流
- 无 submit / approve
- 无 pending / approved
- 无库存锁定
- 无 locked_quantity
- 无库存查询页面真实实现
- 无库存流水前端页面
- 无数据库 schema 修改
- 后端 `cargo fmt --check` 通过
- 后端 `cargo build` 通过
- 前端 `npm run build` 通过

Epic 5 完成后，系统应支持：

- 创建出库单
- 编辑 draft 出库单
- 完成出库
- 库存扣减
- 库存不足拒绝出库
- 流水记录
- 查看出库详情
- 出库列表查询
- 取消出库单
- 无审批流
- 无库存锁定

---

> 本文档为 Epic 5 唯一实施细则文档，不是单任务派发包。Reasonix 执行时仍必须以 `_ai/tasks/TASK_BOARD.md` 当前任务为唯一状态真相源，并按 T5.1.1 -> T5.1.2 -> T5.1.3 -> T5.1.4 -> T5.2.1 -> T5.2.2 -> T5.2.3 的 Review Gate 顺序逐项推进。
