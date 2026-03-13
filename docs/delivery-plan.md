# План доведения проекта

## Цель плана

Этот документ фиксирует не абстрактное “что было бы круто”, а конкретный порядок работ, в котором EventDesign должен быть доведён до demo-ready состояния.

Работа разбита на 5 фаз.
Они должны выполняться последовательно.

---

## Фаза 1. Детерминированный запуск, миграции, bootstrap

### Цель

Убрать все причины, по которым стек поднимается нестабильно, падает на старте или зависит от удачного расположения планет.

### Что должно быть сделано

- healthchecks для `postgres`, `redis`, `nats`, `minio`;
- `depends_on` long syntax с `condition: service_healthy`;
- отдельный one-shot `db-migrator`;
- миграции применяются один раз и до старта прикладных сервисов;
- MinIO bucket bootstrap;
- JetStream bootstrap или явная инициализация stream/consumer;
- запуск `docker compose up --build -d` воспроизводим для нового разработчика.

### Критерий завершённости фазы

- стек стартует одной командой;
- сервисы не падают из-за отсутствия готовой инфраструктуры;
- нет гонок миграций;
- нет “иногда стартует, иногда нет”.

---

## Фаза 2. Event backbone, outbox, projections, идемпотентность

### Цель

Сделать async backbone настоящим, а не декоративным.

### Что должно быть сделано

- write-side мутации гарантированно создают outbox rows;
- outbox relay публикует в JetStream;
- JetStream stream/consumers создаются детерминированно;
- projector consumers обновляют projection-таблицы;
- используется `processed_messages` или аналогичный durable dedupe layer;
- ack происходит только после DB commit;
- read-side экраны реально читают projections;
- event create/update/delete отражаются на dashboard/calendar/reports.

### Критерий завершённости фазы

- command -> outbox -> JetStream -> projector -> read model работает end-to-end;
- duplicate delivery не ломает состояние;
- projections не расходятся с write-side state.

---

## Фаза 3. Безопасность, auth flow, export pipeline

### Цель

Закрыть сценарии, где всё “как будто работает”, но на деле ломается в браузере или пропускает чужие данные.

### Что должно быть сделано

- HttpOnly session cookie;
- Secure/SameSite policy для non-local окружений;
- корректный CORS allowlist;
- рабочий CSRF flow;
- ownership checks на events/categories/export jobs/settings;
- отсутствие object-level authorization leaks;
- export jobs проходят путь до реального downloadable artifact;
- report service выдаёт безопасный download flow или presigned URL;
- completed export jobs реально скачиваются.

### Критерий завершённости фазы

- frontend стабильно работает с `credentials: include`;
- не появляется случайных 401/403 без причины;
- пользователь не может получить доступ к чужим данным или экспортам;
- export pipeline работает до конца.

---

## Фаза 4. Зачистка сервисных границ, observability, docs, DX

### Цель

Сделать систему архитектурно аккуратной и операционно наблюдаемой.

### Что должно быть сделано

- убрать прямой domain SQL из Edge API;
- перевести чтение/запись соответствующих областей через внутренние сервисы;
- ввести request_id / trace_id propagation;
- добавить `TraceLayer` и structured logs;
- привести `/metrics` и `/healthz` к единому подходу;
- настроить базовые Prometheus + Grafana dashboards;
- нормализовать cache invalidation или TTL;
- обновить документацию;
- перевести все docs и README на русский язык;
- привести `.env.example`, runbook и demo-script к актуальному состоянию.

### Критерий завершённости фазы

- архитектура совпадает с документацией;
- Edge API не нарушает сервисные границы;
- request correlation работает;
- новый разработчик понимает, как поднять и проверить проект;
- документация на русском языке и соответствует реальности.

---

## Фаза 5. Тесты, smoke, финальная полировка под защиту

### Цель

Сделать так, чтобы проект был не просто “собран”, а реально проверяем и пригоден для защиты.

### Что должно быть сделано

- `scripts/smoke.sh`;
- backend integration tests для auth, ownership, projections, exports;
- frontend smoke / integration tests для login flow, route guards, events, exports;
- seed/demo dataset;
- финальный review checklist;
- проверка полного demo-flow;
- выравнивание README и стартовых инструкций;
- финальный прогон всех проверок.

### Критерий завершённости фазы

- smoke script проходит;
- базовые тесты проходят;
- demo-flow стабилен;
- проект можно показывать без ручных подпорок.

---

## Общий Definition of Done

Проект считается доведённым только когда одновременно выполняются все условия:

- `docker compose up --build -d` стабильно поднимает стек;
- миграции применяются один раз и в правильный момент;
- frontend работает только через Edge API;
- auth/cookie/CSRF/CORS flow стабилен;
- command changes обновляют read model через async backbone;
- duplicate delivery не портит projections;
- export jobs заканчиваются скачиваемыми файлами;
- ownership checks везде соблюдены;
- observability baseline есть;
- документация и README переведены на русский и актуальны;
- smoke/tests проходят;
- demo-script воспроизводим.
