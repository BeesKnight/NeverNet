# Frontend EventDesign

Frontend построен на React, TypeScript, Vite, React Router и TanStack Query.

Клиент общается только с `edge-api`. Прямые вызовы внутренних сервисов запрещены.

## Команды

```bash
npm install
npm run dev
npm run lint
npm run typecheck
npm run build
npm test
```

## Переменные окружения

Базовая локальная конфигурация:

```bash
VITE_API_BASE_URL=/api
VITE_EDGE_API_ORIGIN=http://localhost:8080
```

`VITE_API_BASE_URL=/api` нужен для Vite proxy и для production-like runtime-конфига.

## Browser auth модель

Frontend ожидает:

- cookie-based session через `credentials: include`;
- CSRF token из `GET /api/auth/csrf` для всех `POST`, `PATCH`, `PUT`, `DELETE`;
- отсутствие bearer token в `localStorage` и `Authorization` header;
- единый error envelope с `code`, `message` и `request_id`.

## Query cache поведение

После фазы 4 клиентский cache настроен так:

- default `staleTime` снижен до `5_000` мс;
- `refetchOnWindowFocus` включен;
- после category/event mutations инвалидируются read-side ключи `categories`, `events`, `dashboard`, `calendar-events`, `reports`;
- dashboard, calendar и reports больше не висят в браузере как “свежие” 30 секунд после write-side изменений.

Это не отменяет eventual consistency проекций, но убирает лишний stale-слой на клиенте.

## Локальный запуск

Для полноценной разработки сначала подними backend и инфраструктуру:

```bash
docker compose up -d db redis nats minio
cd ../backend
cargo run -p db-migrator
cargo run -p infra-bootstrap
cargo run -p identity-svc
cargo run -p event-command-svc
cargo run -p event-query-svc
cargo run -p report-svc
cargo run -p worker
cargo run -p edge-api
```

После этого:

```bash
npm run dev
```

Vite dev server проксирует `/api` на `VITE_EDGE_API_ORIGIN`, поэтому cookie-based auth flow сохраняется и локально.
