use std::{collections::HashSet, sync::Arc};

use crate::{ServerEnvironmentVariables, fhir_client::ServerCTX, services::create_services};
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use haste_artifacts::ARTIFACT_RESOURCES;
use haste_config::Config;
use haste_fhir_client::{
    FHIRClient,
    request::{FHIRSearchTypeRequest, SearchRequest},
    url::{Parameter, ParsedParameter, ParsedParameters},
};
use haste_fhir_model::r4::generated::{
    resources::{Resource, ResourceType, SearchParameter, StructureDefinition},
    terminology::IssueType,
    types::{Coding, FHIRCode, FHIRUri, Meta},
};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_fhir_search::{SearchEngine, SearchOptions};
use haste_jwt::{ProjectId, TenantId};

use haste_repository::{Repository, fhir::CachePolicy, types::SupportedFHIRVersions};
use sha1::{Digest, Sha1};

fn generate_sha256_hash(value: &Resource) -> String {
    let json = haste_fhir_serialization_json::to_string(value).expect("failed to serialize value.");
    let mut sha_hasher = Sha1::new();
    sha_hasher.update(json.as_bytes());
    let sha1 = sha_hasher.finalize();

    let sha_string = URL_SAFE_NO_PAD.encode(&sha1);

    sha_string
}

static HASH_TAG_SYSTEM: &str = "https://haste.health/fhir/CodeSystem/hash";

fn _add_hash_tag(meta: &mut Option<Box<Meta>>, sha_hash: String) {
    let hash_tag = Box::new(Coding {
        system: Some(Box::new(FHIRUri {
            value: Some(HASH_TAG_SYSTEM.to_string()),
            ..Default::default()
        })),
        code: Some(Box::new(FHIRCode {
            value: Some(sha_hash),
            ..Default::default()
        })),
        ..Default::default()
    });

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
        _ => todo!("Unsupported resource type"),
    }
}

pub fn get_resource_type(resource: &Resource) -> ResourceType {
    match resource {
        Resource::StructureDefinition(_) => ResourceType::StructureDefinition,
        Resource::CodeSystem(_) => ResourceType::CodeSystem,
        Resource::ValueSet(_) => ResourceType::ValueSet,
        Resource::SearchParameter(_) => ResourceType::SearchParameter,
        _ => todo!("Unsupported resource type"),
    }
}

pub async fn load_artifacts(
    config: Arc<dyn Config<ServerEnvironmentVariables>>,
) -> Result<(), OperationOutcomeError> {
    let services = create_services(config.clone()).await?;

    let ctx = Arc::new(ServerCTX::system(
        TenantId::System,
        ProjectId::System,
        services.fhir_client.clone(),
        services.rate_limit.clone(),
    ));

    let mut hashes = HashSet::new();

    for resource in ARTIFACT_RESOURCES.iter() {
        let sha_hash = generate_sha256_hash(*&resource);
        hashes.insert(sha_hash);

        match &**resource {
            Resource::SearchParameter(_)
            | Resource::CodeSystem(_)
            | Resource::ValueSet(_)
            | Resource::StructureDefinition(_) => {
                let mut resource = (**resource).clone();
                let resource_type = get_resource_type(&resource);
                let id = get_id(&resource);
                let sha_hash = generate_sha256_hash(&resource);

                add_hash_tag(&mut resource, sha_hash.clone());

                let res = services
                    .fhir_client
                    .conditional_update(
                        ctx.clone(),
                        resource_type.clone(),
                        ParsedParameters::new(vec![
                            ParsedParameter::Resource(Parameter {
                                name: "_id".to_string(),
                                value: vec![id.clone()],
                                modifier: None,
                                chains: None,
                            }),
                            ParsedParameter::Resource(Parameter {
                                name: "_tag".to_string(),
                                value: vec![HASH_TAG_SYSTEM.to_string() + "|" + &sha_hash],
                                modifier: Some("not".to_string()),
                                chains: None,
                            }),
                        ]),
                        resource.clone(),
                    )
                    .await;

                if let Ok(_res) = res {
                    println!("Updated {}", resource_type.as_ref());
                } else if let Err(err) = res {
                    if let IssueType::Invalid(_) = err.outcome().issue[0].code.as_ref() {
                        println!("BACKTRACE: {}", err.backtrace().unwrap());
                        panic!("INVALID");
                    }
                }
            }
            _ => {
                // println!("Skipping resource.");
            }
        }
    }

    println!(
        "Loaded a total of '{}' artifacts with unique hashes '{}'",
        ARTIFACT_RESOURCES.len(),
        hashes.len(),
    );

    Ok(())
}

pub async fn get_all_sds<Repo: Repository, Search: SearchEngine>(
    kinds: &[&str],
    repo: &Repo,
    search_engine: &Search,
) -> Result<Vec<StructureDefinition>, OperationOutcomeError> {
    let sd_search = FHIRSearchTypeRequest {
        resource_type: ResourceType::StructureDefinition,
        parameters: ParsedParameters::new(vec![
            ParsedParameter::Resource(Parameter {
                name: "kind".to_string(),
                value: kinds.iter().map(|s| s.to_string()).collect(),
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
            &[&ProjectId::System],
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

pub async fn get_all_sps<Repo: Repository, Search: SearchEngine>(
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
            &[&ProjectId::System],
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
