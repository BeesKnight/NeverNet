use chrono::{DateTime, Utc};
use contracts::identity::identity_service_client::IdentityServiceClient;
use contracts::identity::{
    GetSettingsRequest as IdentityGetSettingsRequest,
    UpdateSettingsRequest as IdentityUpdateSettingsRequest,
};
use tonic::transport::Channel;
use uuid::Uuid;

use crate::{
    app_state::AppState,
    error::AppError,
    settings::models::{UiSettings, UpdateSettingsRequest},
    shared::grpc,
};

pub async fn get(state: &AppState, user_id: Uuid) -> Result<UiSettings, AppError> {
    let mut client = identity_client(state).await?;
    let reply = client
        .get_settings(IdentityGetSettingsRequest {
            user_id: user_id.to_string(),
        })
        .await
        .map_err(map_status)?
        .into_inner();

    map_settings(
        reply.settings.ok_or_else(|| {
            AppError::Internal("Identity response is missing settings".to_string())
        })?,
    )
}

pub async fn update(
    state: &AppState,
    user_id: Uuid,
    payload: UpdateSettingsRequest,
) -> Result<UiSettings, AppError> {
    let mut client = identity_client(state).await?;
    let reply = client
        .update_settings(IdentityUpdateSettingsRequest {
            user_id: user_id.to_string(),
            theme: payload.theme,
            accent_color: payload.accent_color,
            default_view: payload.default_view,
        })
        .await
        .map_err(map_status)?
        .into_inner();

    map_settings(
        reply.settings.ok_or_else(|| {
            AppError::Internal("Identity response is missing settings".to_string())
        })?,
    )
}

async fn identity_client(
    state: &AppState,
) -> Result<
    IdentityServiceClient<
        tonic::service::interceptor::InterceptedService<Channel, grpc::RequestIdInterceptor>,
    >,
    AppError,
> {
    let channel =
        grpc::connect_channel(&state.config.identity_service_url, "Identity service").await?;

    Ok(IdentityServiceClient::with_interceptor(
        channel,
        grpc::RequestIdInterceptor,
    ))
}

fn map_settings(settings: contracts::identity::UiSettings) -> Result<UiSettings, AppError> {
    Ok(UiSettings {
        user_id: parse_uuid(&settings.user_id, "settings user id")?,
        theme: settings.theme,
        accent_color: settings.accent_color,
        default_view: settings.default_view,
        created_at: parse_timestamp(&settings.created_at, "settings created_at")?,
        updated_at: parse_timestamp(&settings.updated_at, "settings updated_at")?,
    })
}

fn parse_uuid(value: &str, field: &str) -> Result<Uuid, AppError> {
    Uuid::parse_str(value)
        .map_err(|_| AppError::Internal(format!("Internal service returned an invalid {field}")))
}

fn parse_timestamp(value: &str, field: &str) -> Result<DateTime<Utc>, AppError> {
    DateTime::parse_from_rfc3339(value)
        .map(|timestamp| timestamp.with_timezone(&Utc))
        .map_err(|_| AppError::Internal(format!("Internal service returned an invalid {field}")))
}

fn map_status(status: tonic::Status) -> AppError {
    match status.code() {
        tonic::Code::InvalidArgument => AppError::BadRequest(status.message().to_string()),
        tonic::Code::NotFound => AppError::NotFound(status.message().to_string()),
        tonic::Code::Unauthenticated => AppError::Unauthorized(status.message().to_string()),
        _ => AppError::Internal(format!("Identity service error: {}", status.message())),
    }
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;

    use super::*;

    #[test]
    fn maps_settings_payload() {
        let user_id = Uuid::new_v4();
        let settings = map_settings(contracts::identity::UiSettings {
            user_id: user_id.to_string(),
            theme: "system".to_string(),
            accent_color: "#b6532f".to_string(),
            default_view: "dashboard".to_string(),
            created_at: Utc
                .with_ymd_and_hms(2026, 3, 13, 10, 0, 0)
                .unwrap()
                .to_rfc3339(),
            updated_at: Utc
                .with_ymd_and_hms(2026, 3, 13, 10, 5, 0)
                .unwrap()
                .to_rfc3339(),
        })
        .expect("settings should map");

        assert_eq!(settings.user_id, user_id);
        assert_eq!(settings.theme, "system");
    }

    #[test]
    fn maps_settings_status_codes() {
        assert!(matches!(
            map_status(tonic::Status::invalid_argument("bad")),
            AppError::BadRequest(_)
        ));
        assert!(matches!(
            map_status(tonic::Status::not_found("missing")),
            AppError::NotFound(_)
        ));
        assert!(matches!(
            map_status(tonic::Status::unauthenticated("denied")),
            AppError::Unauthorized(_)
        ));
        assert!(matches!(
            map_status(tonic::Status::internal("oops")),
            AppError::Internal(_)
        ));
    }
}
