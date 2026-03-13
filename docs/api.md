# External API

## Purpose

This document describes the REST API exposed by Edge API / BFF to the frontend.

The frontend must talk only to this API.
It must not call internal services directly.

## Auth model

The target auth model is browser-friendly session auth using HttpOnly cookies.

Frontend requests that require auth must use credentials-enabled fetches.

Recommended browser behavior:
- `credentials: include`
- CSRF token mechanism for state-changing requests if implemented

## Phase 1 compatibility note

The current Phase 1 external API still keeps the existing compatibility envelope and field naming:

- responses are wrapped as `{ "data": ... }`
- request and response bodies still use existing snake_case fields such as `full_name`, `category_id`, `start_date`, `end_date`, `default_view`, and `report_type`
- `POST /api/auth/register` and `POST /api/auth/login` set the `eventdesign_session` HttpOnly cookie
- `GET /api/auth/me` currently returns the authenticated user payload only; settings remain on `GET /api/settings`

## Route groups

### Auth

#### POST /api/auth/register
Creates a user account.

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
Creates a session and sets auth cookie.

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
Clears the current session.

Response:
```json
{
  "data": "logged_out"
}
```

#### GET /api/auth/me
Returns the current authenticated user.

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
  "items": [
    {
      "id": "uuid",
      "name": "Conference",
      "color": "#7c3aed"
    }
  ]
}
```

#### POST /api/categories
Creates a category.

Request:
```json
{
  "name": "Conference",
  "color": "#7c3aed"
}
```

#### PATCH /api/categories/:id
Updates a category.

Request:
```json
{
  "name": "Meetup",
  "color": "#2563eb"
}
```

#### DELETE /api/categories/:id
Deletes a category.

Response:
```json
{
  "ok": true
}
```

### Events

#### GET /api/events
Returns filtered and sorted events.

Query params may include:
- `search`
- `status`
- `categoryId`
- `dateFrom`
- `dateTo`
- `sortBy`
- `sortDir`
- `page`
- `pageSize`

Response:
```json
{
  "items": [
    {
      "id": "uuid",
      "title": "Frontend Meetup",
      "category": {
        "id": "uuid",
        "name": "Meetup",
        "color": "#2563eb"
      },
      "location": "Amsterdam",
      "startsAt": "2026-04-10T18:00:00Z",
      "endsAt": "2026-04-10T21:00:00Z",
      "budget": 1200,
      "status": "planned"
    }
  ],
  "page": 1,
  "pageSize": 20,
  "total": 1
}
```

#### GET /api/events/:id
Returns one event.

#### POST /api/events
Creates an event.

Request:
```json
{
  "title": "Frontend Meetup",
  "description": "Community event",
  "location": "Amsterdam",
  "categoryId": "uuid",
  "startsAt": "2026-04-10T18:00:00Z",
  "endsAt": "2026-04-10T21:00:00Z",
  "budget": 1200,
  "status": "planned"
}
```

#### PATCH /api/events/:id
Updates an event.

#### DELETE /api/events/:id
Deletes an event.

### Dashboard

#### GET /api/dashboard
Returns dashboard summary data.

Response:
```json
{
  "cards": {
    "totalEvents": 12,
    "upcomingEvents": 4,
    "completedEvents": 6,
    "cancelledEvents": 2,
    "totalBudget": 8450
  },
  "upcoming": [],
  "recentActivity": []
}
```

### Calendar

#### GET /api/calendar
Returns calendar projection data.

Query params:
- `month`
- `year`

Response:
```json
{
  "year": 2026,
  "month": 4,
  "items": [
    {
      "eventId": "uuid",
      "title": "Frontend Meetup",
      "date": "2026-04-10",
      "startsAt": "2026-04-10T18:00:00Z",
      "endsAt": "2026-04-10T21:00:00Z",
      "status": "planned",
      "categoryColor": "#2563eb"
    }
  ]
}
```

### Reports

#### GET /api/reports/summary
Returns report preview and aggregates.

Query params may include:
- `dateFrom`
- `dateTo`
- `categoryId`
- `status`
- `sortBy`
- `sortDir`

Response:
```json
{
  "summary": {
    "totalEvents": 12,
    "totalBudget": 8450,
    "averageBudget": 704.17
  },
  "items": []
}
```

### Exports

#### POST /api/exports
Creates an export job.

Request:
```json
{
  "report_type": "summary",
  "format": "pdf",
  "filters": {
    "start_date": "2026-04-01",
    "end_date": "2026-04-30",
    "category_id": null,
    "status": null
  }
}
```

Response:
```json
{
  "job": {
    "id": "uuid",
    "status": "queued"
  }
}
```

#### GET /api/exports
Returns current user's export jobs.

#### GET /api/exports/:id
Returns one export job status.

Response:
```json
{
  "job": {
    "id": "uuid",
    "status": "completed",
    "format": "pdf",
    "downloadUrl": "/api/exports/uuid/download"
  }
}
```

#### GET /api/exports/:id/download
Returns or redirects to the export file if ready and authorized.

### Settings

#### GET /api/settings
Returns the current user's UI settings.

#### PATCH /api/settings
Updates UI settings.

Request:
```json
{
  "theme": "dark",
  "default_view": "dashboard"
}
```

## Error contract

Recommended error shape:

```json
{
  "error": {
    "code": "validation_error",
    "message": "Title is required",
    "details": {
      "field": "title"
    }
  }
}
```

Guidelines:
- keep codes stable
- keep messages human-readable
- avoid exposing internal stack traces

## Internal service note

These routes are the external contract only.
Internal gRPC contracts should be designed around service boundaries, not copied 1:1 from the REST surface.
