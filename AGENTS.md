# AGENTS.md

## Project identity

This repository contains **EventDesign**, a high-ambition graduation project for event planning and event operations.

The current repository already includes a working baseline implementation.
Do not discard it.
Rewrite it incrementally toward the target architecture described in `docs/`.

## How to read instructions

Before starting any substantial task, read these files in this order:

1. `docs/architecture.md`
2. `docs/domain-model.md`
3. `docs/messaging.md`
4. `docs/api.md`
5. `docs/delivery-plan.md`
6. `docs/demo-script.md`
7. `docs/erd.md`

If a task changes architecture, contracts, data flow, infra, deployment, auth, queueing, or demo behavior, update the affected docs in the same change.

## Primary migration rule

This repository must be migrated in **phases**.
Do not attempt a big-bang rewrite.
Preserve end-to-end functionality while moving to the target architecture.

Phase execution order:
1. foundation and workspace split
2. command/query separation and async spine
3. hardening, observability, polish, and defense preparation

Only work on the active phase unless explicitly asked to continue.

## Repository goals

The end state should look like a compact high-load style product while still being runnable locally.

Target qualities:
- a single external API entrypoint
- explicit service boundaries
- command/write and query/read separation
- asynchronous background processing
- durable event publication through outbox
- projection-driven fast reads
- secure browser auth
- production-like observability
- clear local startup path through Docker Compose

## Target stack

### Frontend
- React
- TypeScript
- Vite
- React Router
- TanStack Query

### Backend
- Rust workspace
- Axum for Edge API / BFF
- tonic gRPC for internal services
- SQLx
- PostgreSQL
- Redis
- NATS JetStream
- MinIO or S3-compatible storage

### Observability
- tracing
- OpenTelemetry
- Prometheus
- Grafana
- Loki

## Target architecture

The target architecture is:

- `frontend/`
- `backend/apps/edge-api`
- `backend/apps/identity-svc`
- `backend/apps/event-command-svc`
- `backend/apps/event-query-svc`
- `backend/apps/report-svc`
- `backend/apps/worker`
- `backend/crates/*`

Communication model:
- REST/JSON from frontend to edge-api
- gRPC between edge-api and internal services
- outbox pattern for durable domain events
- NATS JetStream for async event delivery
- PostgreSQL write model + projection/read model
- Redis for cache, rate limiting, locks, session acceleration
- MinIO/S3-compatible storage for exports

## Non-negotiable rules

- Do not delete working user-facing features unless they are replaced in the same phase.
- Do not introduce unnecessary services.
- Do not add Kafka when NATS JetStream is sufficient.
- Do not add wasm-based complexity.
- Do not let the frontend call internal services directly.
- Do not keep browser auth tokens in localStorage in the final architecture.
- Do not leave docs stale after architecture changes.
- Do not keep fake TODO implementations presented as complete.

## Product scope that must remain supported

The following features must remain available throughout migration:

- registration
- login/logout
- categories
- event CRUD
- filtering
- sorting
- reports by period and category
- PDF/XLSX export
- UI settings
- calendar view

## Security rules

Target auth model:
- HttpOnly cookie-based auth
- secure password hashing
- CSRF protection for state-changing browser requests
- ownership checks on all user resources
- CORS restricted to configured frontend origins
- no hardcoded secrets

Temporary compatibility may be used only during migration and must be removed before phase completion.

## Data and messaging rules

All business writes must go through command-side ownership boundaries.
All major write-side changes must become domain events through the outbox pipeline.
Read-side screens should rely on projection tables where the target architecture requires it.

Important event families:
- user
- category
- event
- export
- activity/audit

## Code quality rules

Before considering work complete:
- run formatting
- run linting
- run type checks
- run tests relevant to the change
- verify local startup still works where applicable
- update docs if behavior or architecture changed

## Review priorities

When reviewing your own work, check for:
- broken auth/session flow
- missing ownership checks
- command/query inconsistency
- projection drift
- outbox publication failures
- stale cache invalidation
- export job failure handling
- Docker Compose breakage
- stale docs
- TypeScript type drift
- missing Rust error handling

## Communication rules

When completing a task:
- summarize what changed
- summarize what remains
- list temporary compatibility layers
- list known limitations honestly

## Safe defaults

If something is ambiguous:
- choose the smallest implementation that still matches the target architecture
- prefer a working migration step over an idealized rewrite
- keep the repository understandable
