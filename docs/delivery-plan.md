# Delivery Plan

## Goal

Migrate the existing repository into the target architecture in three controlled phases.

This is a rewrite by transformation, not a rewrite by demolition.

## Phase 1: Foundation and skeleton

### Objectives

- rebrand the whole product to EventDesign
- convert backend into a workspace
- introduce service binaries
- introduce Edge API as the only public backend entrypoint
- migrate auth toward HttpOnly cookie-based flow
- keep current user-facing features functional
- add Redis, NATS JetStream, and MinIO to local Compose
- introduce internal contracts crate and initial gRPC definitions

### Expected result

After Phase 1:

- the repo has the target folder skeleton
- frontend talks to Edge API
- auth no longer depends on localStorage in final form
- current features still work
- Docker Compose includes the future infra backbone

### Main tasks

- backend workspace split
- edge-api creation
- identity-svc creation
- temporary compatibility adapters from edge-api to legacy logic if needed
- frontend auth client migration
- environment cleanup
- docs update

### Phase 1 implementation snapshot

Delivered in Phase 1:

- repo rebranded to EventDesign
- backend workspace and target app/crate skeleton created
- `edge-api` became the only published backend service
- `identity-svc` came online through internal gRPC contracts
- frontend moved off `localStorage` and onto a cookie flow
- Compose gained PostgreSQL, Redis, NATS JetStream, and MinIO

## Phase 2: CQRS and async backbone

### Objectives

- move all writes for categories and events into `event-command-svc`
- implement the outbox pattern
- publish domain events to NATS JetStream
- create projection tables and a read-side service
- move dashboard, calendar, and reporting reads to query-side projections
- move export processing into the worker flow
- store generated exports in MinIO

### Expected result

After Phase 2:

- command/write and query/read responsibilities are split
- outbox is real and used
- projections power the read-heavy screens
- export jobs survive API restarts
- the async spine is operational

### Main tasks

- outbox table and relay
- event-command-svc migration
- event-query-svc implementation
- projection worker implementation
- report-svc implementation
- export worker implementation
- cache strategy introduction

### Phase 2 implementation snapshot

Delivered in Phase 2:

- `event-command-svc` took ownership of category and event writes
- category and event mutations began writing to `outbox_events` in the same transaction
- worker relayed outbox rows to NATS JetStream
- projection tables began powering event list, calendar, dashboard, and report preview reads
- Redis began caching dashboard reads and worker jobs started using Redis locks
- `report-svc` took ownership of export job creation, lookup, and MinIO-backed downloads
- worker started generating PDF and XLSX exports asynchronously

## Phase 3: Hardening, polish, defense

### Objectives

- harden cookie auth, CSRF, CORS, and ownership checks
- add observability stack and useful metrics
- improve dashboard, reports, sorting, loading, and empty states
- prepare demo seed data
- add realistic backend and frontend tests
- finalize docs and defense materials

### Expected result

After Phase 3:

- the system is presentable for defense
- the system is observable
- the demo is reliable
- the repository is defendable

### Main tasks

- security hardening
- observability wiring
- UX polish
- sorting and reporting upgrades
- seed and demo data
- backend and frontend tests
- demo script and documentation finalization

### Phase 3 implementation status

Implemented:

- durable session persistence now backs browser auth, and logout revokes the session row
- Edge API enforces CSRF for state-changing requests and restricts CORS to configured frontend origins
- Redis-backed rate limiting protects auth routes and the broader REST surface
- request ids, structured tracing, Prometheus metrics, and Grafana dashboards are wired into local Compose
- dashboard, reports, calendar, categories, settings, and login views were polished with stronger loading, empty, and error states
- visible sorting support now exists on the event list and report preview
- `demo-seed` can populate a defense-ready user, categories, 14 events, and export history
- frontend tests cover login, route guard, event list, reports, and calendar smoke paths
- backend tests cover session flow, ownership checks, projection reads, and export job lifecycle

Remaining rough edges:

- settings still execute in `edge-api`
- Loki is not wired yet, so local log aggregation is still limited to structured container logs
- DB-backed backend tests require a reachable local Postgres and storage stack when run outside Compose

## Rules For Every Phase

For every phase:

- keep the repo runnable
- do not remove working features without replacement
- update docs with architecture and behavior changes
- prefer small verified steps
- commit each milestone separately

## Definition Of Done

The migration is done when:

- frontend talks only to Edge API
- internal services communicate through explicit contracts
- command and query flows are separated
- async work is driven through outbox and NATS
- projections power dashboard, calendar, report, and event-list reads
- browser auth is durable-session cookie auth with CSRF and config-driven CORS
- rate limiting and ownership checks protect user-scoped resources
- observability works locally through Prometheus and Grafana
- exports are asynchronous and stored outside local API container filesystems
- critical frontend and backend paths have automated test coverage
- local startup is documented and repeatable
- docs and demo flow match the implementation
