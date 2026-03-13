# EventDesign Architecture

## Overview

EventDesign is a high-load inspired full-stack system for event planning and event operations.

The architecture should look like a production-grade product, while still being practical to implement under deadline pressure.

## Phase 1 Status

The repository is currently in the foundation phase.

Implemented now:

- backend workspace split into app crates and shared crates
- `edge-api` is the only published backend entrypoint
- `identity-svc` is live and handles auth over gRPC
- frontend auth uses an HttpOnly cookie flow and `credentials: include`
- Docker Compose starts PostgreSQL, Redis, NATS JetStream, and MinIO

Phase 1 compatibility layers:

- categories, events, calendar, reports, settings, and exports still execute inside `edge-api`
- `event-command-svc`, `event-query-svc`, `report-svc`, and `worker` are service skeletons with explicit contracts
- export files still land in shared local storage while MinIO is introduced for the next phase

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
  -> Phase 1 compatibility modules for categories, events, reports, settings, and exports
  -> Event Command / Query / Report service contracts

Identity Service
  -> PostgreSQL
  -> signed cookie session compatibility

Phase 2 target
  -> Event Command Service -> PostgreSQL write schema + outbox
  -> Outbox Relay / Publisher -> NATS JetStream
  -> NATS JetStream -> workers and projections
  -> Event Query Service -> projection/read schema + Redis cache
  -> Report Service -> read schema + MinIO + export jobs

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
- generate PDF / XLSX
- store generated files in object storage
- report job status lookup
- secure download flow

### Workers

Workers are responsible for:

- consuming domain events
- updating read models / projections
- invalidating or warming cache
- processing export jobs
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
