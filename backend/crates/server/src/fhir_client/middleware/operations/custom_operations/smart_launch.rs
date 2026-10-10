//! `ClientApplication/{id}/$launch`: starts a SMART App Launch EHR launch.
//!
//! Haste Health plays the EHR. The caller names a patient (and optionally one
//! of their encounters) and gets back an opaque `launch` value and the
//! client's `launchUri` with `iss` and `launch` added to its query. Opening
//! that URL starts the app, which passes `launch` back when it asks for
//! authorization.
//!
//! A launch is an authorization code of kind `smart_launch` holding the
//! context in its `meta`, so it expires like any other code. It is tied to the
//! user who started it, since the same user must then authorize the app; a
//! client signed in on its own cannot start one.

use crate::{
    fhir_client::{
        ServerCTX,
        middleware::{ServerMiddlewareState, operations::ServerOperationContext},
    },
    route_path::api_fhir_root_url,
};
use haste_fhir_client::{FHIRClient, request::InvocationRequest};
use haste_fhir_generated_ops::generated::HasteHealthSmartLaunch;
use haste_fhir_model::r4::generated::{
    resources::{Encounter, Resource, ResourceType},
    terminology::IssueType,
    types::{FHIRString, FHIRUri, Reference},
};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_fhir_ops::OperationExecutor;
use haste_fhir_search::SearchEngine;
use haste_fhir_terminology::FHIRTerminology;
use haste_jwt::{AuthorId, AuthorKind, ProjectId, ResourceId, TenantId};
use haste_repository::{
    Repository,
    admin::ProjectModelAdmin,
    types::authorization_code::{AuthorizationCodeKind, CreateAuthorizationCode},
};
use serde_json::json;
use std::{sync::Arc, time::Duration};
use url::Url;

/// How long a launch can be redeemed. An app is opened and asks for
/// authorization within seconds, so this matches an authorization code.
const LAUNCH_EXPIRY: Duration = Duration::from_secs(5 * 60);

fn invalid(message: impl Into<String>) -> OperationOutcomeError {
    OperationOutcomeError::error(IssueType::invalid(), message.into())
}

fn not_found(message: impl Into<String>) -> OperationOutcomeError {
    OperationOutcomeError::error(IssueType::not_found(), message.into())
}

/// The id `reference` points at, which must be a relative reference to a
/// `resource_type`: `123` for `Patient/123`.
fn reference_id(
    reference: &Reference,
    resource_type: ResourceType,
    parameter: &str,
) -> Result<String, OperationOutcomeError> {
    let value = reference
        .reference
        .as_ref()
        .and_then(|reference| reference.value.as_deref())
        .unwrap_or_default();

    match value.split_once('/') {
        Some((type_, id))
            if type_ == resource_type.as_ref() && !id.is_empty() && !id.contains('/') =>
        {
            Ok(id.to_string())
        }
        _ => Err(invalid(format!(
            "'{parameter}' must reference a {type_}, as {type_}/<id>.",
            type_ = resource_type.as_ref()
        ))),
    }
}

/// Whether `encounter` is one of `patient_id`'s.
fn belongs_to_patient(encounter: &Encounter, patient_id: &str) -> bool {
    encounter
        .subject
        .as_ref()
        .and_then(|subject| subject.reference.as_ref())
        .and_then(|reference| reference.value.as_deref())
        == Some(format!("Patient/{patient_id}").as_str())
}

/// `launch_uri` parsed, so `iss` and `launch` can be added to its query. It
/// is opened in a browser, so only an absolute http or https URL will do.
fn parse_launch_uri(launch_uri: &str) -> Result<Url, OperationOutcomeError> {
    Url::parse(launch_uri)
        .ok()
        .filter(|url| matches!(url.scheme(), "http" | "https"))
        .ok_or_else(|| invalid("The client's launchUri must be an absolute http or https URL."))
}

pub fn smart_launch_op<
    Repo: Repository + Send + Sync + 'static,
    Search: SearchEngine + Send + Sync + 'static,
    Terminology: FHIRTerminology + Send + Sync + 'static,
    Client: FHIRClient<Arc<ServerCTX<Client>>, OperationOutcomeError> + 'static,
>() -> OperationExecutor<
    ServerOperationContext<ServerMiddlewareState<Repo, Search, Terminology>, Client>,
    HasteHealthSmartLaunch::Input,
    HasteHealthSmartLaunch::Output,
