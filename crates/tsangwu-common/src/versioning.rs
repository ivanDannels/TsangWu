use axum::{
    extract::Request,
    http::{header, StatusCode},
    middleware::Next,
    response::Response,
    Router,
};

/// API 版本
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApiVersion {
    V1,
    V2,
}

impl ApiVersion {
    /// 从请求头解析版本
    pub fn from_header(value: &str) -> Option<Self> {
        match value {
            "application/vnd.tsangwu.v1+json" => Some(Self::V1),
            "application/vnd.tsangwu.v2+json" => Some(Self::V2),
            _ => None,
        }
    }

    /// 从路径前缀解析版本
    pub fn from_path(path: &str) -> Option<Self> {
        if path.starts_with("/v1/") || path.starts_with("/api/v1/") {
            Some(Self::V1)
        } else if path.starts_with("/v2/") || path.starts_with("/api/v2/") {
            Some(Self::V2)
        } else {
            None
        }
    }

    /// 获取版本字符串
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::V1 => "v1",
            Self::V2 => "v2",
        }
    }
}

impl Default for ApiVersion {
    fn default() -> Self {
        Self::V1
    }
}

/// 版本检测中间件
pub async fn version_middleware(mut req: Request, next: Next) -> Result<Response, StatusCode> {
    // 优先从 Accept 头获取版本
    let version = req
        .headers()
        .get(header::ACCEPT)
        .and_then(|v| v.to_str().ok())
        .and_then(ApiVersion::from_header)
        .or_else(|| {
            // 其次从路径获取版本
            ApiVersion::from_path(req.uri().path())
        })
        .unwrap_or_default();

    // 将版本信息添加到请求扩展
    req.extensions_mut().insert(version);

    Ok(next.run(req).await)
}

/// 版本路由构建器
pub struct VersionedRouter {
    v1: Option<Router>,
    v2: Option<Router>,
}

impl VersionedRouter {
    pub fn new() -> Self {
        Self {
            v1: None,
            v2: None,
        }
    }

    /// 添加 v1 路由
    pub fn v1(mut self, router: Router) -> Self {
        self.v1 = Some(router);
        self
    }

    /// 添加 v2 路由
    pub fn v2(mut self, router: Router) -> Self {
        self.v2 = Some(router);
        self
    }

    /// 构建最终路由
    pub fn build(self) -> Router {
        let mut router = Router::new();

        if let Some(v1) = self.v1 {
            router = router.nest("/v1", v1);
        }

        if let Some(v2) = self.v2 {
            router = router.nest("/v2", v2);
        }

        router
    }
}

impl Default for VersionedRouter {
    fn default() -> Self {
        Self::new()
    }
}

/// 版本兼容性检查宏
#[macro_export]
macro_rules! check_version {
    ($req:expr, $min_version:expr) => {
        {
            let version = $req.extensions().get::<$crate::ApiVersion>()
                .copied()
                .unwrap_or_default();

            if version < $min_version {
                return Err($crate::AppError::Validation(
                    format!("此功能需要 API 版本 {} 或更高", $min_version.as_str())
                ));
            }
        }
    };
}
