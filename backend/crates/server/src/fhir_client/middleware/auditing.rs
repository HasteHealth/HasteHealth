use crate::fhir_client::{
    ClientState, ServerCTX,
    middleware::{Backend, Middleware, Next, Output},
};
use haste_fhir_client::{
    FHIRClient,
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
        terminology::{self, AuditEventSubType, BoundCode},
        types::{Coding, FHIRBoolean, FHIRInstant, FHIRString, FHIRUri, Reference},
    },
};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_repository::fhir::FHIRRepository;
use std::sync::Arc;
use tracing::instrument;

struct Audit<'a, Client: FHIRClient<Arc<ServerCTX<Client>>, OperationOutcomeError>>(
    Arc<ServerCTX<Client>>,
    &'a FHIRRequest,
);

fn coding(system: &str, code: Option<String>, display: &str) -> Coding {
    Coding {
        system: Some(Box::new(FHIRUri::from(system.to_string()))),
        code: code.map(|code| Box::new(code.into())),
        display: Some(Box::new(FHIRString::from(display.to_string()))),
        ..Default::default()
    }
}

fn detail(name: &str, value: String) -> AuditEventEntityDetail {
    AuditEventEntityDetail {
        type_: Box::new(FHIRString::from(name.to_string())),
        value: AuditEventEntityDetailValueTypeChoice::String(Box::new(FHIRString::from(value))),
        ..Default::default()
    }
}

fn response_kind(response: &FHIRResponse) -> String {
    match response {
        FHIRResponse::Create(_) => "create".to_string(),
        FHIRResponse::Read(_) => "read".to_string(),
        FHIRResponse::VersionRead(_) => "version-read".to_string(),
        FHIRResponse::Update(_) => "update".to_string(),
        FHIRResponse::Patch(_) => "patch".to_string(),
        FHIRResponse::Delete(delete) => match delete {
            DeleteResponse::Instance(_) => "delete-instance".to_string(),
            DeleteResponse::Type(_) => "delete-type".to_string(),
            DeleteResponse::System(_) => "delete-system".to_string(),
        },
        FHIRResponse::Capabilities(_) => "capabilities".to_string(),
        FHIRResponse::Search(search) => match search {
            SearchResponse::Type(_) => "search-type".to_string(),
            SearchResponse::System(_) => "search-system".to_string(),
        },
        FHIRResponse::History(_) => "history".to_string(),
        FHIRResponse::Invoke(invoke) => match invoke {
            InvokeResponse::Instance(_) => "invoke-instance".to_string(),
            InvokeResponse::Type(_) => "invoke-type".to_string(),
            InvokeResponse::System(_) => "invoke-system".to_string(),
        },
        FHIRResponse::Batch(_) => "batch".to_string(),
        FHIRResponse::Transaction(_) => "transaction".to_string(),
        FHIRResponse::Accepted(_) => "accepted".to_string(),
    }
}

