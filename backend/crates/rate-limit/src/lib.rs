use std::pin::Pin;

pub enum RateLimitError {
    Error(String),
    Exceeded,
}

pub trait RateLimit: Sync + Send {
    /// Spends `points` against `rate_key` and returns the running total, or
    /// [`RateLimitError::Exceeded`] once it would pass `max`. May be served
    /// from a short-lived in-memory cache, so concurrent callers can each read
    /// the same pre-spend total and slip past the cap by a small margin before
    /// the backing store catches up.
    fn check<'a>(
        &'a self,
        rate_key: &'a str,
        max: i32,
        points: i32,
        window_in_seconds: i32,
    ) -> Pin<Box<dyn Future<Output = Result<i32, RateLimitError>> + Send + 'a>>;

    /// Like [`RateLimit::check`], but always consults the backing store rather
    /// than any in-memory cache, so every concurrent call is counted exactly.
    /// Use it to gate security-sensitive actions where the cache's slack would
    /// let parallel requests exceed the cap, such as verification-code guesses.
    /// The default delegates to [`RateLimit::check`]; back it with a serialized
    /// store operation in real implementations.
    fn check_uncached<'a>(
        &'a self,
        rate_key: &'a str,
        max: i32,
        points: i32,
        window_in_seconds: i32,
    ) -> Pin<Box<dyn Future<Output = Result<i32, RateLimitError>> + Send + 'a>> {
        self.check(rate_key, max, points, window_in_seconds)
    }
}
