# EventDesign Architecture

## Overview

EventDesign is a high-load inspired full-stack system for event planning and event operations.

The repository has now reached the Phase 3 hardening and defense-polish state:
- the public surface is a single Edge API / BFF
- write and read responsibilities are split across internal Rust services
- major write-side changes flow through an outbox plus NATS JetStream
- read-heavy screens use projection tables
- browser auth is cookie-based and CSRF-protected
- local observability is wired through Prometheus and Grafana

## Phase 3 Status

Implemented now:

- backend workspace split into focused app crates and shared crates
- `edge-api` is the only published backend entrypoint
- `identity-svc` persists durable session rows in PostgreSQL and revokes them on logout
- browser auth uses an HttpOnly `eventdesign_session` cookie and the normal UI flow no longer uses `localStorage` bearer tokens
- `GET /api/auth/csrf` issues a CSRF token and Edge API enforces double-submit CSRF checks on `POST`, `PATCH`, and `DELETE`
- CORS is restricted by `FRONTEND_ORIGINS`
- Edge API rate limits general API traffic and auth traffic separately through Redis counters
- request ids propagate from incoming HTTP requests into internal gRPC calls
- each Rust service exposes `/metrics` and `/healthz` on a dedicated metrics port
- `event-command-svc` owns category and event mutations and writes outbox rows transactionally
- worker relays outbox rows to NATS JetStream and processes projection and export consumers
- `event-query-svc` serves event list, calendar, dashboard, and report-summary reads from projections
- dashboard responses are cached in Redis and invalidated from projection updates
- `report-svc` owns export job creation, metadata lookup, and MinIO-backed downloads
- `demo-seed` can populate a defense-ready user, categories, events, projections, and export history
- frontend dashboard, reports, calendar, settings, and auth flows now include stronger loading, empty, and error states
- backend and frontend tests cover the main product paths

Remaining rough edges:

- settings still execute inside `edge-api`
- Loki or Promtail is not wired yet; local logs are structured JSON or pretty stdout only
- full DB-backed backend tests still require a reachable local PostgreSQL and storage stack when run outside Docker Compose

Architecture style:

- React + TypeScript frontend
- Edge API / BFF as the only public backend entrypoint
- internal Rust services over gRPC
- CQRS split for write and read paths
- async event backbone through outbox plus NATS JetStream
- PostgreSQL as the main source of truth
- Redis for dashboard cache, rate limiting, and worker coordination
- MinIO/S3-compatible storage for generated export files
- worker processes for projections and heavy background tasks
- local observability stack for logs, metrics, and tracing

## Top-level Architecture

```text
Browser
  -> Frontend static hosting
  -> Edge API / BFF

Edge API / BFF
  -> request ids + CORS + CSRF + Redis rate limiting
  -> Identity Service (gRPC)
  -> Event Command Service (gRPC)
  -> Event Query Service (gRPC)
  -> Report Service (gRPC)
  -> settings compatibility module

Identity Service
  -> PostgreSQL users + sessions

Event Command Service
  -> PostgreSQL write schema + outbox

Worker
  -> outbox relay -> NATS JetStream
  -> projection consumer
  -> export consumer

Event Query Service
  -> projection/read schema
  -> Redis dashboard cache

Report Service
  -> export job metadata
  -> MinIO-backed download reads

Observability
  -> Prometheus scrapes service metrics + NATS varz
  -> Grafana dashboards

Frontend
  -> communicates only with Edge API / BFF
```

## Architectural Goals

The architecture provides:

- a stable external API for the frontend
- separation of authentication from business logic
- separation of write and read concerns
- support for fast dashboard, calendar, and reporting reads
- support for asynchronous report export generation
- support for future scaling of hot paths
- a defense-ready observability and security story

## Main Components

### Frontend

Frontend responsibilities:

- authentication UI
- dashboard
- categories
- events
- calendar
- reports
- settings
- export job status UI

Frontend contains UI behavior and client-side request orchestration only.
It does not call internal services directly.

### Edge API / BFF

This is the only public API.

Responsibilities:

- authenticate incoming requests
- validate and normalize input
- enforce CSRF checks on state-changing browser requests
- enforce config-driven CORS
- apply request correlation and rate limiting
- provide a stable REST API for the frontend
- orchestrate calls to internal services
- hide internal topology from the frontend
- expose UI-oriented response shapes

