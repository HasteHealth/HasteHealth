//! Checks what a `ClientApplication` registers for `private_key_jwt` client
//! authentication as it is written, so a mistake shows up on the write rather
//! than as `invalid_client` at the token endpoint later.

use crate::{
    auth_n::oidc::client_assertion::jwks::parse_jwks,
    fhir_client::{
        ClientState, ServerCTX,
        middleware::{Backend, Middleware, Next, Output},
    },
    safe_fetch::public_https_url,
};
use haste_fhir_client::request::{FHIRRequest, UpdateRequest};
use haste_fhir_model::r4::generated::{
    resources::{ClientApplication, Resource},
    terminology::IssueType,
};
use haste_fhir_operation_error::OperationOutcomeError;
use std::sync::Arc;

fn value(string: Option<&str>) -> Option<&str> {
    string.map(str::trim).filter(|value| !value.is_empty())
}

/// `jwks` must be a JWK Set with a `kid` on every key, and `jwksUri` an
/// `https` URL the server could fetch. Whether the host is public is checked
/// when it is fetched, since what a name resolves to can change.
fn validate(client: &ClientApplication) -> Result<(), OperationOutcomeError> {
    if let Some(jwks) = value(client.jwks.as_ref().and_then(|jwks| jwks.value.as_deref())) {
        parse_jwks(jwks.as_bytes()).map_err(|_| {
            OperationOutcomeError::error(
                IssueType::invalid(),
                "ClientApplication.jwks must be a JSON Web Key Set document with at least one key, each with a kid."
                    .to_string(),
            )
        })?;
    }

    if let Some(uri) = value(client.jwksUri.as_ref().and_then(|uri| uri.value.as_deref())) {
        public_https_url(uri).map_err(|_| {
            OperationOutcomeError::error(
                IssueType::invalid(),
                "ClientApplication.jwksUri must be an https URL with a host name and no credentials."
                    .to_string(),
            )
        })?;
    }

    Ok(())
}

/// Refuses a `ClientApplication` whose registered keys could never
/// authenticate a client.
pub struct ClientApplicationValidation;

impl<B: Backend> Middleware<B> for ClientApplicationValidation {
    fn call<'a>(
        &'a self,
        state: &'a ClientState<B>,
        ctx: Arc<ServerCTX<B>>,
        request: FHIRRequest,
        next: Next<'a, B>,
    ) -> Output<'a> {
        let written = match &request {
            FHIRRequest::Create(create) => Some(&create.resource),
            FHIRRequest::Update(UpdateRequest::Instance(update)) => Some(&update.resource),
            FHIRRequest::Update(UpdateRequest::Conditional(update)) => Some(&update.resource),
            _ => None,
        };

        if let Some(Resource::ClientApplication(client)) = written
            && let Err(error) = validate(client)
        {
            return Box::pin(std::future::ready(Err(error)));
        }

        next.run(state, ctx, request)
    }
}
