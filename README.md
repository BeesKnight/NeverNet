# NeverNet

NeverNet is a full-stack graduation project for event planning and event management. It is implemented as a practical modular monolith with a React frontend, a Rust backend, PostgreSQL persistence, asynchronous report exports, and a calendar view.

## Implemented scope

- registration, login, logout, password hashing, and protected API routes
- per-user category management with ownership checks
- full event CRUD with validation, filtering, and text search
- dashboard summaries for total, upcoming, completed, and cancelled events
- reports by date range and category with event-level detail
- PDF and XLSX export jobs with asynchronous processing
- persisted UI preferences: theme, accent color, and default start page
- monthly calendar view for event inspection

## Repository layout

- `frontend/` React + TypeScript + Vite
- `backend/` Rust + Axum + SQLx
- `docs/` architecture and project notes

## Docker Compose quick start

Run the full stack with one command:

```bash
docker compose up --build
```

Services started by Compose:

- `frontend` on `http://localhost:3000`
- `backend` on `http://localhost:8080`
- `postgres` on the internal Compose network

Compose already wires:

- PostgreSQL persistence
- backend migrations on startup
- frontend reverse proxy to `/api`
- persistent export storage via a Docker volume

To stop and remove containers:

```bash
docker compose down
```

To also remove the database and export volumes:

```bash
docker compose down -v
```

If you want a non-demo JWT secret, export `JWT_SECRET` before starting Compose.

## Build optimization

Container builds are optimized for repeat runs:

- backend Dockerfile reuses Cargo dependency layers before source copy
- BuildKit cache mounts are enabled for Cargo registry/git/target data
- frontend Dockerfile caches the npm package store and isolates dependency install
- both services use service-specific `.dockerignore` files to keep build context small
- frontend ships as static assets behind nginx, so the runtime image stays small

## Local setup without Docker

### 1. Start PostgreSQL

```bash
docker compose up -d db
```

### 2. Create environment files

Backend:

```bash
copy backend\.env.example backend\.env
```

Frontend:

```bash
copy frontend\.env.example frontend\.env
```

`backend/.env` uses the local Docker database by default. Replace `JWT_SECRET` before non-demo use.

### 3. Run the backend

```bash
cd backend
cargo run
```

The backend runs on `http://localhost:8080` and applies migrations automatically.

### 4. Run the frontend

```bash
cd frontend
npm install
npm run dev
```

The frontend runs on `http://localhost:5173`.

## API endpoints

Auth:

- `POST /api/auth/register`
- `POST /api/auth/login`
- `POST /api/auth/logout`
- `GET /api/auth/me`

Categories:

- `GET /api/categories`
- `POST /api/categories`
- `PATCH /api/categories/{id}`
- `DELETE /api/categories/{id}`

Events:

- `GET /api/events`
- `GET /api/events/{id}`
- `POST /api/events`
- `PATCH /api/events/{id}`
- `DELETE /api/events/{id}`

Reports and exports:

- `GET /api/reports/summary`
- `GET /api/reports/by-category`
- `GET /api/calendar`
- `GET /api/exports`
- `GET /api/exports/{id}`
- `POST /api/exports`
- `GET /api/exports/{id}/download`

Settings:

- `GET /api/settings`
- `PATCH /api/settings`

## Quality checks

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

- export files are written to `backend/storage/exports/`
- export jobs are asynchronous but intentionally stay inside the backend process to keep local setup simple
- category deletion is blocked while events still reference that category
- no demo seed is included; use the registration flow to create the first user

## Known limitations

- export processing is in-process, not a separate worker binary
- the dashboard recent activity is event-based and does not yet include an audit log
- PDF export is text-first and optimized for defendable completeness rather than polished print layout

See [docs/architecture.md](/docs/architecture.md) for the module breakdown.
