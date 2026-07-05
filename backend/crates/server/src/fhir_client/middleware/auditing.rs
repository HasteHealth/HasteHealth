use crate::fhir_client::{
    ServerCTX,
    middleware::{
        ServerMiddlewareContext, ServerMiddlewareNext, ServerMiddlewareOutput,
        ServerMiddlewareState,
    },
};
use haste_fhir_client::{
    FHIRClient,
    middleware::MiddlewareChain,
    request::{
        DeleteRequest, DeleteResponse, FHIRRequest, FHIRResponse, HistoryRequest,
        InvocationRequest, InvokeResponse, SearchRequest, SearchResponse, UpdateRequest,
    },
};
use haste_fhir_model::r4::{
    datetime::Instant,
    generated::{
        resources::{
            AuditEvent, AuditEventAgent, AuditEventEntity, AuditEventEntityDetail,
            AuditEventEntityDetailValueTypeChoice, AuditEventSource, Resource,
        },
        terminology::{self, AuditEventSubType},
        types::{Coding, FHIRBoolean, FHIRInstant, FHIRString, FHIRUri, Reference},
    },
};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_fhir_search::SearchEngine;
use haste_fhir_terminology::FHIRTerminology;
use haste_repository::{Repository, fhir::FHIRRepository};
use std::sync::Arc;
use tracing::instrument;

struct Audit<'a, Client: FHIRClient<Arc<ServerCTX<Client>>, OperationOutcomeError>>(
    Arc<ServerCTX<Client>>,
    &'a FHIRRequest,
);

fn coding(system: &str, code: Option<String>, display: &str) -> Box<Coding> {
    Box::new(Coding {
        system: Some(Box::new(FHIRUri::from(system.to_string()))),
        code: code.map(|code| Box::new(code.into())),
        display: Some(Box::new(FHIRString::from(display.to_string()))),
        ..Default::default()
    })
}

fn detail(name: &str, value: String) -> AuditEventEntityDetail {
    AuditEventEntityDetail {
        type_: Box::new(FHIRString::from(name.to_string())),
        value: AuditEventEntityDetailValueTypeChoice::String(Box::new(FHIRString::from(value))),
        ..Default::default()
    }
}

fn response_kind(response: Option<&FHIRResponse>) -> String {
    match response {
        Some(FHIRResponse::Create(_)) => "create".to_string(),
        Some(FHIRResponse::Read(_)) => "read".to_string(),
        Some(FHIRResponse::VersionRead(_)) => "version-read".to_string(),
        Some(FHIRResponse::Update(_)) => "update".to_string(),
        Some(FHIRResponse::Patch(_)) => "patch".to_string(),
        Some(FHIRResponse::Delete(delete)) => match delete {
            DeleteResponse::Instance(_) => "delete-instance".to_string(),
            DeleteResponse::Type(_) => "delete-type".to_string(),
            DeleteResponse::System(_) => "delete-system".to_string(),
        },
        Some(FHIRResponse::Capabilities(_)) => "capabilities".to_string(),
        Some(FHIRResponse::Search(search)) => match search {
            SearchResponse::Type(_) => "search-type".to_string(),
            SearchResponse::System(_) => "search-system".to_string(),
        },
        Some(FHIRResponse::History(_)) => "history".to_string(),
        Some(FHIRResponse::Invoke(invoke)) => match invoke {
            InvokeResponse::Instance(_) => "invoke-instance".to_string(),
            InvokeResponse::Type(_) => "invoke-type".to_string(),
            InvokeResponse::System(_) => "invoke-system".to_string(),
        },
        Some(FHIRResponse::Batch(_)) => "batch".to_string(),
        Some(FHIRResponse::Transaction(_)) => "transaction".to_string(),
        None => "none".to_string(),
    }
}

fn failure_response_type(error: &OperationOutcomeError) -> String {
    error
        .outcome()
        .issue
        .first()
        .and_then(|issue| {
            let code: Option<String> = issue.code.as_ref().into();
            code
        })
        .unwrap_or_else(|| "unknown".to_string())
}

fn add_response_outcome_to_audit_event<
    Client: FHIRClient<Arc<ServerCTX<Client>>, OperationOutcomeError>,
