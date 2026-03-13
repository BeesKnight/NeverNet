# Runbook: запуск, проверка и локальная диагностика

## Назначение

Этот документ нужен для локального запуска, базовой диагностики и smoke-проверки EventDesign.

Он должен быть достаточным, чтобы новый человек без контекста смог:

- поднять стек;
- применить миграции;
- проверить, что всё healthy;
- пройти минимальный happy-path;
- понять, где искать проблему, если что-то сломалось.

## Предварительные требования

Необходимо иметь:

- Docker
- Docker Compose
- Node.js / npm для локальной работы с фронтендом, если нужно
- Rust toolchain для локальной разработки backend, если нужно
- свободные порты, указанные в `docker-compose.yml`

## Переменные окружения

Перед запуском нужно:

1. скопировать `backend/.env.example` в `backend/.env`, если такой сценарий предусмотрен;
2. скопировать `frontend/.env.example` в `frontend/.env`, если такой сценарий предусмотрен;
3. проверить:
   - PostgreSQL connection string;
   - Redis URL;
   - NATS URL;
   - MinIO endpoint / access key / secret key / bucket;
   - `FRONTEND_ORIGINS`;
   - cookie/security config.

Если `.env.example` устарели, их нужно обновить в рамках соответствующей фазы.

## Запуск всего стека

Целевая команда запуска:

```bash
docker compose up --build -d
```

Ожидаемое поведение:

- infra-сервисы стартуют первыми;
- migrator применяет миграции и завершает работу;
- прикладные сервисы стартуют после готовности infra;
- frontend становится доступным;
- metrics stack поднимается без ручных действий.

## Проверка состояния

Полезные команды:

```bash
docker compose ps
docker compose logs -f
docker compose logs -f edge-api
docker compose logs -f worker
```

Проверить health endpoints по нужным адресам и портам, если они задокументированы в коде и compose.

## Базовый smoke-path

Минимальный сценарий, который должен работать:

1. открыть frontend;
2. получить CSRF token;
3. зарегистрировать пользователя;
4. выполнить login;
5. создать категорию;
6. создать событие;
7. открыть список событий;
8. открыть dashboard;
9. открыть calendar;
10. запросить export;
11. дождаться completed status;
12. скачать файл.

Если хотя бы один из этих шагов нестабилен, проект не считается demo-ready.

## Диагностика типовых проблем

### Проблема: сервисы падают на старте

Проверь:

- есть ли healthchecks у infra;
- правильно ли настроен `depends_on`;
- отработал ли `db-migrator`;
- готов ли Postgres;
- готов ли NATS;
- готов ли MinIO;
- создан ли bucket.

### Проблема: SQL-ошибки при запуске

Проверь:

- применились ли миграции;
- совпадает ли схема БД с текущим кодом;
- не стартуют ли несколько сервисов, пытаясь мигрировать БД одновременно;
- корректен ли формат имён SQLx migration files.

### Проблема: события создаются, но dashboard/calendar не обновляются

Проверь:

- создаются ли outbox rows;
- публикуются ли они relay-процессом;
- существует ли stream/consumer в JetStream;
- не падает ли projector;
- не stale ли Redis cache.

### Проблема: login/register “иногда работает, иногда нет”

Проверь:

- выставляется ли auth cookie;
- совпадает ли CORS allowlist с origin фронтенда;
- используется ли `credentials: include`;
- работает ли CSRF flow;
- не конфликтуют ли SameSite/Secure настройки.

### Проблема: export completed, но скачать нельзя

Проверь:

- существует ли bucket;
- реально ли загружен объект в MinIO;
- корректен ли `object_key`;
- совпадает ли статус job с реальным состоянием storage;
- проходит ли ownership check;
- корректно ли формируется presigned URL.

## Метрики и observability

Минимум должно быть доступно:

- Prometheus;
- Grafana;
- метрики HTTP;
- метрики export jobs;
- метрики projection lag / queue lag, если реализованы;
- request correlation через `request_id`.

## Локальная разработка по частям

Если нужен частичный запуск:

- можно запускать frontend отдельно локально;
- можно запускать конкретный backend сервис локально;
- но end-to-end сценарий должен в итоге проверяться через полный Compose-стек.

## Обязательный финальный прогон

Перед тем как считать задачу завершённой, нужно прогнать:

```bash
docker compose up --build -d
docker compose ps
scripts/smoke.sh
```

И при необходимости:

```bash
cargo test --workspace --all-features
npm run lint
npm run typecheck
npm run build
npm test
```

## Требование к документации

Если меняется:

- структура запуска;
- порты;
- env variables;
- зависимости;
- healthcheck strategy;
- smoke flow;

то этот runbook должен быть обновлён в том же изменении.
