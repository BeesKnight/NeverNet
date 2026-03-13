# Messaging и async backbone

## Назначение

Этот документ описывает внутреннюю асинхронную модель EventDesign.

Архитектура использует:

- PostgreSQL outbox для durable event capture;
- NATS JetStream для transport слоя событий;
- worker consumers для projection updates и export processing.

## Базовые гарантии

Async backbone считается корректным только если одновременно соблюдаются следующие условия:

- бизнес-запись и соответствующий outbox event коммитятся в одной транзакции;
- unpublished outbox rows безопасно ретраятся;
- bootstrap stream и consumer в JetStream детерминирован;
- projector consumers устойчивы к at-least-once delivery;
- projection changes идемпотентны;
- ack сообщения происходит только после durable local handling;
- export processing защищён от duplicate claim и duplicate work.

## Зачем это вообще нужно

Async backbone нужен, чтобы:

- держать write-запросы быстрыми;
- не привязывать тяжёлую фоновую работу к latency HTTP-запроса;
- обновлять projection-модели независимо от write path;
- выполнять экспорты вне жизненного цикла request-response;
- иметь defendable production-style архитектуру.

## Канонический write path

1. command-side или report-side service принимает валидную мутацию;
2. мутация коммитится в PostgreSQL;
3. matching outbox event пишется в той же транзакции;
4. worker relay читает unpublished outbox rows;
5. relay публикует их в NATS JetStream;
6. consumer workers обрабатывают события;
7. projections или export states обновляются;
8. при необходимости выполняется cache invalidation.

## Таблица outbox

Ожидаемые поля outbox:

- id
- aggregate_type
- aggregate_id
- event_type
- event_version
- payload_json
- occurred_at
- published_at
- publish_attempts
- last_error

Правила:

- вставка в outbox должна происходить в той же DB-транзакции, что и запись authoritative state;
- `published_at` выставляется только после успешной публикации;
- каждая неудача публикации увеличивает `publish_attempts`;
- `last_error` должен сохраняться;
- relay не имеет права “просто молча” терять плохие строки.

## Требования к bootstrap JetStream

Стек должен явно гарантировать:

- stream существует;
- consumer существует;
- subjects объявлены и согласованы;
- durable consumer names стабильны там, где это нужно;
- локальный запуск не зависит от ручной настройки NATS.

Если создание stream/consumer неявное или manual-only, это надо исправить.

## Subject naming

Subject names должны быть явными и стабильными.

Рекомендуемые subjects:

- `eventdesign.category.created`
- `eventdesign.category.updated`
- `eventdesign.category.deleted`
- `eventdesign.event.created`
- `eventdesign.event.updated`
- `eventdesign.event.deleted`
- `eventdesign.event.status_changed`
- `eventdesign.export.requested`
- `eventdesign.export.started`
- `eventdesign.export.completed`
- `eventdesign.export.failed`

Не используй расплывчатые или перегруженные subject names.

## Ответственность relay

Outbox relay отвечает за:

- выбор unpublished outbox rows;
- публикацию в JetStream;
- пометку успешной публикации;
- сохранение ошибок без потери данных;
- bounded retry behavior;
- достаточные logging/metrics для диагностики lag и failures.

## Ответственность consumers

### Projection consumer

Потребляет:

- category events;
- event events.

Обновляет:

- `event_list_projection`
- `calendar_projection`
- `dashboard_projection`
- `report_projection`
- `recent_activity_projection`

Правила:

- обработка должна быть идемпотентной;
- дубли не должны создавать duplicate rows или ломать агрегаты;
- там, где это подходит, предпочтительны UPSERT-паттерны.

### Export consumer

Потребляет:

- `export.requested`

Выполняет:

- claim queued job;
- перевод job в `processing`;
- генерацию артефакта на основе projection-backed данных;
- загрузку артефакта в MinIO;
- перевод job в `completed` или `failed`;
- при необходимости эмитит lifecycle events экспорта.

Правила:

- duplicate work должен быть предотвращён или безопасно переживаться;
- статусные переходы job должны быть явными и монотонными;
- `completed` job обязан соответствовать реальному объекту в storage.

## processed_messages и дедупликация

Для идемпотентных consumers должна использоваться таблица `processed_messages`.

Рекомендуемые поля:

- consumer_name
- message_id
- processed_at

Правила:

- один consumer не должен дважды применять одно и то же сообщение;
- duplicate delivery должна превращаться в no-op после durable deduplication check;
- deduplication по возможности должна жить в той же транзакции, что и mutation projection state.

## Семантика ack

Правильный порядок обработки сообщения:

1. consumer получает сообщение;
2. открывает локальную транзакцию;
3. применяет business/projection/export state changes;
4. записывает processed message или dedupe marker;
5. коммитит транзакцию;
6. только после этого делает ack.

Нельзя ack’ать до durable commit.
Именно так люди получают “всё вроде ок, но данные почему-то пропали”.

## Канонический export flow

1. frontend запрашивает экспорт через Edge API;
2. Edge API идёт в report service;
3. report service создаёт `export_jobs` со статусом `queued`;
4. report service пишет `export.requested` в outbox;
5. relay публикует событие в JetStream;
6. export worker потребляет событие;
7. worker claim’ит job и переводит её в `processing`;
8. worker строит файл на основе projection-backed read model;
9. worker загружает объект в MinIO;
10. worker обновляет метаданные job и переводит её в `completed` или `failed`;
11. report service выдаёт secure download access или presigned URL.

## Использование Redis в async flow

Redis допустимо использовать для:

- distributed locks или claim coordination;
- cache invalidation;
- hot read caches;
- rate limit counters.

Redis не должен подменять собой durable source-of-truth storage.

## Метрики, которые желательно иметь

Как минимум нужны или должны быть легко вычисляемы:

- количество unpublished outbox rows;
- количество ошибок публикации outbox;
- consumer lag в JetStream;
- latency обработки projection updates;
- длительность export jobs;
- количество duplicate/deduped messages;
- cache hit/miss, если кэш используется на projection-backed endpoints.

## Типовые failure modes, которые надо предотвратить

Самые опасные сценарии:

- stream не создан на старте;
- consumer не создан на старте;
- projector делает ack до DB commit;
- после redelivery в projections появляются дубли;
- export job помечается completed без реальной загрузки объекта;
- dashboard stale из-за отсутствия cache invalidation;
- outbox rows никогда не публикуются, потому что relay падает молча.

## Критерий завершённости async backbone

Messaging/backbone считается завершённым только когда:

- command writes создают outbox rows;
- relay публикует эти rows;
- consumers обновляют projections идемпотентно;
- read-side UI отражает обновлённое projection state;
- export jobs заканчиваются реальными downloadable artifacts;
- duplicate delivery не повреждает состояние системы.
