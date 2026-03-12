# AGENTS.md

## Project summary

This repository contains **EventDesign**, a full-stack graduation project for event planning and event management.

The required product scope is:
- user registration and authentication
- event category management
- event CRUD
- filtering and search
- reports by period and category
- export reports to PDF and XLSX
- UI customization
- calendar view as the additional feature

This project must remain practical, clean, and easy to run locally.

## Architecture rules

Use a **modular monolith** architecture.

Preferred top-level structure:
- `frontend/` for React + TypeScript + Vite
- `backend/` for Rust + Axum + SQLx
- `worker/` if background export processing is implemented
- `docs/` for architecture and project documentation

Do not introduce unnecessary microservices, gRPC layers, brokers, or wasm-based complexity.

## Tech stack

### Frontend
- React
- TypeScript
- Vite
- React Router
- TanStack Query

### Backend
- Rust
- Axum
- SQLx
- PostgreSQL

### Optional
- Tailwind CSS or another lightweight UI styling solution
- a separate worker for heavy export jobs

## Product rules

Implement only what supports the EventDesign scope.

Core entities:
- users
- categories
- events
- ui_settings
- export_jobs

Required event fields:
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

Allowed event statuses:
- planned
- in_progress
- completed
- cancelled

## Development priorities

Always prioritize in this order:
1. correctness
2. working end-to-end functionality
3. maintainable structure
4. validation and error handling
5. UI polish

Do not prioritize visual polish over core functionality.

## Backend rules

Backend should be organized by domain modules, for example:
- auth
- users
- events
- categories
- reports
- settings

Use clear separation between:
- routes / handlers
- services / use cases
- repositories / db layer
- models / dto
- validation
- errors

Prefer explicit code over clever abstractions.

Do not create generic abstractions too early.

Use SQL migrations and keep schema changes tracked in the repository.

## Frontend rules

Frontend should be:
- simple
- typed
- maintainable
- not overengineered

Prefer:
- feature-oriented folder structure
- reusable UI components only when reuse is real
- server state via TanStack Query
- local UI state via React state unless something more is clearly justified

Avoid premature global state management.

## API rules

Use REST/JSON APIs.

Keep request and response shapes consistent.
Validate inputs on the backend.
Return meaningful errors.
Protect private endpoints.

## Export rules

PDF and XLSX export must be implemented.
If export generation is heavy, do not block the main request path.
Use a background job or worker pattern.

Track export state in `export_jobs`.

## Settings rules

Persist user interface preferences per user.
At minimum support theme switching.

## Additional feature rule

The required additional feature for this project is:
- calendar view of events

Do not replace this with another feature unless explicitly requested.

## Code quality rules

Before considering work complete:
- run formatting
- run linting if configured
- run type checks
- run tests relevant to the change
- verify the app still starts locally

When changing behavior:
- update or add tests when reasonable
- update documentation if setup, architecture, or behavior changed

Do not leave fake implementations presented as complete.

## Completion rules

A task is not complete unless:
- code compiles
- the changed flow works end-to-end
- obvious edge cases are handled
- documentation stays consistent
- no unrelated code is broken

## Review rules

When reviewing your own work:
- look for broken auth flows
- look for missing authorization checks
- look for invalid ownership checks on categories and events
- look for broken filtering
- look for incorrect report aggregation
- look for export flow failures
- look for schema / migration drift
- look for TypeScript type mismatches
- look for poor error handling

## Communication rules

When working on a task:
- explain major architectural decisions briefly
- keep plans concrete
- do not flood responses with unnecessary theory
- call out assumptions clearly
- mention limitations honestly

## Safe defaults

If something is ambiguous:
- choose the simplest robust solution
- keep the architecture compact
- prefer a finished working implementation over a broader incomplete one