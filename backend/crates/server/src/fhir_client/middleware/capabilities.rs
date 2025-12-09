use crate::fhir_client::{
    ServerCTX,
    middleware::{
        ServerMiddlewareContext, ServerMiddlewareNext, ServerMiddlewareOutput,
        ServerMiddlewareState,
    },
};
use haste_fhir_client::{
    middleware::MiddlewareChain,
    request::{
        FHIRCapabilitiesResponse, FHIRRequest, FHIRResponse, FHIRSearchTypeRequest, SearchRequest,
    },
    url::{Parameter, ParsedParameter, ParsedParameters},
};
use haste_fhir_model::r4::generated::{
    resources::{
        CapabilityStatement, CapabilityStatementRest, CapabilityStatementRestResource,
        CapabilityStatementRestResourceInteraction, CapabilityStatementRestResourceSearchParam,
        CapabilityStatementRestSecurity, Resource, ResourceType, SearchParameter,
        StructureDefinition,
    },
    terminology::{
        IssueType, ResourceTypes, RestfulCapabilityMode, TypeRestfulInteraction, VersioningPolicy,
    },
    types::{FHIRBoolean, FHIRString},
};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_fhir_search::{SearchEngine, SearchOptions};
use haste_fhir_terminology::FHIRTerminology;
use haste_jwt::{ProjectId, TenantId};
use haste_repository::{Repository, fhir::CachePolicy, types::SupportedFHIRVersions};
use std::sync::{Arc, LazyLock};
use tokio::sync::Mutex;

static CAPABILITIES: LazyLock<Mutex<Option<CapabilityStatement>>> =
    LazyLock::new(|| Mutex::new(None));

