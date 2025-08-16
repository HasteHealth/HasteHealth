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
pub struct OIDCParameters {
    pub parameters: HashMap<String, String>,
    pub launch_parameters: Option<HashMap<String, String>>,
}

#[derive(Clone, Debug)]
pub struct ParameterConfig {
    pub required_parameters: Vec<String>,
    pub optional_parameters: Vec<String>,
    pub allow_launch_parameters: bool,
}

#[derive(Clone)]
pub struct OIDCParameterInjectLayer {
    state: Arc<ParameterConfig>,
}

impl<S> Layer<S> for OIDCParameterInjectLayer {
    type Service = OIDCParameterInjectService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        OIDCParameterInjectService {
            inner,
            state: self.state.clone(),
        }
    }
}

impl OIDCParameterInjectLayer {
    pub fn new(state: Arc<ParameterConfig>) -> Self {
        OIDCParameterInjectLayer { state: state }
    }
}

#[derive(Clone)]
pub struct OIDCParameterInjectService<S> {
    inner: S,
    state: Arc<ParameterConfig>,
}

fn validate_parameter(param_name: &str, param_value: &str) -> Result<(), String> {
    if param_value.is_empty() {
        return Err("Parameter cannot be empty".to_string());
    }

    if param_name == "response_type" && !["code", "token"].contains(&param_value) {
        return Err(format!("Invalid response_type: {}", param_value));
    }

    Ok(())
}

impl<'a, T> Service<Request<Body>> for OIDCParameterInjectService<T>
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
            // Either check the body if serializes or check the query params.
            let unvalidated_parameters = serde_json::from_slice::<HashMap<String, String>>(&bytes)
                .unwrap_or_else(|_e| query_params.0);

            let mut oidc_parameters = OIDCParameters {
                parameters: HashMap::new(),
                launch_parameters: None,
            };

            // Check for required parameters
            for (is_required, parameter_name) in parameter_config
                .required_parameters
                .iter()
                .map(|p| (true, p))
                .chain(
                    parameter_config
                        .optional_parameters
                        .iter()
                        .map(|p| (false, p)),
                )
            {
                if let Some(parameter_value) = unvalidated_parameters.get(parameter_name) {
                    if let Err(_e) = validate_parameter(parameter_name, parameter_value) {
                        return Ok((
                            StatusCode::BAD_REQUEST,
                            format!("Invalid parameter: '{}'", parameter_name),
                        )
                            .into_response());
                    }

                    oidc_parameters
                        .parameters
                        .insert(parameter_name.clone(), parameter_value.clone());
                } else if is_required {
                    return Ok((
                        StatusCode::BAD_REQUEST,
                        format!("Missing required parameter: '{parameter_name}'",),
                    )
                        .into_response());
                }
            }
            // Launch parameters are for SMART apps e.g. launch/patient
            if parameter_config.allow_launch_parameters {
                let mut launch_parameters = HashMap::new();
                for launch_param_name in unvalidated_parameters
                    .keys()
                    .filter(|k| k.starts_with("launch/"))
                {
                    let launch_param_value = unvalidated_parameters
                        .get(launch_param_name)
                        .cloned()
                        .unwrap_or_default();

                    let parts = launch_param_name.split('/').collect::<Vec<_>>();
                    if parts.len() != 2 {
                        return Ok((
                            StatusCode::BAD_REQUEST,
                            format!("Invalid launch parameter: '{launch_param_name}'"),
                        )
                            .into_response());
                    }

                    launch_parameters.insert(launch_param_name.to_string(), launch_param_value);
                }
                oidc_parameters.launch_parameters = Some(launch_parameters);
            }

            let new_body = Body::from(bytes);
            let mut new_request = Request::from_parts(parts, new_body);
            new_request.extensions_mut().insert(oidc_parameters);

            let future = inner.call(new_request);
            let response: Response = future.await?;
            Ok(response)
        })
    }
}