> {
    OperationExecutor::new(
        HasteHealthSmartLaunch::CODE.to_string(),
        Box::new(
            |context: ServerOperationContext<
                ServerMiddlewareState<Repo, Search, Terminology>,
                Client,
            >,
             tenant: TenantId,
             project: ProjectId,
             request: &InvocationRequest,
             input: HasteHealthSmartLaunch::Input| {
                let request = request.clone();

                Box::pin(async move {
                    let client_app_id = match &request {
                        InvocationRequest::Instance(instance)
                            if instance.resource_type == ResourceType::ClientApplication =>
                        {
                            instance.id.clone()
                        }
                        _ => {
                            return Err(invalid(
                                "$launch is invoked on a client: ClientApplication/<id>/$launch.",
                            ));
                        }
                    };

                    // The user who starts a launch is the one who must then
                    // authorize the app, so a client on its own cannot.
                    let claims = &context.ctx.user.claims;
                    if claims.resource_type != AuthorKind::Membership
                        || !matches!(claims.sub, AuthorId::User(_))
                    {
                        return Err(OperationOutcomeError::error(
                            IssueType::forbidden(),
                            "Only a signed-in user can start a launch: the same user must then \
                             authorize the app."
                                .to_string(),
                        ));
                    }

                    // The invocation itself was authorized, and a user starting
                    // a launch need not be able to read client registrations,
                    // which hold secrets.
                    let Some(Resource::ClientApplication(client_app)) = context
                        .state
                        .repo
                        .read_latest(
                            &tenant,
                            &project,
                            &ResourceType::ClientApplication,
                            &ResourceId::new(client_app_id.clone()),
                        )
                        .await?
                    else {
                        return Err(not_found(format!(
                            "ClientApplication/{client_app_id} was not found."
                        )));
                    };

                    let launch_uri = client_app
                        .launchUri
                        .as_ref()
                        .and_then(|uri| uri.value.as_deref())
                        .ok_or_else(|| {
                            invalid(format!(
                                "ClientApplication/{client_app_id} has no launchUri to open the \
                                 app at."
                            ))
                        })?;
                    let mut url = parse_launch_uri(launch_uri)?;

                    // Read as the caller, so their scopes and access policies
                    // decide whether they may launch an app for this patient.
                    let patient_id =
                        reference_id(&input.patient, ResourceType::Patient, "patient")?;
                    if context
                        .ctx
                        .client
                        .read(
                            context.ctx.clone(),
                            ResourceType::Patient,
                            patient_id.clone(),
                        )
                        .await?
                        .is_none()
                    {
                        return Err(not_found(format!("Patient/{patient_id} was not found.")));
                    }

                    let encounter_id = match &input.encounter {
                        None => None,
                        Some(reference) => {
                            let encounter_id =
                                reference_id(reference, ResourceType::Encounter, "encounter")?;
                            let Some(Resource::Encounter(encounter)) = context
                                .ctx
                                .client
                                .read(
                                    context.ctx.clone(),
                                    ResourceType::Encounter,
                                    encounter_id.clone(),
                                )
                                .await?
                            else {
                                return Err(not_found(format!(
                                    "Encounter/{encounter_id} was not found."
                                )));
                            };
                            if !belongs_to_patient(&encounter, &patient_id) {
                                return Err(invalid(format!(
                                    "Encounter/{encounter_id} is not Patient/{patient_id}'s."
                                )));
                            }
                            Some(encounter_id)
                        }
                    };

                    let iss = api_fhir_root_url(&context.state.config.api_uri, &tenant, &project)?;

                    let mut launch_context = json!({ "patient": patient_id });
                    if let Some(encounter_id) = encounter_id {
                        launch_context["encounter"] = json!(encounter_id);
                    }

                    let launch = ProjectModelAdmin::create(
                        context.state.repo.as_ref(),
                        &tenant,
                        &project,
                        CreateAuthorizationCode {
                            membership: claims.membership.clone(),
                            expires_in: LAUNCH_EXPIRY,
                            kind: AuthorizationCodeKind::SmartLaunch,
                            user_id: claims.sub.as_ref().to_string(),
                            client_id: Some(client_app_id),
                            pkce_code_challenge: None,
                            pkce_code_challenge_method: None,
                            redirect_uri: None,
                            meta: Some(sqlx::types::Json(launch_context)),
                        },
                    )
                    .await?;

                    url.query_pairs_mut()
                        .append_pair("iss", iss.as_str())
                        .append_pair("launch", &launch.code);

                    Ok(HasteHealthSmartLaunch::Output {
                        launch: FHIRString {
                            value: Some(launch.code),
                            ..Default::default()
                        },
                        iss: FHIRUri {
                            value: Some(iss.to_string()),
                            ..Default::default()
                        },
                        url: FHIRUri {
                            value: Some(url.to_string()),
                            ..Default::default()
                        },
                    })
                })
            },
        ),
    )
}