fn create_capability_rest_statement(
    sd: StructureDefinition,
    all_sps: &Vec<SearchParameter>,
) -> Result<CapabilityStatementRestResource, OperationOutcomeError> {
    let sd_type = sd.type_.value.unwrap_or_default();
    let shared_base_types = vec!["DomainResource".to_string(), "Resource".to_string()];

    let resource_parameters = all_sps
        .iter()
        .filter(|sp| {
            let types = sp
                .base
                .iter()
                .map(|b| b.as_ref().into())
                .filter_map(|b: Option<String>| b)
                .collect::<Vec<_>>();

            if types.contains(&shared_base_types[0])
                || types.contains(&shared_base_types[1])
                || types.contains(&sd_type)
            {
                true
            } else {
                false
            }
        })
        .collect::<Vec<&SearchParameter>>();

    Ok(CapabilityStatementRestResource {
        type_: Box::new(ResourceTypes::try_from(sd_type).map_err(|e| {
            OperationOutcomeError::error(
                IssueType::Invalid(None),
                format!(
                    "Failed to parse resource type in capabilities generation: '{}'",
                    e
                ),
            )
        })?),
        profile: Some(Box::new(FHIRString {
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
                        Box::new(FHIRString {
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
                TypeRestfulInteraction::Read(None),
                TypeRestfulInteraction::Vread(None),
                TypeRestfulInteraction::Update(None),
                TypeRestfulInteraction::Delete(None),
                TypeRestfulInteraction::SearchType(None),
                TypeRestfulInteraction::Create(None),
                TypeRestfulInteraction::HistoryInstance(None),
                TypeRestfulInteraction::HistoryType(None),
            ]
            .into_iter()
            .map(|code| CapabilityStatementRestResourceInteraction {
                code: Box::new(code),
                ..Default::default()
            })
            .collect(),
        ),
        versioning: Some(Box::new(VersioningPolicy::Versioned(None))),
        ..Default::default()
    })
}

async fn get_all_sds<Repo: Repository, Search: SearchEngine>(
    repo: &Repo,
    search_engine: &Search,
) -> Result<Vec<StructureDefinition>, OperationOutcomeError> {
    let sd_search = FHIRSearchTypeRequest {
        resource_type: ResourceType::StructureDefinition,
        parameters: ParsedParameters::new(vec![
            ParsedParameter::Resource(Parameter {
                name: "kind".to_string(),
                value: vec!["resource".to_string()],
                modifier: None,
                chains: None,
            }),
            ParsedParameter::Resource(Parameter {
                name: "abstract".to_string(),
                value: vec!["false".to_string()],
                modifier: None,
                chains: None,
            }),
            ParsedParameter::Resource(Parameter {
                name: "derivation".to_string(),
                value: vec!["specialization".to_string()],
                modifier: None,
                chains: None,
            }),
            // ParsedParameter::Result(Parameter {
            //     name: "_sort".to_string(),
            //     value: vec!["url".to_string()],
            //     modifier: None,
            //     chains: None,
            // }),
        ]),
    };
    let sd_results = search_engine
        .search(
            &SupportedFHIRVersions::R4,
            &TenantId::System,
            &ProjectId::System,
            &SearchRequest::Type(sd_search),
            Some(SearchOptions { count_limit: false }),
        )
        .await?;

    let version_ids = sd_results
        .entries
        .iter()
        .map(|v| &v.version_id)
        .collect::<Vec<_>>();

    let sds = repo
        .read_by_version_ids(
            &TenantId::System,
            &ProjectId::System,
            version_ids.as_slice(),
            CachePolicy::NoCache,
        )
        .await?
        .into_iter()
        .filter_map(|r| match r {
            Resource::StructureDefinition(sd) => Some(sd),
            _ => None,
        });

    Ok(sds.collect())
}

async fn get_all_sps<Repo: Repository, Search: SearchEngine>(
    repo: &Repo,
    search_engine: &Search,
) -> Result<Vec<SearchParameter>, OperationOutcomeError> {
    let sp_search = FHIRSearchTypeRequest {
        resource_type: ResourceType::SearchParameter,
        parameters: ParsedParameters::new(vec![]),
    };
    let sp_results = search_engine
        .search(
            &SupportedFHIRVersions::R4,
            &TenantId::System,
            &ProjectId::System,
            &SearchRequest::Type(sp_search),
            Some(SearchOptions { count_limit: false }),
        )
        .await?;

    let version_ids = sp_results
        .entries
        .iter()
        .map(|v| &v.version_id)
        .collect::<Vec<_>>();

    let sps = repo
        .read_by_version_ids(
            &TenantId::System,
            &ProjectId::System,
            version_ids.as_slice(),
            CachePolicy::NoCache,
        )
        .await?
        .into_iter()
        .filter_map(|r| match r {
            Resource::SearchParameter(sp) => Some(sp),
            _ => None,
        });

    Ok(sps.collect())
}

async fn generate_capabilities<Repo: Repository, Search: SearchEngine>(
    repo: &Repo,
    search_engine: &Search,
) -> Result<CapabilityStatement, OperationOutcomeError> {
    let (sds, sps) = tokio::join!(
        get_all_sds(repo, search_engine),
        get_all_sps(repo, search_engine)
    );

    let sds = sds?;
    let sps = sps?;

    Ok(CapabilityStatement {
        rest: Some(vec![CapabilityStatementRest {
            mode: Box::new(RestfulCapabilityMode::Server(None)),
            security: Some(CapabilityStatementRestSecurity {
                cors: Some(Box::new(FHIRBoolean {
                    value: Some(true),
                    ..Default::default()
                })),
                ..Default::default()
            }),
            resource: Some(
                sds.into_iter()
                    .map(|sd| create_capability_rest_statement(sd, &sps))
                    .collect::<Result<Vec<_>, _>>()?,
            ),
            ..Default::default()
        }]),
        ..Default::default()
    })
}

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
>
    MiddlewareChain<
        ServerMiddlewareState<Repo, Search, Terminology>,
        Arc<ServerCTX<Repo, Search, Terminology>>,
        FHIRRequest,
        FHIRResponse,
        OperationOutcomeError,
    > for Middleware
{
    fn call(
        &self,

        state: ServerMiddlewareState<Repo, Search, Terminology>,
        mut context: ServerMiddlewareContext<Repo, Search, Terminology>,
        next: Option<Arc<ServerMiddlewareNext<Repo, Search, Terminology>>>,
    ) -> ServerMiddlewareOutput<Repo, Search, Terminology> {
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
                                .await
                                .unwrap();
                        *guard = Some(capabilities.clone());

                        context.response =
                            Some(FHIRResponse::Capabilities(FHIRCapabilitiesResponse {
                                capabilities: capabilities,
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