>(
    event: &mut AuditEvent,
    result: &Result<ServerMiddlewareContext<Client>, OperationOutcomeError>,
) {
    let (success, response_type, failure_type, outcome_desc, outcome) = match result {
        Ok(context) => (
            true,
            response_kind(context.response.as_ref()),
            "none".to_string(),
            "request completed successfully".to_string(),
            terminology::AuditEventOutcome::V0(None),
        ),
        Err(error) => (
            false,
            "failure".to_string(),
            failure_response_type(error),
            error.to_string(),
            terminology::AuditEventOutcome::V8(None),
        ),
    };

    event.outcome = Some(Box::new(outcome));
    event.outcomeDesc = Some(Box::new(FHIRString::from(outcome_desc)));

    let details = vec![
        detail("success", success.to_string()),
        detail("response_type", response_type),
        detail("failure_response_type", failure_type),
    ];

    match event.entity.as_mut() {
        Some(entities) if !entities.is_empty() => {
            let entity_details = entities[0].detail.get_or_insert_with(Vec::new);
            entity_details.extend(details);
        }
        _ => {
            event.entity = Some(vec![AuditEventEntity {
                detail: Some(details),
                ..Default::default()
            }]);
        }
    }
}

fn build_audit_event<Client: FHIRClient<Arc<ServerCTX<Client>>, OperationOutcomeError>>(
    ctx: &Arc<ServerCTX<Client>>,
    action: Option<terminology::AuditEventAction>,
    subtype_code: &AuditEventSubType,
    subtype_display: &str,
    resource_type: Option<String>,
    resource_id: Option<String>,
    mut details: Vec<AuditEventEntityDetail>,
) -> Option<AuditEvent> {
    let tracing_id = ctx.tracing_id.as_ref()?;
    details.push(detail("tracing_id", tracing_id.clone()));
    details.push(detail("tenant", ctx.tenant.as_ref().to_string()));
    details.push(detail("project", ctx.project.as_ref().to_string()));
    details.push(detail(
        "user_role",
        format!("{:?}", ctx.user.claims.user_role).to_lowercase(),
    ));

    let resource_reference = match (&resource_type, &resource_id) {
        (Some(resource_type), Some(id)) => Some(format!("{resource_type}/{id}")),
        (Some(resource_type), None) => Some(resource_type.clone()),
        _ => None,
    };

    let entity = if resource_reference.is_some() || !details.is_empty() {
        Some(vec![AuditEventEntity {
            what: resource_reference.map(|reference| {
                Box::new(Reference {
                    reference: Some(Box::new(FHIRString::from(reference))),
                    ..Default::default()
                })
            }),
            name: resource_id.map(|id| Box::new(FHIRString::from(id))),
            description: resource_type
                .map(|resource_type| Box::new(FHIRString::from(resource_type))),
            detail: Some(details),
            ..Default::default()
        }])
    } else {
        None
    };

    Some(AuditEvent {
        type_: coding(
            "http://dicom.nema.org/resources/ontology/DCM",
            Some("110100".to_string()),
            "Application Activity",
        ),
        subtype: Some(vec![coding(
            "http://hl7.org/fhir/restful-interaction",
            subtype_code.into(),
            subtype_display,
        )]),
        action: action.map(Box::new),
        recorded: Box::new(FHIRInstant::from(Instant::Iso8601(chrono::Utc::now()))),
        agent: vec![AuditEventAgent {
            who: Some(Box::new(Reference {
                reference: Some(Box::new(FHIRString::from(format!(
                    "{}/{}",
                    ctx.user.claims.resource_type.as_ref(),
                    ctx.user.claims.user_id.as_ref()
                )))),
                ..Default::default()
            })),
            altId: Some(Box::new(FHIRString::from(
                ctx.user.claims.user_id.as_ref().to_string(),
            ))),
            requestor: Box::new(FHIRBoolean::from(true)),
            ..Default::default()
        }],
        source: AuditEventSource {
            site: Some(Box::new(FHIRString::from(format!(
                "{}/{}",
                ctx.tenant.as_ref(),
                ctx.project.as_ref()
            )))),
            observer: Box::new(Reference {
                reference: Some(Box::new(FHIRString::from(
                    "Device/haste-server".to_string(),
                ))),
                display: Some(Box::new(FHIRString::from("haste-server".to_string()))),
                ..Default::default()
            }),
            ..Default::default()
        },
        entity,
        ..Default::default()
    })
}

