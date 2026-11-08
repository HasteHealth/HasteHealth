use std::{collections::HashSet, sync::Arc};

use crate::{config::ServerConfig, fhir_client::ServerCTX, services::create_services};
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use haste_artifacts::ARTIFACT_RESOURCES;
use haste_fhir_client::{
    FHIRClient,
    request::{FHIRSearchTypeRequest, SearchRequest},
};
use haste_fhir_model::r4::generated::{
    resources::{
        Bundle, BundleEntry, BundleEntryRequest, OperationDefinition, Resource, ResourceType,
        SearchParameter, StructureDefinition,
    },
    terminology::{BundleType, HttpVerb, IssueType},
    types::{Coding, FHIRCode, FHIRUri, Meta},
};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_fhir_search::{MAX_COUNT, SearchEngine, SearchOptions};
use haste_jwt::{ProjectId, TenantId};
use haste_repository::{Repository, fhir::CachePolicy, types::SupportedFHIRVersions};
use sha1::{Digest, Sha1};

fn generate_sha256_hash(value: &Resource) -> String {
    let json = serde_json::to_string(value).expect("failed to serialize value.");
    let mut sha_hasher = Sha1::new();
    sha_hasher.update(json.as_bytes());
    let sha1 = sha_hasher.finalize();

    URL_SAFE_NO_PAD.encode(sha1)
}

static HASH_TAG_SYSTEM: &str = "https://haste.health/fhir/CodeSystem/hash";

fn _add_hash_tag(meta: &mut Option<Box<Meta>>, sha_hash: String) {
    let hash_tag = Coding {
        system: Some(Box::new(FHIRUri {
            value: Some(HASH_TAG_SYSTEM.to_string()),
            ..Default::default()
        })),
        code: Some(Box::new(FHIRCode {
            value: Some(sha_hash),
            ..Default::default()
        })),
        ..Default::default()
    };

    let meta = if let Some(meta) = meta {
        meta
    } else {
        *meta = Some(Box::new(Meta::default()));
        meta.as_mut().unwrap()
    };

    match &mut meta.tag {
        Some(tags) => tags.push(hash_tag),
        None => meta.tag = Some(vec![hash_tag]),
    }
}

fn add_hash_tag(resource: &mut Resource, sha_hash: String) {
    match resource {
        Resource::StructureDefinition(structure_definition) => {
            _add_hash_tag(&mut structure_definition.meta, sha_hash)
        }
        Resource::CodeSystem(code_system) => _add_hash_tag(&mut code_system.meta, sha_hash),
        Resource::ValueSet(value_set) => _add_hash_tag(&mut value_set.meta, sha_hash),
        Resource::SearchParameter(search_parameter) => {
            _add_hash_tag(&mut search_parameter.meta, sha_hash)
        }
        _ => {}
    }
}

fn get_id(resource: &Resource) -> String {
    match resource {
        Resource::StructureDefinition(structure_definition) => {
            structure_definition.id.clone().unwrap_or_default()
        }
        Resource::CodeSystem(code_system) => code_system.id.clone().unwrap_or_default(),
        Resource::ValueSet(value_set) => value_set.id.clone().unwrap_or_default(),
        Resource::SearchParameter(search_parameter) => {
            search_parameter.id.clone().unwrap_or_default()
        }
        _ => todo!(
            "Unsupported resource type '{}'",
            resource.resource_type().as_ref()
        ),
    }
}

pub fn get_resource_type(resource: &Resource) -> ResourceType {
    match resource {
        Resource::StructureDefinition(_) => ResourceType::StructureDefinition,
        Resource::CodeSystem(_) => ResourceType::CodeSystem,
        Resource::ValueSet(_) => ResourceType::ValueSet,
        Resource::SearchParameter(_) => ResourceType::SearchParameter,
        _ => todo!(
            "Unsupported resource type '{}'",
            resource.resource_type().as_ref()
        ),
    }
}

