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

### H2. Нет smoke script

Риск:

- end-to-end happy-path еще не зафиксирован одной командой.

План:

- `scripts/smoke.sh` в фазе 5.

### H3. Нет финального CI hygiene

Риск:

- регрессии могут проходить, если человек ограничился локальной сборкой.

План:

- зафиксировать полный набор quality gates в фазе 5.

### M1. Dashboard cache зависит от корректной инвалидации worker-а

Риск:

- при поломке projection consumer stale dashboard вернется.

Смягчение:

- TTL 60 секунд;
- cache hit/miss и projection lag метрики;
- отсутствие отдельных backend caches для calendar и reports.

### M2. Worker все еще имеет dead-code warning

Риск:

- это не runtime bug, но сигнал, что phase 5 должна дочистить quality baseline.