impl<'a, Client: FHIRClient<Arc<ServerCTX<Client>>, OperationOutcomeError>> From<Audit<'a, Client>>
    for Option<AuditEvent>
{
    fn from(audit: Audit<'a, Client>) -> Self {
        match audit.1 {
            FHIRRequest::Create(request) => build_audit_event(
                &audit.0,
                Some(terminology::AuditEventAction::C(None)),
                &AuditEventSubType::Create(None),
                "create",
                Some(request.resource_type.as_ref().to_string()),
                request.resource.id().as_ref().map(|s| s.to_string()),
                vec![],
            ),
            FHIRRequest::Read(request) => build_audit_event(
                &audit.0,
                Some(terminology::AuditEventAction::R(None)),
                &AuditEventSubType::Read(None),
                "read",
                Some(request.resource_type.as_ref().to_string()),
                Some(request.id.clone()),
                vec![],
            ),
            FHIRRequest::VersionRead(request) => build_audit_event(
                &audit.0,
                Some(terminology::AuditEventAction::R(None)),
                &AuditEventSubType::Read(None),
                "versioned read",
                Some(request.resource_type.as_ref().to_string()),
                Some(request.id.clone()),
                vec![detail(
                    "version_id",
                    request.version_id.as_ref().to_string(),
                )],
            ),
            FHIRRequest::Update(request) => match request {
                UpdateRequest::Instance(instance) => build_audit_event(
                    &audit.0,
                    Some(terminology::AuditEventAction::U(None)),
                    &AuditEventSubType::Update(None),
                    "update",
                    Some(instance.resource_type.as_ref().to_string()),
                    Some(instance.id.clone()),
                    vec![],
                ),
                UpdateRequest::Conditional(conditional) => build_audit_event(
                    &audit.0,
                    Some(terminology::AuditEventAction::U(None)),
                    &AuditEventSubType::Update(None),
                    "conditional update",
                    Some(conditional.resource_type.as_ref().to_string()),
                    conditional.resource.id().as_ref().map(|s| s.to_string()),
                    vec![detail("conditional", "true".to_string())],
                ),
            },
            FHIRRequest::Patch(request) => build_audit_event(
                &audit.0,
                Some(terminology::AuditEventAction::U(None)),
                &AuditEventSubType::Patch(None),
                "patch",
                Some(request.resource_type.as_ref().to_string()),
                Some(request.id.clone()),
                vec![],
            ),
            FHIRRequest::Delete(request) => match request {
                DeleteRequest::Instance(instance) => build_audit_event(
                    &audit.0,
                    Some(terminology::AuditEventAction::D(None)),
                    &AuditEventSubType::Delete(None),
                    "delete",
                    Some(instance.resource_type.as_ref().to_string()),
                    Some(instance.id.clone()),
                    vec![],
                ),
                DeleteRequest::Type(type_request) => build_audit_event(
                    &audit.0,
                    Some(terminology::AuditEventAction::D(None)),
                    &AuditEventSubType::Delete(None),
                    "type delete",
                    Some(type_request.resource_type.as_ref().to_string()),
                    None,
                    vec![],
                ),
                DeleteRequest::System(_) => build_audit_event(
                    &audit.0,
                    Some(terminology::AuditEventAction::D(None)),
                    &AuditEventSubType::Delete(None),
                    "system delete",
                    None,
                    None,
                    vec![detail("scope", "system".to_string())],
                ),
            },
            FHIRRequest::Capabilities => build_audit_event(
                &audit.0,
                Some(terminology::AuditEventAction::R(None)),
                &AuditEventSubType::Capabilities(None),
                "capability statement",
                None,
                None,
                vec![],
            ),
            FHIRRequest::Search(request) => match request {
                SearchRequest::Type(type_request) => build_audit_event(
                    &audit.0,
                    Some(terminology::AuditEventAction::R(None)),
                    &AuditEventSubType::Search(None),
                    "type search",
                    Some(type_request.resource_type.as_ref().to_string()),
                    None,
                    vec![],
                ),
                SearchRequest::System(_) => build_audit_event(
                    &audit.0,
                    Some(terminology::AuditEventAction::R(None)),
                    &AuditEventSubType::Search(None),
                    "system search",
                    None,
                    None,
                    vec![detail("scope", "system".to_string())],
                ),
            },
            FHIRRequest::History(request) => match request {
                HistoryRequest::Instance(instance) => build_audit_event(
                    &audit.0,
                    Some(terminology::AuditEventAction::R(None)),
                    &AuditEventSubType::HistoryInstance(None),
                    "instance history",
                    Some(instance.resource_type.as_ref().to_string()),
                    Some(instance.id.clone()),
                    vec![],
                ),
                HistoryRequest::Type(type_request) => build_audit_event(
                    &audit.0,
                    Some(terminology::AuditEventAction::R(None)),
                    &AuditEventSubType::HistoryType(None),
                    "type history",
                    Some(type_request.resource_type.as_ref().to_string()),
                    None,
                    vec![],
                ),
                HistoryRequest::System(_) => build_audit_event(
                    &audit.0,
                    Some(terminology::AuditEventAction::R(None)),
                    &AuditEventSubType::HistorySystem(None),
                    "system history",
                    None,
                    None,
                    vec![detail("scope", "system".to_string())],
                ),
            },
            FHIRRequest::Invocation(request) => match request {
                InvocationRequest::Instance(instance) => build_audit_event(
                    &audit.0,
                    Some(terminology::AuditEventAction::E(None)),
                    &AuditEventSubType::Operation(None),
                    "instance operation",
                    Some(instance.resource_type.as_ref().to_string()),
                    Some(instance.id.clone()),
                    vec![detail("operation", instance.operation.name().to_string())],
                ),
                InvocationRequest::Type(type_request) => build_audit_event(
                    &audit.0,
                    Some(terminology::AuditEventAction::E(None)),
                    &AuditEventSubType::Operation(None),
                    "type operation",
                    Some(type_request.resource_type.as_ref().to_string()),
                    None,
                    vec![detail(
                        "operation",
                        type_request.operation.name().to_string(),
                    )],
                ),
                InvocationRequest::System(system_request) => build_audit_event(
                    &audit.0,
                    Some(terminology::AuditEventAction::E(None)),
                    &AuditEventSubType::Operation(None),
                    "system operation",
                    None,
                    None,
                    vec![
                        detail("scope", "system".to_string()),
                        detail("operation", system_request.operation.name().to_string()),
                    ],
                ),
            },
            FHIRRequest::Batch(_) => build_audit_event(
                &audit.0,
                Some(terminology::AuditEventAction::E(None)),
                &AuditEventSubType::Batch(None),
                "batch",
                Some("Bundle".to_string()),
                None,
                vec![],
            ),
            FHIRRequest::Transaction(_) => build_audit_event(
                &audit.0,
                Some(terminology::AuditEventAction::E(None)),
                &AuditEventSubType::Transaction(None),
                "transaction",
                Some("Bundle".to_string()),
                None,
                vec![],
            ),
            FHIRRequest::Compartment(request) => build_audit_event(
                &audit.0,
                Some(terminology::AuditEventAction::R(None)),
                &AuditEventSubType::Search(None),
                "compartment search",
                Some(request.resource_type.as_ref().to_string()),
                Some(request.id.clone()),
                vec![detail("scope", "compartment".to_string())],
            ),
        }
    }
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct Middleware {}
impl Middleware {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Middleware {}
    }
}
impl<
    Repo: Repository + Send + Sync + 'static,
    Search: SearchEngine + Send + Sync + 'static,
    Terminology: FHIRTerminology + Send + Sync + 'static,
    Client: FHIRClient<Arc<ServerCTX<Client>>, OperationOutcomeError> + 'static,
