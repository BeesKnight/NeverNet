# Domain Model

## Product summary

EventDesign is an event planning and operations system.

Users manage their own categories and events.
They can track statuses, filter and sort events, view calendar data, generate reports, and export reports to PDF/XLSX.

## Bounded contexts

### Identity
Handles:

- users
- sessions
- authentication
- authorization context

### Event Management
Handles:

- categories
- events
- event statuses
- ownership
- event lifecycle

### Reporting
Handles:

- report previews
- aggregates
- export jobs
- generated artifacts

### Preferences
Handles:

- UI theme
- default page / default view
- other persistent user UI preferences

### Activity
Handles:

- recent actions
- audit-like user-visible timeline
- event mutation history projections if implemented

## Core entities

### User

Fields:

- id
- email
- password_hash
- display_name
- created_at
- updated_at

Rules:

- email must be unique
- password must never be stored in plain text
- a user owns categories, events, settings, and export jobs

### Session

Fields:

- id
- user_id
- refresh_token_hash or session_secret_hash
- created_at
- expires_at
- revoked_at
- user_agent
- ip_address (optional if needed)

Rules:

- sessions belong to users
- revoked sessions are invalid
- session validation must support logout and cookie revocation
- Phase 1 compatibility currently uses a signed HttpOnly cookie and defers durable session persistence

### Category

Fields:

- id
- user_id
- name
- color
- created_at
- updated_at

Rules:

- category names should be unique per user where practical
- categories are user-scoped
- deleting a category must have a defined behavior for related events

### Event

Fields:

- id
- user_id
- category_id
- title
- description
- location
- starts_at
- ends_at
- budget
- status
- priority (optional but recommended)
- expected_attendees (optional but recommended)
- created_at
- updated_at

Rules:

- an event belongs to exactly one user
- an event may belong to one category
- start time must be before end time
- status must be valid
- ownership must always be enforced

### UI Settings

Fields:

- user_id
- theme
- accent_color
- default_view
- updated_at

Rules:

- exactly one settings record per user
- should be returned as part of authenticated UI bootstrap when useful

### Export Job

Fields:

- id
- user_id
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

Rules:

- export jobs belong to a user
- export jobs must survive API process restarts
- generated file metadata must not rely only on local container filesystem

## Event statuses

Supported statuses:

- planned
- in_progress
- completed
- cancelled

Recommended transition policy:

- planned -> in_progress
- planned -> cancelled
- in_progress -> completed
- in_progress -> cancelled

Avoid magic transitions unless explicitly documented.

## Main user capabilities

A user must be able to:

- register and log in
- manage categories
- create and update events
- filter and sort events
- view a calendar of events
- see dashboard summaries
- view reports by period and category
- export reports to PDF/XLSX
- configure interface settings

## Read model projections

Recommended read-side projections:

### event_list_projection
Purpose:
- power filtered and sorted event list UI

Fields may include:
- event_id
- user_id
- title
- category_name
- category_color
- status
- starts_at
- ends_at
- budget
- location
- updated_at

### calendar_projection
Purpose:
- power month/week calendar rendering

Fields may include:
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
- power dashboard cards and upcoming event summaries

Fields may include:
- user_id
- total_events
- upcoming_events
- completed_events
- cancelled_events
- total_budget
- updated_at

### report_projection
Purpose:
- power report preview screens and exports

Fields may include denormalized event data optimized for report filters.

### recent_activity_projection
Purpose:
- power recent actions widget / timeline

## Domain events

Important domain events include:

- user.registered
- user.logged_in
- category.created
- category.updated
- category.deleted
- event.created
- event.updated
- event.deleted
- event.status_changed
- export.requested
- export.started
- export.completed
- export.failed

## Ownership model

Every user-scoped resource must be checked against the authenticated user id.

This applies to:

- categories
- events
- settings
- export jobs
- generated exports

Never rely only on client-side filtering for access control.
