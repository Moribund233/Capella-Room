use axum::{
    extract::{Path, Query, State},
    http::{header, StatusCode},
    response::IntoResponse,
    Extension, Json,
};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    error::{AppError, Result},
    middleware::admin::CurrentUserId,
    models::audit::{
        AlertQuery, AuditLogExportQuery, AuditLogQuery, AuditStats, AuditStatsQuery, ExportFormat,
        UpdateAlertRuleRequest, UpdateAlertStatusRequest,
    },
    models::response::ApiResponse,
    state::AppState,
};

/// 查询审计日志
/// GET /api/v1/admin/audit/logs
pub async fn list_audit_logs(
    State(state): State<Arc<AppState>>,
    Query(query): Query<AuditLogQuery>,
) -> Result<Json<ApiResponse<serde_json::Value>>> {
    let (logs, total) = state.audit_service().query_logs(query.clone()).await?;

    let response = serde_json::json!({
        "logs": logs,
        "total": total,
        "limit": query.limit.unwrap_or(50),
        "offset": query.offset.unwrap_or(0),
    });

    Ok(Json(ApiResponse::success(response)))
}

/// 获取单条审计日志详情
/// GET /api/v1/admin/audit/logs/:id
pub async fn get_audit_log_detail(
    State(state): State<Arc<AppState>>,
    Path(log_id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>> {
    let log = state
        .audit_service()
        .get_log_by_id(log_id)
        .await?
        .ok_or(AppError::NotFound)?;

    Ok(Json(ApiResponse::success(serde_json::json!(log))))
}

/// 获取审计统计信息
/// GET /api/v1/admin/audit/stats
pub async fn get_audit_stats(
    State(state): State<Arc<AppState>>,
    Query(query): Query<AuditStatsQuery>,
) -> Result<Json<ApiResponse<AuditStats>>> {
    let stats = state
        .audit_service()
        .get_audit_stats(query.start_time, query.end_time)
        .await?;

    Ok(Json(ApiResponse::success(stats)))
}

/// 导出审计日志
/// GET /api/v1/admin/audit/export
pub async fn export_audit_logs(
    State(state): State<Arc<AppState>>,
    Query(query): Query<AuditLogExportQuery>,
) -> Result<impl IntoResponse> {
    let log_query = AuditLogQuery {
        event_type: query.event_type,
        severity: query.severity,
        actor_id: query.actor_id,
        target_id: None,
        target_type: None,
        status: None,
        start_time: query.start_time,
        end_time: query.end_time,
        limit: Some(10000),
        offset: Some(0),
    };

    match query.format {
        ExportFormat::Json => {
            let json_data = state.audit_service().export_logs_json(log_query).await?;
            Ok((
                StatusCode::OK,
                [(header::CONTENT_TYPE, "application/json")],
                [(
                    header::CONTENT_DISPOSITION,
                    "attachment; filename=audit_logs.json",
                )],
                json_data,
            ))
        }
        ExportFormat::Csv => {
            let csv_data = state.audit_service().export_logs_csv(log_query).await?;
            Ok((
                StatusCode::OK,
                [(header::CONTENT_TYPE, "text/csv")],
                [(
                    header::CONTENT_DISPOSITION,
                    "attachment; filename=audit_logs.csv",
                )],
                csv_data,
            ))
        }
    }
}

/// 查询告警列表
/// GET /api/v1/admin/audit/alerts
pub async fn list_alerts(
    State(state): State<Arc<AppState>>,
    Query(query): Query<AlertQuery>,
) -> Result<Json<ApiResponse<serde_json::Value>>> {
    let (alerts, total) = state.audit_service().query_alerts(query.clone()).await?;

    let response = serde_json::json!({
        "alerts": alerts,
        "total": total,
        "limit": query.limit.unwrap_or(50),
        "offset": query.offset.unwrap_or(0),
    });

    Ok(Json(ApiResponse::success(response)))
}

/// 更新告警状态
/// PUT /api/v1/admin/audit/alerts/:id/status
pub async fn update_alert_status(
    State(state): State<Arc<AppState>>,
    Path(alert_id): Path<Uuid>,
    Extension(CurrentUserId(user_id)): Extension<CurrentUserId>,
    Json(request): Json<UpdateAlertStatusRequest>,
) -> Result<Json<ApiResponse<serde_json::Value>>> {
    let alert = state
        .audit_service()
        .update_alert_status(alert_id, request.status, user_id)
        .await?;

    Ok(Json(ApiResponse::success(serde_json::json!(alert))))
}

/// 获取告警规则列表
/// GET /api/v1/admin/audit/rules
pub async fn list_alert_rules(
    State(state): State<Arc<AppState>>,
) -> Result<Json<ApiResponse<serde_json::Value>>> {
    let rules = state.audit_service().get_alert_rules().await?;

    Ok(Json(ApiResponse::success(serde_json::json!({
        "rules": rules
    }))))
}

/// 更新告警规则
/// PUT /api/v1/admin/audit/rules/:id
pub async fn update_alert_rule(
    State(state): State<Arc<AppState>>,
    Path(rule_id): Path<Uuid>,
    Json(request): Json<UpdateAlertRuleRequest>,
) -> Result<Json<ApiResponse<serde_json::Value>>> {
    let rule = state
        .audit_service()
        .update_alert_rule(rule_id, request)
        .await?;

    Ok(Json(ApiResponse::success(serde_json::json!(rule))))
}

/// 清理过期审计日志
/// POST /api/v1/admin/audit/cleanup
#[derive(Debug, Deserialize)]
pub struct CleanupRequest {
    pub days: i64,
    pub archive: Option<bool>,
    pub archive_dir: Option<String>,
}

pub async fn cleanup_audit_logs(
    State(state): State<Arc<AppState>>,
    Json(request): Json<CleanupRequest>,
) -> Result<Json<ApiResponse<serde_json::Value>>> {
    let before = chrono::Utc::now() - chrono::Duration::days(request.days);

    let deleted = if request.archive.unwrap_or(false) {
        // 先归档再删除
        state
            .audit_service()
            .archive_old_logs(before, request.archive_dir.as_deref())
            .await?
    } else {
        // 直接清理
        state.audit_service().cleanup_old_logs(before).await?
    };

    Ok(Json(ApiResponse::success(serde_json::json!({
        "deleted": deleted,
        "before": before,
        "archived": request.archive.unwrap_or(false)
    }))))
}
