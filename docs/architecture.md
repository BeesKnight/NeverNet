# Архитектура EventDesign

## Обзор

EventDesign — это highload-inspired full-stack система для планирования мероприятий и управления событиями.

Архитектура специально сделана заметно сильнее обычного “дипломного CRUD-проекта”, но при этом она обязана оставаться запускаемой локально, понятной и стабильной для защиты.

Целевая форма системы:

- один публичный фронтенд;
- один публичный Edge API / BFF;
- внутренние Rust-сервисы с явными gRPC-границами;
- CQRS-inspired разделение write и read путей;
- PostgreSQL outbox для durable domain events;
- NATS JetStream для async transport;
- workers для проекций и экспортов;
- Redis для cache, rate limiting и coordination;
- MinIO для export artifacts;
- Prometheus и Grafana для observability.

## Топология

```text
Browser
  -> Frontend (React/Vite)
  -> Edge API / BFF (REST/JSON)

Edge API / BFF
  -> Identity Service (gRPC)
  -> Event Command Service (gRPC)
  -> Event Query Service (gRPC)
  -> Report Service (gRPC)

Identity Service
  -> PostgreSQL
  -> Redis (опционально для session acceleration / rate-control integration)

Event Command Service
  -> PostgreSQL write model
  -> outbox_events

Report Service
  -> PostgreSQL
  -> outbox_events
  -> MinIO metadata / presign integration

Worker
  -> outbox relay
  -> JetStream publisher
  -> projector consumer
  -> export job processor

Event Query Service
  -> PostgreSQL read-model / projection tables
  -> Redis cache

Observability
  -> Prometheus
  -> Grafana
```

## Архитектурные цели

Система должна обеспечивать:

- чистый внешний REST API для фронтенда;
- безопасную browser auth-модель на cookies;
- явное разделение write и read ответственности;
- durable async processing для projections и export jobs;
- быстрые read-пути для dashboard, calendar и reports;
- детерминированный startup;
- достаточную observability для локального дебага и защиты.

## Ответственность компонентов

### Frontend

Frontend отвечает за:

- пользовательский интерфейс и маршрутизацию;
- простую client-side валидацию;
- auth-aware UX;
- вызовы только Edge API;
- polling/refresh статуса экспортов там, где это нужно.

Frontend не должен знать внутреннюю топологию сервисов.

### Edge API / BFF

Edge API — единственная публичная backend-точка входа.

Он отвечает за:

- browser auth/session boundary;
- CSRF и CORS enforcement;
- нормализацию входящих запросов;
- генерацию и прокидывание request id;
- orchestration между внутренними сервисами;
- shaping ответов под нужды фронтенда.

Edge API не должен оставаться “скрытым монолитом” с прямым SQL-доступом к доменным таблицам.

### Identity Service

Отвечает за:

- регистрацию;
- login;
- logout;
- выпуск и валидацию сессий;
- password hashing;
- current-user lookup;
- revocation checks.

### Event Command Service

Отвечает за:

- create/update/delete категорий;
- create/update/delete мероприятий;
- ownership validation;
- enforcement status transitions;
- write-side бизнес-правила;
- запись authoritative state;
- создание outbox events в той же транзакции.

### Event Query Service

Отвечает за:

- event list reads;
- dashboard reads;
- calendar reads;
- report preview reads;
- filtered/sorted query paths;
- recent activity reads, если реализованы.

Query service должен читать projection-таблицы, а не write-таблицы, там где flow уже доведён до целевой архитектуры.

### Report Service

Отвечает за:

- создание export jobs;
- чтение export metadata;
- download authorization;
- выдачу presigned URL;
- report-related orchestration;
- при необходимости summary/report-specific reads, если они не живут в query service.

### Worker

Отвечает за:

- outbox relay в JetStream;
- bootstrap JetStream stream/consumers;
- обновление projection-моделей;
- обработку export jobs;
- идемпотентную обработку сообщений;
- cache invalidation / cache refresh side effects.

## Write path

Целевой write path:

1. Frontend вызывает Edge API.
2. Edge API аутентифицирует и валидирует запрос.
3. Edge API вызывает внутренний command service.
4. Command service пишет authoritative state.
5. Command service пишет matching outbox row в той же DB-транзакции.
6. Worker relay публикует unpublished outbox rows в JetStream.
7. Projector consumers обновляют projection-таблицы.
8. Query service читает обновлённые projections.

## Read path

Целевой read path:

1. Frontend вызывает Edge API.
2. Edge API аутентифицирует пользователя и обращается к query/report/internal service.
3. Query service отдаёт оптимизированные projection-backed данные.
4. Redis при необходимости кэширует hot reads.
5. Edge API возвращает frontend-shaped JSON.

## Зачем здесь CQRS

CQRS здесь нужен потому, что у EventDesign реально разные требования к двум классам операций:

- write-side требует корректности, явной бизнес-валидации и транзакционной надёжности;
- read-side требует скорости, удобных агрегатов, календарного представления, фильтрации, сортировки и отчётных выборок.

Это не “архитектура ради слова CQRS”.
Это способ одновременно иметь чистый write-путь и быстрый read-путь.

## Зачем нужен async backbone

Async backbone нужен, чтобы:

- не привязывать write-запросы к тяжёлой фоновой работе;
- держать request latency короткой;
- обновлять read-model независимо от write-транзакции;
- генерировать экспорты вне жизненного цикла request-response;
- иметь defendable production-style архитектуру.

## Startup и миграции

Стек обязан стартовать детерминированно.

Это означает:

- у инфраструктурных сервисов есть healthchecks;
- прикладные сервисы зависят от healthy infra;
- миграции идут через отдельный one-shot migrator;
- сервисы не соревнуются друг с другом за изменение схемы;
- JetStream bootstrap делается явно;
- MinIO bucket bootstrap делается явно.

## Модель безопасности

Целевая browser security-модель:

- HttpOnly session cookie;
- Secure cookie в production-like окружениях;
- явная SameSite policy;
- CSRF token для state-changing browser requests;
- строгий CORS allowlist;
- ownership checks на всех user-scoped сущностях;
- отсутствие выдачи лишних внутренних полей;
- отсутствие прямого доступа фронтенда к внутренним сервисам.

## Базовый observability baseline

Обязательный минимум:

- генерация request id на Edge;
- прокидывание correlation metadata через gRPC;
- structured logs во всех Rust-сервисах;
- Prometheus metrics на сервис;
- Grafana dashboard(ы);
- health endpoints и metrics endpoints;
- queue/projection/export метрики там, где это практически возможно.

OpenTelemetry должен быть как минимум учтён архитектурно, но не ценой развала запуска.

## Ключевые риски, которые нужно устранить

Архитектура считается настоящей только если устранены:

- race conditions при старте docker compose;
- гонки миграций и нестабильный порядок старта;
- отсутствие bootstrap stream/consumer в JetStream;
- отсутствие идемпотентности у projector/export flow;
- export jobs, у которых `completed`, но файл недоступен;
- прямой domain SQL внутри Edge API;
- слабая request correlation между сервисами;
- stale projection или stale cache;
- отсутствие smoke-проверки всего сквозного сценария.

## Критерий завершённости архитектуры

Архитектура считается доведённой только когда:

- весь стек стартует одной командой;
- write path гарантированно обновляет read path;
- export files реально скачиваются;
- фронтенд стабильно работает с cookie auth;
- документация совпадает с кодом;
- demo-flow проходит end-to-end без ручных костылей.
