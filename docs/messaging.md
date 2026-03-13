# Messaging And Async Backbone

## Purpose

This document defines how asynchronous communication works inside EventDesign.

The architecture uses:

- PostgreSQL outbox for durable event capture
- NATS JetStream for event transport
- worker consumers for projections and exports

## Current Status

Implemented now:

- category and event mutations insert outbox rows in the same transaction as the write
- `report-svc` creates export jobs and emits `export.requested` through the same outbox pattern
- worker relays unpublished outbox rows into NATS JetStream
- projection consumers update read-model tables and invalidate Redis dashboard cache
- export jobs are executed by workers and stored in MinIO

Still intentionally synchronous:

- authentication and session validation
- settings reads and writes

## Why This Exists

The async backbone is used to:

- keep write requests fast
- avoid coupling write transactions to heavy background work
- update read models asynchronously
- support export job processing independently of request lifetime
- provide a defendable production-style architecture

## Core Pattern

### Write path

1. command-side or report-side service handles a valid mutation
2. mutation is committed to PostgreSQL
3. matching outbox event is written in the same transaction
4. worker relay publishes the event to NATS JetStream
5. worker consumers process the event

### Read path

1. projector worker consumes domain event
2. projection tables are updated
3. Redis dashboard cache is invalidated when needed
4. query service serves optimized reads from projections

## Outbox Table

Current outbox fields:

- id
- aggregate_type
- aggregate_id
- event_type
- event_version
- payload_json
- occurred_at
- published_at
- publish_attempts
- last_error

Rules:

- outbox insert happens in the same transaction as the mutation
- unpublished rows are retried
- publishing is idempotent where practical

## NATS Subjects

Current subject naming:

- `eventdesign.category.created`
- `eventdesign.category.updated`
- `eventdesign.category.deleted`
- `eventdesign.event.created`
- `eventdesign.event.updated`
- `eventdesign.event.deleted`
- `eventdesign.event.status_changed`
- `eventdesign.export.requested`
- `eventdesign.export.started`
- `eventdesign.export.completed`
- `eventdesign.export.failed`

Keep subject naming explicit and stable.

## Consumers

### Projection worker

Consumes:

- category events
- event events

Updates:

- `event_list_projection`
- `calendar_projection`
- `dashboard_projection`
- `report_projection`
- `recent_activity_projection`

### Export worker

Consumes:

- `export.requested`

Performs:

- marks job started
- builds export from `report_projection`
- uploads file to MinIO
- marks job completed or failed
- emits `export.completed` or `export.failed`

## Retry And Idempotency

Workers must be resilient to transient failures.

Current implementation details:

- outbox relay retries unpublished rows on each polling cycle and records `last_error`
- projection consumer deduplicates with `processed_messages`
- export worker uses Redis locks plus job status transitions to avoid duplicate work

Guidance:

- projection and export logic must tolerate repeats
- UPSERT patterns are preferred for projections
- export state transitions must remain monotonic and explicit

## Cache Invalidation

Current cache behavior:

- dashboard reads are cached in Redis
- projection updates invalidate the dashboard cache after relevant mutations

Report preview is projection-backed but is not currently cached separately.

## Operational Signals

The async backbone is instrumented so the demo and local stack can show meaningful behavior.

Currently exposed signals include:

- HTTP request rate, latency, and error status distribution
- export duration
- projection lag
- worker queue lag
- dashboard cache hit or miss
- security events such as invalid sessions, CSRF rejection, and rate limiting

Local monitoring stack:

- Prometheus scrapes service metrics plus `nats:8222/varz`
- Grafana loads the `EventDesign Overview` dashboard from the repo
- Loki is not yet wired locally

## Export Job Flow

1. frontend requests export through Edge API
2. Edge API forwards to `report-svc`
3. `report-svc` creates `export_jobs` row with `queued` status
4. `export.requested` outbox event is written
5. worker relay publishes the event to JetStream
6. export worker consumes the event
7. export file is generated from the read model
8. artifact is uploaded to MinIO
9. export job is marked completed or failed
10. frontend polls the job list or refreshes status

Current flow notes:

- downloads go back through `report-svc`
- the seed path can also create completed and queued jobs so the demo always has export history

## What Should Not Go Through Async Flow

Keep these synchronous:

- authentication
- category and event write confirmation
- current user bootstrap
- immediate validation errors
- settings reads and writes

## Local Development Expectations

The async backbone must run locally through Docker Compose.

A developer should be able to:

- start PostgreSQL
- start Redis
- start NATS JetStream
- start MinIO
- start services and workers
- create an event
- observe projections eventually update
- request an export and receive a generated file
- inspect metrics in Prometheus or Grafana
