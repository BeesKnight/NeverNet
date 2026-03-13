mod app_state;
mod config;
mod error;
mod models;
mod repository;

use std::sync::Arc;

use cache::dashboard_key;
use chrono::{NaiveDate, Utc};
use contracts::event_query::event_query_service_server::{
    EventQueryService, EventQueryServiceServer,
};
use contracts::event_query::{
    ActivityItem, CalendarItem, CalendarReply, Category as GrpcCategory, DashboardCards,
    DashboardReply, EventItem as GrpcEventItem, EventReply, GetCalendarRequest,
    GetDashboardRequest, GetEventRequest, GetReportSummaryRequest, ListCategoriesReply,
    ListCategoriesRequest, ListEventsReply, ListEventsRequest, ReportCategoryRow, ReportStatusRow,
    ReportSummaryReply,
};
use persistence::connect_pool;
use redis::AsyncCommands;
use tonic::{Request, Response, Status, transport::Server};
use uuid::Uuid;

use crate::{
    app_state::AppState,
    config::Config,
    error::AppError,
    models::{
        ActivityRow, CalendarItemRow, CategoryRow, DashboardProjectionRow, DashboardSnapshot,
        EventFilters, EventItemRow, ReportCategoryRow as LocalReportCategoryRow,
        ReportStatusRow as LocalReportStatusRow, ReportSummary,
    },
};

#[derive(Clone)]
struct EventQueryGrpcService {
    state: AppState,
}

#[tonic::async_trait]
impl EventQueryService for EventQueryGrpcService {
    async fn list_categories(
        &self,
        request: Request<ListCategoriesRequest>,
    ) -> Result<Response<ListCategoriesReply>, Status> {
        let span = observability::grpc_request_span("event_query.list_categories", &request);
        tracing::info!(parent: &span, "grpc request received");
        let user_id = parse_uuid(&request.get_ref().user_id, "user_id")?;
        let items = repository::list_categories(&self.state.pool, user_id)
            .await
            .map_err(AppError::from)
            .map_err(status_from_error)?
            .into_iter()
            .map(map_category)
            .collect();

        Ok(Response::new(ListCategoriesReply { items }))
    }

    async fn list_events(
        &self,
        request: Request<ListEventsRequest>,
    ) -> Result<Response<ListEventsReply>, Status> {
        let span = observability::grpc_request_span("event_query.list_events", &request);
        tracing::info!(parent: &span, "grpc request received");
        let user_id = parse_uuid(&request.get_ref().user_id, "user_id")?;
        let filters = parse_event_filters(request.get_ref())?;
        let items = repository::list_events(&self.state.pool, user_id, &filters)
            .await
            .map_err(AppError::from)
            .map_err(status_from_error)?
            .into_iter()
            .map(map_event)
            .collect();

        Ok(Response::new(ListEventsReply { items }))
    }

    async fn get_event(
        &self,
        request: Request<GetEventRequest>,
    ) -> Result<Response<EventReply>, Status> {
        let span = observability::grpc_request_span("event_query.get_event", &request);
        tracing::info!(parent: &span, "grpc request received");
        let user_id = parse_uuid(&request.get_ref().user_id, "user_id")?;
        let event_id = parse_uuid(&request.get_ref().event_id, "event_id")?;
        let event = repository::get_event(&self.state.pool, user_id, event_id)
            .await
            .map_err(AppError::from)
            .map_err(status_from_error)?
            .ok_or_else(|| Status::not_found("Event not found"))?;

        Ok(Response::new(EventReply {
            event: Some(map_event(event)),
        }))
    }

    async fn get_calendar(
        &self,
        request: Request<GetCalendarRequest>,
    ) -> Result<Response<CalendarReply>, Status> {
        let span = observability::grpc_request_span("event_query.get_calendar", &request);
        tracing::info!(parent: &span, "grpc request received");
        let user_id = parse_uuid(&request.get_ref().user_id, "user_id")?;
        let start_date = parse_date(&request.get_ref().start_date, "start_date")?;
        let end_date = parse_date(&request.get_ref().end_date, "end_date")?;
        let items = repository::get_calendar(&self.state.pool, user_id, start_date, end_date)
            .await
            .map_err(AppError::from)
            .map_err(status_from_error)?
            .into_iter()
            .map(map_calendar_item)
            .collect();

        Ok(Response::new(CalendarReply { items }))
    }

    async fn get_dashboard(
        &self,
        request: Request<GetDashboardRequest>,
    ) -> Result<Response<DashboardReply>, Status> {
        let span = observability::grpc_request_span("event_query.get_dashboard", &request);
        tracing::info!(parent: &span, "grpc request received");
        let user_id = parse_uuid(&request.get_ref().user_id, "user_id")?;
        let snapshot = get_dashboard(&self.state, user_id)
            .await
            .map_err(status_from_error)?;

        Ok(Response::new(DashboardReply {
            cards: Some(map_dashboard_cards(snapshot.cards)),
            upcoming: snapshot.upcoming.into_iter().map(map_event).collect(),
            recent_activity: snapshot
                .recent_activity
                .into_iter()
                .map(map_activity)
                .collect(),
        }))
    }

