use axum::{
    extract::{Extension, Path, State},
    http::StatusCode,
    Json,
};
use std::sync::Arc;
use uuid::Uuid;
use validator::Validate;

use crate::{
    error::{AppError, Result},
    models::{
        response::ApiResponse,
        webhook::*,
    },
    services::auth_service::Claims,
    state::AppState,
};

fn extract_user_id(claims: &Claims) -> Result<Uuid> {
    Uuid::parse_str(&claims.sub).map_err(|_| AppError::Auth("无效的用户 ID".to_string()))
}

pub async fn create_subscription(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Json(request): Json<CreateWebhookSubscriptionRequest>,
) -> Result<(StatusCode, Json<ApiResponse<WebhookSubscriptionResponse>>)> {
    let user_id = extract_user_id(&claims)?;
    request.validate().map_err(|e| AppError::Validation(e.to_string()))?;

    // Get user's first OAuth app (simplified — in production, pass app_id explicitly)
    let apps = state.oauth_service().list_apps(user_id).await?;
    let app = apps.first().ok_or_else(|| AppError::Validation("需要先注册 OAuth 应用".to_string()))?;

    let secret = request.secret.unwrap_or_else(|| {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        (0..32).map(|_| format!("{:02x}", rng.gen::<u8>())).collect()
    });

    let sub = state.webhook_service().create_subscription(
        app.id, &request.url, &secret, &request.events,
    ).await?;

    let response = WebhookSubscriptionResponse::from(sub);
    Ok((StatusCode::CREATED, Json(ApiResponse::success(response))))
}

pub async fn list_subscriptions(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<ApiResponse<Vec<WebhookSubscriptionResponse>>>> {
    let user_id = extract_user_id(&claims)?;
    let apps = state.oauth_service().list_apps(user_id).await?;
    let app = apps.first().ok_or_else(|| AppError::Validation("需要先注册 OAuth 应用".to_string()))?;

    let subs = state.webhook_service().list_subscriptions(app.id).await?;
    let responses: Vec<WebhookSubscriptionResponse> = subs.into_iter().map(Into::into).collect();
    Ok(Json(ApiResponse::success(responses)))
}

pub async fn update_subscription(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(sub_id): Path<Uuid>,
    Json(request): Json<UpdateWebhookSubscriptionRequest>,
) -> Result<Json<ApiResponse<WebhookSubscriptionResponse>>> {
    let user_id = extract_user_id(&claims)?;
    let apps = state.oauth_service().list_apps(user_id).await?;
    let app = apps.first().ok_or_else(|| AppError::Validation("需要先注册 OAuth 应用".to_string()))?;

    let sub = state.webhook_service().update_subscription(
        sub_id, app.id,
        request.url.as_deref(),
        request.secret.as_deref(),
        request.events.as_deref(),
        request.is_active,
    ).await?;

    Ok(Json(ApiResponse::success(WebhookSubscriptionResponse::from(sub))))
}

pub async fn delete_subscription(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(sub_id): Path<Uuid>,
) -> Result<Json<ApiResponse<()>>> {
    let user_id = extract_user_id(&claims)?;
    let apps = state.oauth_service().list_apps(user_id).await?;
    let app = apps.first().ok_or_else(|| AppError::Validation("需要先注册 OAuth 应用".to_string()))?;

    state.webhook_service().delete_subscription(sub_id, app.id).await?;
    Ok(Json(ApiResponse::success_with_message("订阅已删除")))
}

pub async fn get_deliveries(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(sub_id): Path<Uuid>,
) -> Result<Json<ApiResponse<Vec<WebhookDeliveryResponse>>>> {
    let user_id = extract_user_id(&claims)?;
    let apps = state.oauth_service().list_apps(user_id).await?;
    let app = apps.first().ok_or_else(|| AppError::Validation("需要先注册 OAuth 应用".to_string()))?;

    let deliveries = state.webhook_service().get_deliveries(sub_id, app.id, 50, 0).await?;
    let responses: Vec<WebhookDeliveryResponse> = deliveries.into_iter().map(Into::into).collect();
    Ok(Json(ApiResponse::success(responses)))
}

pub async fn redeliver(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path((sub_id, delivery_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<ApiResponse<()>>> {
    let user_id = extract_user_id(&claims)?;
    let apps = state.oauth_service().list_apps(user_id).await?;
    let app = apps.first().ok_or_else(|| AppError::Validation("需要先注册 OAuth 应用".to_string()))?;

    state.webhook_service().redeliver(delivery_id, sub_id, app.id).await?;
    Ok(Json(ApiResponse::success_with_message("投递已重新触发")))
}
