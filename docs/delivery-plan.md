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

## Phase 2: CQRS and async backbone

### Objectives
- move all writes for categories and events into event-command-svc
- implement outbox pattern
- publish domain events to NATS JetStream
- create projection tables and read-side service
- move dashboard / calendar / reporting reads to query-side projections
- move export processing into worker flow
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

## Phase 3: Hardening, polish, defense

### Objectives
- harden cookie auth, CSRF, CORS, and ownership checks
- add observability stack and useful metrics
- improve dashboard, reports, sorting, loading, and empty states
- prepare demo seed data
- add test baseline
- finalize docs and defense materials

### Expected result
After Phase 3:
- the system looks serious
- the system is observable
- the demo is reliable
- the repository is defendable

### Main tasks
- security hardening
- observability wiring
- UX polish
- sorting and reporting upgrades
- seed/demo data
- integration tests
- demo script and documentation finalization

## Rules for every phase

For every phase:
- keep the repo runnable
- do not remove working features without replacement
- update docs with architecture and behavior changes
- prefer small verified steps
- commit each milestone separately

## Definition of done

The migration is done when:
- frontend talks only to edge-api
- internal services communicate through explicit contracts
- command and query flows are separated
- async work is driven through outbox and NATS
- projections power dashboard/calendar/report reads
- exports are asynchronous and stored outside local API container filesystems
- local startup is documented and repeatable
- docs and demo flow match the implementation
