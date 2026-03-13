# Messaging and Async Backbone

## Purpose

This document defines how asynchronous communication works inside EventDesign.

The architecture uses:

- PostgreSQL outbox for durable event capture
- NATS JetStream for event transport
- worker consumers for projections, exports, and side effects

## Phase 2 note

The repository now runs the async backbone through the real `worker` app.

Implemented now:

- category and event mutations insert outbox rows in the same transaction as the write
- worker relays unpublished outbox rows into NATS JetStream
- projection consumers update read-model tables and invalidate Redis dashboard cache
- export jobs are created by `report-svc`, executed by workers, and stored in MinIO

Still intentionally synchronous:

- authentication
- settings reads and writes

## Why this exists

The async backbone is used to:

- keep write requests fast
- avoid coupling write transactions to heavy background work
- update read models asynchronously
- support export job processing independently of request lifetime
- make the architecture look and behave like a serious system

## Core pattern

### Write path
1. command service handles a valid mutation
2. mutation is committed to the write model
3. matching outbox event is written in the same transaction
4. outbox relay publishes event to NATS JetStream
5. worker consumers process the event

### Read path
1. projector worker consumes domain event
2. projection tables are updated
3. Redis cache is invalidated or warmed as needed
4. query service serves optimized reads from projections

## Outbox table

Recommended fields:

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

- outbox insert must happen in the same transaction as the mutation
- unpublished rows must be retried
- publishing must be idempotent where practical

## NATS subjects

Recommended subject naming:

- `eventdesign.user.registered`
- `eventdesign.user.logged_in`
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

Keep subject naming consistent and explicit.

## Consumers

### Projection worker
Consumes:
- category events
- event events

Updates:
- event_list_projection
- calendar_projection
- dashboard_projection
- report_projection
- recent_activity_projection

### Export worker
Consumes:
- export.requested

Performs:
- mark job started
- build export from read model
- upload file to MinIO
- mark job completed or failed
- publish export.completed or export.failed

### Activity / audit worker
Optional but recommended.

Consumes:
- major user-visible mutations

Updates:
- recent activity feed
- audit tables if implemented

## Retry behavior

Workers must be resilient to transient failures.

Recommended behavior:

- retry failed publishes and consumptions
- record errors in DB where appropriate
- do not lose export jobs if a worker crashes
- ensure idempotent projection updates where possible

Current implementation details:

- outbox relay retries unpublished rows on each polling cycle and records `last_error`
- projection consumer deduplicates with `processed_messages`
- export worker uses Redis locks plus job status transitions to avoid duplicate processing

## Idempotency guidance

Workers may process the same logical event more than once.
Projection and export logic should be designed to tolerate repeats.

Examples:

- use event ids or outbox ids for deduplication
- use UPSERT patterns for projections
- mark export state transitions carefully

## Cache invalidation

Redis cache invalidation should be triggered by domain events or projection updates.

Examples:

- invalidate dashboard cache on event.created or event.updated
- invalidate calendar cache on category or event changes
- invalidate report preview cache on event mutations

Avoid manual invalidation scattered across unrelated codepaths.

## Export job flow

1. frontend requests export
2. edge-api forwards to report-svc
3. report-svc creates `export_jobs` row with `queued` status
4. export.requested event is emitted
5. export worker consumes event
6. report is generated from read model
7. artifact is uploaded to MinIO
8. export job is marked completed or failed
9. frontend polls or refreshes job status

Current flow:

- `edge-api` forwards export creation and lookup to `report-svc`
- workers generate the file asynchronously from `report_projection`
- artifacts are uploaded to MinIO and downloaded back through `report-svc`

## What should not go through async flow

Keep these synchronous:

- authentication
- category and event write confirmation
- primary event detail fetch
- current user bootstrap
- immediate validation errors

Not every action needs a queue just because people enjoy drawing arrows.

## Local development expectations

The async backbone must run locally through Docker Compose.

A developer should be able to:

- start NATS JetStream
- start Redis
- start PostgreSQL
- start MinIO
- start services and workers
- create an event
- observe projections eventually update
- request an export and receive a generated file