/// This deletes existing artifacts and then reloads them. In a single transaction.
pub async fn reset_artifacts(config: Arc<ServerConfig>) -> Result<(), OperationOutcomeError> {
    let services = create_services(config.clone()).await?;

    let transaction = services.transaction().await?;

    {
        let ctx = Arc::new(ServerCTX::system(
            TenantId::System,
            ProjectId::System,
            transaction.fhir_client.clone(),
            transaction.rate_limit.clone(),
        ));

        tracing::info!("Deleting existing CodeSystems");
        ctx.client
            .delete_type(
                ctx.clone(),
                ResourceType::CodeSystem,
                (vec![] as Vec<(String, Vec<String>)>).into(),
            )
            .await?;
        tracing::info!("Deleting existing ValueSets");
        ctx.client
            .delete_type(
                ctx.clone(),
                ResourceType::ValueSet,
                (vec![] as Vec<(String, Vec<String>)>).into(),
            )
            .await?;
        tracing::info!("Deleting existing StructureDefinitions");
        ctx.client
            .delete_type(
                ctx.clone(),
                ResourceType::StructureDefinition,
                (vec![] as Vec<(String, Vec<String>)>).into(),
            )
            .await?;
        tracing::info!("Deleting existing SearchParameters");
        ctx.client
            .delete_type(
                ctx.clone(),
                ResourceType::SearchParameter,
                (vec![] as Vec<(String, Vec<String>)>).into(),
            )
            .await?;
        _load_artifacts(ctx.clone()).await?;
    }

    transaction.commit().await?;

    Ok(())
}

// Used for both reloading artifacts and reset.
async fn _load_artifacts<Client: FHIRClient<Arc<ServerCTX<Client>>, OperationOutcomeError>>(
    ctx: Arc<ServerCTX<Client>>,
) -> Result<(), OperationOutcomeError> {
    let mut hashes = HashSet::new();
    let mut artifact_transaction_bundle_entries: Vec<BundleEntry> = vec![];
    let mut entry_meta: Vec<(ResourceType, String, String)> = vec![];

    for resource in ARTIFACT_RESOURCES.iter() {
        let sha_hash = generate_sha256_hash(resource);
        hashes.insert(sha_hash);

        match &resource {
            Resource::SearchParameter(_)
            | Resource::CodeSystem(_)
            | Resource::ValueSet(_)
            | Resource::StructureDefinition(_) => {
                let mut resource = resource.clone();
                let resource_type = get_resource_type(&resource);
                let resource_type_str = resource_type.as_ref();
                let id = get_id(&resource);
                let sha_hash = generate_sha256_hash(&resource);

                add_hash_tag(&mut resource, sha_hash.clone());

                entry_meta.push((resource_type.clone(), id.clone(), sha_hash.clone()));

                artifact_transaction_bundle_entries.push(BundleEntry {
                    resource: Some(Box::new(resource.clone())),
                    request: Some(BundleEntryRequest {
                        method: HttpVerb::put(),
                        url: Box::new(
                            format!(
                                "{resource_type_str}?_id={id}&_tag:not={HASH_TAG_SYSTEM}|{sha_hash}"
                            )
                            .into(),
                        ),
                        ..Default::default()
                    }),

                    ..Default::default()
                });
            }
            _ => {
                // println!("Skipping resource.");
            }
        }
    }

    let batch_response = ctx
        .client
        .batch(
            ctx.clone(),
            Bundle {
                type_: BundleType::batch(),
                entry: Some(artifact_transaction_bundle_entries),
                ..Default::default()
            },
        )
        .await?;

    let mut total_loaded = 0;
    for ((resource_type, id, sha_hash), entry) in
        entry_meta.iter().zip(batch_response.entry.iter().flatten())
    {
        match entry.response.as_ref().and_then(|r| r.outcome.as_deref()) {
            Some(Resource::OperationOutcome(outcome)) if !outcome.issue.is_empty() => {
                let issue = &outcome.issue[0];
                let diagnostic = issue
                    .diagnostics
                    .as_deref()
                    .and_then(|d| d.value.as_deref())
                    .unwrap_or("unknown");

                if issue.code == IssueType::invalid() {
                    tracing::error!("{:#?}", outcome);
                    panic!("INVALID");
                } else if issue.code == IssueType::conflict() {
                    // Ignore.
                } else {
                    tracing::error!(
                        "Failed to update '{}' with id '{}'. Issue code: '{:?}', diagnostic: '{}'",
                        resource_type.as_ref(),
                        id,
                        issue.code,
                        diagnostic
                    );
                }
            }
            _ => {
                total_loaded += 1;
                tracing::info!(
                    "Updated '{}' with id '{}' and sha '{}'",
                    resource_type.as_ref(),
                    entry
                        .resource
                        .as_deref()
                        .and_then(|r| r.id().as_deref())
                        .unwrap_or("unknown"),
                    sha_hash.as_str()
                );
            }
        }
    }

    tracing::info!(
        "Loaded a total of '{}' artifacts with unique hashes '{}'",
        total_loaded,
        hashes.len()
    );

    Ok(())
}

