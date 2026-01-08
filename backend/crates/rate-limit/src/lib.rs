pub enum RateLimitError {
    Exceeded,
}

pub trait RateLimit {
    fn check(
        &self,
        rate_key: &str,
        max: i32,
        points: i32,
        window_in_seconds: i32,
    ) -> impl Future<Output = Result<(), RateLimitError>> + Send;
}