>
    MiddlewareChain<
        ServerMiddlewareState<Repo, Search, Terminology>,
        Arc<ServerCTX<Client>>,
        FHIRRequest,
        FHIRResponse,
        OperationOutcomeError,
    > for Middleware
{
    #[instrument(name = "Auditing Middleware", skip(self, state, context, next))]
    fn call(
        &self,
        state: ServerMiddlewareState<Repo, Search, Terminology>,
        context: ServerMiddlewareContext<Client>,
        next: Option<
            Arc<ServerMiddlewareNext<Client, ServerMiddlewareState<Repo, Search, Terminology>>>,
        >,
    ) -> ServerMiddlewareOutput<Client> {
        Box::pin(async move {
            let mut audit_event =
                Option::<AuditEvent>::from(Audit(context.ctx.clone(), &context.request));

            let repo = state.repo.clone();
            let tenant = context.ctx.tenant.clone();
            let project = context.ctx.project.clone();
            let user = context.ctx.user.claims.clone();
            let fhir_version = context.ctx.fhir_version.clone();

            let result = if let Some(next) = next {
                next(state, context).await
            } else {
                Ok(context)
            };

            if let Some(event) = audit_event.as_mut() {
                add_response_outcome_to_audit_event(event, &result);

                let mut resource = Resource::AuditEvent(event.clone());
                tokio::spawn(async move {
                    if let Err(error) = FHIRRepository::create(
                        repo.as_ref(),
                        &tenant,
                        &project,
                        &user,
                        &fhir_version,
                        &mut resource,
                    )
                    .await
                    {
                        tracing::warn!("Failed to persist audit event: {error:?}");
                    }
                });
            }

            result
        })
    }
}
