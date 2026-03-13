# ERD Notes

## Purpose

This document describes the conceptual schema for EventDesign.

The actual SQL schema may evolve, but these relationships are the baseline.

## Write-side tables

### users
- id (pk)
- email (unique)
- password_hash
- display_name
- created_at
- updated_at

### sessions
- id (pk)
- user_id (fk -> users.id)
- session_secret_hash or refresh_token_hash
- created_at
- expires_at
- revoked_at
- user_agent
- ip_address

Phase 1 compatibility note:
- the table is part of the target model, but the current implementation still uses a signed cookie compatibility session

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
- category_id (fk -> categories.id, nullable depending on business rule)
- title
- description
- location
- starts_at
- ends_at
- budget
- status
- priority (optional)
- expected_attendees (optional)
- created_at
- updated_at

### ui_settings
- user_id (pk, fk -> users.id)
- theme
- accent_color
- default_view
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
- filters_json
- object_key
- content_type
- error_message
- created_at
- started_at
- finished_at

### processed_messages
- consumer_name
- message_id
- processed_at

Purpose:
- deduplicate idempotent consumers such as projection updates

## Read-side projection tables

### event_list_projection
Purpose:
- event list page
- filter/sort UI

Suggested fields:
- event_id
- user_id
- title
- category_id
- category_name
- category_color
- location
- starts_at
- ends_at
- budget
- status
- updated_at

### calendar_projection
Purpose:
- calendar month/week rendering

Suggested fields:
- event_id
- user_id
- date_bucket
- title
- starts_at
- ends_at
- status
- category_color

### dashboard_projection
Purpose:
- dashboard summary cards and quick widgets

Suggested fields:
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

Suggested fields:
- event_id
- user_id
- category_name
- title
- starts_at
- ends_at
- status
- budget
- location

### recent_activity_projection
Purpose:
- recent actions feed

Suggested fields:
- id
- user_id
- entity_type
- entity_id
- action
- title
- occurred_at

## Relationship summary

- one user has many sessions
- one user has many categories
- one user has many events
- one category belongs to one user
- one category can have many events
- one user has one ui_settings record
- one user has many export_jobs
- projection rows are derived from write-side events

## Lifecycle notes

### Event write lifecycle
1. command-side service writes `events`
2. command-side service writes `outbox_events`
3. relay publishes domain event
4. projector updates read-side tables

### Export lifecycle
1. report service creates `export_jobs` row
2. report service writes `export.requested` to outbox
3. outbox relay publishes the event to JetStream
4. export worker processes job
5. generated file is uploaded to MinIO
6. `export_jobs` row is updated with result

## Optional future additions

These may be added later if useful:
- attachments table
- event_notes table
- audit_log table
- notification_jobs table
- user_preferences expansion
