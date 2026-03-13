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
- при необходимости обновить dataset командой `docker compose run --rm demo-seed`;
- убедиться, что `frontend`, `edge-api`, `Prometheus`, `Grafana` доступны;
- использовать готового demo user `demo@eventdesign.local` / `DemoPass123!`;
- убедиться, что в системе уже видны категории, события, dashboard, calendar и история export jobs;
- при желании перед выступлением прогнать `scripts/smoke.sh`.

## Сценарий

### 1. Вход

- показать login;
- нажать `Use demo user`, чтобы не тратить время на ручной ввод;
- отметить cookie-based auth и CSRF;
- показать bootstrap current user.

### 2. Categories

- показать, что категории уже заполнены demo seed;
- создать еще одну category;
- изменить category;
- отметить, что write-side идет в internal command service.

### 3. Events

- показать, что в списке уже есть события с разными статусами;
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

- показать, что история export jobs уже не пустая;
- создать новый export job;
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
- не надеяться на ручное наполнение данных под глазами комиссии;
- не спорить с реальным состоянием кода;
- не обещать прямой SQL в BFF “только временно”;
- не тратить много времени на логи без запроса комиссии.
