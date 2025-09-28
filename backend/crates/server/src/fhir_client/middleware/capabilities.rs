use crate::fhir_client::middleware::{
    ServerMiddlewareContext, ServerMiddlewareNext, ServerMiddlewareOutput, ServerMiddlewareState,
};
use oxidized_fhir_client::{
    request::{FHIRCapabilitiesResponse, FHIRRequest, FHIRResponse, FHIRSearchTypeRequest},
    url::{Parameter, ParsedParameter},
};
use oxidized_fhir_model::r4::generated::{
    resources::{
        CapabilityStatement, CapabilityStatementRest, CapabilityStatementRestResource,
        CapabilityStatementRestResourceInteraction, CapabilityStatementRestSecurity, Resource,
        ResourceType,
    },
    terminology::{IssueType, RestfulCapabilityMode, TypeRestfulInteraction, VersioningPolicy},
    types::{FHIRBoolean, FHIRCode, FHIRString},
};
use oxidized_fhir_operation_error::OperationOutcomeError;
use oxidized_fhir_search::{SearchEngine, SearchOptions, SearchRequest};
use oxidized_repository::{
    Repository,
    types::{ProjectId, SupportedFHIRVersions, TenantId, VersionIdRef},
};
use std::sync::{Arc, LazyLock};
use tokio::sync::Mutex;

static CAPABILITIES: LazyLock<Mutex<Option<CapabilityStatement>>> =
    LazyLock::new(|| Mutex::new(None));

pub async fn generate_capabilities<Repo: Repository, Search: SearchEngine>(
    repo: &Repo,
    search_engine: &Search,
) -> Result<CapabilityStatement, OperationOutcomeError> {
    let sd_search = FHIRSearchTypeRequest {
        resource_type: ResourceType::StructureDefinition,
        parameters: vec![
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
        ],
    };
    let sd_results = search_engine
        .search(
            &SupportedFHIRVersions::R4,
            &TenantId::System,
            &ProjectId::System,
            SearchRequest::TypeSearch(&sd_search),
            Some(SearchOptions { count_limit: false }),
        )
        .await?;
    let sds = repo
        .read_by_version_ids(
            &TenantId::System,
            &ProjectId::System,
            sd_results
                .entries
                .iter()
                .map(|v| VersionIdRef::new(v.version_id.as_ref()))
                .collect(),
        )
        .await?
        .into_iter()
        .filter_map(|r| match r {
            Resource::StructureDefinition(sd) => Some(sd),
            _ => None,
        });

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
                sds.map(|sd| CapabilityStatementRestResource {
                    type_: Box::new(FHIRCode {
                        value: sd.type_.value,
                        ..Default::default()
                    }),
                    profile: Some(Box::new(FHIRString {
                        value: sd.url.value,
                        ..Default::default()
                    })),
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
                .collect(),
            ),
            ..Default::default()
        }]),
        ..Default::default()
    })
}

pub fn capabilities<
    Repo: Repository + Send + Sync + 'static,
    Search: SearchEngine + Send + Sync + 'static,
>(
    state: ServerMiddlewareState<Repo, Search>,
    mut context: ServerMiddlewareContext,
    next: Option<Arc<ServerMiddlewareNext<Repo, Search>>>,
) -> ServerMiddlewareOutput {
    Box::pin(async move {
        match context.request {
            FHIRRequest::Capabilities => {
                let mut guard = CAPABILITIES.lock().await;

                if let Some(capabilities) = &*guard {
                    context.response = Some(FHIRResponse::Capabilities(FHIRCapabilitiesResponse {
                        capabilities: capabilities.clone(),
                    }));
                } else {
                    let capabilities =
                        generate_capabilities(state.repo.as_ref(), state.search.as_ref())
                            .await
                            .unwrap();
                    *guard = Some(capabilities.clone());

                    context.response = Some(FHIRResponse::Capabilities(FHIRCapabilitiesResponse {
                        capabilities: capabilities,
                    }));
                }

                Ok(context)
            }
            _ => {
                if let Some(next) = next {
                    next(state, context).await
                } else {
                    Err(OperationOutcomeError::fatal(
                        IssueType::Exception(None),
                        "No next middleware found".to_string(),
                    ))
                }
            }
        }
    })
}
