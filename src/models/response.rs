use serde::Serialize;

/// 统一的 API 响应结构
/// 
/// 所有 API 端点都应使用此结构返回响应，确保响应格式一致性
/// 
/// # 示例
/// ```json
/// {
///     "success": true,
///     "data": { ... },
///     "message": "操作成功"
/// }
/// ```
#[derive(Debug, Clone, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

impl<T: Serialize> ApiResponse<T> {
    /// 创建成功响应（带数据）
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            message: None,
        }
    }

    /// 创建成功响应（带消息）
    pub fn success_with_message(message: impl Into<String>) -> Self {
        Self {
            success: true,
            data: None,
            message: Some(message.into()),
        }
    }

    /// 创建成功响应（带数据和消息）
    pub fn success_with_message_and_data(data: T, message: impl Into<String>) -> Self {
        Self {
            success: true,
            data: Some(data),
            message: Some(message.into()),
        }
    }

    /// 创建失败响应
    pub fn error(message: impl Into<String>) -> Self {
        Self {
            success: false,
            data: None,
            message: Some(message.into()),
        }
    }
}

/// 分页数据响应
#[derive(Debug, Clone, Serialize)]
pub struct PaginatedData<T> {
    pub items: Vec<T>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

impl<T> PaginatedData<T> {
    pub fn new(items: Vec<T>, total: i64, limit: i64, offset: i64) -> Self {
        Self {
            items,
            total,
            limit,
            offset,
        }
    }
}
