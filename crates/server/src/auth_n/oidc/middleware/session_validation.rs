use axum::RequestExt;
use axum::response::{IntoResponse, Redirect};
use axum::{body::Body, extract::Request, response::Response};
use std::pin::Pin;
use std::task::{Context, Poll};
use tower::Service;
use tower_sessions::Session;

use crate::auth_n::session;

struct SessionValidation<T> {
    inner: T,
    redirect_url: String,
}

impl<'a, T> Service<Request<Body>> for SessionValidation<T>
where
    T: Service<Request, Response = Response> + Send + 'static + Clone,
    T::Future: Send + 'static,
{
    type Response = Response;
    type Error = T::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, mut request: Request) -> Self::Future {
        // https://docs.rs/tower/latest/tower/trait.Service.html#be-careful-when-cloning-inner-services
        let clone = self.inner.clone();
        // take the service that was ready
        let mut inner = std::mem::replace(&mut self.inner, clone);
        let redirect_url = self.redirect_url.clone();

        // Return the response as an immediate future
        Box::pin(async move {
            let current_session = request
                .extract_parts::<Session>()
                .await
                .expect("Could not extract session.");

            if let Ok(Some(user)) = session::user::get_user(current_session).await {
                let response = inner.call(request).await?;
                Ok(response)
            } else {
                let redirect = Redirect::to(&redirect_url);
                Ok(redirect.into_response())
            }
        })
    }
}
