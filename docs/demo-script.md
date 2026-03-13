# Demo script для защиты

## Цель

Показать не только UI, но и зрелую архитектурную историю:

- frontend -> edge-api;
- gRPC между внутренними сервисами;
- CQRS/read projections;
- outbox + JetStream + worker;
- observability baseline;
- predictable cache behavior.

## Подготовка

Перед демонстрацией:

- выполнить `docker compose up --build -d`;
- убедиться, что `frontend`, `edge-api` и metrics endpoints доступны;
- иметь хотя бы одного пользователя и несколько событий;
- иметь хотя бы один completed export job.

## Сценарий

### 1. Вход

- показать login;
- отметить cookie-based auth и CSRF;
- показать bootstrap current user.

### 2. Categories

- создать category;
- изменить category;
- отметить, что write-side идет в internal command service.

### 3. Events

- создать event;
- изменить event;
- удалить event или сменить статус;
- проговорить, что mutation не идет напрямую в read-side.

### 4. Dashboard

- открыть dashboard;
- показать summary cards и recent activity;
- отметить, что данные приходят из projections, а dashboard cache инвалидируется worker-ом.

### 5. Calendar

- открыть month view;
- показать, что event появился в calendar после обновления read-side;
- отметить, что calendar не держится в отдельном backend cache.

### 6. Reports

- открыть reports summary;
- сменить фильтры;
- показать breakdown по category и status.

### 7. Export

- создать export job;
- показать `queued -> processing -> completed`;
- скачать PDF или XLSX.

### 8. Observability

Если есть время:

- показать Prometheus или Grafana;
- показать `x-request-id` в ответе `edge-api`;
- кратко сказать, что тот же `request_id` проходит в gRPC chain.

## Что проговорить словами

- `edge-api` не ходит в доменную БД напрямую.
- Session validation и `ui_settings` вынесены в `identity-svc`.
- Category/event writes идут через `event-command-svc`.
- Dashboard, calendar и reports читают query-side.
- Export не держит HTTP request и выполняется worker-ом.
- Cache ускоряет dashboard, но не маскирует stale read-side на десятки секунд.

## Чего не делать

- не показывать пустую систему;
- не спорить с реальным состоянием кода;
- не обещать прямой SQL в BFF “только временно”;
- не тратить много времени на логи без запроса комиссии.
