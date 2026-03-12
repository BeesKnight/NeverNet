# EventDesign Architecture

## Overview

EventDesign is implemented as a modular monolith:

- `frontend/`: React + TypeScript + Vite UI
- `backend/`: Axum + SQLx + PostgreSQL API
- `docs/`: architecture and setup notes

The backend stays in one deployable service and is organized by domain modules:

- `auth`
- `users`
- `categories`
- `events`
- `reports`
- `settings`
- `exports`

Each backend domain keeps route handlers, service logic, repository access, models, and validation close together. The project avoids generic abstraction layers and only separates what is needed to keep behavior explicit.

## Data Model

Tracked tables:

- `users`
- `categories`
- `events`
- `ui_settings`
- `export_jobs`

Ownership is enforced per user for categories, events, settings, and exports.

## API Shape

All API endpoints are REST/JSON under `/api`.

Success payloads use:

```json
{
  "data": {}
}
```

Error payloads use:

```json
{
  "error": {
    "message": "..."
  }
}
```

## Export Flow

Export requests create an `export_jobs` row first and return immediately.

The backend then processes the job asynchronously:

1. mark job as `processing`
2. reuse report filters to build a summary
3. write a PDF or XLSX file into `backend/storage/exports/<user-id>/`
4. mark job as `completed` or `failed`

This keeps the main request path short while still remaining a single-process local setup.

## Frontend Structure

The frontend uses:

- React Router for pages
- TanStack Query for server state
- local auth context for session state
- feature folders for forms and auth behavior

Theme preference is stored in the backend and synchronized to the document root from the UI.
