# External API

## Purpose

This document describes the REST API exposed by Edge API / BFF to the frontend.

The frontend talks only to this API.
It does not call internal services directly.

## Auth And Browser Security

The implemented browser auth model is cookie-based session auth:

- the auth cookie is `eventdesign_session`
- the browser must send `credentials: include`
- state-changing requests use a double-submit CSRF token
- allowed frontend origins come from `FRONTEND_ORIGINS`
- most responses expose `x-request-id` for correlation

Browser flow:

1. `GET /api/auth/csrf`
2. send `POST`, `PATCH`, or `DELETE` with the returned token in `X-CSRF-Token`
3. after login or register, continue using the HttpOnly session cookie

Normal browser auth does not use `localStorage` bearer tokens.

## Common Response And Error Format

Successful JSON responses use:

```json
{
  "data": {}
}
```

Error responses use:

```json
{
  "error": {
    "message": "Human readable message"
  }
}
```

Common status codes:

- `400` bad input
- `401` unauthenticated or CSRF failure
- `404` resource not found
- `409` conflict
- `429` rate limited
- `500` internal error

Field names are snake_case throughout the public API.

## Route Groups

### Auth

#### GET /api/auth/csrf

Returns a CSRF token and sets the `eventdesign_csrf` cookie.
This route is unauthenticated and safe to call before login.

Response:

```json
{
  "data": {
    "csrf_token": "4d1458f4deda4cf6bbd91f7743e0d4b0"
  }
}
```

#### POST /api/auth/register

Creates a user account and sets the `eventdesign_session` HttpOnly cookie.
Requires a valid CSRF token.

Request:

```json
{
  "email": "user@example.com",
  "password": "secret123",
  "full_name": "Alex"
}
```

Response:

```json
{
  "data": {
    "user": {
      "id": "uuid",
      "email": "user@example.com",
      "full_name": "Alex",
      "created_at": "2026-03-13T10:00:00Z"
    }
  }
}
```

#### POST /api/auth/login

Creates a durable session row, sets the `eventdesign_session` HttpOnly cookie, and returns the current user.
Requires a valid CSRF token.

Request:

```json
{
  "email": "user@example.com",
  "password": "secret123"
}
```

Response:

```json
{
  "data": {
    "user": {
      "id": "uuid",
      "email": "user@example.com",
      "full_name": "Alex",
      "created_at": "2026-03-13T10:00:00Z"
    }
  }
}
```

#### POST /api/auth/logout

Revokes the current durable session and clears the auth cookie.
Requires a valid CSRF token.

Response:

```json
{
  "data": "logged_out"
}
```

#### GET /api/auth/me

Returns the current authenticated user if the cookie is present and the backing session row is still active.

Response:

```json
{
  "data": {
    "user": {
      "id": "uuid",
      "email": "user@example.com",
      "full_name": "Alex",
      "created_at": "2026-03-13T10:00:00Z"
    }
  }
}
```

### Categories

#### GET /api/categories

Returns all categories for the authenticated user.

Response:

```json
{
  "data": [
    {
      "id": "uuid",
      "user_id": "uuid",
      "name": "Conference",
      "color": "#0f766e",
      "created_at": "2026-03-13T10:00:00Z",
      "updated_at": "2026-03-13T10:00:00Z"
    }
  ]
}
```

#### POST /api/categories

Creates a category.
Requires a valid CSRF token.

Request:

```json
{
  "name": "Conference",
  "color": "#0f766e"
}
```

#### PATCH /api/categories/:id

Updates a category.
Requires a valid CSRF token.

Request:

```json
{
  "name": "Meetup",
  "color": "#2563eb"
}
```

#### DELETE /api/categories/:id

Deletes a category owned by the current user.
Requires a valid CSRF token.

Response:

```json
{
  "data": "deleted"
}
```

### Events

#### GET /api/events

Returns filtered and sorted event projection rows for the authenticated user.

Query params may include:

- `search`
- `status`
- `category_id`
- `start_date`
- `end_date`
- `sort_by`
- `sort_dir`

Supported `sort_by` values:

- `starts_at`
- `ends_at`
- `budget`
- `title`
- `status`
- `updated_at`
- `category_name`

Response:

```json
{
  "data": [
    {
      "id": "uuid",
      "user_id": "uuid",
      "category_id": "uuid",
      "category_name": "Conference",
      "category_color": "#0f766e",
      "title": "Defense rehearsal",
      "description": "Dry run",
      "location": "Room 301",
      "starts_at": "2026-03-15T10:00:00Z",
      "ends_at": "2026-03-15T12:00:00Z",
      "budget": 850.0,
      "status": "planned",
      "created_at": "2026-03-13T10:00:00Z",
      "updated_at": "2026-03-13T10:00:00Z"
    }
  ]
}
```

