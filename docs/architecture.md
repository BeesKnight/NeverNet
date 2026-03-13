# Архитектура EventDesign

## Обзор

EventDesign — service-oriented full-stack система для планирования мероприятий.

После фазы 4 целевая архитектура в коде выглядит так:

- `frontend` говорит только с `edge-api`;
- `edge-api` не ходит в доменную БД напрямую;
- внутренние backend-сервисы разделены по ответственности и доступны только по gRPC;
- write-side изменения публикуются через outbox;
- read-heavy экраны читают projection tables;
- экспорт работает асинхронно через worker и MinIO;
- observability собрана вокруг `x-request-id`, structured logs, `/healthz` и `/metrics`.

## Топология

```text
Browser
  -> Frontend (React/Vite)
  -> Edge API (REST/JSON, cookies, CSRF)

Edge API
  -> Identity Service (gRPC)
  -> Event Command Service (gRPC)
  -> Event Query Service (gRPC)
  -> Report Service (gRPC)

Identity Service
  -> users
  -> sessions
  -> ui_settings

Event Command Service
  -> categories
  -> events
  -> outbox_events

Event Query Service
  -> event_list_projection
  -> calendar_projection
  -> dashboard_projection
  -> report_projection
  -> recent_activity_projection
  -> Redis dashboard cache

Report Service
  -> export_jobs
  -> outbox_events
  -> MinIO metadata / download boundary

Worker
  -> outbox relay
  -> JetStream publisher
  -> projection consumer
  -> export consumer
```

## Роли сервисов

### Frontend

- UI, маршрутизация, формы, client-side validation.
- Только вызовы `edge-api`.
- Нет знания о внутренней топологии сервисов.

### Edge API

- BFF и browser boundary.
- CORS, CSRF, cookies, rate limiting.
- Генерация `x-request-id` на HTTP-входе.
- gRPC orchestration между внутренними сервисами.
- Shaping JSON-ответов под frontend.

`edge-api` после фазы 4:

- не требует `DATABASE_URL`;
- не держит `PgPool`;
- не валидирует сессию через SQL;
- не читает и не пишет `ui_settings` напрямую.

### Identity Service

- Регистрация, login, logout.
- Валидация активной session.
- `GetCurrentUser`.
- Чтение и обновление `ui_settings`.

### Event Command Service

- Create/update/delete categories.
- Create/update/delete events.
- Ownership и business validation.
- Запись outbox event в той же транзакции.

### Event Query Service

- Categories list.
- Events list / single event.
- Dashboard.
- Calendar.
- Reports preview.
- Redis cache только для dashboard snapshot.

### Report Service

- Create/list/get/download export jobs.
- Проверка ownership для export artifacts.
- Чтение файлов из MinIO.

### Worker

- Relay из outbox в JetStream.
- Обновление projection tables.
- Инвалидация dashboard cache после релевантных domain events.
- Обработка export lifecycle.

## Request correlation

Синхронная цепочка запроса устроена так:

1. `edge-api` создает `x-request-id`.
2. `TraceLayer` кладет `request_id` в HTTP span.
3. `RequestIdInterceptor` пробрасывает `x-request-id` в gRPC metadata.
4. Внутренние сервисы создают gRPC span через `observability::grpc_request_span`.
5. Ошибки внешнего API возвращают `request_id` в error envelope.

Для async backbone основным коррелятором остается `message_id` / `outbox_event_id`.

## Observability baseline

### HTTP и gRPC

- `edge-api` использует `TraceLayer`.
- Логи поддерживают `LOG_FORMAT=json`.
- `x-request-id` выставляется наружу в HTTP response headers.

### Health и metrics

- Все Rust-сервисы поднимают отдельный metrics server.
- На metrics port доступны `/metrics` и `/healthz`.
- У `edge-api` на публичном порту также доступен `GET /healthz`.

### Базовые метрики

- HTTP request count и latency на `edge-api`.
- Cache hit/miss для dashboard cache.
- Security event counters.
- Export duration.
- Projection lag.
- Queue lag.

## Cache behavior

### Backend

- `event-query-svc` кэширует только dashboard snapshot в Redis.
- TTL dashboard cache: 60 секунд.
- Worker удаляет dashboard cache после category/event изменений.

### Backend без кэша

- Calendar и reports preview читаются напрямую из projection tables.
- Дополнительного Redis cache для них нет, чтобы не создавать еще один stale-слой.

### Frontend

- Default `staleTime` в TanStack Query: 5 секунд.
- После category/event mutations инвалидируются `categories`, `events`, `dashboard`, `calendar-events`, `reports`.

## Архитектурные инварианты

- Frontend не ходит к внутренним сервисам.
- `edge-api` не ходит напрямую в domain SQL.
- Write-side изменения обязаны создавать outbox events.
- Read-heavy экраны обязаны читать projections.
- Export artifact считается валидным только если файл реально существует в MinIO.

## Что остается на фазу 5

- Smoke script и повторяемый end-to-end прогон.
- CI hygiene и более жесткие quality gates.
- При необходимости расширение correlation story через async backbone.
