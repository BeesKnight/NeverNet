# Доменная модель EventDesign

## Bounded contexts

### Identity

Отвечает за:

- `users`
- `sessions`
- `ui_settings`
- browser auth контекст

### Event Management

Отвечает за:

- `categories`
- `events`
- ownership
- lifecycle и business validation write-side

### Reporting and Exports

Отвечает за:

- preview агрегаты по projections
- `export_jobs`
- downloadable artifacts

## Основные сущности

### User

- `id`
- `email`
- `password_hash`
- `full_name`
- `created_at`
- `updated_at`

### Session

- `id`
- `user_id`
- `created_at`
- `expires_at`
- `revoked_at`

Инварианты:

- session принадлежит одному user;
- revoked или expired session не должна проходить в `edge-api`;
- проверка session выполняется через `identity-svc`.

### UI Settings

- `user_id`
- `theme`
- `accent_color`
- `default_view`
- `created_at`
- `updated_at`

Инварианты:

- одна запись на пользователя;
- настройки user-scoped;
- владельцем `ui_settings` считается `identity-svc`, а не `edge-api`.

### Category

- `id`
- `user_id`
- `name`
- `color`
- `created_at`
- `updated_at`

Инварианты:

- category user-scoped;
- нельзя мутировать category другого пользователя;
- удаление запрещено, если category все еще используется event-ами.

### Event

- `id`
- `user_id`
- `category_id`
- `title`
- `description`
- `location`
- `starts_at`
- `ends_at`
- `budget`
- `status`
- `created_at`
- `updated_at`

Инварианты:

- event принадлежит одному пользователю;
- `category_id` обязан ссылаться на category того же пользователя;
- `starts_at < ends_at`;
- допустимые статусы: `planned`, `in_progress`, `completed`, `cancelled`;
- write-side mutation обязана создавать outbox event.

### Export Job

- `id`
- `user_id`
- `report_type`
- `format`
- `status`
- `filters`
- `object_key`
- `content_type`
- `error_message`
- `created_at`
- `started_at`
- `updated_at`
- `finished_at`

Инварианты:

- export job user-scoped;
- `completed` допустим только при наличии реального объекта в MinIO;
- скачивание обязано проверять ownership.

## Projection-backed read model

В read-side используются:

- `event_list_projection`
- `calendar_projection`
- `dashboard_projection`
- `report_projection`
- `recent_activity_projection`

Dashboard дополнительно может кэшироваться в Redis.

Calendar и report preview в фазе 4 backend-кэшем не прикрываются.

## Архитектурная граница

`edge-api` не считается владельцем ни одной доменной сущности.

Его роль:

- принять browser request;
- аутентифицировать пользователя;
- вызвать правильный внутренний сервис;
- вернуть frontend-shaped ответ.
