use std::pin::Pin;

pub enum RateLimitError {
    Error(String),
    Exceeded,
}

pub trait RateLimit: Sync + Send {
    fn check<'a>(
        &'a self,
        rate_key: &'a str,
        max: i32,
        points: i32,
        window_in_seconds: i32,
    ) -> Pin<Box<dyn Future<Output = Result<i32, RateLimitError>> + Send + 'a>>;
}
