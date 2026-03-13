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
5. Поднимается `edge-api`.
6. Поднимается `frontend`.

## Базовые проверки

```bash
docker compose ps
docker compose logs --no-color
docker compose logs --no-color db-migrator infra-bootstrap
```

Критерии:

- one-shot сервисы завершились успешно;
- runtime-сервисы не рестартуются циклически;
- `edge-api` и metrics endpoints доступны.

## Health и metrics

### Публичный health

```bash
curl http://localhost:8080/healthz
```

### Служебные endpoints

- `http://localhost:9100/metrics` и `http://localhost:9100/healthz` — `edge-api`
- `http://localhost:9101/metrics` — `identity-svc`
- `http://localhost:9102/metrics` — `event-command-svc`
- `http://localhost:9103/metrics` — `event-query-svc`
- `http://localhost:9104/metrics` — `report-svc`
- `http://localhost:9105/metrics` — `worker`

## Проверка request correlation

```bash
curl -i ^
  -H "x-request-id: demo-phase4-001" ^
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

## Типовые проблемы

### `edge-api` отвечает 401/403

Проверить:

- есть ли `eventdesign_session` cookie;
- был ли запрошен `GET /api/auth/csrf`;
- отправляется ли `X-CSRF-Token`;
- жив ли `identity-svc`.

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
