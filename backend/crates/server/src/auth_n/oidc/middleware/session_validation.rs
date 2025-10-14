use axum::RequestExt;
use axum::extract::OriginalUri;
use axum::response::{IntoResponse, Redirect};
use axum::{body::Body, extract::Request, response::Response};
use std::pin::Pin;
use std::task::{Context, Poll};
use tower::{Layer, Service};
use tower_sessions::Session;

use crate::auth_n::session;

#[derive(Clone)]
pub struct AuthSessionValidationLayer {
    from: &'static str,
    to: &'static str,
}

impl<S> Layer<S> for AuthSessionValidationLayer {
    type Service = AuthSessionValidationService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        AuthSessionValidationService {
            inner,
            from: self.from,
            to: self.to,
        }
    }
}

impl AuthSessionValidationLayer {
    pub fn new(from: &'static str, to: &'static str) -> Self {
        AuthSessionValidationLayer { from, to }
    }
}

#[derive(Clone)]
pub struct AuthSessionValidationService<T> {
    inner: T,
    from: &'static str,
    to: &'static str,
}

impl<'a, T> Service<Request<Body>> for AuthSessionValidationService<T>
where
    T: Service<Request, Response = Response> + Send + 'static + Clone,
    T::Future: Send + 'static,
{
    type Response = Response;
    type Error = T::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, mut request: Request) -> Self::Future {
        // https://docs.rs/tower/latest/tower/trait.Service.html#be-careful-when-cloning-inner-services
        let clone = self.inner.clone();
        // take the service that was ready
        let mut inner = std::mem::replace(&mut self.inner, clone);
        let from = self.from;
        let to = self.to;

        // Return the response as an immediate future
        Box::pin(async move {
            let current_session = request
                .extract_parts::<Session>()
                .await
                .expect("Could not extract session.");

            if let Ok(Some(_user)) = session::user::get_user(&current_session).await {
                let response = inner.call(request).await?;
                Ok(response)
            } else {
                let uri = request
                    .extract_parts::<OriginalUri>()
                    .await
                    .expect("Could not extract original URI.");
                let login_redirect = Redirect::to(
                    &(uri.path().to_string().replace(&from, &to) + "?" + uri.query().unwrap_or("")),
                );

                Ok(login_redirect.into_response())
            }
        })
    }
}