### Identity Service

Responsibilities:

- user registration
- password hashing
- login and logout
- durable session issuing and validation
- user profile lookup

### Event Command Service

Responsibilities:

- create, update, and delete categories
- create, update, and delete events
- enforce ownership checks
- enforce write-path business rules
- write domain events into the outbox transactionally

### Event Query Service

Responsibilities:

- fast event list reads
- dashboard summaries
- calendar reads
- filtered and sorted event projections
- report previews and aggregates
- dashboard cache usage and invalidation

This service reads from projection tables rather than the write model.

### Report Service

Responsibilities:

- create export jobs
- report job status lookup
- secure artifact metadata lookup
- secure MinIO-backed download flow

### Workers

Workers are responsible for:

- relaying unpublished outbox rows to JetStream
- consuming domain events
- updating read models and recent activity projections
- invalidating dashboard cache
- processing export jobs
- generating PDF and XLSX artifacts
- uploading files to MinIO

## Data Flow Types

### Synchronous flows

Used for user-facing critical operations:

- register
- login
- logout
- create category
- create event
- update event
- open dashboard
- open calendar
- open reports

### Asynchronous flows

Used for system decoupling and heavy work:

- projection refresh
- dashboard cache invalidation
- export generation
- recent activity projection updates

## Why CQRS Is Used

CQRS separates:

- the write model for correctness and transactional safety
- the read model for fast UI queries, sorting, filtering, calendar rendering, and reporting

The product has very different write and read access patterns, so the split is practical rather than decorative.

## Why Event-Driven Async Flow Is Used

The async event backbone makes it possible to:

- keep write requests fast
- decouple projection updates from command handling
- support export processing independently of request latency
- retry durable publication instead of losing events between DB write and queue publish

## Security Model

Implemented browser security controls:

- Argon2 password hashing
- HttpOnly `eventdesign_session` cookie
- `SameSite=Lax` cookies with configurable `Secure`
- durable `sessions` table in PostgreSQL
- logout revocation through the session row
- ownership checks across categories, events, settings, export jobs, and downloads
- `eventdesign_csrf` cookie plus `X-CSRF-Token` header on state-changing requests
- CORS restricted to configured `FRONTEND_ORIGINS`
- Redis-backed Edge API rate limiting for auth and general API windows
- secrets and service endpoints loaded from environment variables

Compatibility note:

- `Authorization: Bearer` fallback still exists for non-browser tooling, but the browser flow uses cookies only

## Storage Model

### PostgreSQL

PostgreSQL is the primary source of truth.

It stores:

- users
- sessions
- categories
- events
- ui settings
- outbox events
- export jobs
- read model tables

### Redis

Redis stores:

- dashboard cache entries
- rate-limit counters
- worker locks and short-lived coordination keys

### MinIO

MinIO stores:

- generated PDF files
- generated XLSX files
- export artifact object keys referenced by `export_jobs`

## Observability

The local stack now supports:

- structured logs through `tracing` and `tracing-subscriber`
- request ids and correlation across HTTP and gRPC
- Prometheus metrics from every Rust service
- Grafana dashboards provisioned from the repository

Current local observability pieces:

- `LOG_FORMAT=json|pretty` switching for service logs
- `/metrics` and `/healthz` on each Rust service metrics port
- `x-request-id` on Edge API responses
- Prometheus scraping:
  - `edge-api:9100`
  - `identity-svc:9101`
  - `event-command-svc:9102`
  - `event-query-svc:9103`
  - `report-svc:9104`
  - `worker:9105`
  - `nats:8222/varz`
- Grafana dashboard `EventDesign Overview`

Useful metrics currently exposed:

- request rate, latency, and status distribution
- export duration
- projection lag
- worker queue lag
- dashboard cache hit or miss
- security events such as CSRF rejection, invalid session use, and rate limiting

Current limitation:

- Loki is not part of the local Compose stack yet

## Development Constraints

This architecture should look production-grade while remaining implementable.

Important constraints:

- do not add unnecessary services
- do not add Kafka when NATS JetStream is sufficient
- do not create internal frameworks
- do not replace working components without clear reason
- keep contracts explicit and documented
- prefer small, verifiable migration steps
