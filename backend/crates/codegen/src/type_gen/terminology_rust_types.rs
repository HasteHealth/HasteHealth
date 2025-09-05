use crate::utilities::{generate::capitalize, load};
use oxidized_config::{ConfigType, get_config};
use oxidized_fhir_generated_ops::generated::ValueSetExpand;
use oxidized_fhir_model::r4::types::{FHIRCode, Resource, ValueSet, ValueSetExpansionContains};
use oxidized_fhir_operation_error::{OperationOutcomeCodes, OperationOutcomeError};
use oxidized_fhir_terminology::{FHIRTerminology, client::FHIRCanonicalTerminology};
use oxidized_repository::types::{Author, ProjectId, TenantId};
use oxidized_server::{fhir_client::ServerCTX, services::create_services};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use std::sync::Arc;
use walkdir::WalkDir;

fn flatten_concepts(contains: ValueSetExpansionContains) -> Vec<Box<FHIRCode>> {
    let mut codes = vec![];
    for concept in contains.contains.unwrap_or_default().into_iter() {
        if let Some(code) = concept.code {
            codes.push(code);
        }
        if let Some(expansions) = concept.contains {
            for contains in expansions {
                codes.extend(flatten_concepts(contains));
            }
        }
    }

    codes
}

fn format_string(id: &str) -> String {
    let k = id
        .split('-')
        .map(|id| capitalize(id))
        .collect::<Vec<_>>()
        .join("")
        .replace(" ", "");

    k
}

fn generate_enum_variants(value_set: ValueSet) -> TokenStream {
    let terminology_enum_name = format_ident!(
        "{}",
        format_string(&value_set.id.expect("ValueSet must have an id"))
    );
    if let Some(expansion) = value_set.expansion {
        let codes = expansion
            .contains
            .unwrap_or_default()
            .into_iter()
            .flat_map(|concept| flatten_concepts(concept))
            .collect::<Vec<_>>();

        if codes.len() < 100 {
            let enum_variants = codes.into_iter().filter_map(|v| v.value).map(|code| {
                let code_ident = format_ident!("{}", format_string(&code));
                quote! {
                    #code_ident(Option<Element>)
                }
            });

            return quote! {
                pub enum #terminology_enum_name {
                    #(#enum_variants),*
                }
            };
        }
    }

    quote! {}
}

fn load_terminologies(file_paths: &Vec<String>) -> Result<Vec<ValueSet>, OperationOutcomeError> {
    let mut value_sets = Vec::new();
    for dir_path in file_paths {
        let walker = WalkDir::new(dir_path).into_iter();
        for entry in walker
            .filter_map(|e| e.ok())
            .filter(|e| e.metadata().unwrap().is_file())
        {
            let resource = load::load_from_file(entry.path())
                .map_err(|f| OperationOutcomeError::error(OperationOutcomeCodes::Exception, f))?;

            match resource {
                Resource::Bundle(bundle) => {
                    bundle.entry.unwrap_or_default().into_iter().for_each(|e| {
                        if let Some(resource) = e.resource {
                            match *resource {
                                Resource::ValueSet(vs) => {
                                    value_sets.push(vs);
                                }
                                _ => {}
                            }
                        }
                    });
                }
                Resource::ValueSet(vs) => {
                    value_sets.push(vs);
                }
                _ => {}
            }
        }
    }

    Ok(value_sets)
}

pub async fn generate_fhir_types_from_files(
    file_paths: &Vec<String>,
) -> Result<TokenStream, OperationOutcomeError> {
    let config = get_config(ConfigType::Environment);
    let services = create_services(config).await?;
    let client = services.fhir_client.clone();

    let value_sets = load_terminologies(file_paths)?;

    let terminology = FHIRCanonicalTerminology::new(client);
    let ctx = Arc::new(ServerCTX {
        author: Author {
            id: "root".to_string(),
            kind: "admin".to_string(),
        },
        tenant: TenantId::System,
        project: ProjectId::System,
        fhir_version: oxidized_repository::types::SupportedFHIRVersions::R4,
    });

    let mut codes = Vec::new();

    for valueset_to_expand in value_sets {
        let expanded_valueset = terminology
            .expand(
                ctx.clone(),
                ValueSetExpand::Input {
                    valueSet: Some(valueset_to_expand),
                    url: None,
                    valueSetVersion: None,
                    context: None,
                    contextDirection: None,
                    filter: None,
                    date: None,
                    offset: None,
                    count: None,
                    includeDesignations: None,
                    designation: None,
                    includeDefinition: None,
                    activeOnly: None,
                    excludeNested: None,
                    excludeNotForUI: None,
                    excludePostCoordinated: None,
                    displayLanguage: None,
                    exclude_system: None,
                    system_version: None,
                    check_system_version: None,
                    force_system_version: None,
                },
            )
            .await?
            .return_;

        codes.push(generate_enum_variants(expanded_valueset));
    }

    Ok(quote! {
        #(#codes)*
    })
}
