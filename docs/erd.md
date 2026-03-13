# ERD и структура данных

## Write-side таблицы

### users

- `id`
- `email`
- `password_hash`
- `full_name`
- `created_at`
- `updated_at`

### sessions

- `id`
- `user_id`
- `created_at`
- `expires_at`
- `revoked_at`

### ui_settings

- `user_id`
- `theme`
- `accent_color`
- `default_view`
- `created_at`
- `updated_at`

Владелец таблицы на уровне сервисной границы: `identity-svc`.

### categories

- `id`
- `user_id`
- `name`
- `color`
- `created_at`
- `updated_at`

### events

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

### export_jobs

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

### outbox_events

- `id`
- `aggregate_type`
- `aggregate_id`
- `event_type`
- `event_version`
- `payload_json`
- `occurred_at`
- `published_at`
- `publish_attempts`
- `last_error`

### processed_messages

- `consumer_name`
- `message_id`
- `processed_at`

## Read-side projection tables

### event_list_projection

Источник списка событий, фильтрации и сортировки.

### calendar_projection

Источник month/day calendar view.

### dashboard_projection

Источник summary cards и части dashboard widgets.

### report_projection

Источник report preview и экспортной генерации.

### recent_activity_projection

Источник recent activity feed.

## Связи

- `users` 1:N `sessions`
- `users` 1:1 `ui_settings`
- `users` 1:N `categories`
- `users` 1:N `events`
- `users` 1:N `export_jobs`
- `categories` 1:N `events`

## Важная практическая оговорка

Наличие таблицы в PostgreSQL не означает, что `edge-api` имеет право ходить в нее напрямую.

После фазы 4 сервисные границы такие:

- `identity-svc` владеет `users`, `sessions`, `ui_settings`;
- `event-command-svc` владеет write-side мутациями `categories` и `events`;
- `event-query-svc` владеет read-side доступом к projection tables;
- `report-svc` владеет `export_jobs` и download boundary;
- `edge-api` напрямую в доменные таблицы не ходит.
