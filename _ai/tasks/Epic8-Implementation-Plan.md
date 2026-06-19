# Epic 8: Seed Data - 实施方案

> 版本：v1.0
> 角色：Tech Lead / Codex
> 目标读者：Reasonix / Codex Reviewer / 后续交接 AI
> 预计总工时：3h
> 依据：`_ai/tasks/TASK_BOARD.md`、`AGENTS.md`、`_ai/AI_协作开发工作流.md`、`_ai/openspec/specs/*`

---

## 1. Epic 概述

Epic 7 已完成并通过 Review。Epic 8 为 WMS MVP 提供最小、可重复执行的演示主数据，使新环境能够立即创建入库单和出库单，而无需人工先录入商品、仓库和库位。

本 Epic 仅覆盖：

- 示例商品、默认仓库和默认库位的种子数据。
- 一个显式 CLI 种子入口。
- 空数据库启动时的自动种子执行。
- 使用既有 API 与前端页面验证种子数据可用。

任务状态、验收标准与依赖关系以 `TASK_BOARD.md` 为唯一真相源。本计划定义实现边界和技术决策，不替代任务板。

---

## 2. 架构决策与范围边界

### 2.1 固定决策

1. 种子脚本固定为 `backend/migrations/seed.sql`，但它不是 SQLx schema migration，不得作为第十张表或修改既有迁移。
2. 手动入口固定为 `cargo run -- --seed`；不新增 `POST /api/v1/seed`，避免将初始化能力暴露为运行时 HTTP 接口。
3. 普通 `cargo run` 启动时，仅当 `products`、`warehouses` 与 `locations` 三类主数据均为空才执行自动种子；任一表已有数据则跳过并记录日志。
4. 手动入口可重复执行；脚本对既有同业务编码的数据不报错、不重复插入，且不覆盖用户已修改的数据。
5. 种子数据只创建主数据，不创建库存、单据或库存流水。

### 2.2 全程禁止

- 新增或修改数据库表、列、索引、约束及现有迁移文件。
- 修改 `_ai/openspec/specs/*`、`_ai/tasks/TASK_BOARD.md` 或 `AGENTS.md`。
- 新增分类、`category_id`、`locked_quantity`、审批状态或库存锁定能力。
- 新增 Seed HTTP API、前端路由、菜单或页面。
- 提前实现 Epic 之外的功能，或一次实现多个 Task。

最终 MVP 决议优先于早期 OpenSpec 中的历史描述：数据库维持 9 张表；商品无分类；库存仅有 `quantity`；单据状态仅为 `draft/completed/cancelled`。

---

## 3. 种子数据设计

### 3.1 业务标识

种子数据以稳定、可读的业务编码识别，UUID 继续由数据库生成。建议采用以下编码，便于演示、排查及幂等冲突处理：

| 实体 | 稳定标识 | 说明 |
|---|---|---|
| 商品 | `DEMO-001` 至 `DEMO-005` | 五个启用状态的示例商品，包含 `sku_code`、名称与单位 |
| 仓库 | `WH-MAIN` | 默认“主仓库”，启用状态 |
| 库位 | `A-01-01`、`A-01-02`、`RECV-01`、`SHIP-01` | 全部属于 `WH-MAIN`，类型与其作业语义一致，启用状态 |

商品名称、规格与条码可作为示例字段，但不得形成分类或库存预置。库位通过仓库编码查询对应 `warehouse_id` 后插入，保证仓库与库位的外键关系正确。

### 3.2 幂等策略

- 商品以 `sku_code` 冲突为准，使用不覆盖既有记录的插入策略。
- 仓库以 `code` 冲突为准，使用不覆盖既有记录的插入策略。
- 库位以既有的 `(warehouse_id, code)` 唯一关系为准，使用不覆盖既有记录的插入策略。
- 脚本在单一数据库事务内执行；任何步骤失败时整体回滚。
- 不用删除、截断或更新操作修复现有数据。

这使 `--seed` 可以作为安全的补种入口，同时保证自动种子只在真正空的主数据环境发生。

---

## 4. 任务实施计划

### T8.1.1 编写 Seed Data SQL 脚本

允许修改：

- `backend/migrations/seed.sql`

实现要求：

- 仅写入任务板指定的商品、仓库、库位示例数据。
- 与现有 `products`、`warehouses`、`locations` 的实际列、枚举值、唯一约束和外键兼容。
- 按“仓库先于库位”的依赖顺序编排，且使用稳定业务编码关联。
- SQL 自身具备可重复执行的幂等性，不依赖手工清库。

不得修改：

- 任何现有 schema migration。
- 后端运行时代码、路由、前端或测试基础设施。
- 设计文档和任务看板。

验证依据：`TASK_BOARD.md` 的 T8.1.1；至少执行 SQL 语法/数据库执行验证，并确认记录数和库位归属。

