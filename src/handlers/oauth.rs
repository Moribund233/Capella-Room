use axum::{
    body::Bytes,
    extract::{Extension, Form, Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::{Html, IntoResponse, Redirect, Response},
    Json,
};
use std::sync::Arc;
use uuid::Uuid;
use validator::Validate;

use crate::{
    error::{AppError, Result},
    middleware::oauth_auth::{extract_app_id_from_auth, extract_user_id_from_auth, AppAuth},
    models::{
        oauth::*,
        response::ApiResponse,
    },
    state::AppState,
    websocket::protocol::WebSocketMessage,
};

/// 解析 OAuth client_id 字符串为 Uuid
/// RFC 6749 定义 client_id 为字符串，但 CapellaRoom 内部使用 UUID 作为 app 的主键
fn parse_client_id(client_id: &str) -> Result<Uuid> {
    Uuid::parse_str(client_id).map_err(|_| {
        AppError::Validation(format!(
            "client_id '{}' 不是合法的 UUID 格式。OAuth 应用 client_id 是 CapellaRoom 中应用的 UUID，\
             请检查配置或通过 CapellaRoom 管理页面获取正确的 client_id",
            client_id
        ))
    })
}

// ═══════════════════════════════════════════════
// 3.1 OAuth App CRUD
// ═══════════════════════════════════════════════

pub async fn create_app(
    State(state): State<Arc<AppState>>,
    Extension(auth): Extension<AppAuth>,
    Json(request): Json<CreateOAuthAppRequest>,
) -> Result<(StatusCode, Json<ApiResponse<OAuthAppCreatedResponse>>)> {
    let user_id = extract_user_id_from_auth(&auth)?;
    request.validate().map_err(|e| AppError::Validation(e.to_string()))?;

    let scopes: Vec<String> = request.scopes.unwrap_or_default();
    let redirect_uri_refs: Vec<&str> = request.redirect_uris.iter().map(|s| s.as_str()).collect();
    let scope_refs: Vec<&str> = scopes.iter().map(|s| s.as_str()).collect();

    let app = state.oauth_service().register_app(
        user_id,
        &request.name,
        request.description.as_deref(),
        &redirect_uri_refs,
        &scope_refs,
    ).await?;

    let response = OAuthAppCreatedResponse {
        id: app.id,
        name: app.name,
        client_id: app.id,
        client_secret: app.client_secret,
        redirect_uris: app.redirect_uris,
        scopes: app.scopes,
        is_active: app.is_active,
        created_at: app.created_at,
    };

    Ok((StatusCode::CREATED, Json(ApiResponse::success(response))))
}

pub async fn list_apps(
    State(state): State<Arc<AppState>>,
    Extension(auth): Extension<AppAuth>,
) -> Result<Json<ApiResponse<Vec<OAuthAppResponse>>>> {
    let user_id = extract_user_id_from_auth(&auth)?;
    let apps = state.oauth_service().list_apps(user_id).await?;

    let responses: Vec<OAuthAppResponse> = apps.into_iter().map(|a| OAuthAppResponse {
        id: a.id,
        name: a.name,
        description: a.description,
        redirect_uris: a.redirect_uris,
        scopes: a.scopes,
        is_active: a.is_active,
        created_at: a.created_at,
        updated_at: a.updated_at,
    }).collect();

    Ok(Json(ApiResponse::success(responses)))
}

pub async fn get_app(
    State(state): State<Arc<AppState>>,
    Extension(auth): Extension<AppAuth>,
    Path(app_id): Path<Uuid>,
) -> Result<Json<ApiResponse<OAuthAppResponse>>> {
    let user_id = extract_user_id_from_auth(&auth)?;
    let app = state.oauth_service().get_app(app_id).await?;
    if app.owner_id != user_id {
        return Err(AppError::Forbidden);
    }

    let response = OAuthAppResponse {
        id: app.id,
        name: app.name,
        description: app.description,
        redirect_uris: app.redirect_uris,
        scopes: app.scopes,
        is_active: app.is_active,
        created_at: app.created_at,
        updated_at: app.updated_at,
    };

    Ok(Json(ApiResponse::success(response)))
}

pub async fn update_app(
    State(state): State<Arc<AppState>>,
    Extension(auth): Extension<AppAuth>,
    Path(app_id): Path<Uuid>,
    Json(request): Json<UpdateOAuthAppRequest>,
) -> Result<Json<ApiResponse<OAuthAppResponse>>> {
    let user_id = extract_user_id_from_auth(&auth)?;

    let name = request.name.as_deref();
    let description_opt = request.description.flatten();
    let description = description_opt.as_deref();
    let redirect_uris = request.redirect_uris.as_deref().map(|v| v.iter().map(|s| s.as_str()).collect::<Vec<_>>());
    let scopes = request.scopes.as_deref().map(|v| v.iter().map(|s| s.as_str()).collect::<Vec<_>>());

    let app = state.oauth_service().update_app(
        app_id,
        user_id,
        name,
        description,
        redirect_uris.as_deref(),
        scopes.as_deref(),
    ).await?;

    let response = OAuthAppResponse {
        id: app.id,
        name: app.name,
        description: app.description,
        redirect_uris: app.redirect_uris,
        scopes: app.scopes,
        is_active: app.is_active,
        created_at: app.created_at,
        updated_at: app.updated_at,
    };

    Ok(Json(ApiResponse::success(response)))
}

pub async fn delete_app(
    State(state): State<Arc<AppState>>,
    Extension(auth): Extension<AppAuth>,
    Path(app_id): Path<Uuid>,
) -> Result<Json<ApiResponse<()>>> {
    let user_id = extract_user_id_from_auth(&auth)?;
    state.oauth_service().delete_app(app_id, user_id).await?;
    Ok(Json(ApiResponse::success_with_message("应用已删除")))
}

pub async fn rotate_secret(
    State(state): State<Arc<AppState>>,
    Extension(auth): Extension<AppAuth>,
    Path(app_id): Path<Uuid>,
) -> Result<Json<ApiResponse<OAuthAppWithSecretResponse>>> {
    let user_id = extract_user_id_from_auth(&auth)?;
    let new_secret = state.oauth_service().rotate_secret(app_id, user_id).await?;
    let app = state.oauth_service().get_app(app_id).await?;

    let response = OAuthAppWithSecretResponse {
        id: app.id,
        name: app.name,
        description: app.description,
        client_secret: new_secret,
        redirect_uris: app.redirect_uris,
        scopes: app.scopes,
        is_active: app.is_active,
        created_at: app.created_at,
        updated_at: app.updated_at,
    };

    Ok(Json(ApiResponse::success(response)))
}

// ═══════════════════════════════════════════════
// 3.2 OAuth Flow — Authorize endpoint (browser)
// ═══════════════════════════════════════════════

pub async fn authorize_get(
    State(state): State<Arc<AppState>>,
    Query(params): Query<AuthorizeRequest>,
) -> Result<Response> {
    let client_id = parse_client_id(&params.client_id)?;
    let app = state.oauth_service().get_app(client_id).await?;
    if !app.is_active {
        return Err(AppError::Auth("应用未激活".to_string()));
    }

    if !app.redirect_uris.contains(&params.redirect_uri) {
        return Err(AppError::Validation("redirect_uri 不匹配".to_string()));
    }

    if params.response_type != "code" {
        return Err(AppError::Validation("仅支持 response_type=code".to_string()));
    }

    let login_html = include_str!("../oauth/templates/login.html");
    let html = login_html
        .replace("{{client_id}}", &params.client_id)
        .replace("{{redirect_uri}}", &params.redirect_uri)
        .replace("{{state}}", &params.state.unwrap_or_default())
        .replace("{{scope}}", &params.scope.unwrap_or_default())
        .replace("{{app_name}}", &app.name)
        .replace("{{error}}", "");

    Ok(Html(html).into_response())
}

pub async fn authorize_post(
    State(state): State<Arc<AppState>>,
    Form(form): Form<AuthorizeFormRequest>,
) -> Result<Response> {
    let client_id = parse_client_id(&form.client_id)?;
    let app = state.oauth_service().get_app(client_id).await?;
    if !app.is_active {
        return Err(AppError::Auth("应用未激活".to_string()));
    }
    if !app.redirect_uris.contains(&form.redirect_uri) {
        return Err(AppError::Validation("redirect_uri 不匹配".to_string()));
    }

    // Authenticate user — on failure re-render login page with error
    let email = form.email.trim().to_lowercase();
    let user = match state.user_service().get_user_by_email(&email).await {
        Ok(Some(u)) => u,
        Ok(None) => {
            let login_html = include_str!("../oauth/templates/login.html");
            let html = login_html
                .replace("{{client_id}}", &form.client_id)
                .replace("{{redirect_uri}}", &form.redirect_uri)
                .replace("{{state}}", &form.state.unwrap_or_default())
                .replace("{{scope}}", &form.scope.unwrap_or_default())
                .replace("{{app_name}}", &app.name)
                .replace("{{error}}", "邮箱或密码错误");
            return Ok(Html(html).into_response());
        }
        Err(_) => return Err(AppError::Internal),
    };

    if user.is_account_disabled() {
        let login_html = include_str!("../oauth/templates/login.html");
        let html = login_html
            .replace("{{client_id}}", &form.client_id)
            .replace("{{redirect_uri}}", &form.redirect_uri)
            .replace("{{state}}", &form.state.unwrap_or_default())
            .replace("{{scope}}", &form.scope.unwrap_or_default())
            .replace("{{app_name}}", &app.name)
            .replace("{{error}}", "账号已被禁用");
        return Ok(Html(html).into_response());
    }

    let password_valid = state.auth_service().verify_password(&form.password, &user.password_hash)?;
    if !password_valid {
        let login_html = include_str!("../oauth/templates/login.html");
        let html = login_html
            .replace("{{client_id}}", &form.client_id)
            .replace("{{redirect_uri}}", &form.redirect_uri)
            .replace("{{state}}", &form.state.unwrap_or_default())
            .replace("{{scope}}", &form.scope.unwrap_or_default())
            .replace("{{app_name}}", &app.name)
            .replace("{{error}}", "邮箱或密码错误");
        return Ok(Html(html).into_response());
    }

    // Create auth session token
    let session_token = state.oauth_service().create_auth_session_token(user.id, client_id)?;

    // Render consent page
    let consent_html = include_str!("../oauth/templates/consent.html");
    let app_name_short = app.name.chars().next().unwrap_or('?').to_string();
    let html = consent_html
        .replace("{{auth_session_token}}", &session_token)
        .replace("{{client_id}}", &form.client_id)
        .replace("{{redirect_uri}}", &form.redirect_uri)
        .replace("{{response_type}}", &form.response_type)
        .replace("{{state}}", &form.state.unwrap_or_default())
        .replace("{{scope}}", &form.scope.unwrap_or_default())
        .replace("{{app_name}}", &app.name)
        .replace("{{app_name_short}}", &app_name_short)
        .replace("{{username}}", &user.username);

    Ok(Html(html).into_response())
}

pub async fn authorize_consent(
    State(state): State<Arc<AppState>>,
    Form(form): Form<ConsentFormRequest>,
) -> Result<Response> {
    let client_id = parse_client_id(&form.client_id)?;
    let session = state.oauth_service().verify_auth_session_token(&form.auth_session_token)?;

    if session.client_id != client_id {
        return Err(AppError::Auth("client_id 不匹配".to_string()));
    }

    let scopes: Vec<String> = form.scope
        .unwrap_or_default()
        .split_whitespace()
        .map(|s| s.to_string())
        .collect();

    if form.approve.as_deref() == Some("true") {
        let auth_code = state.oauth_service().create_authorization_code(
            client_id,
            session.user_id,
            &form.redirect_uri,
            &scopes,
        ).await?;

        let state_param = form.state.unwrap_or_default();
        let redirect_url = format!("{}?code={}&state={}", form.redirect_uri, auth_code.code, state_param);

        Ok(Redirect::to(&redirect_url).into_response())
    } else {
        let state_param = form.state.unwrap_or_default();
        let redirect_url = format!("{}?error=access_denied&state={}", form.redirect_uri, state_param);
        Ok(Redirect::to(&redirect_url).into_response())
    }
}

// ═══════════════════════════════════════════════
// 3.2 OAuth Flow — Token endpoint
// ═══════════════════════════════════════════════

pub async fn token(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Json<TokenResponse>> {
    let content_type = headers
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    let request: TokenRequest = if content_type.contains("json") {
        serde_json::from_slice(&body)
            .map_err(|e| AppError::Validation(format!("JSON 请求体格式错误: {}", e)))?
    } else {
        serde_urlencoded::from_bytes(&body)
            .map_err(|e| AppError::Validation(format!("表单请求体格式错误: {}", e)))?
    };

    let client_id = parse_client_id(&request.client_id)?;

    match request.grant_type.as_str() {
        "authorization_code" => {
            let code = request.code.ok_or_else(|| AppError::Validation("缺少 code".to_string()))?;
            let redirect_uri = request.redirect_uri.ok_or_else(|| AppError::Validation("缺少 redirect_uri".to_string()))?;

            let (auth_code, _) = state.oauth_service().exchange_code(
                &code,
                client_id,
                &request.client_secret,
                &redirect_uri,
            ).await?;

            let scopes = auth_code.scopes.unwrap_or_default();
            let response = state.oauth_service().generate_tokens(
                auth_code.user_id,
                client_id,
                &scopes,
            ).await?;

            Ok(Json(response))
        }
        "refresh_token" => {
            let refresh_token = request.refresh_token.ok_or_else(|| AppError::Validation("缺少 refresh_token".to_string()))?;

            let response = state.oauth_service().exchange_refresh_token(
                &refresh_token,
                client_id,
                &request.client_secret,
            ).await?;

            Ok(Json(response))
        }
        "client_credentials" => {
            let response = state.oauth_service().client_credentials_grant(
                client_id,
                &request.client_secret,
            ).await?;

            Ok(Json(response))
        }
        _ => Err(AppError::Auth("unsupported_grant_type".to_string())),
    }
}

// ═══════════════════════════════════════════════
// 3.2 UserInfo endpoint
// ═══════════════════════════════════════════════

pub async fn userinfo(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<UserInfoResponse>> {
    let token = headers
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or_else(|| AppError::Auth("Missing or invalid Authorization header".to_string()))?;
    let claims = state.oauth_service().verify_access_token(token)?;
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::Auth("无效的用户 ID".to_string()))?;
    let user_info = state.oauth_service().get_user_info(user_id).await?;
    Ok(Json(user_info))
}

// ═══════════════════════════════════════════════
// 3.2 Identity Mapping endpoints
// ═══════════════════════════════════════════════

pub async fn create_mapping(
    State(state): State<Arc<AppState>>,
    Extension(auth): Extension<AppAuth>,
    Json(request): Json<CreateMappingRequest>,
) -> Result<(StatusCode, Json<ApiResponse<UserIdentityMapping>>)> {
    request.validate().map_err(|e| AppError::Validation(e.to_string()))?;

    // 权限验证：OAuth token 必须匹配被映射的用户和 app
    match &auth {
        AppAuth::OAuth(oauth_claims) => {
            let token_user_id = Uuid::parse_str(&oauth_claims.sub)
                .map_err(|_| AppError::Auth("无效的 token sub".to_string()))?;
            let token_app_id = Uuid::parse_str(&oauth_claims.aud)
                .map_err(|_| AppError::Auth("无效的 token aud".to_string()))?;

            if token_user_id != request.user_id {
                return Err(AppError::Forbidden);
            }
            if token_app_id != request.app_id {
                return Err(AppError::Forbidden);
            }
        }
        AppAuth::User(_) => {
            // CapellaRoom 内部用户可以直接创建映射
        }
    }

    let mapping = state.oauth_service().create_mapping(
        request.app_id,
        request.user_id,
        &request.external_user_id,
        request.external_username.as_deref(),
    ).await?;

    Ok((StatusCode::CREATED, Json(ApiResponse::success(mapping))))
}

pub async fn lookup_mapping(
    State(state): State<Arc<AppState>>,
    Extension(_auth): Extension<AppAuth>,
    Query(query): Query<MappingLookupQuery>,
) -> Result<Json<ApiResponse<Option<UserIdentityMapping>>>> {
    let mapping = state.oauth_service().lookup_mapping(query.app_id, &query.external_user_id).await?;
    Ok(Json(ApiResponse::success(mapping)))
}

pub async fn delete_mapping_handler(
    State(state): State<Arc<AppState>>,
    Extension(_auth): Extension<AppAuth>,
    Path(mapping_id): Path<Uuid>,
) -> Result<Json<ApiResponse<()>>> {
    state.oauth_service().delete_mapping(mapping_id).await?;
    Ok(Json(ApiResponse::success_with_message("映射已解除")))
}

// ═══════════════════════════════════════════════
// 3.5 Room Resource Binding endpoints
// ═══════════════════════════════════════════════

pub async fn bind_resource(
    State(state): State<Arc<AppState>>,
    Extension(auth): Extension<AppAuth>,
    Path(room_id): Path<Uuid>,
    Json(request): Json<CreateResourceBindingRequest>,
) -> Result<(StatusCode, Json<ApiResponse<RoomResourceBinding>>)> {
    let user_id = extract_user_id_from_auth(&auth)?;
    request.validate().map_err(|e| AppError::Validation(e.to_string()))?;

    let member = state.room_service().get_room_member(room_id, user_id).await?;
    match member {
        Some(m) if matches!(m.role, crate::models::room::MemberRole::Owner | crate::models::room::MemberRole::Admin) => {}
        _ => return Err(AppError::Forbidden),
    }

    let binding = state.oauth_service().create_resource_binding(room_id, request).await?;

    // WS broadcast: resource bound
    if let Ok(json) = (WebSocketMessage::ResourceBound {
        room_id,
        binding: serde_json::to_value(&binding).unwrap_or_default(),
    }).to_json() {
        let _ = state.ws_manager().broadcast_to_room_all(room_id, json).await;
    }

    Ok((StatusCode::CREATED, Json(ApiResponse::success(binding))))
}

pub async fn list_bindings(
    State(state): State<Arc<AppState>>,
    Extension(_auth): Extension<AppAuth>,
    Path(room_id): Path<Uuid>,
) -> Result<Json<ApiResponse<Vec<RoomResourceBinding>>>> {
    let bindings = state.oauth_service().list_resource_bindings(room_id).await?;
    Ok(Json(ApiResponse::success(bindings)))
}

pub async fn lookup_resource(
    State(state): State<Arc<AppState>>,
    Extension(_auth): Extension<AppAuth>,
    Query(query): Query<ResourceLookupQuery>,
) -> Result<Json<ApiResponse<Option<RoomResourceBinding>>>> {
    let binding = state.oauth_service().lookup_resource(query.app_id, &query.resource_type, &query.resource_id).await?;
    Ok(Json(ApiResponse::success(binding)))
}

pub async fn update_binding(
    State(state): State<Arc<AppState>>,
    Extension(auth): Extension<AppAuth>,
    Path((room_id, binding_id)): Path<(Uuid, Uuid)>,
    Json(request): Json<UpdateResourceBindingRequest>,
) -> Result<Json<ApiResponse<RoomResourceBinding>>> {
    let user_id = extract_user_id_from_auth(&auth)?;

    let member = state.room_service().get_room_member(room_id, user_id).await?;
    match member {
        Some(m) if matches!(m.role, crate::models::room::MemberRole::Owner | crate::models::room::MemberRole::Admin) => {}
        _ => return Err(AppError::Forbidden),
    }

    let binding = state.oauth_service().update_resource_binding(
        binding_id,
        request.resource_url.as_deref(),
        request.resource_name.as_deref(),
        request.metadata,
    ).await?;

    // WS broadcast: resource binding updated
    if let Ok(json) = (WebSocketMessage::ResourceBindingUpdated {
        room_id,
        binding: serde_json::to_value(&binding).unwrap_or_default(),
    }).to_json() {
        let _ = state.ws_manager().broadcast_to_room_all(room_id, json).await;
    }

    Ok(Json(ApiResponse::success(binding)))
}

pub async fn unbind_resource(
    State(state): State<Arc<AppState>>,
    Extension(auth): Extension<AppAuth>,
    Path((room_id, binding_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<ApiResponse<()>>> {
    let user_id = extract_user_id_from_auth(&auth)?;

    let member = state.room_service().get_room_member(room_id, user_id).await?;
    match member {
        Some(m) if matches!(m.role, crate::models::room::MemberRole::Owner | crate::models::room::MemberRole::Admin) => {}
        _ => return Err(AppError::Forbidden),
    }

    let binding = state.oauth_service().get_resource_binding(binding_id).await?;
    state.oauth_service().delete_resource_binding(binding_id).await?;

    // WS broadcast: resource unbound
    if let Ok(json) = (WebSocketMessage::ResourceUnbound {
        room_id,
        binding_id,
        resource_type: binding.resource_type.clone(),
        resource_id: binding.resource_id.clone(),
    }).to_json() {
        let _ = state.ws_manager().broadcast_to_room_all(room_id, json).await;
    }

    Ok(Json(ApiResponse::success_with_message("资源已解绑")))
}

pub async fn bind_resource_auto_create(
    State(state): State<Arc<AppState>>,
    Extension(auth): Extension<AppAuth>,
    Json(request): Json<AutoCreateResourceRequest>,
) -> Result<(StatusCode, Json<ApiResponse<serde_json::Value>>)> {
    request.validate().map_err(|e| AppError::Validation(e.to_string()))?;

    let user_id = extract_user_id_from_auth(&auth)?;
    let app_id = extract_app_id_from_auth(&auth)
        .ok_or_else(|| AppError::Auth("此接口需要 OAuth access_token 认证".to_string()))?;

    // 重复检查：同 app + resource_type + resource_id 返回 409
    if let Some(existing) = state.oauth_service().lookup_resource(app_id, &request.resource_type, &request.resource_id).await? {
        return Err(AppError::Conflict(format!(
            "资源绑定已存在：room_id={}, binding_id={}", existing.room_id, existing.id
        )));
    }

    let room_name = request.resource_name
        .clone()
        .map(|n| format!("{} 聊天室", n))
        .unwrap_or_else(|| format!("{}:{} 聊天室", request.resource_type, request.resource_id));

    let room = state.room_service().create_room(
        &room_name,
        Some(&format!("资源 {}:{} 的自动创建聊天室", request.resource_type, request.resource_id)),
        user_id,
        false,
        100,
    ).await?;

    let binding_request = CreateResourceBindingRequest {
        app_id,
        resource_type: request.resource_type.clone(),
        resource_id: request.resource_id.clone(),
        resource_url: None,
        resource_name: request.resource_name,
        metadata: None,
    };
    let binding = state.oauth_service().create_resource_binding(room.id, binding_request).await?;

    // WS 广播
    if let Ok(json) = (WebSocketMessage::ResourceBound {
        room_id: room.id,
        binding: serde_json::to_value(&binding).unwrap_or_default(),
    }).to_json() {
        let _ = state.ws_manager().broadcast_to_room_all(room.id, json).await;
    }

    let response = serde_json::json!({
        "room": room,
        "binding": binding,
    });

    Ok((StatusCode::CREATED, Json(ApiResponse::success(response))))
}
