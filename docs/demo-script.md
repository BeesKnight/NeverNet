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

## Recommended demo environment

Use a seeded environment with:
- one demo user
- several categories
- at least 10 to 15 events
- varied statuses
- several events across one visible month
- at least one completed export job
- at least one queued or processing export job
- a populated dashboard

## Demo flow

### 1. Show the login page
Explain:
- browser-facing app is React + TypeScript
- auth uses secure cookie-based session flow
- Edge API is the only public backend entrypoint

### 2. Log in
Show:
- successful login
- authenticated landing page
- current user-aware UI

Mention:
- auth is isolated in Identity Service
- frontend does not manage raw bearer tokens in localStorage

### 3. Dashboard
Show:
- total events
- upcoming events
- completed events
- cancelled events
- total budget
- recent activity

Mention:
- dashboard reads come from optimized read models
- this is why the screen can stay fast even as the system grows

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
- sorting
- event detail or edit flow

Mention:
- write and read paths are separated
- list rendering is backed by query projections

### 6. Create or update an event
Show:
- create or edit a real event
- status transition
- category assignment

Mention:
- mutation is handled by the command side
- change creates domain events
- read model updates asynchronously

### 7. Calendar
Show:
- month view
- events placed on dates
- click-through to event info

Mention:
- calendar is the required extra product feature
- calendar data is projection-driven

### 8. Reports
Show:
- report filters
- report summary
- sorted results

Mention:
- report preview is served by query-side read models
- this is separate from heavy export generation

### 9. Export
Show:
- create PDF or XLSX export
- job appears in queue / status list
- completed file download

Mention:
- export is asynchronous
- files are stored in object storage
- API request does not wait for heavy report generation

### 10. Architecture slide / docs
Show a short architecture diagram.

Mention:
- Edge API / BFF
- Identity Service
- Event Command Service
- Event Query Service
- Report Service
- Workers
- PostgreSQL
- Redis
- NATS JetStream
- MinIO
- observability stack

## Key phrases for defense

Useful phrasing:
- “We separated write and read paths because event mutation and reporting/calendar access have different load patterns.”
- “We use an outbox pattern so domain events are durable and do not get lost between database write and async publication.”
- “Exports are asynchronous and processed by workers to avoid blocking request latency.”
- “The frontend only knows the Edge API, so internal topology can evolve without breaking the UI.”
- “The read model is projection-based, which helps fast dashboard, calendar, and reporting queries.”

## What not to waste time on

Do not spend demo time:
- opening random config files
- explaining every crate
- showing raw Redis keys
- clicking through dead-end pages
- improvising data entry for five minutes

Humans adore chaos, but a good defense should not.
