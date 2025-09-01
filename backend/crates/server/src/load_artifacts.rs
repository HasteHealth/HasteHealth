use std::sync::Arc;

use crate::{fhir_client::ServerCTX, services::create_services};
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use oxidized_artifacts::ARTIFACT_RESOURCES;
use oxidized_config::Config;
use oxidized_fhir_client::{
    FHIRClient,
    url::{Parameter, ParsedParameter},
};
use oxidized_fhir_model::r4::types::{Coding, FHIRCode, FHIRUri, Meta, Resource, ResourceType};
use oxidized_fhir_operation_error::OperationOutcomeError;
use oxidized_repository::types::{Author, ProjectId, SupportedFHIRVersions, TenantId};
use sha1::{Digest, Sha1};
// use tokio::task::JoinSet;

fn generate_sha256_hash(value: &Resource) -> String {
    let json =
        oxidized_fhir_serialization_json::to_string(value).expect("failed to serialize value.");
    let mut sha_hasher = Sha1::new();
    sha_hasher.update(json.as_bytes());
    let sha1 = sha_hasher.finalize();

    let sha_string = URL_SAFE_NO_PAD.encode(&sha1);

    sha_string
}

static HASH_TAG_SYSTEM: &str = "https://oxidized-health.app/fhir/CodeSystem/hash";

pub fn add_hash_tag(meta: &mut Option<Box<Meta>>, sha_hash: String) {
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

pub async fn load_artifacts(config: Box<dyn Config>) -> Result<(), OperationOutcomeError> {
    let services = create_services(config).await?;

    let ctx = Arc::new(ServerCTX {
        tenant: TenantId::System,
        project: ProjectId::System,
        fhir_version: SupportedFHIRVersions::R4,
        author: Author {
            id: "system".into(),
            kind: "admin".into(),
        },
    });

    for resource in ARTIFACT_RESOURCES.iter() {
        match &**resource {
            Resource::StructureDefinition(structure_definition) => {
                let sha_hash = generate_sha256_hash(*&resource);
                let mut structure_definition = structure_definition.clone();
                add_hash_tag(&mut structure_definition.meta, sha_hash.clone());

                let res = services
                    .fhir_client
                    .conditional_update(
                        ctx.clone(),
                        ResourceType::StructureDefinition,
                        vec![ParsedParameter::Resource(Parameter {
                            name: "_tag".to_string(),
                            value: vec![HASH_TAG_SYSTEM.to_string() + "|" + &sha_hash],
                            modifier: Some("not".to_string()),
                            chains: None,
                        })],
                        Resource::StructureDefinition(structure_definition.clone()),
                    )
                    .await;

                if let Ok(_res) = res {
                    println!("Updated StructureDefinition");
                } else if let Err(err) = res {
                    if err.outcome().issue[0].code.value == Some("invalid".to_string()) {
                        println!("BACKTRACE: {}", err.backtrace().unwrap());
                        panic!("INVALID");
                    }
                    // println!("Did not update StructureDefinition {:?}", err);
                }
            }
            Resource::ValueSet(valueset) => {
                let sha_hash = generate_sha256_hash(*&resource);
                let mut valueset = valueset.clone();
                add_hash_tag(&mut valueset.meta, sha_hash.clone());

                let res = services
                    .fhir_client
                    .conditional_update(
                        ctx.clone(),
                        ResourceType::ValueSet,
                        vec![ParsedParameter::Resource(Parameter {
                            name: "_tag".to_string(),
                            value: vec![HASH_TAG_SYSTEM.to_string() + "|" + &sha_hash],
                            modifier: Some("not".to_string()),
                            chains: None,
                        })],
                        Resource::ValueSet(valueset.clone()),
                    )
                    .await;

                if let Ok(_res) = res {
                    println!("Updated ValueSet");
                } else if let Err(err) = res {
                    if err.outcome().issue[0].code.value == Some("invalid".to_string()) {
                        println!("BACKTRACE: {}", err.backtrace().unwrap());
                        panic!("INVALID");
                    }
                    // println!("Did not update ValueSet {:?}", err);
                }
            }
            Resource::CodeSystem(code_system) => {
                let sha_hash = generate_sha256_hash(*&resource);
                let mut code_system = code_system.clone();
                add_hash_tag(&mut code_system.meta, sha_hash.clone());

                let res = services
                    .fhir_client
                    .conditional_update(
                        ctx.clone(),
                        ResourceType::CodeSystem,
                        vec![ParsedParameter::Resource(Parameter {
                            name: "_tag".to_string(),
                            value: vec![HASH_TAG_SYSTEM.to_string() + "|" + &sha_hash],
                            modifier: Some("not".to_string()),
                            chains: None,
                        })],
                        Resource::CodeSystem(code_system.clone()),
                    )
                    .await;

                if let Ok(_res) = res {
                    println!("Updated CodeSystem");
                } else if let Err(err) = res {
                    if err.outcome().issue[0].code.value == Some("invalid".to_string()) {
                        println!("BACKTRACE: {}", err.backtrace().unwrap());
                        panic!("INVALID");
                    }
                    // println!("Did not update StructureDefinition {:?}", err);
                }
            }
            Resource::SearchParameter(search_param) => {
                let sha_hash = generate_sha256_hash(*&resource);
                let mut search_param = search_param.clone();
                add_hash_tag(&mut search_param.meta, sha_hash.clone());

                let res = services
                    .fhir_client
                    .conditional_update(
                        ctx.clone(),
                        ResourceType::SearchParameter,
                        vec![ParsedParameter::Resource(Parameter {
                            name: "_tag".to_string(),
                            value: vec![HASH_TAG_SYSTEM.to_string() + "|" + &sha_hash],
                            modifier: Some("not".to_string()),
                            chains: None,
                        })],
                        Resource::SearchParameter(search_param.clone()),
                    )
                    .await;

                if let Ok(_res) = res {
                    println!("Updated SearchParameter");
                } else if let Err(err) = res {
                    if err.outcome().issue[0].code.value == Some("invalid".to_string()) {
                        println!("BACKTRACE: {}", err.backtrace().unwrap());
                        panic!("INVALID");
                    }
                    // println!("Did not update StructureDefinition {:?}", err);
                }
            }
            _ => {
                println!("Skipping resource.");
            }
        }
    }

    Ok(())
}
