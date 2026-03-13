# Messaging и async backbone

## Назначение

EventDesign использует outbox + NATS JetStream, чтобы:

- не держать write-side запросы на тяжелой фоновой работе;
- обновлять projections отдельно от command transaction;
- выполнять экспорт асинхронно;
- переживать at-least-once delivery без порчи состояния.

## Канонический flow

1. Command-side сервис коммитит доменную запись.
2. В той же транзакции пишет строку в `outbox_events`.
3. Worker relay публикует envelope в JetStream.
4. Projection consumer обновляет read-model.
5. Export consumer забирает `export.requested` и строит файл.

## Ответственность компонентов

### Event Command Service

- authoritative write-side для categories и events;
- запись domain event в outbox.

### Report Service

- создание `export_jobs`;
- запись `export.requested` в outbox.

### Worker relay

- читает unpublished outbox rows;
- публикует в `EVENTDESIGN_DOMAIN_EVENTS`;
- отдельно фиксирует успешную публикацию или ошибку.

### Projection consumer

- читает category/event события;
- обновляет `event_list_projection`, `calendar_projection`, `dashboard_projection`, `report_projection`, `recent_activity_projection`;
- после релевантных изменений инвалидирует dashboard cache в Redis.

### Export consumer

- обрабатывает `export.requested`;
- переводит job в `processing`;
- строит PDF/XLSX из `report_projection`;
- загружает артефакт в MinIO;
- завершает job как `completed` или `failed`.

## Subject naming

- `eventdesign.category.created`
- `eventdesign.category.updated`
- `eventdesign.category.deleted`
- `eventdesign.event.created`
- `eventdesign.event.updated`
- `eventdesign.event.deleted`
- `eventdesign.event.status_changed`
- `eventdesign.export.requested`
- `eventdesign.export.completed`
- `eventdesign.export.failed`

## Идемпотентность

Для consumer side используется `processed_messages`.

Правила:

- одно и то же `message_id` не должно применяться повторно;
- ack происходит только после локального durable commit;
- duplicate delivery превращается в no-op, а не в повторную мутацию.

## Cache side effects

В фазе 4 к async backbone привязано только одно серверное cache side effect:

- после обработки category/event событий worker удаляет Redis key dashboard cache.

Calendar и reports preview не имеют отдельного backend cache и читаются напрямую из projections.

## Метрики

Минимум, который уже должен быть доступен:

- lag projection consumer;
- lag export consumer;
- export duration;
- cache hit/miss для dashboard cache.

## Ограничение текущей фазы

`request_id` живет на синхронной HTTP -> gRPC цепочке.

После выхода запроса в async backbone основным коррелятором остаются:

- `outbox_events.id`
- `processed_messages.message_id`
- JetStream `message_id`

Это осознанное ограничение текущего baseline, а не скрытая магия.