fn failure_response_type(error: &OperationOutcomeError) -> String {
    error
        .outcome()
        .issue
        .first()
        .and_then(|issue| issue.code.as_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

fn add_response_outcome_to_audit_event(
    event: &mut AuditEvent,
    result: &Result<FHIRResponse, OperationOutcomeError>,
) {
    let (success, response_type, failure_type, outcome_desc, outcome) = match result {
        Ok(response) => (
            true,
            response_kind(response),
            "none".to_string(),
            "request completed successfully".to_string(),
            terminology::AuditEventOutcome::v0(),
        ),
        Err(error) => (
            false,
            "failure".to_string(),
            failure_response_type(error),
            error.to_string(),
            terminology::AuditEventOutcome::v8(),
        ),
    };

    event.outcome = Some(outcome);
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
    action: Option<BoundCode<terminology::AuditEventAction>>,
    subtype_code: &BoundCode<AuditEventSubType>,
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
        type_: Box::new(coding(
            "http://dicom.nema.org/resources/ontology/DCM",
            Some("110100".to_string()),
            "Application Activity",
        )),
        subtype: Some(vec![coding(
            "http://hl7.org/fhir/restful-interaction",
            subtype_code.as_str().map(|s| s.to_string()),
            subtype_display,
        )]),
        action,
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
                Some(terminology::AuditEventAction::c()),
                &AuditEventSubType::create(),
                "create",
                Some(request.resource_type.as_ref().to_string()),
                request.resource.id().as_ref().map(|s| s.to_string()),
                vec![],
            ),
            FHIRRequest::Read(request) => build_audit_event(
                &audit.0,
                Some(terminology::AuditEventAction::r()),
                &AuditEventSubType::read(),
                "read",
                Some(request.resource_type.as_ref().to_string()),
                Some(request.id.clone()),
                vec![],
            ),
            FHIRRequest::VersionRead(request) => build_audit_event(
                &audit.0,
                Some(terminology::AuditEventAction::r()),
                &AuditEventSubType::read(),
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
                    Some(terminology::AuditEventAction::u()),
                    &AuditEventSubType::update(),
                    "update",
                    Some(instance.resource_type.as_ref().to_string()),
                    Some(instance.id.clone()),
                    vec![],
                ),
                UpdateRequest::Conditional(conditional) => build_audit_event(
                    &audit.0,
                    Some(terminology::AuditEventAction::u()),
                    &AuditEventSubType::update(),
                    "conditional update",
                    Some(conditional.resource_type.as_ref().to_string()),
                    conditional.resource.id().as_ref().map(|s| s.to_string()),
                    vec![detail("conditional", "true".to_string())],
                ),
            },
            FHIRRequest::Patch(request) => build_audit_event(
                &audit.0,
                Some(terminology::AuditEventAction::u()),
                &AuditEventSubType::patch(),
                "patch",
                Some(request.resource_type.as_ref().to_string()),
                Some(request.id.clone()),
                vec![],
            ),
            FHIRRequest::Delete(request) => match request {
                DeleteRequest::Instance(instance) => build_audit_event(
                    &audit.0,
                    Some(terminology::AuditEventAction::d()),
                    &AuditEventSubType::delete(),
                    "delete",
                    Some(instance.resource_type.as_ref().to_string()),
                    Some(instance.id.clone()),
                    vec![],
                ),
                DeleteRequest::Type(type_request) => build_audit_event(
                    &audit.0,
                    Some(terminology::AuditEventAction::d()),
                    &AuditEventSubType::delete(),
                    "type delete",
                    Some(type_request.resource_type.as_ref().to_string()),
                    None,
                    vec![],
                ),
                DeleteRequest::System(_) => build_audit_event(
                    &audit.0,
                    Some(terminology::AuditEventAction::d()),
                    &AuditEventSubType::delete(),
                    "system delete",
                    None,
                    None,
                    vec![detail("scope", "system".to_string())],
                ),
            },
            FHIRRequest::Capabilities => build_audit_event(
                &audit.0,
                Some(terminology::AuditEventAction::r()),
                &AuditEventSubType::capabilities(),
                "capability statement",
                None,
                None,
                vec![],
            ),
            FHIRRequest::Search(request) => match request {
                SearchRequest::Type(type_request) => build_audit_event(
                    &audit.0,
                    Some(terminology::AuditEventAction::r()),
                    &AuditEventSubType::search(),
                    "type search",
                    Some(type_request.resource_type.as_ref().to_string()),
                    None,
                    vec![],
                ),
                SearchRequest::System(_) => build_audit_event(
                    &audit.0,
                    Some(terminology::AuditEventAction::r()),
                    &AuditEventSubType::search(),
                    "system search",
                    None,
                    None,
                    vec![detail("scope", "system".to_string())],
                ),
            },
            FHIRRequest::History(request) => match request {
                HistoryRequest::Instance(instance) => build_audit_event(
                    &audit.0,
                    Some(terminology::AuditEventAction::r()),
                    &AuditEventSubType::history_instance(),
                    "instance history",
                    Some(instance.resource_type.as_ref().to_string()),
                    Some(instance.id.clone()),
                    vec![],
                ),
                HistoryRequest::Type(type_request) => build_audit_event(
                    &audit.0,
                    Some(terminology::AuditEventAction::r()),
                    &AuditEventSubType::history_type(),
                    "type history",
                    Some(type_request.resource_type.as_ref().to_string()),
                    None,
                    vec![],
                ),
                HistoryRequest::System(_) => build_audit_event(
                    &audit.0,
                    Some(terminology::AuditEventAction::r()),
                    &AuditEventSubType::history_system(),
                    "system history",
                    None,
                    None,
                    vec![detail("scope", "system".to_string())],
                ),
            },
            FHIRRequest::Invocation(request) => match request {
                InvocationRequest::Instance(instance) => build_audit_event(
                    &audit.0,
                    Some(terminology::AuditEventAction::e()),
                    &AuditEventSubType::operation(),
                    "instance operation",
                    Some(instance.resource_type.as_ref().to_string()),
                    Some(instance.id.clone()),
                    vec![detail("operation", instance.operation.name().to_string())],
                ),
                InvocationRequest::Type(type_request) => build_audit_event(
                    &audit.0,
                    Some(terminology::AuditEventAction::e()),
                    &AuditEventSubType::operation(),
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
                    Some(terminology::AuditEventAction::e()),
                    &AuditEventSubType::operation(),
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
                Some(terminology::AuditEventAction::e()),
                &AuditEventSubType::batch(),
                "batch",
                Some("Bundle".to_string()),
                None,
                vec![],
            ),
            FHIRRequest::Transaction(_) => build_audit_event(
                &audit.0,
                Some(terminology::AuditEventAction::e()),
                &AuditEventSubType::transaction(),
                "transaction",
                Some("Bundle".to_string()),
                None,
                vec![],
            ),
            FHIRRequest::Compartment(request) => build_audit_event(
                &audit.0,
                Some(terminology::AuditEventAction::r()),
                &AuditEventSubType::search(),
                "compartment search",
                Some(request.resource_type.as_ref().to_string()),
                Some(request.id.clone()),
                vec![detail("scope", "compartment".to_string())],
            ),
        }
    }
}

/// Records an AuditEvent for every request when the audit repository is
/// configured. The event is written in the background once the response is
/// known, so a slow audit store never slows the caller.
#[derive(Debug)]
pub struct Auditing;

impl<B: Backend> Middleware<B> for Auditing {
    #[instrument(name = "Auditing Middleware", skip(self, state, ctx, request, next))]
    fn call<'a>(
        &'a self,
        state: &'a ClientState<B>,
        ctx: Arc<ServerCTX<B>>,
        request: FHIRRequest,
        next: Next<'a, B>,
    ) -> Output<'a> {
        let Some(audit_repo) = state.audit_repo.clone() else {
            return next.run(state, ctx, request);
        };

        Box::pin(async move {
            let audit_event = Option::<AuditEvent>::from(Audit(ctx.clone(), &request));

            let tenant = ctx.tenant.clone();
            let project = ctx.project.clone();
            let user = ctx.user.claims.clone();
            let fhir_version = ctx.fhir_version.clone();

            let result = next.run(state, ctx, request).await;

            if let Some(mut event) = audit_event {
                add_response_outcome_to_audit_event(&mut event, &result);

                tokio::spawn(async move {
                    if let Err(error) = FHIRRepository::create(
                        audit_repo.as_ref(),
                        &tenant,
                        &project,
                        &user,
                        &fhir_version,
                        Resource::AuditEvent(event),
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
