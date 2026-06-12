# WMS

WMS is a warehouse management system MVP built with Rust, React, and PostgreSQL.

## Features

Current foundation:

- Rust backend service with Axum
- PostgreSQL 16 database setup with Docker Compose
- SQLx migration for the MVP schema
- Unified backend response format
- Request tracing, request id, and CORS middleware
- React + Vite frontend shell
- Ant Design application layout
- React Router based SPA routing
- Axios API client

Planned business modules:

- Product management
- Warehouse management
- Location management
- Inbound order management
- Outbound order management
- Inventory query

## Tech Stack

Backend:

- Rust
- Axum
- Tokio
- SQLx
- Serde
- tracing
- tower-http
- dotenvy

Frontend:

- React 18
- Vite 5
- TypeScript
- Ant Design 5
- React Router 6
- Axios
- TanStack Query

Database:

- PostgreSQL 16
- SQLx migrations

## Project Structure

```text
.
├── backend/             # Rust API service
├── frontend/            # React SPA
├── docker-compose.yml   # Local PostgreSQL service
├── README.md
└── .gitignore
```

## Database Schema

The MVP database contains 9 tables:

- `products`
- `warehouses`
- `locations`
- `inventories`
- `inbound_orders`
- `inbound_order_items`
- `outbound_orders`
- `outbound_order_items`
- `inventory_transactions`

The MVP intentionally does not include product categories, authentication, RBAC, approval workflows, or locked inventory quantities.

## Getting Started

### 1. Start PostgreSQL

```bash
docker compose up -d
```

PostgreSQL will be available at:

```text
localhost:5432
```

Default local credentials from `docker-compose.yml`:

```text
POSTGRES_USER=wms
POSTGRES_PASSWORD=wms123
POSTGRES_DB=wms
```

### 2. Configure Backend Environment

Create `backend/.env`:

```env
DATABASE_URL=postgres://wms:wms123@localhost:5432/wms
SERVER_PORT=3000
```

### 3. Run Database Migrations

From the backend directory:

```bash
sqlx migrate run
```

### 4. Start Backend

```bash
cargo run --manifest-path backend/Cargo.toml
```

Health check:

```bash
curl http://localhost:3000/api/v1/health
```

Expected response:

```json
{"code":0,"data":null,"message":"ok"}
```

### 5. Start Frontend

```bash
npm --prefix frontend install
npm --prefix frontend run dev
```

The frontend dev server will usually run at:

```text
http://localhost:5173
```

## Build

Backend:

```bash
cargo build --manifest-path backend/Cargo.toml
```

Frontend:

```bash
npm --prefix frontend run build
```

## API Status

The backend infrastructure is ready, and business routes are registered as placeholders. Business APIs will be implemented module by module.

Implemented:

- `GET /api/v1/health`

Business route groups prepared:

- Products
- Warehouses
- Locations
- Inbound orders
- Outbound orders
- Inventory

## License

License information has not been specified yet.
