use crate::{
    fhir_client::{
        ServerCTX,
        middleware::{
            ServerMiddlewareContext, ServerMiddlewareNext, ServerMiddlewareOutput,
            ServerMiddlewareState,
        },
    },
    load_artifacts::{get_all_operation_definitions, get_all_sds, get_all_sps},
};
use haste_fhir_client::{
    FHIRClient,
    middleware::MiddlewareChain,
    request::{FHIRCapabilitiesResponse, FHIRRequest, FHIRResponse},
};
use haste_fhir_model::r4::{
    datetime::DateTime,
    generated::{
        resources::{
            CapabilityStatement, CapabilityStatementRest, CapabilityStatementRestResource,
            CapabilityStatementRestResourceInteraction, CapabilityStatementRestResourceOperation,
            CapabilityStatementRestResourceSearchParam, CapabilityStatementRestSecurity,
            OperationDefinition, SearchParameter, StructureDefinition,
        },
        terminology::{
            BoundCode, CapabilityStatementKind, FHIRVersion, IssueType, PublicationStatus,
            ResourceTypes, RestfulCapabilityMode, TypeRestfulInteraction, VersioningPolicy,
        },
        types::{FHIRBoolean, FHIRCanonical, FHIRCode, FHIRDateTime, FHIRString},
    },
};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_fhir_search::SearchEngine;
use haste_fhir_terminology::FHIRTerminology;
use haste_repository::Repository;
use std::sync::{Arc, LazyLock};
use tokio::sync::Mutex;
use tracing::instrument;

static CAPABILITIES: LazyLock<Mutex<Option<CapabilityStatement>>> =
    LazyLock::new(|| Mutex::new(None));

/// Build a `CapabilityStatementRestResourceOperation` entry from an `OperationDefinition`.
///
/// `name` is the code clients invoke it by (`$expand`), and `definition` points at
/// the OperationDefinition carrying its parameters.
fn operation_entry(
    operation: &OperationDefinition,
) -> Option<CapabilityStatementRestResourceOperation> {
    let code = operation.code.value.clone()?;

    Some(CapabilityStatementRestResourceOperation {
        name: Box::new(FHIRString {
            value: Some(code),
            ..Default::default()
        }),
        definition: Box::new(FHIRCanonical {
            value: operation.url.as_ref().and_then(|u| u.value.clone()),
            ..Default::default()
        }),
        documentation: operation.description.as_ref().map(|d| {
            Box::new(haste_fhir_model::r4::generated::types::FHIRMarkdown {
                value: d.value.clone(),
                ..Default::default()
            })
        }),
        ..Default::default()
    })
}

/// Operations this resource type supports at type or instance level.
fn operations_for_type<'a>(
    sd_type: &str,
    operations: &'a [OperationDefinition],
) -> Vec<&'a OperationDefinition> {
    operations
        .iter()
        .filter(|operation| {
            // Only type- and instance-level operations belong on a resource;
            // system-level ones are listed once on the rest element.
            if !(operation.type_.value.unwrap_or(false)
                || operation.instance.value.unwrap_or(false))
            {
                return false;
            }

            operation
                .resource
                .as_deref()
                .unwrap_or_default()
                .iter()
                .any(|r| r.as_str() == Some(sd_type))
        })
        .collect()
}

