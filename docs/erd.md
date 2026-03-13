# ERD Notes

## Purpose

This document describes the conceptual schema for EventDesign.

The actual SQL schema may evolve, but these relationships reflect the current Phase 3 implementation.

## Write-side Tables

### users

- id (pk)
- email (unique)
- password_hash
- full_name
- created_at
- updated_at

### sessions

- id (pk)
- user_id (fk -> users.id)
- created_at
- expires_at
- revoked_at
- user_agent
- ip_address

Notes:

- browser auth cookies carry a JWT with the session id claim
- the session row is validated on authenticated requests
- logout revokes the row by setting `revoked_at`

### categories

- id (pk)
- user_id (fk -> users.id)
- name
- color
- created_at
- updated_at

### events

- id (pk)
- user_id (fk -> users.id)
- category_id (fk -> categories.id)
- title
- description
- location
- starts_at
- ends_at
- budget
- status
- created_at
- updated_at

### ui_settings

- user_id (pk, fk -> users.id)
- theme
- accent_color
- default_view
- created_at
- updated_at

### outbox_events

- id (pk)
- aggregate_type
- aggregate_id
- event_type
- event_version
- payload_json
- occurred_at
- published_at
- publish_attempts
- last_error

### export_jobs

- id (pk)
- user_id (fk -> users.id)
- report_type
- format
- status
- filters
- object_key
- content_type
- error_message
- created_at
- started_at
- updated_at
- finished_at

### processed_messages

- consumer_name
- message_id
- processed_at

Purpose:

- deduplicate idempotent consumers such as projection updates

## Read-side Projection Tables

### event_list_projection

Purpose:

- event list page
- filter and sort UI

Fields:

- event_id
- user_id
- category_id
- category_name
- category_color
- title
- description
- location
- starts_at
- ends_at
- budget
- status
- created_at
- updated_at

### calendar_projection

Purpose:

- calendar month rendering

Fields:

- event_id
- user_id
- date_bucket
- title
- starts_at
- ends_at
- status
- category_color
- updated_at

### dashboard_projection

Purpose:

- dashboard summary cards and quick widgets

Fields:

- user_id
- total_events
- upcoming_events
- completed_events
- cancelled_events
- total_budget
- updated_at

### report_projection

Purpose:

- report preview and export generation queries

Fields:

- event_id
- user_id
- category_id
- category_name
- category_color
- title
- description
- location
- starts_at
- ends_at
- budget
- status
- created_at
- updated_at

### recent_activity_projection

Purpose:

- recent actions feed

Fields:

- id
- source_message_id
- user_id
- entity_type
- entity_id
- action
- title
- occurred_at
- created_at

## Relationship Summary

- one user has many sessions
- one user has many categories
- one user has many events
- one user has one `ui_settings` record
- one user has many `export_jobs`
- one category belongs to one user
- one category can have many events
- projection rows are derived from write-side events

## Lifecycle Notes

### Auth/session lifecycle

1. register or login creates a `sessions` row
2. Edge API sets the `eventdesign_session` cookie
3. authenticated requests validate both the JWT and the backing session row
4. logout revokes the row and clears the cookie

### Event write lifecycle

1. command-side service writes `events`
2. command-side service writes `outbox_events`
3. relay publishes the domain event
4. projector updates read-side tables

### Export lifecycle

1. report service creates `export_jobs` row
2. report service writes `export.requested` to outbox
3. outbox relay publishes the event to JetStream
4. export worker processes the job
5. generated file is uploaded to MinIO
6. `export_jobs` row is updated with result

## Optional Future Additions

These may be added later if useful:

- attachments table
- event_notes table
- audit_log table
- notification_jobs table
- user_preferences expansion
