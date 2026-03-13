# ERD и структура данных

## Назначение

Этот документ описывает концептуальную схему EventDesign.

Это не автогенерированная SQL-спецификация, а человекочитаемая карта write-side, read-side и связей между сущностями.

## Write-side таблицы

### users

Поля:

- id (pk)
- email (unique)
- password_hash
- full_name
- created_at
- updated_at

### sessions

Поля:

- id (pk)
- user_id (fk -> users.id)
- created_at
- expires_at
- revoked_at
- user_agent
- ip_address

Назначение:

- durable browser sessions;
- server-side session validation;
- revoke on logout.

### categories

Поля:

- id (pk)
- user_id (fk -> users.id)
- name
- color
- created_at
- updated_at

### events

Поля:

- id (pk)
- user_id (fk -> users.id)
- category_id (fk -> categories.id)
- title
- description
- location
- starts_at
- ends_at
- budget
- status
- created_at
- updated_at

### ui_settings

Поля:

- user_id (pk, fk -> users.id)
- theme
- accent_color
- default_view
- created_at
- updated_at

### outbox_events

Поля:

- id (pk)
- aggregate_type
- aggregate_id
- event_type
- event_version
- payload_json
- occurred_at
- published_at
- publish_attempts
- last_error

### export_jobs

Поля:

- id (pk)
- user_id (fk -> users.id)
- report_type
- format
- status
- filters_json
- object_key
- content_type
- size_bytes
- error_message
- created_at
- started_at
- updated_at
- finished_at

### processed_messages

Поля:

- consumer_name
- message_id
- processed_at

Назначение:

- durable deduplication для idempotent consumers.
- используется и projector consumer, и export consumer.

## Read-side projection tables

### event_list_projection

Назначение:

- список событий;
- фильтрация;
- сортировка.

Поля:

- event_id
- user_id
- category_id
- category_name
- category_color
- title
- description
- location
- starts_at
- ends_at
- budget
- status
- created_at
- updated_at

### calendar_projection

Назначение:

- month/date-bucket rendering календаря.

Поля:

- event_id
- user_id
- date_bucket
- title
- starts_at
- ends_at
- status
- category_color
- updated_at

### dashboard_projection

Назначение:

- summary cards;
- dashboard widgets.

Поля:

- user_id
- total_events
- upcoming_events
- completed_events
- cancelled_events
- total_budget
- updated_at

### report_projection

Назначение:

- report preview;
- база для export generation.

Поля:

- event_id
- user_id
- category_id
- category_name
- category_color
- title
- description
- location
- starts_at
- ends_at
- budget
- status
- created_at
- updated_at

### recent_activity_projection

Назначение:

- recent activity feed;
- traceability user-visible действий.

Поля:

- id
- source_message_id
- user_id
- entity_type
- entity_id
- action
- title
- occurred_at
- created_at

## Основные связи

- один user имеет много sessions;
- один user имеет много categories;
- один user имеет много events;
- один user имеет одну запись `ui_settings`;
- один user имеет много `export_jobs`;
- одна category принадлежит одному user;
- одна category может иметь много events;
- projection rows производны от write-side событий.

## Жизненный цикл данных

### При создании события

1. запись пишется в `events`;
2. outbox event пишется в `outbox_events`;
3. relay публикует событие;
4. projector обновляет `event_list_projection`, `calendar_projection`, `dashboard_projection`, `report_projection`;
5. read-side eventually видит новое событие.

### При обновлении события

1. запись в `events` меняется;
2. outbox event фиксирует изменение;
3. projector переобновляет projections;
4. dashboard/calendar/reports eventually показывают новую версию.

### При удалении события

1. событие удаляется или помечается удалённым;
2. outbox фиксирует deletion event;
3. projector удаляет/обновляет projection rows;
4. read-side больше не показывает событие.

### При создании export job

1. `export_jobs` получает `queued`;
2. outbox фиксирует `export.requested`;
3. worker claim'ит job, переводит её в `processing` и не допускает второй активный claim для того же export;
4. worker генерирует файл, пишет объект в MinIO;
5. `export_jobs` получает `completed` или `failed`;
6. terminal transition одновременно пишет marker в `processed_messages`.

## Критические инварианты

Нужно гарантировать:

- согласованность ownership между `events.user_id` и `categories.user_id`;
- отсутствие foreign key drift;
- соответствие `export_jobs.status` реальному состоянию storage;
- отсутствие duplicate processing одного и того же message_id для одного consumer;
- отсутствие повторного terminal update для уже завершённого export job;
- eventual consistency между write-side state и projection-таблицами.

## Практическая заметка

Этот ERD надо держать в актуальном состоянии.
Если схема БД изменилась, а документ остался старым, он перестаёт быть полезным и начинает вредить.
