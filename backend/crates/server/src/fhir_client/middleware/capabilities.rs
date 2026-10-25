use crate::{
    fhir_client::{
        ClientState, ServerCTX,
        middleware::{Backend, Handler, Middleware, Next, Output},
    },
    load_artifacts::{get_all_operation_definitions, get_all_sds, get_all_sps},
};
use haste_fhir_client::{
    FHIRClient,
    request::{FHIRCapabilitiesResponse, FHIRRequest, FHIRResponse},
};
use haste_fhir_model::r4::{
    datetime::DateTime,
    generated::{
        resources::{
            CapabilityStatement, CapabilityStatementRest, CapabilityStatementRestInteraction,
            CapabilityStatementRestResource, CapabilityStatementRestResourceInteraction,
            CapabilityStatementRestResourceOperation, CapabilityStatementRestResourceSearchParam,
            CapabilityStatementRestSecurity, OperationDefinition, Resource, ResourceType,
            SearchParameter, StructureDefinition,
        },
        terminology::{
            BoundCode, CapabilityStatementKind, FHIRVersion, IssueType, PublicationStatus,
            ResourceTypes, RestfulCapabilityMode, SystemRestfulInteraction, TypeRestfulInteraction,
            VersioningPolicy,
        },
        types::{FHIRBoolean, FHIRCanonical, FHIRCode, FHIRDateTime, FHIRString},
    },
};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_fhir_search::SearchEngine;
use haste_repository::Repository;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, LazyLock};
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use tracing::instrument;

/// How long a generated statement is served before profiles and search
/// parameters loaded since are picked up.
const CAPABILITIES_TTL: Duration = Duration::from_secs(30);

/// Page size for the project's profile and search parameter lookups.
const PROJECT_RESOURCES_COUNT: &str = "1000";

// Keyed by (tenant, project): the statement lists that project's custom
// operations, profiles and search parameters, so a single process-wide value
// would serve one project's to every other tenant (and, through the public
// metadata route, to unauthenticated callers).
#[allow(clippy::type_complexity)]
static CAPABILITIES: LazyLock<
    Mutex<HashMap<(haste_jwt::TenantId, haste_jwt::ProjectId), (Instant, CapabilityStatement)>>,
> = LazyLock::new(|| Mutex::new(HashMap::new()));

/// Drops a project's cached statement so the next `metadata` request sees
/// profiles and search parameters written since.
pub async fn invalidate(tenant: &haste_jwt::TenantId, project: &haste_jwt::ProjectId) {
    CAPABILITIES
        .lock()
        .await
        .remove(&(tenant.clone(), project.clone()));
}

/// The project's active resources of one type matching `filters`.
async fn project_resources<
    Client: FHIRClient<Arc<ServerCTX<Client>>, OperationOutcomeError> + 'static,
>(
    ctx: Arc<ServerCTX<Client>>,
    resource_type: ResourceType,
    filters: &[(&str, &str)],
) -> Result<Vec<Resource>, OperationOutcomeError> {
    let parameters: Vec<(String, Vec<String>)> = filters
        .iter()
        .chain(&[("status", "active"), ("_count", PROJECT_RESOURCES_COUNT)])
        .map(|(name, value)| ((*name).to_string(), vec![(*value).to_string()]))
        .collect();

    Ok(ctx
        .client
        .search_type(ctx.clone(), resource_type, parameters.into())
        .await?
        .entry
        .unwrap_or_default()
        .into_iter()
        .filter_map(|e| e.resource)
        .map(|r| *r)
        .collect())
}

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
    profiles: &[FHIRCanonical],
    all_sps: &[SearchParameter],
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
        supportedProfile: (!profiles.is_empty()).then(|| profiles.to_vec()),
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
                TypeRestfulInteraction::patch(),
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

async fn generate_capabilities<
    Repo: Repository + Send + Sync + 'static,
    Search: SearchEngine + Send + Sync + 'static,
    Client: FHIRClient<Arc<ServerCTX<Client>>, OperationOutcomeError> + 'static,
