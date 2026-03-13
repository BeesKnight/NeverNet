# Реестр рисков и несостыковок

## Закрыто в фазе 4

### R1. Direct domain SQL в `edge-api`

Статус: закрыто.

Что сделано:

- `edge-api` больше не держит `PgPool`;
- session validation идет через `identity-svc`;
- `ui_settings` читаются и обновляются через `identity-svc`;
- `edge-api` больше не зависит от `DATABASE_URL`.

### R2. Слабая request correlation

Статус: закрыто для синхронной цепочки.

Что сделано:

- `x-request-id` генерируется на HTTP-входе;
- пробрасывается в gRPC metadata;
- попадает в HTTP error envelope;
- логируется во внутренних сервисах через gRPC span.

### R3. Непредсказуемый frontend cache

Статус: частично закрыто.

Что сделано:

- global `staleTime` снижен до 5 секунд;
- включен `refetchOnWindowFocus`;
- read-side query keys инвалидируются после category/event mutations.

Что осталось:

- eventual consistency проекций все еще существует по природе архитектуры;
- async side не коррелируется исходным HTTP `request_id`.

### R4. Неполный error envelope

Статус: закрыто.

Что сделано:

- ошибки `edge-api` имеют `code`, `message`, `request_id`.

## Актуальные риски

### H1. Async correlation ограничена message id

Риск:

- после выхода в outbox трассировка уже не держится на исходном HTTP `request_id`.

План:

- при необходимости расширить envelope событий или observability pipeline в фазе 5.

### H2. Smoke script и e2e happy-path

Статус: закрыто.

Что сделано:

- добавлен `scripts/smoke.sh`;
- smoke script проходит end-to-end на поднятом compose-стеке;
- smoke покрывает auth bootstrap, category/event flow, projection polling, export completion и logout.

### H3. Финальный CI hygiene зависит от установленного `cargo-audit`

Статус: частично закрыто.

Что сделано:

- добавлены и выровнены quality gates для `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test`, `npm ci`, `npm run lint`, `npm run typecheck`, `npm run build`, `npm test`;
- compose-старт и smoke-путь воспроизводимы;
- SQL-backed backend tests и frontend integration tests проходят.

Что осталось:

- `cargo audit` требует установленного `cargo-audit` в окружении запуска.

### M1. Dashboard cache зависит от корректной инвалидации worker-а

Риск:

- при поломке projection consumer stale dashboard вернется.

Смягчение:

- TTL 60 секунд;
- cache hit/miss и projection lag метрики;
- отсутствие отдельных backend caches для calendar и reports.

### M2. Worker все еще имеет dead-code warning

Статус: закрыто.

Что сделано:

- backend проходит `cargo clippy --workspace --all-targets --all-features -- -D warnings`.
