# Внешний API

## Назначение

Этот документ описывает browser-facing REST API, которое отдает `edge-api`.

Frontend общается только с ним. Внутренние gRPC контракты сюда не выносятся.

## Browser security модель

- Auth хранится в `HttpOnly` cookie `eventdesign_session`.
- CSRF token выдается через `GET /api/auth/csrf`.
- State-changing запросы обязаны отправлять `X-CSRF-Token`.
- Browser использует `credentials: include`.
- `FRONTEND_ORIGINS` — строгий allowlist без `*`.
- Для non-local origin требуется `AUTH_COOKIE_SECURE=true`.

## Correlation

- `edge-api` генерирует `x-request-id`.
- Заголовок возвращается в HTTP response.
- При ошибке `request_id` также попадает в JSON envelope.

## Формат успешного ответа

```json
{
  "data": {}
}
```

## Формат ошибки

```json
{
  "error": {
    "code": "bad_request",
    "message": "Человекочитаемое описание ошибки",
    "request_id": "6f4d0d7a-..."
  }
}
```

### Поддерживаемые error codes

- `bad_request`
- `unauthorized`
- `not_found`
- `conflict`
- `rate_limited`
- `config_error`
- `internal_error`

## Health endpoint

- `GET /healthz` — публичный health-check `edge-api`.
- `/metrics` не торчит на публичном порту и доступен только на metrics port.

## Группы маршрутов

### Auth

- `GET /api/auth/csrf`
- `POST /api/auth/register`
- `POST /api/auth/login`
- `POST /api/auth/logout`
- `GET /api/auth/me`

Особенности:

- `edge-api` не валидирует session через SQL.
- Проверка session и current user идет через `identity-svc`.

### Categories

- `GET /api/categories`
- `POST /api/categories`
- `PUT|PATCH /api/categories/:id`
- `DELETE /api/categories/:id`

Command-side операции уходят в `event-command-svc`, чтение — в `event-query-svc`.

### Events

- `GET /api/events`
- `GET /api/events/:id`
- `POST /api/events`
- `PUT|PATCH /api/events/:id`
- `DELETE /api/events/:id`

Query параметры списка:

- `search`
- `status`
- `category_id`
- `start_date`
- `end_date`
- `sort_by`
- `sort_dir`

### Dashboard

- `GET /api/dashboard`

Источник данных: `dashboard_projection` через `event-query-svc`, при необходимости через Redis dashboard cache.

### Calendar

- `GET /api/calendar?start_date=YYYY-MM-DD&end_date=YYYY-MM-DD`

Источник данных: `calendar_projection` через `event-query-svc`.

### Reports

- `GET /api/reports/summary`
- `GET /api/reports/by-category`

Источник данных: `report_projection` через `event-query-svc`.

### Settings

- `GET /api/settings`
- `PUT|PATCH /api/settings`

`ui_settings` принадлежат `identity-svc`. `edge-api` больше не читает их из БД напрямую.

### Exports

- `GET /api/exports`
- `POST /api/exports`
- `GET /api/exports/:id`
- `GET /api/exports/:id/download`

Export job создается через `report-svc`, а фоновая обработка идет через outbox + JetStream + worker.

## Инварианты внешнего API

- Нет прямого доступа frontend к внутренним сервисам.
- Нет bearer auth через `localStorage`.
- Нет object-level leaks между пользователями.
- Ответы и ошибки имеют предсказуемую JSON-форму.
