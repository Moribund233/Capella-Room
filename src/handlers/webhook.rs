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
    middleware::oauth_auth::{extract_user_id_from_auth, AppAuth},
    models::{
        response::ApiResponse,
        webhook::*,
    },
    state::AppState,
};

/// 从认证信息中获取 app_id
/// OAuth token: 从 aud claim 提取
/// CapellaRoom JWT: 从用户已注册的 OAuth 应用中取第一个
async fn resolve_app_id(auth: &AppAuth, state: &AppState) -> Result<Uuid> {
    match auth {
        AppAuth::OAuth(oauth_claims) => {
            Uuid::parse_str(&oauth_claims.aud)
                .map_err(|_| AppError::Auth("无效的 OAuth app ID".to_string()))
        }
        AppAuth::User(_claims) => {
            let user_id = extract_user_id_from_auth(auth)?;
            let apps = state.oauth_service().list_apps(user_id).await?;
            apps.first().map(|a| a.id)
                .ok_or_else(|| AppError::Validation("需要先注册 OAuth 应用".to_string()))
        }
    }
}

pub async fn create_subscription(
    State(state): State<Arc<AppState>>,
    Extension(auth): Extension<AppAuth>,
    Json(request): Json<CreateWebhookSubscriptionRequest>,
) -> Result<(StatusCode, Json<ApiResponse<WebhookSubscriptionResponse>>)> {
    request.validate().map_err(|e| AppError::Validation(e.to_string()))?;
    let app_id = resolve_app_id(&auth, &state).await?;

    let secret = request.secret.unwrap_or_else(|| {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        (0..32).map(|_| format!("{:02x}", rng.gen::<u8>())).collect()
    });

    let sub = state.webhook_service().create_subscription(
        app_id, &request.url, &secret, &request.events,
    ).await?;

    let response = WebhookSubscriptionResponse::from(sub);
    Ok((StatusCode::CREATED, Json(ApiResponse::success(response))))
}

pub async fn list_subscriptions(
    State(state): State<Arc<AppState>>,
    Extension(auth): Extension<AppAuth>,
) -> Result<Json<ApiResponse<Vec<WebhookSubscriptionResponse>>>> {
    let app_id = resolve_app_id(&auth, &state).await?;
    let subs = state.webhook_service().list_subscriptions(app_id).await?;
    let responses: Vec<WebhookSubscriptionResponse> = subs.into_iter().map(Into::into).collect();
    Ok(Json(ApiResponse::success(responses)))
}

pub async fn update_subscription(
    State(state): State<Arc<AppState>>,
    Extension(auth): Extension<AppAuth>,
    Path(sub_id): Path<Uuid>,
    Json(request): Json<UpdateWebhookSubscriptionRequest>,
) -> Result<Json<ApiResponse<WebhookSubscriptionResponse>>> {
    let app_id = resolve_app_id(&auth, &state).await?;

    let sub = state.webhook_service().update_subscription(
        sub_id, app_id,
        request.url.as_deref(),
        request.secret.as_deref(),
        request.events.as_deref(),
        request.is_active,
    ).await?;

    Ok(Json(ApiResponse::success(WebhookSubscriptionResponse::from(sub))))
}

pub async fn delete_subscription(
    State(state): State<Arc<AppState>>,
    Extension(auth): Extension<AppAuth>,
    Path(sub_id): Path<Uuid>,
) -> Result<Json<ApiResponse<()>>> {
    let app_id = resolve_app_id(&auth, &state).await?;
    state.webhook_service().delete_subscription(sub_id, app_id).await?;
    Ok(Json(ApiResponse::success_with_message("订阅已删除")))
}

pub async fn get_deliveries(
    State(state): State<Arc<AppState>>,
    Extension(auth): Extension<AppAuth>,
    Path(sub_id): Path<Uuid>,
) -> Result<Json<ApiResponse<Vec<WebhookDeliveryResponse>>>> {
    let app_id = resolve_app_id(&auth, &state).await?;
    let deliveries = state.webhook_service().get_deliveries(sub_id, app_id, 50, 0).await?;
    let responses: Vec<WebhookDeliveryResponse> = deliveries.into_iter().map(Into::into).collect();
    Ok(Json(ApiResponse::success(responses)))
}

pub async fn redeliver(
    State(state): State<Arc<AppState>>,
    Extension(auth): Extension<AppAuth>,
    Path((sub_id, delivery_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<ApiResponse<()>>> {
    let app_id = resolve_app_id(&auth, &state).await?;
    state.webhook_service().redeliver(delivery_id, sub_id, app_id).await?;
    Ok(Json(ApiResponse::success_with_message("投递已重新触发")))
}
