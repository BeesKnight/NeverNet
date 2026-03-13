# Domain Model

## Product Summary

EventDesign is an event planning and operations system.

Users manage their own categories and events.
They can track statuses, filter and sort events, view calendar data, generate reports, and export reports to PDF or XLSX.

## Bounded Contexts

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
- grouped aggregates
- export jobs
- export history
- generated artifacts

### Preferences

Handles:

- UI theme
- accent color
- default page or default view
- other persistent user UI preferences

### Activity

Handles:

- recent actions
- user-visible timeline entries
- event mutation history projections

## Core Entities

### User

Fields:

- id
- email
- password_hash
- full_name
- created_at
- updated_at

Rules:

- email must be unique
- password must never be stored in plain text
- a user owns categories, events, settings, sessions, and export jobs

### Session

Fields:

- id
- user_id
- created_at
- expires_at
- revoked_at
- user_agent
- ip_address

Rules:

- sessions belong to users
- sessions are persisted in PostgreSQL
- the browser cookie carries a JWT with the user id plus the session id claim
- the session row remains the source of truth for logout and revocation
- revoked or expired sessions are invalid
- normal browser auth does not rely on `localStorage` bearer tokens

### Category

Fields:

- id
- user_id
- name
- color
- created_at
- updated_at

Rules:

- category names are unique per user
- categories are user-scoped
- deleting a category that is still referenced by events is blocked by the write model

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
- created_at
- updated_at

Rules:

- an event belongs to exactly one user
- an event belongs to exactly one category
- start time must be before end time
- status must be valid
- ownership must always be enforced

### UI Settings

Fields:

- user_id
- theme
- accent_color
- default_view
- created_at
- updated_at

Rules:

- exactly one settings record exists per user
- settings are user-scoped like the rest of the product

### Export Job

Fields:

- id
- user_id
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

Rules:

- export jobs belong to a user
- export jobs survive API process restarts
- generated file metadata does not rely on local container filesystem paths
- supported job states are `queued`, `processing`, `completed`, and `failed`

## Event Statuses

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

## Main User Capabilities

A user must be able to:

- register and log in
- manage categories
- create and update events
- filter and sort events
- view a calendar of events
- see dashboard summaries
- view reports by period and category
- export reports to PDF or XLSX
- configure interface settings

## Read Model Projections

### event_list_projection

Purpose:

- power filtered and sorted event list UI

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

- power month calendar rendering

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

- power dashboard cards and upcoming event summaries

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

- power report preview screens, sorting, grouped summaries, and exports

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

- power the recent activity widget on the dashboard

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

## Domain Events

Current async event families include:

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

Auth lifecycle remains synchronous through `identity-svc`; login and logout are enforced through durable sessions rather than the async backbone.

## Ownership Model

Every user-scoped resource is checked against the authenticated user id.

This applies to:

- categories
- events
- settings
- sessions
- export jobs
- generated exports

Never rely on client-side filtering for access control.
