pub mod error;
pub mod pagination;
pub mod protection;
pub mod response;
pub mod trace;
pub mod types;
pub mod versioning;

pub use error::AppError;
pub use pagination::{PageParams, PageResult};
pub use protection::{CircuitBreaker, CircuitBreakerConfig, RateLimiter, RateLimiterConfig};
pub use response::ApiResponse;
pub use trace::{trace_middleware, TraceId, TRACE_ID_HEADER};
pub use versioning::{version_middleware, ApiVersion, VersionedRouter};