### T8.1.2 实现 Seed 运行入口

允许修改：

- `backend/src/main.rs`
- `backend/src/seed.rs`
- `backend/src/lib.rs`（仅注册 `seed` 模块）
- 必要时 `backend/src/config.rs`（仅为现有配置模式补充最小支持）
- `backend/Cargo.toml`（仅在确有必要时增加轻量依赖）

实现要求：

- 读取 `seed.sql` 的内容并在数据库连接池上执行；不重新实现 SQL 数据内容。
- `cargo run -- --seed` 执行种子后正常退出，不启动 HTTP 服务。
- 正常启动时检查三类主数据是否均为空；仅满足该条件时运行种子，然后继续启动服务。
- 同一启动路径的种子执行使用事务并输出明确的 tracing 日志。
- 数据库连接、错误传播与启动流程沿用既有配置和错误处理模式。

不得修改：

- Router、Handler、DTO、Repository、Service 的业务语义。
- API 路由或响应格式。
- 数据库迁移、前端、设计文档和任务看板。

验证依据：`TASK_BOARD.md` 的 T8.1.2。须分别验证显式 CLI、空库自动种子、重复运行的幂等性，以及普通服务启动行为。

### T8.1.3 验证 Seed 数据可用性

允许修改：

- `backend/tests/*`（仅在需要将验证固化为自动化测试时）
- 与验证记录直接相关的最小文档文件

验证要求：

- 先以 T8.1.2 的入口完成种子写入。
- 通过既有商品、仓库和库位查询能力确认预置数据可见。
- 在既有入库单创建界面确认商品、默认仓库与库位可被选择。
- 验证过程不新增前端功能、不写入库存、不创建或完成单据。

验证依据：`TASK_BOARD.md` 的 T8.1.3。若增加自动化测试，必须使用现有 `TEST_DATABASE_URL` 安全约束和可重复 fixture 规则。

---

## 5. 验证基线

每个 Task 完成时，按修改面执行最小充分验证：

```text
cd backend
cargo fmt --check
cargo build
cargo test
```

涉及前端可用性确认时，再执行：

```text
cd frontend
npm.cmd run build
```

数据验证使用隔离的开发或测试数据库。测试不得把 `TEST_DATABASE_URL` 回退到开发数据库，亦不得通过清空共享数据库来证明幂等性。

---

## 6. Review Gate

每一项按以下顺序独立推进：

```text
T8.1.1 实现 -> Codex Review Approve -> 更新 TASK_BOARD
-> T8.1.2 实现 -> Codex Review Approve -> 更新 TASK_BOARD
-> T8.1.3 验证 -> Codex Review Approve -> 更新 TASK_BOARD
```

Review 必须以代码、实际 SQL 和命令结果为准；Reasonix 报告只用于定位改动。发现需求外功能、schema 变更、HTTP Seed 端点、非幂等写入或未隔离验证时，应 Reject 并生成单独 Repair Prompt。

---

## 7. 风险与缓解

| 风险 | 缓解措施 |
|---|---|
| 种子脚本被误当作 schema migration 执行 | 明确 `seed.sql` 只由种子模块读取；不改现有 migration 文件。 |
| 重复执行污染或覆盖人工数据 | 使用业务唯一键的非覆盖插入；禁止 `DELETE`、`TRUNCATE` 与无条件 `UPDATE`。 |
| 部分初始化环境被自动补写 | 自动执行条件为三类主数据均为空；其他情况只记录跳过日志。 |
| HTTP 攻击面扩大 | 固定 CLI 方案，不新增 Seed API 与路由。 |
| 示例数据越界到库存或交易数据 | 种子脚本仅操作三张主数据表。 |
| 历史 Spec 与最终 MVP 决议冲突 | 以 `TASK_BOARD.md` 与最终版 API/UI/数据库约束为准。 |

---

## 8. Epic 完成条件

Epic 8 在以下条件全部满足后完成：

- T8.1.1、T8.1.2、T8.1.3 均获得 Codex Review Approve。
- 新数据库可获得任务板指定的示例主数据。
- CLI 与空库启动路径都能安全运行，重复执行不产生重复记录或覆盖人工数据。
- 现有商品、仓库、库位查询与入库单创建流程可使用这些数据。
- 没有 schema、禁用设计、前端路由或非必要 API 扩张。

---

## 9. 下一次派发

当前唯一允许派发的任务是：

```text
T8.1.1 - 编写 Seed Data SQL 脚本
```

派发和审查时应引用：

```text
AGENTS.md
_ai/AI_协作开发工作流.md
_ai/tasks/TASK_BOARD.md
_ai/tasks/Epic8-Implementation-Plan.md
```

不得复制任务板验收标准或本计划全文；在 T8.1.1 Review Approve 且由 Codex 更新任务板前，不得进入 T8.1.2 或 T8.1.3。
