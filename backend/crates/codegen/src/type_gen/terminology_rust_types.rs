use oxidized_config::{ConfigType, get_config};
use oxidized_fhir_generated_ops::generated::ValueSetExpand;
use oxidized_fhir_model::r4::types::{FHIRCode, ValueSet, ValueSetExpansionContains};
use oxidized_fhir_operation_error::OperationOutcomeError;
use oxidized_fhir_terminology::{FHIRTerminology, client::FHIRClientTerminology};
use oxidized_repository::types::{Author, ProjectId, TenantId};
use oxidized_server::{fhir_client::ServerCTX, services::create_services};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use std::sync::Arc;

pub fn flatten_concepts(contains: ValueSetExpansionContains) -> Vec<Box<FHIRCode>> {
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

pub fn generate_enum_variants(valueset: ValueSet) -> TokenStream {
    let terminology_enum_name = format_ident!("{}", valueset.id.expect("ValueSet must have an id"));
    if let Some(expansion) = valueset.expansion {
        let codes = expansion
            .contains
            .unwrap_or_default()
            .into_iter()
            .flat_map(|concept| flatten_concepts(concept));

        let enum_variants = codes.filter_map(|v| v.value).map(|code| {
            let code_ident = format_ident!("{}", code);
            quote! {
                #code_ident(Element)
            }
        });

        quote! {
            pub enum #terminology_enum_name {
                #(#enum_variants),*
            }
        }
    } else {
        quote! {}
    }
}

pub async fn create_terminologies(
    valuesets: Vec<ValueSet>,
) -> Result<TokenStream, OperationOutcomeError> {
    let config = get_config(ConfigType::Environment);
    let services = create_services(config).await?;
    let client = services.fhir_client.clone();

    let terminology = FHIRClientTerminology::new(client);
    let ctx = Arc::new(ServerCTX {
        author: Author {
            id: "root".to_string(),
            kind: "admin".to_string(),
        },
        tenant: TenantId::System,
        project: ProjectId::System,
        fhir_version: oxidized_repository::types::SupportedFHIRVersions::R4,
    });

    for valueset in valuesets {
        let k = terminology
            .expand(
                ctx.clone(),
                ValueSetExpand::Input {
                    valueSet: Some(valueset),
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
    }

    todo!();
}
