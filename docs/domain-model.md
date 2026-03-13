# Доменная модель EventDesign

## Краткое описание продукта

EventDesign — система для планирования мероприятий и управления событиями.

Пользователь управляет своими категориями и событиями, может фильтровать и сортировать списки, смотреть календарь, видеть dashboard summary, строить отчёты и экспортировать их в PDF/XLSX.

## Bounded contexts

### Identity

Отвечает за:

- users;
- sessions;
- authentication;
- authorization context.

### Event Management

Отвечает за:

- categories;
- events;
- event lifecycle;
- ownership;
- write-side business validation.

### Reporting And Exports

Отвечает за:

- report previews;
- aggregates;
- export jobs;
- generated artifacts;
- download authorization.

### Preferences

Отвечает за:

- UI theme;
- accent color;
- default view / default page;
- другие user-scoped interface settings.

## Основные сущности

### User

Поля:

- id
- email
- password_hash
- full_name
- created_at
- updated_at

Правила:

- email должен быть уникальным;
- пароль никогда не хранится в открытом виде;
- пользователь владеет категориями, событиями, сессиями, UI settings и export jobs.

### Session

Поля:

- id
- user_id
- created_at
- expires_at
- revoked_at
- user_agent
- ip_address

Правила:

- сессия принадлежит одному пользователю;
- сессия бывает активной или отозванной;
- browser auth cookie представляет session-bound identity;
- browser auth не использует bearer token в `localStorage` или `Authorization` header;
- session cookie должна быть `HttpOnly`, а для non-local origin также `Secure`;
- session и CSRF cookie используют явную `SameSite` policy;
- logout и принудительная инвалидизация должны отзывать session row.

### Category

Поля:

- id
- user_id
- name
- color
- created_at
- updated_at

Правила:

- категории user-scoped;
- ограничения имени могут быть глобальными по пользователю, если продукт это предполагает;
- ownership должен проверяться всегда;
- удаление категории, которая ещё используется событиями, должно быть явно запрещено или корректно обработано.

### Event

Поля:

- id
- user_id
- category_id
- title
- description
- location
- starts_at
- ends_at
- budget
- status
- created_at
- updated_at

Правила:

- событие принадлежит одному пользователю;
- событие принадлежит одной категории того же пользователя;
- `starts_at` должен быть раньше `ends_at`;
- status должен быть одним из допустимых значений;
- ownership должен проверяться всегда;
- write-side изменения обязаны генерировать domain events.

### UI Settings

Поля:

- user_id
- theme
- accent_color
- default_view
- created_at
- updated_at

Правила:

- на пользователя должна существовать одна запись настроек;
- настройки user-scoped;
- если строки нет, должны безопасно применяться значения по умолчанию.

### Export Job

Поля:

- id
- user_id
- report_type
- format
- status
- filters_json
- object_key
- content_type
- size_bytes (опционально, если метаданные размера сохраняются отдельно)
- error_message
- created_at
- started_at
- updated_at
- finished_at

Правила:

- export job принадлежит одному пользователю;
- файловая метаинформация не должна зависеть от локального пути внутри контейнера;
- как минимум должны поддерживаться статусы `queued`, `processing`, `completed`, `failed`;
- `completed` допустим только после успешной загрузки файла в MinIO/S3-compatible storage и заполнения `object_key` / `content_type`;
- completed-job обязан указывать на реальный объект в MinIO/S3-compatible storage;
- скачивание должно проверять ownership.

## Допустимые статусы события

Минимальный набор:

- planned
- in_progress
- completed
- cancelled

Рекомендуемая политика переходов:

- planned -> in_progress
- planned -> cancelled
- in_progress -> completed
- in_progress -> cancelled

Если код позволяет больше переходов, это должно быть явно задокументировано и сделано осознанно.

## Основные пользовательские возможности

Пользователь должен уметь:

- регистрироваться и входить;
- выходить, теряя активную сессию;
- управлять категориями;
- создавать и редактировать события;
- фильтровать и сортировать списки событий;
- смотреть календарь;
- видеть dashboard summaries;
- строить отчёты по периоду и категориям;
- экспортировать отчёты в PDF и XLSX;
- менять настройки интерфейса.

## Projection-backed read models

### event_list_projection

Назначение:

- страница списка событий;
- UI для сортировки и фильтрации.

Ожидаемые поля:

- event_id
- user_id
- category_id
- category_name
- category_color
- title
- description
- location
- starts_at
- ends_at
- budget
- status
- created_at
- updated_at

### calendar_projection

Назначение:

- отображение календаря по датам / месяцам.

Ожидаемые поля:

- event_id
- user_id
- date_bucket
- title
- starts_at
- ends_at
- status
- category_color
- updated_at

### dashboard_projection

Назначение:

- summary cards и dashboard widgets.

Ожидаемые поля:

- user_id
- total_events
- upcoming_events
- completed_events
- cancelled_events
- total_budget
- updated_at

### report_projection

Назначение:

- предпросмотр отчётов и база для export-запросов.

Ожидаемые поля:

- event_id
- user_id
- category_id
- category_name
- category_color
- title
- description
- location
- starts_at
- ends_at
- budget
- status
- created_at
- updated_at

### recent_activity_projection

Назначение:

- recent activity feed;
- audit-flavored UX, если реализовано.

Ожидаемые поля:

- id
- source_message_id
- user_id
- entity_type
- entity_id
- action
- title
- occurred_at
- created_at

## Инварианты, которые должны соблюдаться

Во всей системе должно оставаться истинным следующее:

- пользователь A не может читать или изменять категории пользователя B;
- пользователь A не может читать или изменять события пользователя B;
- пользователь A не может скачивать или смотреть export jobs пользователя B;
- событие не может ссылаться на категорию другого пользователя;
- projection rows должны eventually отражать authoritative write-side state;
- повторная доставка одного и того же события не должна портить projections;
- completed export job должен соответствовать реально скачиваемому артефакту.

## Зоны риска доменной модели

Доменная модель быстро деградирует, если:

- Edge API обходит внутренние сервисы и ходит в DB напрямую;
- projection updates неидемпотентны;
- export jobs двигаются по статусам без durable state transitions;
- DTO наружу отдают внутренние поля, которые не нужны фронтенду.

Это не абстрактные страшилки. Эти места надо проверять кодом и чинить там, где они есть.
