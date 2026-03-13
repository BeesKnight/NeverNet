# Demo Script

## Goal

Demonstrate EventDesign as a polished event operations system, not just a CRUD project.

The demo should show:

- product value
- architecture maturity
- security awareness
- asynchronous processing
- reporting and exports
- calendar and dashboard usefulness
- observability readiness

## Recommended Demo Environment

Start the stack:

```powershell
docker compose up --build -d db redis nats minio identity-svc event-command-svc event-query-svc report-svc worker edge-api frontend prometheus grafana
docker compose --profile demo up demo-seed
```

Useful local URLs:

- app: `http://localhost:3000`
- edge api: `http://localhost:8080`
- prometheus: `http://localhost:9090`
- grafana: `http://localhost:3001`

Grafana credentials:

- username: `admin`
- password: `admin`

Seeded demo credentials:

- email: `demo@eventdesign.local`
- password: `DemoPass123!`

The seed creates:

- one demo user
- five categories
- fourteen events
- mixed `planned`, `in_progress`, `completed`, and `cancelled` statuses
- a month with visible calendar activity
- completed PDF and XLSX exports
- one queued export job for the worker pipeline

## Demo Flow

### 1. Show the login page

Explain:

- browser-facing app is React + TypeScript
- auth uses secure cookie-based session flow
- the login page exposes demo credentials for a predictable defense flow
- Edge API is the only public backend entrypoint

### 2. Log in

Show:

- successful login
- authenticated landing page
- current user-aware UI

Mention:

- auth is isolated in `identity-svc`
- the browser does not manage raw bearer tokens in `localStorage`
- session validity is backed by a durable PostgreSQL session row
- state-changing requests are CSRF-protected

### 3. Dashboard

Show:

- total events
- upcoming events
- completed events
- cancelled events
- total budget
- recent activity
- export queue visibility

Mention:

- dashboard reads come from optimized read models
- Redis is used for dashboard cache acceleration
- the screen stays fast because it reads projections instead of the write model

### 4. Categories

Show:

- category list
- create category
- rename category

Mention:

- categories are user-owned resources
- write operations are validated and go through command-side rules

### 5. Events list

Show:

- event list
- filtering
- visible sorting controls
- event edit flow

Mention:

- write and read paths are separated
- list rendering is backed by query projections
- sorting is performed through read-side query contracts rather than client-only table tricks

### 6. Create or update an event

Show:

- create or edit a real event
- status transition
- category assignment

Mention:

- mutation is handled by the command side
- the same transaction writes the outbox event
- read models update asynchronously through the worker pipeline

### 7. Calendar

Show:

- month view
- events placed on dates
- overflow handling on busy days
- current day highlighting

Mention:

- calendar is projection-driven
- the seeded month is intentionally populated so the calendar looks alive

### 8. Reports

Show:

- report filters
- summary cards
- grouped category and status aggregates
- sorted preview rows
- export history table

Mention:

- report preview is served by query-side read models
- heavy export generation is separate from preview reads

### 9. Export

Show:

- create PDF or XLSX export
- job appears in the queue and history list
- completed file download

Mention:

- export is asynchronous
- files are stored in object storage
- the API request does not wait for PDF or XLSX generation
- queued and processing job states are visible in the UI

### 10. Observability

Show:

- Prometheus targets page or a scrape graph
- Grafana `EventDesign Overview` dashboard

Mention:

- every Rust service exposes `/metrics`
- request rate, latency, errors, export duration, projection lag, queue lag, cache hit or miss, and security events are visible locally
- request ids are propagated through the Edge API into internal gRPC calls

### 11. Architecture slide or docs

Show a short architecture diagram.

Mention:

- Edge API / BFF
- Identity Service
- Event Command Service
- Event Query Service
- Report Service
- Worker
- PostgreSQL
- Redis
- NATS JetStream
- MinIO
- Prometheus and Grafana

## Key Phrases For Defense

Useful phrasing:

- "We separated write and read paths because event mutation and reporting or calendar access have different load patterns."
- "We use an outbox pattern so domain events are durable and do not get lost between the database write and async publication."
- "Exports are asynchronous and processed by workers to avoid blocking request latency."
- "The frontend only knows the Edge API, so internal topology can evolve without breaking the UI."
- "The read model is projection-based, which keeps dashboard, calendar, and reporting queries fast."
- "Cookie auth is backed by a durable session row, and state-changing requests require a CSRF token."
- "We can show latency, errors, cache behavior, projection lag, and export duration from the local observability stack."

## What Not To Waste Time On

Do not spend demo time:

- opening random config files
- explaining every crate
- showing raw Redis keys
- clicking through dead-end pages
- improvising data entry for five minutes

The point is to show a reliable product and a coherent architecture, not to improvise under pressure.
