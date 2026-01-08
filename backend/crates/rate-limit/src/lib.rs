pub enum RateLimitError {
    Error(String),
    Exceeded,
}

pub trait RateLimit {
    fn check(
        &self,
        rate_key: &str,
        max: i32,
        points: i32,
        window_in_seconds: i32,
    ) -> impl Future<Output = Result<i32, RateLimitError>> + Send;
}
