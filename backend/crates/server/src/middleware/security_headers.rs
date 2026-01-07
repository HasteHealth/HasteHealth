use axum::response::IntoResponse;
use axum::{body::Body, extract::Request, response::Response};
use std::pin::Pin;
use std::task::{Context, Poll};
use tower::{Layer, Service};

#[derive(Clone)]
pub struct SecurityHeaderLayer {}

impl<S> Layer<S> for SecurityHeaderLayer {
    type Service = SecurityHeaderService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        SecurityHeaderService { inner }
    }
}

impl SecurityHeaderLayer {
    pub fn new() -> Self {
        SecurityHeaderLayer {}
    }
}

#[derive(Clone)]
pub struct SecurityHeaderService<S> {
    inner: S,
}

impl<'a, T> Service<Request<Body>> for SecurityHeaderService<T>
where
    T: Service<Request, Response = Response> + Send + 'static + Clone,
    T::Future: Send + 'static,
    T::Error: IntoResponse,
{
    type Response = T::Response;
    type Error = T::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, request: Request) -> Self::Future {
        // https://docs.rs/tower/latest/tower/trait.Service.html#be-careful-when-cloning-inner-services
        let clone = self.inner.clone();
        // take the service that was ready
        let mut inner = std::mem::replace(&mut self.inner, clone);

        Box::pin(async move {
            let future = inner.call(request);
            let mut response: Response = future.await?;
            let headers = response.headers_mut();

            // Most of these headers are pulled from default helmet in other products.
            headers.insert("Content-Security-Policy" ,"default-src 'self';base-uri 'self';font-src 'self' https: data:;frame-ancestors 'self';img-src 'self' data:;object-src 'none';script-src 'self';script-src-attr 'none';style-src 'self' https: 'unsafe-inline';upgrade-insecure-requests".parse().unwrap());
            headers.insert("Cross-Origin-Opener-Policy", "same-origin".parse().unwrap());
            headers.insert(
                "Cross-Origin-Resource-Policy",
                "same-origin".parse().unwrap(),
            );
            headers.insert("Origin-Agent-Cluster", "?1".parse().unwrap());
            headers.insert("Referrer-Policy", "no-referrer".parse().unwrap());
            headers.insert(
                "Strict-Transport-Security",
                "max-age=31536000; includeSubDomains".parse().unwrap(),
            );
            headers.insert("X-Content-Type-Options", "nosniff".parse().unwrap());
            headers.insert("X-DNS-Prefetch-Control", "off".parse().unwrap());
            headers.insert("X-Download-Options", "noopen".parse().unwrap());
            headers.insert("X-Frame-Options", "SAMEORIGIN".parse().unwrap());
            headers.insert(
                "X-Permitted-Cross-Domain-Policies",
                " none".parse().unwrap(),
            );
            headers.insert("X-Powered-By", "0".parse().unwrap());

            Ok(response)
        })
    }
}
