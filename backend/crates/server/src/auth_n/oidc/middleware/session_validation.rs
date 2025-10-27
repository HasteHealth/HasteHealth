use axum::RequestExt;
use axum::extract::OriginalUri;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Redirect};
use axum::{body::Body, extract::Request, response::Response};
use axum_extra::extract::Cached;
use std::pin::Pin;
use std::task::{Context, Poll};
use tower::{Layer, Service};
use tower_sessions::Session;

use crate::auth_n::oidc::routes::route_string::oidc_route_string;
use crate::auth_n::session;
use crate::extract::path_tenant::{ProjectIdentifier, TenantIdentifier};

#[derive(Clone)]
pub struct AuthSessionValidationLayer {
    to: &'static str,
}

impl<S> Layer<S> for AuthSessionValidationLayer {
    type Service = AuthSessionValidationService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        AuthSessionValidationService { inner, to: self.to }
    }
}

impl AuthSessionValidationLayer {
    pub fn new(to: &'static str) -> Self {
        AuthSessionValidationLayer { to }
    }
}

#[derive(Clone)]
pub struct AuthSessionValidationService<T> {
    inner: T,
    to: &'static str,
}

impl<'a, T> Service<Request<Body>> for AuthSessionValidationService<T>
where
    T: Service<Request, Response = Response> + Send + 'static + Clone,
    T::Future: Send + 'static,
    T::Error: IntoResponse,
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
        let to = self.to;

        // Return the response as an immediate future
        Box::pin(async move {
            let Ok(Cached(TenantIdentifier { tenant })) =
                request.extract_parts::<Cached<TenantIdentifier>>().await
            else {
                return Ok((
                    StatusCode::BAD_REQUEST,
                    "Tenant id not found on request".to_string(),
                )
                    .into_response());
            };
            let Ok(Cached(ProjectIdentifier { project })) =
                request.extract_parts::<Cached<ProjectIdentifier>>().await
            else {
                return Ok((
                    StatusCode::BAD_REQUEST,
                    "Project id not found on request".to_string(),
                )
                    .into_response());
            };

            let Cached(current_session) = request
                .extract_parts::<Cached<Session>>()
                .await
                .expect("Could not extract session.");

            let to_route = oidc_route_string(&tenant, &project, &to);

            if let Ok(Some(user)) = session::user::get_user(&current_session, &tenant).await
                && user.tenant == tenant
            {
                let response = inner.call(request).await?;
                Ok(response)
            } else {
                let uri = request
                    .extract_parts::<OriginalUri>()
                    .await
                    .expect("Could not extract original URI.");
                let login_redirect = Redirect::to(
                    &(to_route
                        .to_str()
                        .expect("Failed to create to route.")
                        .to_string()
                        + "?"
                        + uri.query().unwrap_or("")),
                );

                Ok(login_redirect.into_response())
            }
        })
    }
}
