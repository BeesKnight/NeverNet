#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

API_BASE_URL="${API_BASE_URL:-http://localhost:8080/api}"
EDGE_HEALTH_URL="${EDGE_HEALTH_URL:-http://localhost:8080/healthz}"
FRONTEND_URL="${FRONTEND_URL:-http://localhost:3000}"
NATS_HEALTH_URL="${NATS_HEALTH_URL:-http://localhost:8222/healthz}"
MINIO_HEALTH_URL="${MINIO_HEALTH_URL:-http://localhost:9000/minio/health/live}"
POLL_ATTEMPTS="${POLL_ATTEMPTS:-45}"
POLL_INTERVAL_SECONDS="${POLL_INTERVAL_SECONDS:-2}"

log() {
  printf '[smoke] %s\n' "$*"
}

fail() {
  printf '[smoke] ERROR: %s\n' "$*" >&2
  exit 1
}

require_command() {
  local command_name="$1"
  command -v "$command_name" >/dev/null 2>&1 || fail "Требуется команда '$command_name'"
}

require_command curl

if command -v python3 >/dev/null 2>&1; then
  PYTHON_BIN="python3"
elif command -v python >/dev/null 2>&1; then
  PYTHON_BIN="python"
else
  fail "Требуется python3 или python"
fi

COOKIE_JAR="$(mktemp)"
BODY_FILE="$(mktemp)"
DOWNLOAD_FILE="$(mktemp)"

cleanup() {
  rm -f "$COOKIE_JAR" "$BODY_FILE" "$DOWNLOAD_FILE"
}

trap cleanup EXIT

json_get() {
  local expression="$1"
  JSON_FILE="$BODY_FILE" JSON_EXPR="$expression" "$PYTHON_BIN" - <<'PY'
import json
import os
import sys
from pathlib import Path

scope = {
    "data": json.loads(Path(os.environ["JSON_FILE"]).read_text(encoding="utf-8")),
    "env": dict(os.environ),
    "any": any,
    "all": all,
    "len": len,
    "sum": sum,
}

globals_scope = {"__builtins__": {}}
globals_scope.update(scope)

value = eval(os.environ["JSON_EXPR"], globals_scope, {})
if value is None:
    sys.exit(1)
if isinstance(value, bool):
    print("true" if value else "false")
elif isinstance(value, (dict, list)):
    print(json.dumps(value, ensure_ascii=False))
else:
    print(value)
PY
}

json_matches() {
  local expression="$1"
  JSON_FILE="$BODY_FILE" JSON_EXPR="$expression" "$PYTHON_BIN" - <<'PY'
import json
import os
import sys
from pathlib import Path

scope = {
    "data": json.loads(Path(os.environ["JSON_FILE"]).read_text(encoding="utf-8")),
    "env": dict(os.environ),
    "any": any,
    "all": all,
    "len": len,
    "sum": sum,
}

globals_scope = {"__builtins__": {}}
globals_scope.update(scope)

result = eval(os.environ["JSON_EXPR"], globals_scope, {})
sys.exit(0 if bool(result) else 1)
PY
}

http_status=""

api_request() {
  local method="$1"
  local url="$2"
  local payload="${3:-}"
  local csrf_token="${4:-}"
  local -a curl_args=(
    -sS
    -X "$method"
    -H "Accept: application/json"
    -c "$COOKIE_JAR"
    -b "$COOKIE_JAR"
    -o "$BODY_FILE"
    -w "%{http_code}"
  )

  if [[ -n "$csrf_token" ]]; then
    curl_args+=(-H "X-CSRF-Token: $csrf_token")
  fi

  if [[ -n "$payload" ]]; then
    curl_args+=(-H "Content-Type: application/json" --data "$payload")
  fi

  http_status="$(curl "${curl_args[@]}" "$url")"
}

expect_status() {
  local expected="$1"
  if [[ "$http_status" != "$expected" ]]; then
    printf '[smoke] Ответ %s от %s не совпал с ожидаемым %s\n' "$http_status" "$2" "$expected" >&2
    printf '[smoke] Тело ответа:\n' >&2
    cat "$BODY_FILE" >&2
    exit 1
  fi
}