pub async fn load_artifacts(config: Arc<ServerConfig>) -> Result<(), OperationOutcomeError> {
    let services = create_services(config.clone()).await?;

    let ctx = Arc::new(ServerCTX::system(
        TenantId::System,
        ProjectId::System,
        services.fhir_client.clone(),
        services.rate_limit.clone(),
    ));

    _load_artifacts(ctx.clone()).await
}

/// Every resource of `resource_type` in the project that matches
/// `parameters`.
pub async fn search_all<Repo: Repository, Search: SearchEngine>(
    repo: &Repo,
    search_engine: &Search,
    tenant: &TenantId,
    project: &ProjectId,
    resource_type: ResourceType,
    parameters: &[(&str, &[&str])],
) -> Result<Vec<Resource>, OperationOutcomeError> {
    let parameters: Vec<(String, Vec<String>)> = parameters
        .iter()
        .map(|(name, values)| {
            (
                (*name).to_string(),
                values.iter().map(|value| (*value).to_string()).collect(),
            )
        })
        .collect();
    let results = search_engine
        .search(
            &SupportedFHIRVersions::R4,
            tenant,
            project,
            &SearchRequest::Type(FHIRSearchTypeRequest {
                resource_type,
                parameters: parameters.into(),
            }),
            Some(SearchOptions {
                count_limit: Some(MAX_COUNT),
                any_of: None,
            }),
        )
        .await?;
    let version_ids: Vec<_> = results.entries.iter().map(|e| &e.version_id).collect();

    repo.read_by_version_ids(tenant, project, &version_ids, CachePolicy::NoCache)
        .await
}

/// The system project's concrete base StructureDefinitions of `kinds`.
pub async fn get_all_sds<Repo: Repository, Search: SearchEngine>(
    kinds: &[&str],
    repo: &Repo,
    search_engine: &Search,
) -> Result<Vec<StructureDefinition>, OperationOutcomeError> {
    let sds = search_all(
        repo,
        search_engine,
        &TenantId::System,
        &ProjectId::System,
        ResourceType::StructureDefinition,
        &[
            ("kind", kinds),
            ("abstract", &["false"]),
            ("derivation", &["specialization"]),
        ],
    )
    .await?;

    Ok(sds
        .into_iter()
        .filter_map(|r| match r {
            Resource::StructureDefinition(sd) => Some(sd),
            _ => None,
        })
        .collect())
}

/// The system project's SearchParameters.
pub async fn get_all_sps<Repo: Repository, Search: SearchEngine>(
    repo: &Repo,
    search_engine: &Search,
) -> Result<Vec<SearchParameter>, OperationOutcomeError> {
    let sps = search_all(
        repo,
        search_engine,
        &TenantId::System,
        &ProjectId::System,
        ResourceType::SearchParameter,
        &[],
    )
    .await?;

    Ok(sps
        .into_iter()
        .filter_map(|r| match r {
            Resource::SearchParameter(sp) => Some(sp),
            _ => None,
        })
        .collect())
}

/// Every `OperationDefinition` installed for the user.
/// Includes the embeded operations we create
pub async fn get_all_operation_definitions<
    Client: FHIRClient<Arc<ServerCTX<Client>>, OperationOutcomeError> + 'static,
>(
    ctx: Arc<ServerCTX<Client>>,
) -> Result<Vec<OperationDefinition>, OperationOutcomeError> {
    let operations = ctx
        .client
        .search_type(
            ctx.clone(),
            ResourceType::OperationDefinition,
            (vec![] as Vec<(String, Vec<String>)>).into(),
        )
        .await?
        .entry
        .unwrap_or_default()
        .into_iter()
        .filter_map(|e| e.resource)
        .filter_map(|r| match *r {
            Resource::OperationDefinition(operation) => Some(operation),
            _ => None,
        })
        .collect();

    Ok(operations)
}
