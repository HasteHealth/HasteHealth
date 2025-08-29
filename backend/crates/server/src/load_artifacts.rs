use std::sync::{Arc, LazyLock};

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

static SYSTEM_PROJECT_TENANT: LazyLock<ProjectId> =
    LazyLock::new(|| ProjectId::new("system".to_string()));

pub async fn load_artifacts(config: Box<dyn Config>) -> Result<(), OperationOutcomeError> {
    let services = create_services(config).await?;

    let ctx: Arc<ServerCTX> = Arc::new(ServerCTX {
        tenant: TenantId::System,
        project: SYSTEM_PROJECT_TENANT.clone(),
        fhir_version: SupportedFHIRVersions::R4,
        author: Author {
            id: "system".into(),
            kind: "admin".into(),
        },
    });

    for resource in ARTIFACT_RESOURCES.iter() {
        match &**resource {
            Resource::StructureDefinition(sd) => {
                let sha_hash = generate_sha256_hash(*&resource);
                let mut sd = sd.clone();
                add_hash_tag(&mut sd.meta, sha_hash.clone());

                let resource_type =
                    unsafe { ResourceType::unchecked("StructureDefinition".to_string()) };

                let res = services
                    .fhir_client
                    .conditional_update(
                        ctx.clone(),
                        resource_type,
                        vec![ParsedParameter::Resource(Parameter {
                            name: "_tag".to_string(),
                            value: vec![HASH_TAG_SYSTEM.to_string() + ":" + &sha_hash],
                            modifier: Some("not".to_string()),
                            chains: None,
                        })],
                        Resource::StructureDefinition(sd),
                    )
                    .await;

                if let Ok(_res) = res {
                    println!("Updated StructureDefinition");
                } else if let Err(err) = res {
                    println!("Did not update StructureDefinition {:?}", err);
                }
            }
            _ => {
                println!("Skipping resource.");
            }
        }
    }

    Ok(())
}
