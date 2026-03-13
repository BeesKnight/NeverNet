# EventDesign

EventDesign is a graduation project for event planning and event operations.

The repository is now in the Phase 1 foundation state:

- the backend is a Rust workspace
- `edge-api` is the only published backend entrypoint
- `identity-svc` handles registration, login, logout, and current-user lookup over gRPC
- browser auth uses an HttpOnly cookie flow with `credentials: include`
- current user-facing features still run end to end
- Docker Compose includes PostgreSQL, Redis, NATS JetStream, and MinIO

## Phase 1 architecture

```text
frontend/
backend/
  apps/
    edge-api/
    identity-svc/
    event-command-svc/
    event-query-svc/
    report-svc/
    worker/
  crates/
    contracts/
    shared-kernel/
    persistence/
    messaging/
    cache/
    observability/
```

What is live now:

- `edge-api` serves the REST API used by the browser
- `identity-svc` is the active internal auth service
- `event-command-svc`, `event-query-svc`, `report-svc`, and `worker` are bootable Phase 1 skeletons with explicit contracts

Phase 1 compatibility layers:

- categories, events, calendar, reports, settings, and exports still execute inside `edge-api`
- export files are still written to the shared local volume at `backend/storage/exports`
- the cookie session is a signed compatibility token and is not yet backed by durable session storage

## Supported product scope

The following features remain available during the migration:

- registration
- login and logout
- categories
- event CRUD
- filtering and search
- reports by period and category
- PDF and XLSX export jobs
- UI settings
- calendar view

## Docker Compose

Start the full stack:

```bash
docker compose up --build
```

Published endpoints:

- frontend: `http://localhost:3000`
- edge API: `http://localhost:8080`
- Redis: `localhost:6379`
- NATS JetStream client port: `localhost:4222`
- NATS monitoring: `http://localhost:8222`
- MinIO API: `http://localhost:9000`
- MinIO console: `http://localhost:9001`

Compose services:

- `frontend`
- `edge-api`
- `identity-svc`
- `event-command-svc`
- `event-query-svc`
- `report-svc`
- `worker`
- `db`
- `redis`
- `nats`
- `minio`

Stop the stack:

```bash
docker compose down
```

Remove the persistent volumes as well:

```bash
docker compose down -v
```

## Local development

1. Start infrastructure:

```bash
docker compose up -d db redis nats minio
```

2. Copy environment files:

```bash
copy backend\.env.example backend\.env
copy frontend\.env.example frontend\.env
```

3. Run backend services:

```bash
cd backend
cargo run -p identity-svc
cargo run -p edge-api
```

4. Run the frontend:

```bash
cd frontend
npm install
npm run dev
```

Local frontend development uses a Vite proxy to forward `/api` requests to `http://localhost:8080`.

## Quality checks

Backend:

```bash
cd backend
cargo fmt --all
cargo check --workspace
```

Frontend:

```bash
cd frontend
npm run lint
npm run build
```

## Known limitations

- the write/query/report services are scaffolded but not authoritative yet
- the async backbone is configured locally but not fully migrated into the business flow
- exports still complete through the `edge-api` compatibility path and shared filesystem storage
- CSRF protection and durable session persistence are still Phase 2 and Phase 3 work

See [docs/architecture.md](/docs/architecture.md) and [docs/delivery-plan.md](/docs/delivery-plan.md) for the migration direction.
