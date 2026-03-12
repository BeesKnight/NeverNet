# EventDesign

EventDesign is a full-stack graduation project for event planning and event management. It covers:

- registration and authentication
- category management
- event CRUD
- filtering and search
- reports by period and category
- PDF and XLSX export jobs
- persisted UI theme settings
- calendar view

The implementation is a practical modular monolith: one React frontend and one Rust backend backed by PostgreSQL.

## Repository Layout

- `frontend/` React + TypeScript + Vite
- `backend/` Rust + Axum + SQLx
- `docs/` project documentation

## Stack

- Frontend: React, TypeScript, Vite, React Router, TanStack Query
- Backend: Rust, Axum, SQLx, PostgreSQL

## Local Run

### 1. Start PostgreSQL

```bash
docker compose up -d db
```

### 2. Configure environment files

Backend:

```bash
copy backend\.env.example backend\.env
```

Frontend:

```bash
copy frontend\.env.example frontend\.env
```

Update `backend/.env` and set a real `JWT_SECRET` before regular use.

### 3. Start the backend

```bash
cd backend
cargo run
```

The backend runs on `http://localhost:8080` and applies SQL migrations automatically on startup.

### 4. Start the frontend

```bash
cd frontend
npm install
npm run dev
```

The frontend runs on `http://localhost:5173`.

## Useful Endpoints

- `GET /health`
- `POST /api/auth/register`
- `POST /api/auth/login`
- `GET /api/auth/me`
- `GET|POST /api/categories/`
- `PUT|DELETE /api/categories/:id`
- `GET|POST /api/events/`
- `GET|PUT|DELETE /api/events/:id`
- `GET /api/reports/summary`
- `GET|PUT /api/settings/`
- `GET|POST /api/exports/`
- `GET /api/exports/:id/download`

## Checks

Backend:

```bash
cd backend
cargo fmt
cargo check
cargo test
```

Frontend:

```bash
cd frontend
npm run lint
npm run build
```

## Notes

- Export files are written to `backend/storage/exports/`.
- Export processing is asynchronous but runs inside the backend process to keep local setup simple.
- Category deletion is blocked when events still reference that category.
- Theme preferences are persisted per user in `ui_settings`.

## Architecture

See [docs/architecture.md](/docs/architecture.md).