    async fn get_report_summary(
        &self,
        request: Request<GetReportSummaryRequest>,
    ) -> Result<Response<ReportSummaryReply>, Status> {
        let span = observability::grpc_request_span("event_query.get_report_summary", &request);
        tracing::info!(parent: &span, "grpc request received");
        let user_id = parse_uuid(&request.get_ref().user_id, "user_id")?;
        let filters = parse_report_filters(request.get_ref())?;
        let summary = get_report_summary(&self.state, user_id, filters)
            .await
            .map_err(status_from_error)?;

        Ok(Response::new(ReportSummaryReply {
            period_start: summary
                .period_start
                .map(|value| value.to_string())
                .unwrap_or_default(),
            period_end: summary
                .period_end
                .map(|value| value.to_string())
                .unwrap_or_default(),
            total_events: summary.total_events as i64,
            total_budget: summary.total_budget,
            by_category: summary
                .by_category
                .into_iter()
                .map(map_report_category)
                .collect(),
            by_status: summary
                .by_status
                .into_iter()
                .map(map_report_status)
                .collect(),
            events: summary.events.into_iter().map(map_event).collect(),
        }))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    observability::init_tracing("event-query-svc", "event_query_svc=info");

    let config = Arc::new(Config::from_env()?);
    observability::spawn_metrics_server("event-query-svc", config.metrics_port);
    let pool = connect_pool(&config.database_url, 10).await?;
    let redis = redis::Client::open(config.redis_url.clone())?;
    let state = AppState::new(pool, redis, config.clone());
    let address = format!("0.0.0.0:{}", config.grpc_port).parse()?;

    tracing::info!("event-query-svc listening on {address}");

    Server::builder()
        .add_service(EventQueryServiceServer::new(EventQueryGrpcService {
            state,
        }))
        .serve(address)
        .await?;

    Ok(())
}

async fn get_dashboard(state: &AppState, user_id: Uuid) -> Result<DashboardSnapshot, AppError> {
    if let Some(snapshot) = read_dashboard_cache(state, user_id).await {
        observability::observe_cache_result("dashboard", "hit");
        return Ok(snapshot);
    }
    observability::observe_cache_result("dashboard", "miss");

    let cards = repository::get_dashboard_projection(&state.pool, user_id)
        .await?
        .unwrap_or(DashboardProjectionRow {
            user_id,
            total_events: 0,
            upcoming_events: 0,
            completed_events: 0,
            cancelled_events: 0,
            total_budget: 0.0,
            updated_at: Utc::now(),
        });
    let upcoming = repository::list_upcoming_events(&state.pool, user_id, 5).await?;
    let recent_activity = repository::list_recent_activity(&state.pool, user_id, 5).await?;

    let snapshot = DashboardSnapshot {
        cards,
        upcoming,
        recent_activity,
    };
    write_dashboard_cache(state, user_id, &snapshot).await;

    Ok(snapshot)
}

async fn get_report_summary(
    state: &AppState,
    user_id: Uuid,
    filters: EventFilters,
) -> Result<ReportSummary, AppError> {
    let rows = repository::list_report_rows(&state.pool, user_id, &filters).await?;
    Ok(ReportSummary::from_events(
        rows,
        filters.start_date,
        filters.end_date,
    ))
}

async fn read_dashboard_cache(state: &AppState, user_id: Uuid) -> Option<DashboardSnapshot> {
    let mut connection = state.redis.get_multiplexed_tokio_connection().await.ok()?;
    let payload: Option<String> = connection
        .get(dashboard_key(&user_id.to_string()))
        .await
        .ok()?;
    payload.and_then(|value| serde_json::from_str(&value).ok())
}

async fn write_dashboard_cache(state: &AppState, user_id: Uuid, snapshot: &DashboardSnapshot) {
    let payload = match serde_json::to_string(snapshot) {
        Ok(payload) => payload,
        Err(error) => {
            tracing::warn!("could not serialize dashboard cache payload: {}", error);
            return;
        }
    };

    match state.redis.get_multiplexed_tokio_connection().await {
        Ok(mut connection) => {
            let result: redis::RedisResult<()> = connection
                .set_ex(dashboard_key(&user_id.to_string()), payload, 60)
                .await;
            if let Err(error) = result {
                tracing::warn!("could not write dashboard cache: {}", error);
            }
        }
        Err(error) => tracing::warn!("could not connect to redis: {}", error),
    }
}

#[allow(clippy::result_large_err)]
fn parse_event_filters(request: &ListEventsRequest) -> Result<EventFilters, Status> {
    Ok(EventFilters {
        search: normalized(&request.search),
        status: normalized(&request.status),
        category_id: optional_uuid(&request.category_id, "category_id")?,
        start_date: optional_date(&request.start_date, "start_date")?,
        end_date: optional_date(&request.end_date, "end_date")?,
        sort_by: normalized(&request.sort_by),
        sort_dir: normalized(&request.sort_dir),
    })
}

#[allow(clippy::result_large_err)]
fn parse_report_filters(request: &GetReportSummaryRequest) -> Result<EventFilters, Status> {
    Ok(EventFilters {
        search: None,
        status: normalized(&request.status),
        category_id: optional_uuid(&request.category_id, "category_id")?,
        start_date: optional_date(&request.start_date, "start_date")?,
        end_date: optional_date(&request.end_date, "end_date")?,
        sort_by: normalized(&request.sort_by),
        sort_dir: normalized(&request.sort_dir),
    })
}

fn normalized(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

#[allow(clippy::result_large_err)]
fn parse_uuid(value: &str, field: &str) -> Result<Uuid, Status> {
    Uuid::parse_str(value).map_err(|_| Status::invalid_argument(format!("Invalid {field}")))
}

#[allow(clippy::result_large_err)]
fn optional_uuid(value: &str, field: &str) -> Result<Option<Uuid>, Status> {
    if value.trim().is_empty() {
        Ok(None)
    } else {
        parse_uuid(value, field).map(Some)
    }
}

#[allow(clippy::result_large_err)]
fn parse_date(value: &str, field: &str) -> Result<NaiveDate, Status> {
    NaiveDate::parse_from_str(value, "%Y-%m-%d")
        .map_err(|_| Status::invalid_argument(format!("Invalid {field}")))
}

#[allow(clippy::result_large_err)]
fn optional_date(value: &str, field: &str) -> Result<Option<NaiveDate>, Status> {
    if value.trim().is_empty() {
        Ok(None)
    } else {
        parse_date(value, field).map(Some)
    }
}

fn map_category(category: CategoryRow) -> GrpcCategory {
    GrpcCategory {
        id: category.id.to_string(),
        user_id: category.user_id.to_string(),
        name: category.name,
        color: category.color,
        created_at: category.created_at.to_rfc3339(),
        updated_at: category.updated_at.to_rfc3339(),
    }
}

fn map_event(event: EventItemRow) -> GrpcEventItem {
    GrpcEventItem {
        id: event.id.to_string(),
        user_id: event.user_id.to_string(),
        category_id: event.category_id.to_string(),
        category_name: event.category_name,
        category_color: event.category_color,
        title: event.title,
        description: event.description,
        location: event.location,
        starts_at: event.starts_at.to_rfc3339(),
        ends_at: event.ends_at.to_rfc3339(),
        budget: event.budget,
        status: event.status,
        created_at: event.created_at.to_rfc3339(),
        updated_at: event.updated_at.to_rfc3339(),
    }
}

fn map_calendar_item(item: CalendarItemRow) -> CalendarItem {
    CalendarItem {
        event_id: item.event_id.to_string(),
        title: item.title,
        date: item.date.to_string(),
        starts_at: item.starts_at.to_rfc3339(),
        ends_at: item.ends_at.to_rfc3339(),
        status: item.status,
        category_color: item.category_color,
    }
}

fn map_dashboard_cards(cards: DashboardProjectionRow) -> DashboardCards {
    DashboardCards {
        total_events: cards.total_events,
        upcoming_events: cards.upcoming_events,
        completed_events: cards.completed_events,
        cancelled_events: cards.cancelled_events,
        total_budget: cards.total_budget,
    }
}

fn map_activity(activity: ActivityRow) -> ActivityItem {
    ActivityItem {
        id: activity.id.to_string(),
        entity_type: activity.entity_type,
        entity_id: activity.entity_id.to_string(),
        action: activity.action,
        title: activity.title,
        occurred_at: activity.occurred_at.to_rfc3339(),
    }
}

fn map_report_category(row: LocalReportCategoryRow) -> ReportCategoryRow {
    ReportCategoryRow {
        category_id: row.category_id.to_string(),
        category_name: row.category_name,
        category_color: row.category_color,
        event_count: row.event_count,
        total_budget: row.total_budget,
    }
}

fn map_report_status(row: LocalReportStatusRow) -> ReportStatusRow {
    ReportStatusRow {
        status: row.status,
        event_count: row.event_count,
        total_budget: row.total_budget,
    }
}

fn status_from_error(error: AppError) -> Status {
    match error {
        AppError::Config(message) => Status::internal(message),
        AppError::Database(inner) => {
            tracing::error!("event-query database error: {}", inner);
            Status::internal("Database operation failed")
        }
    }
}
