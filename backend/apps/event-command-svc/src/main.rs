mod app_state;
mod config;
mod error;
mod models;
mod repository;
mod validation;

use std::sync::Arc;

use chrono::{DateTime, Utc};
use contracts::event_command::event_command_service_server::{
    EventCommandService, EventCommandServiceServer,
};
use contracts::event_command::{
    Category as GrpcCategory, CategoryReply, CreateCategoryRequest, CreateEventRequest,
    DeleteCategoryRequest, DeleteEventRequest, Empty, EventRecord as GrpcEventRecord, EventReply,
    UpdateCategoryRequest, UpdateEventRequest,
};
use persistence::connect_pool;
use sqlx::{PgPool, Postgres, Transaction};
use tonic::{Request, Response, Status, transport::Server};
use uuid::Uuid;

use crate::{
    app_state::AppState,
    config::Config,
    error::{AppError, is_constraint},
    models::{
        Category, CategoryEventPayload, EventPayload, EventRecord, EventStatusChangedPayload,
    },
    repository::EventMutation,
};

#[derive(Clone)]
struct EventCommandGrpcService {
    state: AppState,
}

#[tonic::async_trait]
impl EventCommandService for EventCommandGrpcService {
    async fn create_category(
        &self,
        request: Request<CreateCategoryRequest>,
    ) -> Result<Response<CategoryReply>, Status> {
        let span = observability::grpc_request_span("event_command.create_category", &request);
        tracing::info!(parent: &span, "grpc request received");
        let user_id = parse_uuid(&request.get_ref().user_id, "user_id")?;
        let category = create_category(
            &self.state.pool,
            user_id,
            &request.get_ref().name,
            &request.get_ref().color,
        )
        .await
        .map_err(status_from_error)?;

        Ok(Response::new(CategoryReply {
            category: Some(map_category(category)),
        }))
    }

    async fn update_category(
        &self,
        request: Request<UpdateCategoryRequest>,
    ) -> Result<Response<CategoryReply>, Status> {
        let span = observability::grpc_request_span("event_command.update_category", &request);
        tracing::info!(parent: &span, "grpc request received");
        let user_id = parse_uuid(&request.get_ref().user_id, "user_id")?;
        let category_id = parse_uuid(&request.get_ref().category_id, "category_id")?;
        let category = update_category(
            &self.state.pool,
            user_id,
            category_id,
            &request.get_ref().name,
            &request.get_ref().color,
        )
        .await
        .map_err(status_from_error)?;

        Ok(Response::new(CategoryReply {
            category: Some(map_category(category)),
        }))
    }

    async fn delete_category(
        &self,
        request: Request<DeleteCategoryRequest>,
    ) -> Result<Response<Empty>, Status> {
        let span = observability::grpc_request_span("event_command.delete_category", &request);
        tracing::info!(parent: &span, "grpc request received");
        let user_id = parse_uuid(&request.get_ref().user_id, "user_id")?;
        let category_id = parse_uuid(&request.get_ref().category_id, "category_id")?;
        delete_category(&self.state.pool, user_id, category_id)
            .await
            .map_err(status_from_error)?;

        Ok(Response::new(Empty {}))
    }

    async fn create_event(
        &self,
        request: Request<CreateEventRequest>,
    ) -> Result<Response<EventReply>, Status> {
        let span = observability::grpc_request_span("event_command.create_event", &request);
        tracing::info!(parent: &span, "grpc request received");
        let user_id = parse_uuid(&request.get_ref().user_id, "user_id")?;
        let payload = build_event_mutation(request.get_ref())?;
        let event = create_event(&self.state.pool, user_id, payload)
            .await
            .map_err(status_from_error)?;

        Ok(Response::new(EventReply {
            event: Some(map_event(event)),
        }))
    }

