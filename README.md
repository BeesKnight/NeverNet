# EventDesign

EventDesign — дипломный full-stack проект для планирования мероприятий, dashboard-аналитики, календарного представления и асинхронного экспорта отчетов.

Репозиторий доведен до состояния фазы 5:

- `frontend` работает только через `edge-api`;
- внутренние вызовы идут через gRPC в `identity-svc`, `event-command-svc`, `event-query-svc` и `report-svc`;
- write-side изменения публикуются через outbox и `NATS JetStream`;
- read-heavy экраны читают projection-таблицы;
- browser auth использует `HttpOnly` cookie и CSRF;
- compose-подъем автоматически прогоняет `demo-seed`;
- есть SQL-backed backend tests, frontend integration/smoke tests и `scripts/smoke.sh`.

## Быстрый старт

Полный стек поднимается одной командой:

```bash
docker compose up --build -d
```

При обычном старте compose:

1. Поднимает инфраструктуру `db`, `redis`, `nats`, `minio`.
2. Прогоняет `db-migrator` и `infra-bootstrap`.
3. Стартует внутренние сервисы и `worker`.
4. Прогоняет `demo-seed`.
5. После успешного seed запускает `edge-api` и `frontend`.

Если нужно повторно восстановить демонстрационный набор данных без полного пересоздания стека:

```bash
docker compose run --rm demo-seed
```

## Demo dataset

После штатного `docker compose up --build -d` доступны:

- demo user: `demo@eventdesign.local`
- пароль: `DemoPass123!`
- несколько категорий;
- события с разными статусами;
- заполненные `dashboard` и `calendar`;
- история export jobs со статусами `completed` и `queued`.

## Smoke

Минимальный happy-path фиксируется скриптом:

```bash
scripts/smoke.sh
```

Smoke script проверяет:

- доступность публичных entrypoint-ов и инфраструктурных health endpoint-ов;
- CSRF + register;
- `GET /api/auth/me`;
- создание категории;
- создание события;
- появление события в `events`, `dashboard`, `calendar`;
- создание export job;
- переход export в `completed`;
- download артефакта;
- logout и инвалидацию сессии.

## Команды проверки

Backend:

```bash
cd backend
export DATABASE_URL=postgres://eventdesign:eventdesign@localhost:5432/eventdesign
cargo fmt --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
cargo audit
```

Frontend:

```bash
cd frontend
npm ci
npm run lint
npm run typecheck
npm run build
npm test
```

Полный production-like прогон:

```bash
docker compose up --build -d
scripts/smoke.sh
```

## Структура

```text
frontend/
backend/apps/edge-api
backend/apps/identity-svc
backend/apps/event-command-svc
backend/apps/event-query-svc
backend/apps/report-svc
backend/apps/worker
backend/apps/demo-seed
backend/crates/*
docs/*
ops/*
scripts/*
```

## Основные адреса

- Frontend: `http://localhost:3000`
- Edge API: `http://localhost:8080`
- Edge API health: `http://localhost:8080/healthz`
- Edge API metrics: `http://localhost:9100/metrics`
- PostgreSQL: `localhost:5432`
- Prometheus: `http://localhost:9090`
- Grafana: `http://localhost:3001`
- MinIO API: `http://localhost:9000`
- MinIO Console: `http://localhost:9001`
- NATS monitor API: `http://localhost:8222`

## Что важно помнить

- `edge-api` не ходит в доменную БД напрямую.
- `ui_settings` и session validation принадлежат `identity-svc`.
- dashboard использует Redis только как короткоживущий cache над projection-backed read model.
- calendar и reports читают projections без отдельного backend cache.
- async correlation после выхода запроса в outbox по-прежнему опирается на `message_id` и `outbox_event_id`, а не на исходный HTTP `request_id`.

## Документация

- [docs/architecture.md](docs/architecture.md)
- [docs/api.md](docs/api.md)
- [docs/messaging.md](docs/messaging.md)
- [docs/runbook.md](docs/runbook.md)
- [docs/demo-script.md](docs/demo-script.md)
- [docs/review-checklist.md](docs/review-checklist.md)
- [docs/risk-register.md](docs/risk-register.md)
