# Runbook: запуск, проверка и локальная диагностика

## Назначение

Этот документ фиксирует рабочий порядок запуска EventDesign в фазе 1 и команды для базовой диагностики.

Цель runbook:

- поднять стек одной командой;
- убедиться, что миграции применились ровно один раз;
- проверить, что MinIO bucket и JetStream bootstrap действительно созданы;
- быстро локализовать проблему, если startup или базовый happy-path сломались.

## Предварительные требования

Нужно иметь:

- Docker;
- Docker Compose;
- Node.js / npm для локальной работы с `frontend`, если требуется;
- Rust toolchain для локального запуска backend-приложений вне compose;
- свободные порты из `docker-compose.yml`.

## Стандартный запуск всего стека

Целевая команда:

```bash
docker compose up --build -d
```

Ожидаемая последовательность:

1. `db`, `redis`, `nats`, `minio` переходят в `healthy`.
2. `db-migrator` применяет SQLx migrations и завершается с кодом `0`.
3. `infra-bootstrap` создаёт или валидирует MinIO bucket и JetStream stream/consumers, затем завершается с кодом `0`.
4. Стартуют `identity-svc`, `event-command-svc`, `event-query-svc`, `report-svc`, `worker`.
5. После их готовности стартует `edge-api`.
6. После `edge-api` стартует `frontend`.

## Базовая проверка состояния

Минимальный набор команд после старта:

```bash
docker compose ps
docker compose logs --no-color
docker compose logs --no-color db-migrator infra-bootstrap
```

Что должно быть видно:

- `db-migrator` завершился со статусом `Exited (0)`;
- `infra-bootstrap` завершился со статусом `Exited (0)`;
- runtime-сервисы находятся в `Up` и не перезапускаются циклически;
- `edge-api`, `worker`, `report-svc`, `identity-svc`, `event-command-svc`, `event-query-svc` доходят до `healthy`.

## Проверка миграций

Проверить, что миграции действительно применились:

```bash
docker exec $(docker compose ps -q db) \
  psql -U eventdesign -d eventdesign \
  -c "SELECT version, description, success FROM _sqlx_migrations ORDER BY version;"
```

Ожидаемый результат:

- все migration files из `backend/migrations` присутствуют в `_sqlx_migrations`;
- `success = true` для каждой записи.

## Проверка MinIO bootstrap

Проверить liveness:

```bash
curl http://localhost:9000/minio/health/live
```

Проверить bucket:

```bash
docker exec $(docker compose ps -q minio) sh -c \
  "mc alias set local http://127.0.0.1:9000 eventdesign eventdesign123 >/dev/null && mc ls local"
```

Ожидаемый результат:

- в выводе присутствует bucket `eventdesign-exports/`.

## Проверка JetStream bootstrap

Проверить stream и consumers:

```bash
curl "http://localhost:8222/jsz?streams=true&consumers=true"
```

Ожидаемый результат:

- есть stream `EVENTDESIGN_DOMAIN_EVENTS`;
- у stream есть consumers `projection-worker` и `export-worker`.

## Базовый smoke-path

Минимальный сценарий, который должен работать после старта:

1. открыть `frontend`;
2. получить CSRF token;
3. зарегистрировать пользователя;
4. выполнить login;
5. создать категорию;
6. создать событие;
7. открыть список событий;
8. открыть dashboard;
9. открыть calendar;
10. создать export job;
11. дождаться статуса `completed`;
12. скачать файл.

Если один из шагов нестабилен, проект нельзя считать demo-ready.

## Локальная разработка по частям

Если нужен запуск без полного compose для приложений:

1. Поднять только инфраструктуру:

```bash
docker compose up -d db redis nats minio
```

2. Применить миграции и bootstrap:

```bash
cd backend
cargo run -p db-migrator
cargo run -p infra-bootstrap
```

3. Запустить backend-сервисы:

```bash
cargo run -p identity-svc
cargo run -p event-command-svc
cargo run -p event-query-svc
cargo run -p report-svc
cargo run -p worker
cargo run -p edge-api
```

4. Запустить фронтенд:

```bash
cd ../frontend
npm install
npm run dev
```

## Диагностика типовых проблем

### Проблема: backend не стартует после `docker compose up`

Проверить:

- действительно ли `db`, `redis`, `nats`, `minio` перешли в `healthy`;
- завершились ли `db-migrator` и `infra-bootstrap` с кодом `0`;
- нет ли SQL-ошибок в логах `db-migrator`;
- нет ли ошибок создания bucket/stream/consumer в логах `infra-bootstrap`.

### Проблема: SQL-ошибки на старте

Проверить:

- что `_sqlx_migrations` содержит все migration files;
- что ни один runtime-сервис не пытается мигрировать БД самостоятельно;
- что `db-migrator` был единственной точкой применения миграций.

### Проблема: export pipeline падает

Проверить:

- существует ли bucket `eventdesign-exports`;
- доступен ли MinIO по `http://localhost:9000/minio/health/live`;
- нет ли ошибок загрузки артефактов в логах `worker` и `report-svc`.

### Проблема: события не доходят до read-side

Проверить:

- существует ли stream `EVENTDESIGN_DOMAIN_EVENTS`;
- существуют ли consumers `projection-worker` и `export-worker`;
- нет ли ошибок outbox relay и projection consumer в логах `worker`;
- не маскирует ли проблему stale Redis cache.

## Метрики и observability

Минимально доступны:

- Prometheus: `http://localhost:9090`;
- Grafana: `http://localhost:3001`;
- `/metrics` и `/healthz` на metrics-портах backend-сервисов;
- queue/projection/export метрики там, где они уже реализованы.

## Обязательный прогон перед завершением задачи

Минимум:

```bash
docker compose up --build -d
docker compose ps
docker compose logs --no-color
```

Дополнительно, если менялся код:

```bash
cd backend
cargo fmt --all
cargo check --workspace
```

```bash
cd frontend
npm run lint
npm run build
```
