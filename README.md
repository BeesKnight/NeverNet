# EventDesign

EventDesign — дипломный full-stack проект для планирования мероприятий, аналитики и асинхронного экспорта отчётов.

Текущее состояние репозитория соответствует фазе 1:

- стек поднимается одной командой `docker compose up --build -d`;
- миграции применяются отдельным one-shot сервисом `db-migrator`;
- bootstrap MinIO и NATS JetStream выполняется отдельным one-shot сервисом `infra-bootstrap`;
- прикладные сервисы стартуют только после healthy infra и успешного завершения bootstrap-этапов;
- фронтенд работает только через `edge-api`.

## Состав стека

```text
frontend/
backend/
  apps/
    db-migrator/
    infra-bootstrap/
    edge-api/
    identity-svc/
    event-command-svc/
    event-query-svc/
    report-svc/
    worker/
  crates/
    cache/
    contracts/
    messaging/
    observability/
    persistence/
    shared-kernel/
```

Compose-стек включает:

- `db` (PostgreSQL);
- `redis`;
- `nats` с JetStream;
- `minio`;
- `db-migrator`;
- `infra-bootstrap`;
- `identity-svc`;
- `event-command-svc`;
- `event-query-svc`;
- `report-svc`;
- `worker`;
- `edge-api`;
- `frontend`;
- `prometheus`;
- `grafana`.

## Быстрый старт

Основная команда запуска:

```bash
docker compose up --build -d
```

Ожидаемая последовательность:

1. Поднимаются `db`, `redis`, `nats`, `minio` и доходят до `healthy`.
2. `db-migrator` применяет SQLx migrations и завершается с кодом `0`.
3. `infra-bootstrap` создаёт или валидирует bucket в MinIO, stream в JetStream и durable consumers, затем завершается с кодом `0`.
4. Стартуют backend-сервисы и `worker`.
5. После их готовности стартует `edge-api`.
6. После `edge-api` стартует `frontend`.

Полезные команды проверки:

```bash
docker compose ps
docker compose logs --no-color
docker compose logs --no-color db-migrator infra-bootstrap
```

## Доступные адреса

- Frontend: `http://localhost:3000`
- Edge API: `http://localhost:8080`
- MinIO API: `http://localhost:9000`
- MinIO Console: `http://localhost:9001`
- NATS client port: `localhost:4222`
- NATS monitor API: `http://localhost:8222`
- Prometheus: `http://localhost:9090`
- Grafana: `http://localhost:3001`

## Что инициализируется автоматически

`db-migrator`:

- применяет все файлы из `backend/migrations`;
- является единственной точкой применения миграций при старте compose-стека.

`infra-bootstrap`:

- создаёт или валидирует bucket `eventdesign-exports` в MinIO;
- создаёт или валидирует stream `EVENTDESIGN_DOMAIN_EVENTS`;
- создаёт или валидирует durable consumers `projection-worker` и `export-worker`.

## Локальная разработка по частям

Если нужен частичный запуск без полного compose-стека:

1. Поднять инфраструктуру:

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

Vite dev server проксирует `/api` в `VITE_EDGE_API_ORIGIN`, поэтому браузерный auth flow остаётся cookie-based.

## Полезные проверки

Backend:

```bash
cd backend
cargo fmt --all
cargo check --workspace
```

Frontend:

```bash
cd frontend
npm run lint
npm run build
```

Инфраструктура:

```bash
docker compose up --build -d
docker compose ps
docker compose logs --no-color
```

## Текущие ограничения

- observability-стек поднят, но часть scrape/provisioning-настроек ещё требует доводки в следующих фазах;
- smoke script ещё не добавлен в репозиторий;
- auth/security-hardening, полный async backbone и финальная полировка export flow остаются задачами следующих фаз.

Подробности по архитектуре и этапам доведения: [docs/architecture.md](/docs/architecture.md), [docs/delivery-plan.md](/docs/delivery-plan.md), [docs/runbook.md](/docs/runbook.md).
