use axum::RequestExt;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{body::Body, extract::Request, response::Response};
use axum::{body::to_bytes, extract::Query};
use serde::Deserialize;
use std::sync::Arc;
use std::task::{Context, Poll};
use std::{collections::HashMap, pin::Pin};
use tower::{Layer, Service};

#[derive(Deserialize, Clone, Debug)]
pub struct OIDCParameters(pub HashMap<String, String>);

#[derive(Clone, Debug)]
pub struct ParameterConfig {
    pub allowed_params: Vec<String>,
    pub optional_params: Vec<String>,
    pub allow_launch_params: bool,
}

#[derive(Clone)]
pub struct ParameterInjectLayer {
    state: Arc<ParameterConfig>,
}

impl<S> Layer<S> for ParameterInjectLayer {
    type Service = ParameterInjectService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        ParameterInjectService {
            inner,
            state: self.state.clone(),
        }
    }
}
impl ParameterInjectLayer {
    pub fn new(state: ParameterConfig) -> Self {
        ParameterInjectLayer {
            state: Arc::new(state),
        }
    }
}

#[derive(Clone)]
pub struct ParameterInjectService<S> {
    inner: S,
    state: Arc<ParameterConfig>,
}

impl<'a, T> Service<Request<Body>> for ParameterInjectService<T>
where
    T: Service<Request, Response = Response> + Send + 'static + Clone,
    T::Future: Send + 'static,
{
    type Response = T::Response;
    type Error = T::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut request: Request) -> Self::Future {
        // https://docs.rs/tower/latest/tower/trait.Service.html#be-careful-when-cloning-inner-services
        let clone = self.inner.clone();
        // take the service that was ready
        let mut inner = std::mem::replace(&mut self.inner, clone);
        let parameter_config = self.state.clone();

        Box::pin(async move {
            println!("{:?}", parameter_config);

            let query_params = request
                .extract_parts::<Query<HashMap<String, String>>>()
                .await;

            let Ok(query_params) = query_params else {
                return Ok((StatusCode::BAD_REQUEST, "".to_string()).into_response());
            };

            let (parts, body) = request.into_parts();
            let bytes = to_bytes(body, 10000).await;
            let Ok(bytes) = bytes else {
                return Ok((
                    StatusCode::BAD_REQUEST,
                    "Body was to large size limit 10k bytes".to_string(),
                )
                    .into_response());
            };

            let oidc_params = serde_json::from_slice::<OIDCParameters>(&bytes)
                .unwrap_or_else(|_e| OIDCParameters(query_params.0));

            let new_body = Body::from(bytes);
            let mut new_request = Request::from_parts(parts, new_body);
            new_request.extensions_mut().insert(oidc_params);

            let future = inner.call(new_request);
            let response: Response = future.await?;
            Ok(response)
        })
    }
}
