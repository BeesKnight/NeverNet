# Внешний API

## Назначение

Этот документ описывает REST API, который Edge API / BFF отдаёт фронтенду.

Фронтенд общается только с этим API.
Он не должен вызывать внутренние сервисы напрямую.

Документ описывает внешний browser-facing контракт, а не внутренние gRPC контракты.

## Browser security-модель

Предполагается следующая модель безопасности:

- браузерная авторизация cookie-based;
- auth cookie называется `eventdesign_session`;
- браузер должен отправлять `credentials: include`;
- state-changing запросы должны отправлять CSRF token в `X-CSRF-Token`;
- допустимые frontend origins задаются явным allowlist, например `FRONTEND_ORIGINS`;
- ответы по возможности содержат `x-request-id` или аналогичный correlation id.

В нормальном browser flow нельзя использовать `localStorage` bearer tokens.

## Общий формат ответов и ошибок

Успешные JSON-ответы должны иметь вид:

```json
{
  "data": {}
}
```

Ошибки должны иметь единый envelope, например:

```json
{
  "error": {
    "code": "validation_error",
    "message": "Человекочитаемое описание ошибки",
    "request_id": "req_123"
  }
}
```

Если текущий код пока возвращает менее богатый формат, он должен быть постепенно приведён к единому виду.

## Основные коды ответа

- `200` успешное чтение
- `201` успешное создание
- `204` успешное удаление без тела
- `400` ошибка валидации или плохой запрос
- `401` неаутентифицированный пользователь или ошибка CSRF
- `403` недостаточно прав / ownership failure
- `404` ресурс не найден
- `409` конфликт
- `429` rate limited
- `500` внутренняя ошибка сервера

## Группы маршрутов

### Auth

#### GET `/api/auth/csrf`

Назначение:

- выдать CSRF token;
- выставить CSRF cookie, если это нужно текущей реализации.

Сценарий:
1. браузер вызывает endpoint перед login/register или перед первым state-changing запросом;
2. token затем отправляется в `X-CSRF-Token`.

#### POST `/api/auth/register`

Назначение:

- создать пользователя;
- создать активную сессию;
- выставить `eventdesign_session` cookie.

Требования:
- валидный CSRF token;
- email uniqueness;
- password hashing;
- возврат текущего пользователя в `data`.

#### POST `/api/auth/login`

Назначение:

- проверить credentials;
- создать durable session;
- выставить `eventdesign_session` cookie.

Требования:
- валидный CSRF token;
- ownership или role checks здесь не нужны, но нужна строгая проверка credentials;
- в ответе должен быть current user.

#### POST `/api/auth/logout`

Назначение:

- отозвать текущую сессию;
- очистить auth cookie.

Требования:
- валидный CSRF token;
- logout должен инвалидировать session row, а не только удалить cookie на клиенте.

#### GET `/api/auth/me`

Назначение:

- вернуть current authenticated user;
- служить bootstrap endpoint для фронтенда.

Требования:
- корректная обработка просроченной или revoked session;
- отсутствие лишних внутренних полей в DTO.

### Categories

#### GET `/api/categories`

Возвращает все категории текущего пользователя.

Требования:
- обязательная аутентификация;
- нельзя отдавать категории других пользователей.

#### POST `/api/categories`

Создаёт категорию текущего пользователя.

Требования:
- CSRF;
- user ownership определяется сервером, а не приходит из клиента;
- цвет и имя валидируются.

#### PATCH `/api/categories/:id`

Обновляет категорию.

Требования:
- CSRF;
- ownership check;
- нельзя позволять менять category другого пользователя.

#### DELETE `/api/categories/:id`

Удаляет категорию.

Требования:
- CSRF;
- ownership check;
- поведение для категории, используемой событиями, должно быть явно определено.

### Events

#### GET `/api/events`

Возвращает список событий текущего пользователя.

Поддерживаемые query-параметры:

