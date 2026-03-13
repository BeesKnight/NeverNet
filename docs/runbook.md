# Runbook: запуск и диагностика

## Полный запуск

```bash
docker compose up --build -d
```

Ожидаемая последовательность:

1. `db`, `redis`, `nats`, `minio` переходят в `healthy`.
2. `db-migrator` завершается с кодом `0`.
3. `infra-bootstrap` завершается с кодом `0`.
4. Поднимаются `identity-svc`, `event-command-svc`, `event-query-svc`, `report-svc`, `worker`.
5. `demo-seed` завершает подготовку демонстрационного набора данных.
6. Поднимается `edge-api`.
7. Поднимается `frontend`.

## Базовые проверки

```bash
docker compose ps
docker compose logs --no-color
docker compose logs --no-color db-migrator infra-bootstrap demo-seed
```

Критерии:

- one-shot сервисы завершились успешно;
- runtime-сервисы не рестартуются циклически;
- `edge-api` и `frontend` доступны;
- demo user существует и может войти без ручного наполнения системы.

## Demo seed

После полного compose-подъема автоматически доступны:

- email: `demo@eventdesign.local`
- пароль: `DemoPass123!`

Если после ручных экспериментов нужно вернуть проект в демонстрационное состояние:

```bash
docker compose run --rm demo-seed
```

Эта команда пере-создает demo user, категории, события, projection-таблицы и историю export jobs.

## Smoke

Минимальный e2e happy-path фиксируется отдельным скриптом:

```bash
scripts/smoke.sh
```

Скрипт проверяет:

- доступность `frontend`, `edge-api`, `NATS monitor`, `MinIO`;
- CSRF + register;
- auth bootstrap через `GET /api/auth/me`;
- создание category и event;
- появление event в `events`, `dashboard`, `calendar`;
- создание и завершение export job;
- download export artifact;
- logout и последующий `401` на `GET /api/auth/me`.

## Health и metrics

### Публичный health

```bash
curl http://localhost:8080/healthz
```

### Публично доступные служебные endpoints

- `http://localhost:9100/metrics` и `http://localhost:9100/healthz` — `edge-api`
- `http://localhost:9090` — `Prometheus`
- `http://localhost:3001` — `Grafana`
- `http://localhost:8222` — `NATS monitor API`
- `http://localhost:9000/minio/health/live` — `MinIO health`

Метрики внутренних сервисов не публикуются наружу отдельными host-port mapping.
Для них используй `Prometheus`, `Grafana` или `docker compose exec`.

## Проверка request correlation

```bash
curl -i \
  -H "x-request-id: demo-phase5-001" \
  http://localhost:8080/healthz
```

Что проверить:

- в ответе есть `x-request-id`;
- в логах `edge-api` запрос виден с тем же `request_id`;
- при gRPC-вызове внутренних сервисов тот же `request_id` попадает в span.

## Проверка compose-зависимостей

`edge-api` в фазе 4 не зависит напрямую от PostgreSQL и не требует `DATABASE_URL`.

Если `edge-api` не стартует, проверять нужно:

- `redis`;
- `identity-svc`;
- `event-command-svc`;
- `event-query-svc`;
- `report-svc`.

## Проверка read-side свежести

После create/update/delete category или event:

1. обновить страницу events;
2. открыть dashboard;
3. открыть calendar;
4. открыть reports summary.

Ожидаемое поведение:

- projections догоняют write-side;
- dashboard cache сбрасывается worker-ом;
- frontend query cache не держит старое состояние 30 секунд.

Практически допустимое окно eventual consistency — секунды, а не десятки секунд клиентского cache.

## Полезные локальные команды

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

## Типовые проблемы

### `edge-api` отвечает 401/403

Проверить:

- есть ли `eventdesign_session` cookie;
- был ли запрошен `GET /api/auth/csrf`;
- отправляется ли `X-CSRF-Token`;
- жив ли `identity-svc`.

Если проблема воспроизводится только у demo user:

- перепрогнать `docker compose run --rm demo-seed`;
- убедиться, что `demo-seed` завершился с кодом `0`.

### stale dashboard

Проверить:

- есть ли новые rows в `dashboard_projection`;
- есть ли ошибки в логах `worker`;
- сбросился ли Redis key dashboard cache;
- не осталось ли старое состояние только во frontend query cache.

### export completed, но download не работает

Проверить:

- `export_jobs.status = 'completed'`;
- заполнены `object_key` и `content_type`;
- объект реально существует в MinIO;
- `report-svc` может прочитать объект.

### smoke script падает на projection polling

Проверить:

- есть ли новые строки в `outbox_events`;
- публикует ли `worker` события в JetStream;
- появляются ли записи в `event_list_projection`, `calendar_projection`, `dashboard_projection`;
- не отстает ли `worker` по логам и метрикам lag.
