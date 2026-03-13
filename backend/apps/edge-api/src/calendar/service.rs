use uuid::Uuid;

use crate::{
    app_state::AppState,
    error::AppError,
    events::{
        models::{EventFilters, EventListItem},
        repository,
    },
};

pub async fn list(
    state: &AppState,
    user_id: Uuid,
    filters: EventFilters,
) -> Result<Vec<EventListItem>, AppError> {
    Ok(repository::list(&state.pool, user_id, &filters).await?)
}