check_public_endpoint() {
  local name="$1"
  local url="$2"
  log "Проверка доступности: $name"
  curl -fsS "$url" >/dev/null || fail "Недоступен endpoint $url"
}

check_nats_endpoint() {
  log "Проверка доступности: nats monitor"
  curl -fsS "$NATS_HEALTH_URL" >/dev/null && return 0
  curl -fsS "${NATS_HEALTH_URL%/healthz}/varz" >/dev/null && return 0
  fail "Недоступен NATS monitor endpoint"
}

poll_json_endpoint() {
  local description="$1"
  local url="$2"
  local expression="$3"

  for ((attempt = 1; attempt <= POLL_ATTEMPTS; attempt++)); do
    api_request "GET" "$url"
    if [[ "$http_status" == "200" ]] && json_matches "$expression"; then
      log "$description"
      return 0
    fi

    sleep "$POLL_INTERVAL_SECONDS"
  done

  printf '[smoke] Не дождались условия: %s\n' "$description" >&2
  printf '[smoke] Последний ответ (%s):\n' "$http_status" >&2
  cat "$BODY_FILE" >&2
  exit 1
}

download_file() {
  local url="$1"
  http_status="$(curl -sS -o "$DOWNLOAD_FILE" -w "%{http_code}" -c "$COOKIE_JAR" -b "$COOKIE_JAR" "$url")"
}

readarray -t EVENT_VALUES < <("$PYTHON_BIN" - <<'PY'
from datetime import datetime, timedelta, timezone

start = (datetime.now(timezone.utc) + timedelta(days=3)).replace(microsecond=0)
end = start + timedelta(hours=2)
calendar_start = (start.date() - timedelta(days=3)).isoformat()
calendar_end = (start.date() + timedelta(days=3)).isoformat()

print(start.isoformat().replace("+00:00", "Z"))
print(end.isoformat().replace("+00:00", "Z"))
print(calendar_start)
print(calendar_end)
PY
)

EVENT_START_AT="${EVENT_VALUES[0]}"
EVENT_END_AT="${EVENT_VALUES[1]}"
CALENDAR_START_DATE="${EVENT_VALUES[2]}"
CALENDAR_END_DATE="${EVENT_VALUES[3]}"

SMOKE_EMAIL="smoke-$(date +%s)-${RANDOM}@eventdesign.local"
SMOKE_PASSWORD="SmokePass123!"
SMOKE_FULL_NAME="Smoke Script"
CATEGORY_NAME="Smoke Category"
EVENT_TITLE="Smoke Event"
export SMOKE_EMAIL

log "Проверка публичных endpoint-ов и инфраструктуры"
check_public_endpoint "frontend" "$FRONTEND_URL"
check_public_endpoint "edge-api" "$EDGE_HEALTH_URL"
check_nats_endpoint
check_public_endpoint "minio" "$MINIO_HEALTH_URL"

log "Запрос CSRF token"
api_request "GET" "$API_BASE_URL/auth/csrf"
expect_status "200" "$API_BASE_URL/auth/csrf"
CSRF_TOKEN="$(json_get 'data["data"]["csrf_token"]')"
[[ -n "$CSRF_TOKEN" ]] || fail "CSRF token пустой"

log "Регистрация smoke-пользователя"
api_request \
  "POST" \
  "$API_BASE_URL/auth/register" \
  "{\"email\":\"$SMOKE_EMAIL\",\"password\":\"$SMOKE_PASSWORD\",\"full_name\":\"$SMOKE_FULL_NAME\"}" \
  "$CSRF_TOKEN"
expect_status "200" "$API_BASE_URL/auth/register"
USER_ID="$(json_get 'data["data"]["user"]["id"]')"
[[ -n "$USER_ID" ]] || fail "Не удалось получить user id"
export USER_ID

log "Проверка auth bootstrap"
api_request "GET" "$API_BASE_URL/auth/me"
expect_status "200" "$API_BASE_URL/auth/me"
json_matches 'data["data"]["user"]["id"] == env["USER_ID"]' || fail "auth bootstrap вернул другого пользователя"

