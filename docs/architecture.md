# EventDesign Architecture

## Overview

EventDesign is a high-load inspired full-stack system for event planning and event operations.

The architecture should look like a production-grade product, while still being practical to implement under deadline pressure.

## Phase 2 Status

The repository is currently in the CQRS and async-backbone phase.

Implemented now:

- backend workspace split into app crates and shared crates
- `edge-api` is the only published backend entrypoint
- `identity-svc` is live and handles auth over gRPC
- frontend auth uses an HttpOnly cookie flow and `credentials: include`
- Docker Compose starts PostgreSQL, Redis, NATS JetStream, and MinIO
- `event-command-svc` owns category and event mutations
- category and event mutations write durable outbox rows in the same transaction
- worker relays outbox rows to NATS JetStream and retries failed publishes
- `event-query-svc` serves event list, calendar, dashboard, and report-preview reads
- read-heavy screens are served from projection tables
- dashboard responses are cached in Redis and invalidated from projection updates
- `report-svc` owns export job creation, metadata lookup, and MinIO-backed download reads
- worker processes export jobs asynchronously and uploads generated files to MinIO

Phase 2 compatibility layers:

- `edge-api` still owns the external REST surface and UI-oriented response mapping
- settings still execute inside `edge-api` until a later migration step
- identity still uses the Phase 1 signed-cookie session compatibility path

Architecture style:

- React + TypeScript frontend
- Edge API / BFF as the only public backend entrypoint
- internal Rust services
- CQRS split for write and read paths
- async event backbone
- Redis for cache, rate limiting, short-lived coordination, and locks
- PostgreSQL as the main source of truth
- NATS JetStream for domain event delivery
- MinIO/S3-compatible storage for generated export files
- worker processes for projections and heavy background tasks
- observability stack for logs, metrics, and tracing

## Top-level architecture

```text
Browser
  -> Frontend static hosting
  -> Edge API / BFF

Edge API / BFF
  -> Identity Service (gRPC)
  -> Event Command Service (gRPC)
  -> Event Query Service (gRPC)
  -> Report Service (gRPC)
  -> settings compatibility module

Identity Service
  -> PostgreSQL
  -> signed cookie session compatibility

Phase 2 live flow
  -> Event Command Service -> PostgreSQL write schema + outbox
  -> Worker outbox relay -> NATS JetStream
  -> NATS JetStream -> projection worker + export worker
  -> Event Query Service -> projection/read schema + Redis dashboard cache
  -> Report Service -> export job metadata + MinIO downloads

Frontend
  -> communicates only with Edge API / BFF
```

## Architectural goals

The architecture must provide:

- a clean external API for the frontend
- separation of authentication from business logic
- separation of write and read concerns
- support for fast dashboard / calendar / reporting reads
- support for asynchronous report export generation
- support for future scaling of hot paths
- strong observability and strong defense value

## Main components

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

Frontend must not contain business logic beyond UI behavior and simple input validation.

### Edge API / BFF

This is the only public API.

Responsibilities:

- authenticate incoming requests
- validate and normalize input
- provide a stable REST API for the frontend
- orchestrate calls to internal services
- hide internal topology from the frontend
- expose a convenient UI-oriented contract

### Identity Service

Responsibilities:

- user registration
- password hashing
- login / logout
- session issuing and validation
- cookie-based authentication support
- user profile lookup

### Event Command Service

Responsibilities:

- create / update / delete categories
- create / update / delete events
- ownership checks
- status transition enforcement
- write-path business rules
- writing domain events to outbox

### Event Query Service

Responsibilities:

- fast event list reads
- dashboard summaries
- calendar reads
- filtered and sorted event projections
- report previews and aggregates

This service reads from projection tables and optimized read models.

### Report Service

Responsibilities:

- create export jobs
- report job status lookup
- secure artifact metadata lookup
- secure MinIO-backed download flow

### Workers

Workers are responsible for:

- relaying outbox rows to JetStream
- consuming domain events
- updating read models / projections
- invalidating dashboard cache
- processing export jobs
- generating PDF / XLSX artifacts
- uploading files to MinIO
- maintaining activity / audit records

## Data flow types

### Synchronous flows

Used for user-facing critical operations:

- register
- login
- create category
- create event
- update event
- open dashboard
- open calendar
- open reports

### Asynchronous flows

Used for system decoupling and heavy work:

- projection refresh
- cache invalidation
- export generation
- activity feed updates
- audit trail

## Why CQRS is used

CQRS is used to separate:

- **write model** for correctness and transactional safety
- **read model** for fast UI queries, sorting, filtering, calendar rendering, and reporting

The project does not use CQRS for theater.
It uses CQRS because the product has very different write and read access patterns.

## Why event-driven async flow is used

The async event backbone makes it possible to:

- keep write requests fast
- decouple projection updates from command handling
- support export processing independently of request latency
- add additional consumers later without rewriting the write path

## Security model

Authentication should use:

- secure password hashing
- HttpOnly cookie-based auth
- session store in PostgreSQL or Redis-assisted validation
- ownership checks on all user resources
- CSRF protection for state-changing requests
- rate limiting at Edge API

Phase 1 note:

- the normal browser flow already uses HttpOnly cookies
- durable session persistence is still a follow-up migration step

## Storage model

### PostgreSQL

PostgreSQL is the primary source of truth.

It stores:

- users
- sessions
- categories
- events
- outbox events
- export jobs
- read model tables

### Redis

Redis stores:

- session acceleration and revocation helpers
- dashboard and report preview caches
- rate-limit counters
- distributed locks / job ownership hints
- idempotency keys when needed

### MinIO

MinIO stores:

- generated PDF files
- generated XLSX files
- export artifact metadata references

## Observability

The system should support:

- structured logs
- request ids / correlation ids
- metrics
- tracing
- dashboard monitoring

Recommended stack:

- tracing / tracing-subscriber
- OpenTelemetry
- Prometheus
- Grafana
- Loki

## Development constraints

This architecture should look production-grade, but remain implementable.

Important constraints:

- do not add unnecessary services
- do not add Kafka if NATS JetStream is enough
- do not create internal frameworks
- do not replace working components without clear reason
- keep contracts explicit and documented
- prefer small, verifiable migration steps