fn create_capability_rest_statement(
    sd: StructureDefinition,
    all_sps: &Vec<SearchParameter>,
    all_operations: &[OperationDefinition],
) -> Result<CapabilityStatementRestResource, OperationOutcomeError> {
    let sd_type = sd.type_.value.unwrap_or_default();
    let shared_base_types = ["DomainResource".to_string(), "Resource".to_string()];

    let resource_parameters = all_sps
        .iter()
        .filter(|sp| {
            let types = sp
                .base
                .iter()
                .map(|b| b.as_str())
                .filter_map(|b: Option<&str>| b)
                .collect::<Vec<_>>();

            types.contains(&shared_base_types[0].as_str())
                || types.contains(&shared_base_types[1].as_str())
                || types.contains(&sd_type.as_str())
        })
        .collect::<Vec<&SearchParameter>>();

    Ok(CapabilityStatementRestResource {
        type_: BoundCode::<ResourceTypes>::new(&sd_type).ok_or_else(|| {
            OperationOutcomeError::error(
                IssueType::invalid(),
                format!(
                    "Failed to parse resource type in capabilities generation: '{}'",
                    sd_type
                ),
            )
        })?,
        profile: Some(Box::new(FHIRCanonical {
            value: sd.url.value,
            ..Default::default()
        })),
        searchParam: Some(
            resource_parameters
                .into_iter()
                .map(|sp| CapabilityStatementRestResourceSearchParam {
                    name: Box::new(FHIRString {
                        value: sp.code.value.clone(),
                        ..Default::default()
                    }),
                    definition: sp.url.value.clone().map(|v| {
                        Box::new(FHIRCanonical {
                            value: Some(v),
                            ..Default::default()
                        })
                    }),
                    type_: sp.type_.clone(),
                    documentation: Some(sp.description.clone()),
                    ..Default::default()
                })
                .collect(),
        ),
        interaction: Some(
            vec![
                TypeRestfulInteraction::read(),
                TypeRestfulInteraction::vread(),
                TypeRestfulInteraction::update(),
                TypeRestfulInteraction::delete(),
                TypeRestfulInteraction::search_type(),
                TypeRestfulInteraction::create(),
                TypeRestfulInteraction::history_instance(),
                TypeRestfulInteraction::history_type(),
            ]
            .into_iter()
            .map(|code| CapabilityStatementRestResourceInteraction {
                code,
                ..Default::default()
            })
            .collect(),
        ),
        versioning: Some(VersioningPolicy::versioned()),
        operation: Some(
            operations_for_type(&sd_type, all_operations)
                .into_iter()
                .filter_map(operation_entry)
                .collect(),
        ),
        ..Default::default()
    })
}

async fn generate_capabilities<Repo: Repository, Search: SearchEngine>(
    repo: &Repo,
    search_engine: &Search,
) -> Result<CapabilityStatement, OperationOutcomeError> {
    let (sds, sps, operations) = tokio::join!(
        get_all_sds(&["resource"], repo, search_engine),
        get_all_sps(repo, search_engine),
        get_all_operation_definitions(repo, search_engine)
    );

    let sds = sds?;
    let sps = sps?;
    let operations = operations?;

    // System-level operations are not tied to a resource type, so they are
    // documented once on the rest element itself.
    let system_operations: Vec<_> = operations
        .iter()
        .filter(|operation| operation.system.value.unwrap_or(false))
        .filter_map(operation_entry)
        .collect();

    Ok(CapabilityStatement {
        status: PublicationStatus::active(),
        kind: CapabilityStatementKind::capability(),
        date: Box::new(FHIRDateTime {
            value: Some(DateTime::Year(2025)),
            ..Default::default()
        }),
        format: vec![FHIRCode {
            value: Some("application/fhir+json".to_string()),
            ..Default::default()
        }],
        fhirVersion: FHIRVersion::v401(),
        rest: Some(vec![CapabilityStatementRest {
            mode: RestfulCapabilityMode::server(),
            security: Some(CapabilityStatementRestSecurity {
                cors: Some(Box::new(FHIRBoolean {
                    value: Some(true),
                    ..Default::default()
                })),
                ..Default::default()
            }),
            resource: Some(
                sds.into_iter()
                    .map(|sd| create_capability_rest_statement(sd, &sps, &operations))
                    .collect::<Result<Vec<_>, _>>()?,
            ),
            operation: Some(system_operations),
            ..Default::default()
        }]),
        ..Default::default()
    })
}

#[derive(Debug)]
pub struct Middleware {}
impl Middleware {
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
    #[instrument(name = "Capabilities Middleware", skip(self, state, context, next))]
    fn call(
        &self,
        state: ServerMiddlewareState<Repo, Search, Terminology>,
        mut context: ServerMiddlewareContext<Client>,
        next: Option<
            Arc<ServerMiddlewareNext<Client, ServerMiddlewareState<Repo, Search, Terminology>>>,
        >,
    ) -> ServerMiddlewareOutput<Client> {
        Box::pin(async move {
            match context.request {
                FHIRRequest::Capabilities => {
                    let mut guard = CAPABILITIES.lock().await;

                    if let Some(capabilities) = &*guard {
                        context.response =
                            Some(FHIRResponse::Capabilities(FHIRCapabilitiesResponse {
                                capabilities: capabilities.clone(),
                            }));
                    } else {
                        let capabilities =
                            generate_capabilities(state.repo.as_ref(), state.search.as_ref())
                                .await?;
                        *guard = Some(capabilities.clone());

                        context.response =
                            Some(FHIRResponse::Capabilities(FHIRCapabilitiesResponse {
                                capabilities,
                            }));
                    }

                    Ok(context)
                }
                _ => {
                    if let Some(next) = next {
                        next(state, context).await
                    } else {
                        Ok(context)
                    }
                }
            }
        })
    }
}