    async fn update_event(
        &self,
        request: Request<UpdateEventRequest>,
    ) -> Result<Response<EventReply>, Status> {
        let span = observability::grpc_request_span("event_command.update_event", &request);
        tracing::info!(parent: &span, "grpc request received");
        let user_id = parse_uuid(&request.get_ref().user_id, "user_id")?;
        let event_id = parse_uuid(&request.get_ref().event_id, "event_id")?;
        let payload = build_event_mutation(request.get_ref())?;
        let event = update_event(&self.state.pool, user_id, event_id, payload)
            .await
            .map_err(status_from_error)?;

        Ok(Response::new(EventReply {
            event: Some(map_event(event)),
        }))
    }

    async fn delete_event(
        &self,
        request: Request<DeleteEventRequest>,
    ) -> Result<Response<Empty>, Status> {
        let span = observability::grpc_request_span("event_command.delete_event", &request);
        tracing::info!(parent: &span, "grpc request received");
        let user_id = parse_uuid(&request.get_ref().user_id, "user_id")?;
        let event_id = parse_uuid(&request.get_ref().event_id, "event_id")?;
        delete_event(&self.state.pool, user_id, event_id)
            .await
            .map_err(status_from_error)?;

        Ok(Response::new(Empty {}))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    observability::init_tracing("event-command-svc", "event_command_svc=info");

    let config = Arc::new(Config::from_env()?);
    observability::spawn_metrics_server("event-command-svc", config.metrics_port);
    let pool = connect_pool(&config.database_url, 10).await?;
    let state = AppState::new(pool, config.clone());
    let address = format!("0.0.0.0:{}", config.grpc_port).parse()?;

    tracing::info!("event-command-svc listening on {address}");

    Server::builder()
        .add_service(EventCommandServiceServer::new(EventCommandGrpcService {
            state,
        }))
        .serve(address)
        .await?;

    Ok(())
}

async fn create_category(
    pool: &PgPool,
    user_id: Uuid,
    name: &str,
    color: &str,
) -> Result<Category, AppError> {
    validation::validate_category(name, color)?;

    let mut tx = pool.begin().await?;
    let created = repository::create_category(&mut tx, user_id, name.trim(), color.trim())
        .await
        .map_err(|error| {
            if is_constraint(&error, "categories_user_name_unique") {
                AppError::Conflict("Category name is already in use".to_string())
            } else {
                AppError::from(error)
            }
        })?;

    let payload = serde_json::to_value(CategoryEventPayload {
        user_id: created.user_id,
        category_id: created.id,
        name: created.name.clone(),
        color: created.color.clone(),
        created_at: created.created_at,
        updated_at: created.updated_at,
    })
    .map_err(|error| AppError::Internal(error.to_string()))?;
    repository::insert_outbox_event(&mut tx, "category", created.id, "category.created", payload)
        .await?;
    tx.commit().await?;

    Ok(created)
}

async fn update_category(
    pool: &PgPool,
    user_id: Uuid,
    category_id: Uuid,
    name: &str,
    color: &str,
) -> Result<Category, AppError> {
    validation::validate_category(name, color)?;

    let mut tx = pool.begin().await?;
    let updated =
        repository::update_category(&mut tx, user_id, category_id, name.trim(), color.trim())
            .await
            .map_err(|error| {
                if is_constraint(&error, "categories_user_name_unique") {
                    AppError::Conflict("Category name is already in use".to_string())
                } else {
                    AppError::from(error)
                }
            })?
            .ok_or_else(|| AppError::NotFound("Category not found".to_string()))?;

    let payload = serde_json::to_value(CategoryEventPayload {
        user_id: updated.user_id,
        category_id: updated.id,
        name: updated.name.clone(),
        color: updated.color.clone(),
        created_at: updated.created_at,
        updated_at: updated.updated_at,
    })
    .map_err(|error| AppError::Internal(error.to_string()))?;
    repository::insert_outbox_event(&mut tx, "category", updated.id, "category.updated", payload)
        .await?;
    tx.commit().await?;

    Ok(updated)
}

async fn delete_category(pool: &PgPool, user_id: Uuid, category_id: Uuid) -> Result<(), AppError> {
    let mut tx = pool.begin().await?;
    let existing = repository::find_category_by_id(&mut tx, user_id, category_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Category not found".to_string()))?;

    let events_count = repository::count_events_for_category(&mut tx, user_id, category_id).await?;
    if events_count > 0 {
        return Err(AppError::Conflict(
            "Category cannot be deleted while events still use it".to_string(),
        ));
    }

    let deleted = repository::delete_category(&mut tx, user_id, category_id).await?;
    if deleted == 0 {
        return Err(AppError::NotFound("Category not found".to_string()));
    }

    let payload = serde_json::to_value(CategoryEventPayload {
        user_id: existing.user_id,
        category_id: existing.id,
        name: existing.name,
        color: existing.color,
        created_at: existing.created_at,
        updated_at: Utc::now(),
    })
    .map_err(|error| AppError::Internal(error.to_string()))?;
    repository::insert_outbox_event(
        &mut tx,
        "category",
        category_id,
        "category.deleted",
        payload,
    )
    .await?;
    tx.commit().await?;

    Ok(())
}

async fn create_event(
    pool: &PgPool,
    user_id: Uuid,
    payload: EventMutation<'_>,
) -> Result<EventRecord, AppError> {
    validation::validate_status(payload.status)?;
    validation::validate_event(
        payload.title,
        payload.location,
        payload.starts_at,
        payload.ends_at,
        payload.budget,
    )?;

    let mut tx = pool.begin().await?;
    ensure_category_belongs_to_user(&mut tx, user_id, payload.category_id).await?;

    let created = repository::create_event(&mut tx, user_id, &payload).await?;
    let event_payload = serde_json::to_value(snapshot_event_payload(&created))
        .map_err(|error| AppError::Internal(error.to_string()))?;
    repository::insert_outbox_event(&mut tx, "event", created.id, "event.created", event_payload)
        .await?;
    tx.commit().await?;

    Ok(created)
}

async fn update_event(
    pool: &PgPool,
    user_id: Uuid,
    event_id: Uuid,
    payload: EventMutation<'_>,
) -> Result<EventRecord, AppError> {
    validation::validate_status(payload.status)?;
    validation::validate_event(
        payload.title,
        payload.location,
        payload.starts_at,
        payload.ends_at,
        payload.budget,
    )?;

    let mut tx = pool.begin().await?;
    ensure_category_belongs_to_user(&mut tx, user_id, payload.category_id).await?;
    let existing = repository::find_event_by_id(&mut tx, user_id, event_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Event not found".to_string()))?;
    validation::validate_transition(&existing.status, payload.status)?;

    let updated = repository::update_event(&mut tx, user_id, event_id, &payload)
        .await?
        .ok_or_else(|| AppError::NotFound("Event not found".to_string()))?;

    let event_payload = serde_json::to_value(snapshot_event_payload(&updated))
        .map_err(|error| AppError::Internal(error.to_string()))?;
    repository::insert_outbox_event(&mut tx, "event", updated.id, "event.updated", event_payload)
        .await?;

    if existing.status != updated.status {
        let status_payload = serde_json::to_value(EventStatusChangedPayload {
            user_id: updated.user_id,
            event_id: updated.id,
            title: updated.title.clone(),
            previous_status: existing.status,
            new_status: updated.status.clone(),
            occurred_at: updated.updated_at,
        })
        .map_err(|error| AppError::Internal(error.to_string()))?;
        repository::insert_outbox_event(
            &mut tx,
            "event",
            updated.id,
            "event.status_changed",
            status_payload,
        )
        .await?;
    }

    tx.commit().await?;

    Ok(updated)
}

async fn delete_event(pool: &PgPool, user_id: Uuid, event_id: Uuid) -> Result<(), AppError> {
    let mut tx = pool.begin().await?;
    let existing = repository::find_event_by_id(&mut tx, user_id, event_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Event not found".to_string()))?;

    let deleted = repository::delete_event(&mut tx, user_id, event_id).await?;
    if deleted == 0 {
        return Err(AppError::NotFound("Event not found".to_string()));
    }

    let payload = serde_json::to_value(snapshot_event_payload(&existing))
        .map_err(|error| AppError::Internal(error.to_string()))?;
    repository::insert_outbox_event(&mut tx, "event", existing.id, "event.deleted", payload)
        .await?;
    tx.commit().await?;

    Ok(())
}

async fn ensure_category_belongs_to_user(
    tx: &mut Transaction<'_, Postgres>,
    user_id: Uuid,
    category_id: Uuid,
) -> Result<(), AppError> {
    repository::find_category_by_id(tx, user_id, category_id)
        .await?
        .ok_or_else(|| {
            AppError::BadRequest("Category does not belong to the current user".to_string())
        })?;

    Ok(())
}

fn snapshot_event_payload(event: &EventRecord) -> EventPayload {
    EventPayload {
        user_id: event.user_id,
        event_id: event.id,
        category_id: event.category_id,
        title: event.title.clone(),
        description: event.description.clone(),
        location: event.location.clone(),
        starts_at: event.starts_at,
        ends_at: event.ends_at,
        budget: event.budget,
        status: event.status.clone(),
        created_at: event.created_at,
        updated_at: event.updated_at,
    }
}

#[allow(clippy::result_large_err)]
fn build_event_mutation<'a, T>(request: &'a T) -> Result<EventMutation<'a>, Status>
where
    T: EventMutationRequest,
{
    Ok(EventMutation {
        category_id: parse_uuid(request.category_id(), "category_id")?,
        title: request.title().trim(),
        description: request.description().trim(),
        location: request.location().trim(),
        starts_at: parse_datetime(request.starts_at(), "starts_at")?,
        ends_at: parse_datetime(request.ends_at(), "ends_at")?,
        budget: request.budget(),
        status: request.status().trim(),
    })
}

trait EventMutationRequest {
    fn category_id(&self) -> &str;
    fn title(&self) -> &str;
    fn description(&self) -> &str;
    fn location(&self) -> &str;
    fn starts_at(&self) -> &str;
    fn ends_at(&self) -> &str;
    fn budget(&self) -> f64;
    fn status(&self) -> &str;
}

impl EventMutationRequest for CreateEventRequest {
    fn category_id(&self) -> &str {
        &self.category_id
    }

    fn title(&self) -> &str {
        &self.title
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn location(&self) -> &str {
        &self.location
    }

    fn starts_at(&self) -> &str {
        &self.starts_at
    }

    fn ends_at(&self) -> &str {
        &self.ends_at
    }

    fn budget(&self) -> f64 {
        self.budget
    }

    fn status(&self) -> &str {
        &self.status
    }
}

impl EventMutationRequest for UpdateEventRequest {
    fn category_id(&self) -> &str {
        &self.category_id
    }

    fn title(&self) -> &str {
        &self.title
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn location(&self) -> &str {
        &self.location
    }

    fn starts_at(&self) -> &str {
        &self.starts_at
    }

    fn ends_at(&self) -> &str {
        &self.ends_at
    }

    fn budget(&self) -> f64 {
        self.budget
    }

    fn status(&self) -> &str {
        &self.status
    }
}

fn map_category(category: Category) -> GrpcCategory {
    GrpcCategory {
        id: category.id.to_string(),
        user_id: category.user_id.to_string(),
        name: category.name,
        color: category.color,
        created_at: category.created_at.to_rfc3339(),
        updated_at: category.updated_at.to_rfc3339(),
    }
}

fn map_event(event: EventRecord) -> GrpcEventRecord {
    GrpcEventRecord {
        id: event.id.to_string(),
        user_id: event.user_id.to_string(),
        category_id: event.category_id.to_string(),
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

#[allow(clippy::result_large_err)]
fn parse_uuid(value: &str, field: &str) -> Result<Uuid, Status> {
    Uuid::parse_str(value).map_err(|_| Status::invalid_argument(format!("Invalid {field}")))
}

#[allow(clippy::result_large_err)]
fn parse_datetime(value: &str, field: &str) -> Result<DateTime<Utc>, Status> {
    DateTime::parse_from_rfc3339(value)
        .map(|timestamp| timestamp.with_timezone(&Utc))
        .map_err(|_| Status::invalid_argument(format!("Invalid {field} timestamp")))
}

fn status_from_error(error: AppError) -> Status {
    match error {
        AppError::BadRequest(message) => Status::invalid_argument(message),
        AppError::NotFound(message) => Status::not_found(message),
        AppError::Conflict(message) => Status::already_exists(message),
        AppError::Config(message) | AppError::Internal(message) => Status::internal(message),
        AppError::Database(inner) => {
            tracing::error!("event-command database error: {}", inner);
            Status::internal("Database operation failed")
        }
    }
}

#[cfg(test)]
mod tests {
    use sqlx::PgPool;

    use super::*;

    fn event_mutation<'a>(category_id: Uuid, title: &'a str, status: &'a str) -> EventMutation<'a> {
        EventMutation {
            category_id,
            title,
            description: "demo",
            location: "Room 301",
            starts_at: DateTime::parse_from_rfc3339("2026-03-15T10:00:00Z")
                .unwrap()
                .with_timezone(&Utc),
            ends_at: DateTime::parse_from_rfc3339("2026-03-15T12:00:00Z")
                .unwrap()
                .with_timezone(&Utc),
            budget: 850.0,
            status,
        }
    }

    async fn insert_user(pool: &PgPool, email: &str) -> Uuid {
        let user_id = Uuid::new_v4();
        sqlx::query(
            r#"
            INSERT INTO users (id, email, password_hash, full_name)
            VALUES ($1, $2, 'hash', 'Demo User')
            "#,
        )
        .bind(user_id)
        .bind(email)
        .execute(pool)
        .await
        .unwrap();
        sqlx::query(
            r#"
            INSERT INTO ui_settings (user_id, theme)
            VALUES ($1, 'system')
            "#,
        )
        .bind(user_id)
        .execute(pool)
        .await
        .unwrap();
        user_id
    }

    #[sqlx::test(migrations = "../../migrations")]
    async fn category_lifecycle_enforces_ownership(pool: PgPool) {
        let owner_id = insert_user(&pool, "owner@eventdesign.local").await;
        let other_id = insert_user(&pool, "other@eventdesign.local").await;

        let category = create_category(&pool, owner_id, "Conference", "#0f766e")
            .await
            .expect("category created");
        let updated = update_category(&pool, owner_id, category.id, "Defense", "#2563eb")
            .await
            .expect("owner can update category");
        assert_eq!(updated.name, "Defense");

        let ownership_error = update_category(&pool, other_id, category.id, "Stolen", "#111111")
            .await
            .expect_err("other users cannot update the category");
        assert!(matches!(ownership_error, AppError::NotFound(_)));

        delete_category(&pool, owner_id, category.id)
            .await
            .expect("owner can delete category");
    }

    #[sqlx::test(migrations = "../../migrations")]
    async fn event_crud_enforces_ownership(pool: PgPool) {
        let owner_id = insert_user(&pool, "events-owner@eventdesign.local").await;
        let other_id = insert_user(&pool, "events-other@eventdesign.local").await;
        let category = create_category(&pool, owner_id, "Conference", "#0f766e")
            .await
            .expect("category created");

        let event = create_event(
            &pool,
            owner_id,
            event_mutation(category.id, "Defense rehearsal", "planned"),
        )
        .await
        .expect("event created");
        let updated = update_event(
            &pool,
            owner_id,
            event.id,
            event_mutation(category.id, "Defense rehearsal", "in_progress"),
        )
        .await
        .expect("owner can update event");
        assert_eq!(updated.status, "in_progress");

        let ownership_error = update_event(
            &pool,
            other_id,
            event.id,
            event_mutation(category.id, "Defense rehearsal", "completed"),
        )
        .await
        .expect_err("other users cannot mutate the event");
        assert!(matches!(
            ownership_error,
            AppError::BadRequest(_) | AppError::NotFound(_)
        ));

        delete_event(&pool, owner_id, event.id)
            .await
            .expect("owner can delete event");
        let deleted =
            repository::find_event_by_id(&mut pool.begin().await.unwrap(), owner_id, event.id)
                .await
                .unwrap();
        assert!(deleted.is_none());
    }

    #[sqlx::test(migrations = "../../migrations")]
    async fn event_crud_writes_expected_outbox_events(pool: PgPool) {
        let user_id = insert_user(&pool, "outbox@eventdesign.local").await;
        let category = create_category(&pool, user_id, "Conference", "#0f766e")
            .await
            .expect("category created");

        let event = create_event(
            &pool,
            user_id,
            event_mutation(category.id, "Defense rehearsal", "planned"),
        )
        .await
        .expect("event created");
        update_event(
            &pool,
            user_id,
            event.id,
            event_mutation(category.id, "Defense rehearsal", "in_progress"),
        )
        .await
        .expect("event updated");
        delete_event(&pool, user_id, event.id)
            .await
            .expect("event deleted");

        let created_count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM outbox_events WHERE aggregate_id = $1 AND event_type = 'event.created'",
        )
        .bind(event.id)
        .fetch_one(&pool)
        .await
        .unwrap();
        let updated_count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM outbox_events WHERE aggregate_id = $1 AND event_type = 'event.updated'",
        )
        .bind(event.id)
        .fetch_one(&pool)
        .await
        .unwrap();
        let status_changed_count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM outbox_events WHERE aggregate_id = $1 AND event_type = 'event.status_changed'",
        )
        .bind(event.id)
        .fetch_one(&pool)
        .await
        .unwrap();
        let deleted_count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM outbox_events WHERE aggregate_id = $1 AND event_type = 'event.deleted'",
        )
        .bind(event.id)
        .fetch_one(&pool)
        .await
        .unwrap();

        assert_eq!(created_count, 1);
        assert_eq!(updated_count, 1);
        assert_eq!(status_changed_count, 1);
        assert_eq!(deleted_count, 1);
    }

    #[sqlx::test(migrations = "../../migrations")]
    async fn event_created_outbox_payload_matches_projection_contract(pool: PgPool) {
        let user_id = insert_user(&pool, "projector@eventdesign.local").await;
        let category = create_category(&pool, user_id, "Conference", "#0f766e")
            .await
            .expect("category created");

        let event = create_event(
            &pool,
            user_id,
            event_mutation(category.id, "Projection payload", "planned"),
        )
        .await
        .expect("event created");

        let payload = sqlx::query_scalar::<_, serde_json::Value>(
            r#"
            SELECT payload_json
            FROM outbox_events
            WHERE aggregate_id = $1 AND event_type = 'event.created'
            ORDER BY occurred_at DESC
            LIMIT 1
            "#,
        )
        .bind(event.id)
        .fetch_one(&pool)
        .await
        .unwrap();

        let user_id_text = user_id.to_string();
        let event_id_text = event.id.to_string();
        let category_id_text = category.id.to_string();

        assert_eq!(payload["user_id"].as_str(), Some(user_id_text.as_str()));
        assert_eq!(payload["event_id"].as_str(), Some(event_id_text.as_str()));
        assert_eq!(
            payload["category_id"].as_str(),
            Some(category_id_text.as_str())
        );
        assert_eq!(payload["title"].as_str(), Some("Projection payload"));
        assert_eq!(payload["status"].as_str(), Some("planned"));
        assert!(payload["updated_at"].as_str().is_some());
    }
}