>(
    ctx: Arc<ServerCTX<Client>>,
    repo: &Repo,
    search_engine: &Search,
) -> Result<CapabilityStatement, OperationOutcomeError> {
    let ctx_for_project = ctx.clone();
    let (sds, sps, operations) = tokio::join!(
        get_all_sds(&["resource"], repo, search_engine),
        get_all_sps(repo, search_engine),
        get_all_operation_definitions(ctx)
    );

    let sds = sds?;
    let mut sps = sps?;
    let operations = operations?;

    // Profiles and search parameters loaded into the project itself.
    let (profile_defs, extra_sps) = tokio::try_join!(
        project_resources(
            ctx_for_project.clone(),
            ResourceType::StructureDefinition,
            &[("derivation", "constraint")]
        ),
        project_resources(ctx_for_project, ResourceType::SearchParameter, &[])
    )?;

    // Constraint profile urls grouped by the resource type they constrain.
    let mut profiles: HashMap<String, Vec<FHIRCanonical>> = HashMap::new();
    for resource in profile_defs {
        if let Resource::StructureDefinition(sd) = resource
            && let (Some(url), Some(type_)) = (sd.url.value, sd.type_.value)
        {
            profiles.entry(type_).or_default().push(FHIRCanonical {
                value: Some(url),
                ..Default::default()
            });
        }
    }

    let known: HashSet<Option<String>> = sps.iter().map(|sp| sp.url.value.clone()).collect();
    sps.extend(extra_sps.into_iter().filter_map(|resource| match resource {
        Resource::SearchParameter(sp) if !known.contains(&sp.url.value) => Some(sp),
        _ => None,
    }));

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
                    .map(|sd| {
                        let sd_profiles = sd
                            .type_
                            .value
                            .as_ref()
                            .and_then(|t| profiles.get(t))
                            .map_or(&[][..], Vec::as_slice);
                        create_capability_rest_statement(sd, sd_profiles, &sps, &operations)
                    })
                    .collect::<Result<Vec<_>, _>>()?,
            ),
            // Interactions on the base URL itself; each resource type lists
            // its own above.
            interaction: Some(
                vec![
                    SystemRestfulInteraction::transaction(),
                    SystemRestfulInteraction::batch(),
                    SystemRestfulInteraction::search_system(),
                    SystemRestfulInteraction::history_system(),
                ]
                .into_iter()
                .map(|code| CapabilityStatementRestInteraction {
                    code,
                    ..Default::default()
                })
                .collect(),
            ),
            operation: Some(system_operations),
            ..Default::default()
        }]),
        ..Default::default()
    })
}

/// Ends the capabilities lane: the CapabilityStatement, generated and then
/// served from memory for a short time.
#[derive(Debug)]
pub struct Capabilities;

impl<B: Backend> Handler<B> for Capabilities {
    #[instrument(name = "Capabilities Middleware", skip(self, state, ctx, request))]
    fn handle<'a>(
        &'a self,
        state: &'a ClientState<B>,
        ctx: Arc<ServerCTX<B>>,
        request: FHIRRequest,
    ) -> Output<'a> {
        Box::pin(async move {
            if !matches!(request, FHIRRequest::Capabilities) {
                return Err(OperationOutcomeError::fatal(
                    IssueType::exception(),
                    "The capabilities handler only serves the capabilities interaction."
                        .to_string(),
                ));
            }

            let cache_key = (ctx.tenant.clone(), ctx.project.clone());
            let mut guard = CAPABILITIES.lock().await;
            let capabilities = match guard.get(&cache_key) {
                Some((generated, capabilities)) if generated.elapsed() < CAPABILITIES_TTL => {
                    capabilities.clone()
                }
                _ => {
                    let capabilities =
                        generate_capabilities(ctx, state.repo.as_ref(), state.search.as_ref())
                            .await?;
                    guard.insert(cache_key, (Instant::now(), capabilities.clone()));
                    capabilities
                }
            };

            Ok(FHIRResponse::Capabilities(FHIRCapabilitiesResponse {
                capabilities,
            }))
        })
    }
}

/// On the artifact lane: a successful write to a StructureDefinition or
/// SearchParameter refreshes the project's cached statement.
pub struct InvalidateCapabilities;

impl<B: Backend> Middleware<B> for InvalidateCapabilities {
    fn call<'a>(
        &'a self,
        state: &'a ClientState<B>,
        ctx: Arc<ServerCTX<B>>,
        request: FHIRRequest,
        next: Next<'a, B>,
    ) -> Output<'a> {
        Box::pin(async move {
            let is_write = matches!(
                request,
                FHIRRequest::Create(_)
                    | FHIRRequest::Update(_)
                    | FHIRRequest::Patch(_)
                    | FHIRRequest::Delete(_)
            );
            let response = next.run(state, ctx.clone(), request).await?;
            if is_write {
                invalidate(&ctx.tenant, &ctx.project).await;
            }
            Ok(response)
        })
    }
}