log "Создание категории"
api_request \
  "POST" \
  "$API_BASE_URL/categories" \
  "{\"name\":\"$CATEGORY_NAME\",\"color\":\"#0f766e\"}" \
  "$CSRF_TOKEN"
expect_status "200" "$API_BASE_URL/categories"
CATEGORY_ID="$(json_get 'data["data"]["id"]')"
[[ -n "$CATEGORY_ID" ]] || fail "Не удалось получить category id"
export CATEGORY_ID

log "Создание события"
api_request \
  "POST" \
  "$API_BASE_URL/events" \
  "{\"category_id\":\"$CATEGORY_ID\",\"title\":\"$EVENT_TITLE\",\"description\":\"Smoke happy-path event\",\"location\":\"Room 301\",\"starts_at\":\"$EVENT_START_AT\",\"ends_at\":\"$EVENT_END_AT\",\"budget\":850,\"status\":\"planned\"}" \
  "$CSRF_TOKEN"
expect_status "200" "$API_BASE_URL/events"
EVENT_ID="$(json_get 'data["data"]["id"]')"
[[ -n "$EVENT_ID" ]] || fail "Не удалось получить event id"
export EVENT_ID

log "Ожидание появления события в projection-backed списке"
poll_json_endpoint \
  "Событие появилось в /events" \
  "$API_BASE_URL/events?category_id=$CATEGORY_ID&sort_by=starts_at&sort_dir=asc" \
  'any(item["id"] == env["EVENT_ID"] for item in data["data"])'

log "Ожидание обновления dashboard"
poll_json_endpoint \
  "Dashboard обновился" \
  "$API_BASE_URL/dashboard" \
  'data["data"]["cards"]["total_events"] >= 1 and any(item["id"] == env["EVENT_ID"] for item in data["data"]["upcoming"])'

log "Ожидание обновления calendar"
poll_json_endpoint \
  "Calendar обновился" \
  "$API_BASE_URL/calendar?start_date=$CALENDAR_START_DATE&end_date=$CALENDAR_END_DATE" \
  'any(item["event_id"] == env["EVENT_ID"] for item in data["data"])'

log "Создание export job"
api_request \
  "POST" \
  "$API_BASE_URL/exports" \
  "{\"report_type\":\"summary\",\"format\":\"pdf\",\"filters\":{\"category_id\":\"$CATEGORY_ID\",\"sort_by\":\"starts_at\",\"sort_dir\":\"asc\"}}" \
  "$CSRF_TOKEN"
expect_status "200" "$API_BASE_URL/exports"
EXPORT_ID="$(json_get 'data["data"]["id"]')"
[[ -n "$EXPORT_ID" ]] || fail "Не удалось получить export id"
export EXPORT_ID

log "Ожидание completed export"
poll_json_endpoint \
  "Export перешел в completed" \
  "$API_BASE_URL/exports/$EXPORT_ID" \
  'data["data"]["status"] == "completed" and data["data"]["object_key"]'

log "Проверка download export artifact"
download_file "$API_BASE_URL/exports/$EXPORT_ID/download"
if [[ "$http_status" != "200" ]]; then
  printf '[smoke] Download завершился кодом %s\n' "$http_status" >&2
  cat "$DOWNLOAD_FILE" >&2 || true
  exit 1
fi
if [[ ! -s "$DOWNLOAD_FILE" ]]; then
  fail "Скачанный export-файл пустой"
fi

log "Проверка logout"
api_request "POST" "$API_BASE_URL/auth/logout" "" "$CSRF_TOKEN"
expect_status "200" "$API_BASE_URL/auth/logout"

api_request "GET" "$API_BASE_URL/auth/me"
if [[ "$http_status" != "401" ]]; then
  printf '[smoke] После logout ожидался 401, получен %s\n' "$http_status" >&2
  cat "$BODY_FILE" >&2
  exit 1
fi

log "Smoke happy-path завершен успешно"
log "Пользователь: $SMOKE_EMAIL"
log "Категория: $CATEGORY_ID"
log "Событие: $EVENT_ID"
log "Экспорт: $EXPORT_ID"
