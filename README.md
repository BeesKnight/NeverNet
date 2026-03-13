# EventDesign

EventDesign — дипломный full-stack проект для планирования мероприятий, аналитики и асинхронного экспорта отчетов.

Репозиторий приведен к состоянию фазы 4:

- `edge-api` работает как BFF и browser boundary;
- прямого domain SQL в `edge-api` больше нет;
- внутренние вызовы идут через gRPC в `identity-svc`, `event-command-svc`, `event-query-svc` и `report-svc`;
- write-side изменения попадают в outbox и дальше в NATS JetStream;
- dashboard использует projection-backed Redis cache с TTL и event-driven invalidation;
- calendar и reports preview читают projections без дополнительного server-side cache;
- browser auth работает через `HttpOnly` cookie и CSRF;
- HTTP и gRPC цепочка коррелируется через `x-request-id`.

## Состав стека

```text
frontend
  -> edge-api
      -> identity-svc
      -> event-command-svc
      -> event-query-svc
      -> report-svc

write-side
  -> PostgreSQL
  -> outbox_events
  -> worker relay
  -> NATS JetStream
  -> projection/export consumers

read-side
  -> projection tables
  -> Redis cache for dashboard

exports
  -> report-svc
  -> worker
  -> MinIO
```

## Быстрый старт

Полный compose-стек поднимается одной командой:

```bash
docker compose up --build -d
```

Ожидаемая последовательность:

1. Поднимаются `db`, `redis`, `nats`, `minio`.
2. `db-migrator` применяет миграции и завершается.
3. `infra-bootstrap` создает bucket и JetStream stream/consumers и завершается.
4. Стартуют внутренние сервисы и `worker`.
5. После их готовности стартует `edge-api`.
6. После `edge-api` стартует `frontend`.

## Основные адреса

- Frontend: `http://localhost:3000`
- Edge API: `http://localhost:8080`
- Edge API health: `http://localhost:8080/healthz`
- Edge API metrics: `http://localhost:9100/metrics`
- Prometheus: `http://localhost:9090`
- Grafana: `http://localhost:3001`
- MinIO API: `http://localhost:9000`
- MinIO Console: `http://localhost:9001`
- NATS monitor API: `http://localhost:8222`

## Локальная разработка

Если нужен запуск по частям:

```bash
docker compose up -d db redis nats minio
cd backend
cargo run -p db-migrator
cargo run -p infra-bootstrap
cargo run -p identity-svc
cargo run -p event-command-svc
cargo run -p event-query-svc
cargo run -p report-svc
cargo run -p worker
cargo run -p edge-api
cd ../frontend
npm install
npm run dev
```

## Обязательные проверки

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
npm run typecheck
npm run build
npm test
```

## Что важно знать про фазу 4

- `edge-api` больше не требует `DATABASE_URL` и не зависит от PostgreSQL напрямую.
- Валидация сессии и работа с `ui_settings` вынесены в `identity-svc`.
- Ошибки внешнего API возвращаются в едином envelope:

```json
{
  "error": {
    "code": "bad_request",
    "message": "Человекочитаемое описание",
    "request_id": "6f4d0d7a-..."
  }
}
```

- `x-request-id` генерируется на HTTP-входе и пробрасывается в gRPC metadata.
- Во frontend default query cache больше не держит данные свежими 30 секунд; read-side ключи инвалидируются после category/event mutations.

## Ограничения до фазы 5

- `scripts/smoke.sh` еще не добавлен.
- Полный CI hygiene и `cargo clippy -- -D warnings` остаются задачами следующей фазы.
- Async request correlation после выхода запроса в outbox по-прежнему опирается на `message_id`/`outbox_event_id`, а не на исходный HTTP `request_id`.

Подробности:

- [docs/architecture.md](docs/architecture.md)
- [docs/api.md](docs/api.md)
- [docs/messaging.md](docs/messaging.md)
- [docs/runbook.md](docs/runbook.md)
- [docs/demo-script.md](docs/demo-script.md)
