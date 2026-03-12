use uuid::Uuid;

use crate::{
    app_state::AppState,
    categories::{
        models::{Category, CreateCategoryRequest, UpdateCategoryRequest},
        repository, validation,
    },
    error::AppError,
};

pub async fn list(state: &AppState, user_id: Uuid) -> Result<Vec<Category>, AppError> {
    Ok(repository::list(&state.pool, user_id).await?)
}

pub async fn create(
    state: &AppState,
    user_id: Uuid,
    payload: CreateCategoryRequest,
) -> Result<Category, AppError> {
    validation::validate_category(&payload.name, &payload.color)?;
    Ok(repository::create(
        &state.pool,
        user_id,
        payload.name.trim(),
        payload.color.trim(),
    )
    .await?)
}

pub async fn update(
    state: &AppState,
    user_id: Uuid,
    category_id: Uuid,
    payload: UpdateCategoryRequest,
) -> Result<Category, AppError> {
    validation::validate_category(&payload.name, &payload.color)?;
    repository::update(
        &state.pool,
        user_id,
        category_id,
        payload.name.trim(),
        payload.color.trim(),
    )
    .await?
    .ok_or_else(|| AppError::NotFound("Category not found".to_string()))
}

pub async fn delete(state: &AppState, user_id: Uuid, category_id: Uuid) -> Result<(), AppError> {
    let category = repository::find_by_id(&state.pool, user_id, category_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Category not found".to_string()))?;

    let events_count = repository::events_count(&state.pool, user_id, category.id).await?;
    if events_count > 0 {
        return Err(AppError::Conflict(
            "Category cannot be deleted while events still use it".to_string(),
        ));
    }

    let rows = repository::delete(&state.pool, user_id, category_id).await?;
    if rows == 0 {
        return Err(AppError::NotFound("Category not found".to_string()));
    }

    Ok(())
}