- `status`
- `category_id`
- `search`
- `from`
- `to`
- `sort_by`
- `sort_order`

Требования:
- данные должны приходить из query-side / projections в целевой архитектуре;
- нельзя возвращать чужие события;
- сортировка и фильтрация должны быть предсказуемыми.

#### GET `/api/events/:id`

Возвращает одно событие текущего пользователя.

Требования:
- ownership check;
- консистентный DTO.

#### POST `/api/events`

Создаёт событие.

Требования:
- CSRF;
- category должна принадлежать тому же пользователю;
- `starts_at < ends_at`;
- write-side mutation должна создавать outbox event.

#### PATCH `/api/events/:id`

Обновляет событие.

Требования:
- CSRF;
- ownership check;
- status transitions должны быть валидны;
- mutation должна отражаться в projections через async backbone.

#### DELETE `/api/events/:id`

Удаляет событие.

Требования:
- CSRF;
- ownership check;
- удаление должно eventually отражаться на read-side.

### Dashboard

#### GET `/api/dashboard`

Возвращает summary cards и быстрые агрегаты текущего пользователя.

Требования:
- projection-backed reads;
- отсутствие stale data за пределами допустимого eventual consistency window;
- при наличии Redis-кэша должен существовать механизм invalidation или TTL.

### Calendar

#### GET `/api/calendar`

Возвращает данные календаря текущего пользователя.

Ожидаемые query-параметры:

- `year`
- `month`

Требования:
- projection-backed reads;
- ownership isolation;
- консистентное представление статусов и category color.

### Reports

#### GET `/api/reports/summary`

Возвращает preview summary для отчёта.

Ожидаемые фильтры:

- период;
- категория;
- статус;
- сортировка.

#### GET `/api/reports/by-category`

Возвращает breakdown по категориям.

Требования:
- query-side read;
- user scope isolation;
- корректный набор агрегатов.

### Exports

#### POST `/api/exports`

Создаёт export job.

Требования:
- CSRF;
- ownership определяется сервером;
- payload должен описывать report type, format и фильтры;
- ответ должен вернуть хотя бы `job_id` и начальный статус.

#### GET `/api/exports`

Возвращает список export jobs текущего пользователя.

Требования:
- ownership isolation;
- сортировка по времени / статусу по возможности;
- completed jobs должны содержать достаточно информации для дальнейшего download flow.

#### GET `/api/exports/:id`

Возвращает состояние одного export job.

Требования:
- ownership check;
- completed job должен содержать информацию, достаточную для скачивания.

#### POST `/api/exports/:id/download` или GET `/api/exports/:id/download`

Назначение:

- инициировать безопасную выдачу presigned URL или прямой проксируемый download.

Требования:
- ownership check;
- completed status;
- object должен реально существовать в MinIO;
- нельзя отдавать чужие артефакты.

### Settings

#### GET `/api/settings`

Возвращает UI settings текущего пользователя.

#### PATCH `/api/settings`

Обновляет UI settings текущего пользователя.

Требования:
- CSRF;
- ownership определяется сервером;
- настройки не должны требовать прямого SQL в Edge API в финальной архитектуре.

## Общие требования к API

Во всех user-scoped endpoints должны быть:

- authentication;
- ownership check;
- предсказуемый JSON response shape;
- request correlation id;
- внятные ошибки для фронтенда.

## Что не должно быть частью внешнего API

Фронтенд не должен видеть:

- внутренние gRPC DTO;
- внутренние service ids;
- outbox metadata;
- служебные технические поля, не нужные UI;
- прямой доступ к `/metrics` и служебным internal endpoints.

## Критерий завершённости внешнего API

Внешний API считается доведённым, когда:

- фронтенд закрывает все ключевые сценарии только через Edge API;
- browser auth стабильно работает с cookies и CSRF;
- ошибки унифицированы;
- ownership leaks отсутствуют;
- export/download flow проходит end-to-end.
