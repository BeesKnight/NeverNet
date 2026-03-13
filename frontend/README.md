# Frontend EventDesign

Фронтенд EventDesign построен на React, TypeScript и Vite.

## Команды

```bash
npm install
npm run dev
npm run lint
npm run build
```

## Переменные окружения

Скопируй `.env.example` в `.env`.

Базовые локальные значения:

```bash
VITE_API_BASE_URL=/api
VITE_EDGE_API_ORIGIN=http://localhost:8080
```

`VITE_API_BASE_URL=/api` нужен для Vite proxy и nginx runtime-конфига.
Браузерный клиент не должен обращаться к внутренним сервисам напрямую.

## Локальный запуск

Для полноценного локального запуска backend сначала нужно подготовить инфраструктуру и bootstrap-этапы:

```bash
docker compose up -d db redis nats minio
cd ../backend
cargo run -p db-migrator
cargo run -p infra-bootstrap
cargo run -p edge-api
```

Если нужен полный backend-стек без Docker для приложений, дополнительно подними:

```bash
cargo run -p identity-svc
cargo run -p event-command-svc
cargo run -p event-query-svc
cargo run -p report-svc
cargo run -p worker
```

После этого фронтенд можно запускать так:

```bash
npm run dev
```

Vite dev server проксирует `/api` на `VITE_EDGE_API_ORIGIN`, поэтому cookie-based auth flow работает без хранения токенов в `localStorage`.
