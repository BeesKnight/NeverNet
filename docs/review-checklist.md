# Review checklist

## Назначение

Этот checklist нужен для self-review перед merge, перед релизом и перед защитой.

Идея простая:
не доверять ощущению “вроде всё норм”, а проверять систему по фиксированному списку.

---

## 1. Startup и инфраструктура

Проверить:

- [ ] `docker compose up --build -d` стартует стек без ручных костылей
- [ ] `postgres`, `redis`, `nats`, `minio` имеют healthchecks
- [ ] прикладные сервисы зависят от healthy infra, а не просто от запуска контейнера
- [ ] миграции применяются через отдельный migrator
- [ ] нет гонок миграций
- [ ] bucket в MinIO создаётся или гарантированно существует
- [ ] JetStream stream/consumer bootstrap выполняется детерминированно

## 2. Auth и browser security

Проверить:

- [ ] auth cookie HttpOnly
- [ ] Secure/SameSite настроены корректно для окружения
- [ ] frontend использует `credentials: include`
- [ ] CSRF flow стабилен
- [ ] CORS задан через allowlist, а не wildcard
- [ ] logout реально инвалидирует session row
- [ ] `GET /api/auth/me` корректно обрабатывает revoked/expired session

## 3. Ownership и object-level authorization

Проверить:

- [ ] пользователь не может читать чужие categories
- [ ] пользователь не может изменять чужие categories
- [ ] пользователь не может читать чужие events
- [ ] пользователь не может изменять чужие events
- [ ] пользователь не может видеть или скачивать чужие export jobs
- [ ] DTO не отдают лишние внутренние поля

## 4. Command / Query согласованность

Проверить:

- [ ] write-side операции идут через command-side boundary
- [ ] read-side страницы читают projections там, где это уже заявлено архитектурой
- [ ] Edge API не держит прямой domain SQL там, где это уже должно быть вынесено
- [ ] create/update/delete event eventually отражаются в dashboard/calendar/reports

## 5. Outbox и JetStream

Проверить:

- [ ] write-side mutation создаёт outbox row в той же транзакции
- [ ] relay публикует unpublished rows
- [ ] публикации не теряются молча
- [ ] consumer делает ack только после DB commit
- [ ] есть защита от duplicate delivery
- [ ] `processed_messages` или аналог работает
- [ ] projections не ломаются при redelivery

## 6. Exports

Проверить:

- [ ] export job создаётся корректно
- [ ] queued -> processing -> completed/failed переходы явные
- [ ] completed export действительно загружен в MinIO
- [ ] object_key валиден
- [ ] download flow защищён ownership-проверкой
- [ ] presigned URL или proxy-download реально работают

## 7. Cache и eventual consistency

Проверить:

- [ ] dashboard не отдаёт stale данные слишком долго
- [ ] calendar не показывает старое состояние после изменений
- [ ] есть TTL или invalidation strategy
- [ ] кэш не маскирует баги projections

## 8. Observability

Проверить:

- [ ] request_id генерируется на Edge
- [ ] request_id или trace metadata прокидывается через gRPC
- [ ] логи структурированы
- [ ] Prometheus видит сервисные метрики
- [ ] Grafana dashboard загружается
- [ ] `/metrics` не торчит наружу без причины
- [ ] есть хотя бы базовые сигналы по exports и projections

## 9. Документация

Проверить:

- [ ] `README.md` на русском языке
- [ ] `frontend/README.md` на русском языке
- [ ] `docs/*` на русском языке
- [ ] `architecture.md` соответствует реальности
- [ ] `runbook.md` соответствует реальному запуску
- [ ] `demo-script.md` соответствует текущему UI и флоу
- [ ] `risk-register.md` обновлён после крупных фиксов

## 10. Тесты и smoke

Проверить:

- [ ] есть `scripts/smoke.sh`
- [ ] smoke script проходит
- [ ] backend integration tests проходят
- [ ] frontend smoke/integration tests проходят
- [ ] `cargo fmt --check` проходит
- [ ] `cargo clippy --workspace --all-targets --all-features -- -D warnings` проходит
- [ ] `cargo test --workspace --all-features` проходит
- [ ] `npm run lint` проходит
- [ ] `npm run typecheck` проходит
- [ ] `npm run build` проходит
- [ ] `npm test` проходит

## 11. Финальный вопрос перед защитой

Если сейчас выключить эмоции и оставить только факты, можно ли сказать:

- [ ] проект поднимается одной командой
- [ ] ключевой happy-path проходит end-to-end
- [ ] архитектура совпадает с тем, что я рассказываю
- [ ] документация соответствует проекту
- [ ] я могу показать систему без импровизационного дебага

Если на что-то ответ “нет”, это и есть то, что ещё надо чинить.
