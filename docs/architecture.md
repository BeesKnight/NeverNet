# NeverNet Architecture

## Overview

NeverNet is implemented as a modular monolith:

- `frontend/`: React + TypeScript + Vite UI
- `backend/`: Axum + SQLx + PostgreSQL API
- `docs/`: architecture and setup notes

The backend remains one deployable service and is organized by domain modules:

- `auth`
- `users`
- `categories`
- `events`
- `reports`
- `settings`
- `exports`
- `calendar`

Each module keeps handlers, services, repositories, models, and validation close together. The code intentionally avoids generic abstraction layers and favors explicit behavior.

## Data model

Tracked tables:

- `users`
- `categories`
- `events`
- `ui_settings`
- `export_jobs`

Important persisted preferences and export metadata:

- `ui_settings.theme`
- `ui_settings.accent_color`
- `ui_settings.default_view`
- `export_jobs.report_type`
- `export_jobs.finished_at`

Ownership is enforced per user for categories, events, settings, and exports.

## API shape

All API endpoints are REST/JSON under `/api`.

Success payload:

```json
{
  "data": {}
}
```

Error payload:

```json
{
  "error": {
    "message": "..."
  }
}
```

## Request flow

Authentication uses stateless JWT bearer tokens. Protected handlers extract the current user from the `Authorization` header and apply ownership checks in the service or repository layer.

Event listing is reused across:

- the main event list
- dashboard summaries
- reports
- the calendar endpoint
- export generation

This keeps filtering rules consistent across the application.

## Export flow

Export requests create an `export_jobs` row first and return immediately.

The backend then processes the job asynchronously:

1. mark the job as `processing`
2. reuse report filters to build a summary
3. generate a PDF or XLSX file inside `backend/storage/exports/<user-id>/`
4. mark the job as `completed` or `failed`

This keeps the main request path short while preserving a simple local deployment model.

## Frontend structure

The frontend uses:

- React Router for pages
- TanStack Query for server state
- local auth context for session state
- feature folders for forms and auth behavior

Interface preferences are loaded from the backend and synchronized to CSS variables so theme and accent color stay persistent per user.
