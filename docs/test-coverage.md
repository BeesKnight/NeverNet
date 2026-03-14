# Покрытие тестами

## Назначение

Этот документ фиксирует текущее состояние тестового покрытия и список зон, которые были дополнительно закрыты в рамках задачи на достижение не менее 70% покрытия проекта.

Метрика считается по `line coverage`.
Для backend и frontend используются разные инструменты, поэтому итоговый процент считается как взвешенная сумма по числу строк:

- backend: `cargo llvm-cov`
- frontend: `vitest --coverage`
- итог по проекту: `(covered_backend + covered_frontend) / (total_backend + total_frontend)`

## Текущее состояние

Результаты последнего прогона:

- backend: `65.42%` lines, `5307 / 8112`
- frontend: `89.32%` lines, `1766 / 1977`
- суммарно по проекту: `70.10%` lines, `7073 / 10089`

## Что было добавлено

### Frontend

Добавлены и/или расширены тесты для следующих зон:

- `frontend/src/api/client.test.ts`
- `frontend/src/api/query-utils.test.ts`
- `frontend/src/app/App.test.tsx`
- `frontend/src/app/router.test.tsx`
- `frontend/src/main.test.tsx`
- `frontend/src/features/auth/auth-context.test.tsx`
- `frontend/src/features/categories/CategoryForm.test.tsx`
- `frontend/src/pages/LoginPage.test.tsx`
- `frontend/src/pages/RegisterPage.test.tsx`
- `frontend/src/pages/SettingsPage.test.tsx`
- `frontend/src/pages/CategoriesPage.test.tsx`
- `frontend/src/pages/DashboardPage.test.tsx`
- `frontend/src/pages/CalendarPage.test.tsx`
- `frontend/src/pages/EventsPage.test.tsx`
- `frontend/src/pages/ReportsPage.test.tsx`

Покрытые сценарии:

- работа API-клиента и обработка ошибок;
- invalidation read-side query keys;
- bootstrap приложения в `main.tsx`;
- синхронизация темы и accent color в `App.tsx`;
- auth context;
- формы login/register/category;
- основные пользовательские страницы dashboard/calendar/events/reports/categories/settings;
- роутинг для гостя и авторизованного пользователя.

### Backend

Добавлены тесты для следующих модулей и сервисов:

- `backend/apps/demo-seed/src/main.rs`
- `backend/apps/edge-api/src/app_state.rs`
- `backend/apps/edge-api/src/auth/handlers.rs`
- `backend/apps/edge-api/src/main.rs`
- `backend/apps/edge-api/src/shared/api.rs`
- `backend/apps/edge-api/src/shared/auth.rs`
- `backend/apps/edge-api/src/shared/grpc.rs`
- `backend/apps/edge-api/src/shared/http.rs`
- `backend/apps/edge-api/src/shared/request_context.rs`
- `backend/apps/event-query-svc/src/repository.rs`
- `backend/apps/identity-svc/src/settings/repository.rs`
- `backend/apps/identity-svc/src/settings/service.rs`
- `backend/apps/infra-bootstrap/src/main.rs`
- `backend/apps/worker/src/main.rs`
- `backend/crates/persistence/src/lib.rs`

Также ранее были добавлены тесты в уже покрытые в этой ветке файлы:

- `backend/crates/cache/src/lib.rs`
- `backend/crates/messaging/src/lib.rs`
- `backend/crates/observability/src/lib.rs`
- `backend/apps/event-query-svc/src/config.rs`
- `backend/apps/event-query-svc/src/main.rs`
- `backend/apps/event-query-svc/src/models.rs`
- `backend/apps/report-svc/src/config.rs`
- `backend/apps/report-svc/src/main.rs`
- `backend/apps/identity-svc/src/config.rs`
- `backend/apps/identity-svc/src/main.rs`
- `backend/apps/event-command-svc/src/config.rs`
- `backend/apps/edge-api/src/error.rs`
- `backend/apps/edge-api/src/auth/service.rs`
- `backend/apps/edge-api/src/categories/service.rs`
- `backend/apps/edge-api/src/events/service.rs`
- `backend/apps/edge-api/src/dashboard/service.rs`
- `backend/apps/edge-api/src/settings/service.rs`
- `backend/apps/edge-api/src/exports/service.rs`
- `backend/apps/edge-api/src/reports/service.rs`
- `backend/apps/edge-api/src/calendar/service.rs`

Покрытые backend-сценарии:

- demo-seed helpers, генерация категорий/событий, refresh projection-таблиц;
- сборка `edge-api` router, health endpoints, CSRF middleware, request context, grpc request-id propagation;
- read-side repository-запросы в `event-query-svc`;
- settings read/write flow в `identity-svc`;
- bootstrap infra для NATS/MinIO по error-path и конфигурации;
- export/report helpers и lifecycle-ветки в `worker`;
- подключение к PostgreSQL в `persistence`.

## Команды проверки

Использовались следующие команды:

- `cargo test --workspace --all-features`
- `cargo +stable-x86_64-pc-windows-msvc llvm-cov --workspace --all-features --summary-only`
- `npm test -- --run`
- `npm exec vitest -- --coverage --run --coverage.reporter=text-summary --coverage.reporter=json-summary`
- `npm run typecheck`
- `npm run lint`
- `npm run build`

Дополнительно для локального backend test-run использовался `DATABASE_URL` на локальный PostgreSQL-инстанс.

## Ограничения

- backend coverage и frontend coverage собираются разными инструментами; итог по проекту рассчитывается отдельно по line counts;
- `infra-bootstrap` tests на retry-ветки занимают заметное время из-за контролируемых неуспешных подключений;
- часть handler-файлов backend всё ещё имеет низкое покрытие, но целевой суммарный порог уже достигнут.