#### GET /api/events/:id

Returns one event from the write-side ownership boundary.

#### POST /api/events

Creates an event.
Requires a valid CSRF token.

Request:

```json
{
  "category_id": "uuid",
  "title": "Frontend Meetup",
  "description": "Community event",
  "location": "Amsterdam",
  "starts_at": "2026-04-10T18:00:00Z",
  "ends_at": "2026-04-10T21:00:00Z",
  "budget": 1200,
  "status": "planned"
}
```

#### PATCH /api/events/:id

Updates an event.
Requires a valid CSRF token.

#### DELETE /api/events/:id

Deletes an event.
Requires a valid CSRF token.

### Dashboard

#### GET /api/dashboard

Returns projection-backed dashboard cards plus upcoming events and recent activity.

Response:

```json
{
  "data": {
    "cards": {
      "total_events": 12,
      "upcoming_events": 4,
      "completed_events": 6,
      "cancelled_events": 2,
      "total_budget": 8450
    },
    "upcoming": [],
    "recent_activity": []
  }
}
```

### Calendar

#### GET /api/calendar

Returns calendar projection rows for a date window.

Query params:

- `start_date`
- `end_date`

Response:

```json
{
  "data": [
    {
      "event_id": "uuid",
      "title": "Frontend Meetup",
      "date": "2026-04-10",
      "starts_at": "2026-04-10T18:00:00Z",
      "ends_at": "2026-04-10T21:00:00Z",
      "status": "planned",
      "category_color": "#2563eb"
    }
  ]
}
```

### Reports

#### GET /api/reports/summary

Returns the report preview, summary cards, grouped aggregates, and sorted preview rows.

Query params may include:

- `status`
- `category_id`
- `start_date`
- `end_date`
- `sort_by`
- `sort_dir`

The current UI uses preview sorting for:

- `starts_at`
- `title`
- `category_name`
- `budget`
- `status`
- `updated_at`

Response:

```json
{
  "data": {
    "filters": {
      "status": "planned",
      "category_id": null,
      "start_date": "2026-03-01",
      "end_date": "2026-03-31",
      "sort_by": "starts_at",
      "sort_dir": "asc"
    },
    "period_start": "2026-03-01",
    "period_end": "2026-03-31",
    "total_events": 12,
    "total_budget": 8450,
    "by_category": [],
    "by_status": [],
    "events": []
  }
}
```

#### GET /api/reports/by-category

Returns only the grouped category rows for the same filter set.

### Exports

#### POST /api/exports

Creates an asynchronous export job.
Requires a valid CSRF token.

Request:

```json
{
  "report_type": "summary",
  "format": "pdf",
  "filters": {
    "start_date": "2026-04-01",
    "end_date": "2026-04-30",
    "category_id": null,
    "status": null,
    "sort_by": "starts_at",
    "sort_dir": "asc"
  }
}
```

Response:

```json
{
  "data": {
    "id": "uuid",
    "user_id": "uuid",
    "report_type": "summary",
    "format": "pdf",
    "status": "queued",
    "filters": {},
    "object_key": null,
    "content_type": null,
    "error_message": null,
    "created_at": "2026-04-10T21:00:00Z",
    "started_at": null,
    "updated_at": "2026-04-10T21:00:00Z",
    "finished_at": null
  }
}
```

#### GET /api/exports

Returns all export jobs for the current user.

#### GET /api/exports/:id

Returns one export job owned by the current user.

#### GET /api/exports/:id/download

Streams the export file if the job is completed and belongs to the current user.

### Settings

#### GET /api/settings

Returns the current user's UI settings.

Response:

```json
{
  "data": {
    "user_id": "uuid",
    "theme": "system",
    "accent_color": "#b6532f",
    "default_view": "dashboard",
    "created_at": "2026-03-13T10:00:00Z",
    "updated_at": "2026-03-13T10:00:00Z"
  }
}
```

#### PATCH /api/settings

Updates the current user's UI settings.
Requires a valid CSRF token.

Request:

```json
{
  "theme": "dark",
  "accent_color": "#0f766e",
  "default_view": "reports"
}
```

## Internal Service Note

These routes are the external contract only.
Internal gRPC contracts follow service boundaries rather than mirroring the REST surface 1:1.
