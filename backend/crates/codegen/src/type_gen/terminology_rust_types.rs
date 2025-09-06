use crate::utilities::{generate::capitalize, load};
use oxidized_fhir_generated_ops::generated::ValueSetExpand;
use oxidized_fhir_model::r4::generated::{
    resources::{Resource, ResourceType, ValueSet, ValueSetExpansionContains},
    types::FHIRCode,
};
use oxidized_fhir_operation_error::{OperationOutcomeCodes, OperationOutcomeError};
use oxidized_fhir_terminology::{
    CanonicalResolution, FHIRTerminology, client::FHIRCanonicalTerminology,
};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use std::{collections::BTreeMap, pin::Pin, sync::Arc};
use walkdir::WalkDir;

fn flatten_concepts(contains: ValueSetExpansionContains) -> Vec<Box<FHIRCode>> {
    let mut codes = vec![];

    if let Some(code) = contains.code {
        codes.push(code);
    }
    for contains in contains.contains.unwrap_or_default().into_iter() {
        codes.extend(flatten_concepts(contains));
    }

    codes
}

fn format_string(id: &str) -> String {
    let safe_string = id
        .split('-')
        .map(|id| capitalize(id))
        .collect::<Vec<_>>()
        .join("")
        .replace(" ", "")
        .replace("<", "Greater")
        .replace(">", "Less")
        .replace("=", "Equal")
        .replace("/", "_")
        .replace("[", "LeftSquareBracket")
        .replace("]", "RightSquareBracket")
        .replace(":", "_")
        .replace("*", "Star")
        .replace("%", "Percent")
        .replace("!", "__")
        .split('.')
        .map(|id| capitalize(id))
        .collect::<Vec<_>>()
        .join("");

    if safe_string.is_empty() {
        println!("Invalid '{}'", id);
        panic!();
    }

    if safe_string.as_bytes()[0].is_ascii_digit() {
        format!("V{}", safe_string)
    } else if safe_string == "Self" {
        format!("_Self")
    } else {
        safe_string
    }
}

fn generate_enum_variants(value_set: ValueSet) -> TokenStream {
    let terminology_enum_name = format_ident!(
        "{}",
        format_string(&value_set.id.clone().expect("ValueSet must have an id"))
    );

    if let Some(expansion) = value_set.expansion {
        let codes = expansion
            .clone()
            .contains
            .unwrap_or_default()
            .into_iter()
            .flat_map(|concept| flatten_concepts(concept))
            .collect::<Vec<_>>();

        if codes.len() > 0 && codes.len() < 100 {
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

type ResolverData = BTreeMap<ResourceType, BTreeMap<String, Resource>>;

fn load_terminologies(
    file_paths: &Vec<String>,
) -> Result<Arc<ResolverData>, OperationOutcomeError> {
    let mut resolver_data: ResolverData = BTreeMap::new();
    resolver_data.insert(ResourceType::ValueSet, BTreeMap::new());
    resolver_data.insert(ResourceType::CodeSystem, BTreeMap::new());

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
                                    let data = resolver_data
                                        .get_mut(&ResourceType::ValueSet)
                                        .expect("Must have ValueSet");
                                    data.insert(
                                        vs.url
                                            .clone()
                                            .expect("VS Must have url")
                                            .value
                                            .expect("VS must have url"),
                                        Resource::ValueSet(vs),
                                    );
                                }
                                Resource::CodeSystem(cs) => {
                                    let data = resolver_data
                                        .get_mut(&ResourceType::CodeSystem)
                                        .expect("Must have CodeSystem");
                                    data.insert(
                                        cs.url
                                            .clone()
                                            .expect("CS Must have url")
                                            .value
                                            .expect("CS must have url"),
                                        Resource::CodeSystem(cs),
                                    );
                                }
                                _ => {}
                            }
                        }
                    });
                }
                Resource::ValueSet(vs) => {
                    let data = resolver_data
                        .get_mut(&ResourceType::ValueSet)
                        .expect("Must have ValueSet");
                    data.insert(
                        vs.url
                            .clone()
                            .expect("VS Must have url")
                            .value
                            .expect("VS must have url"),
                        Resource::ValueSet(vs),
                    );
                }
                Resource::CodeSystem(cs) => {
                    let data = resolver_data
                        .get_mut(&ResourceType::CodeSystem)
                        .expect("Must have CodeSystem");
                    data.insert(
                        cs.url
                            .clone()
                            .expect("CS Must have url")
                            .value
                            .expect("CS must have url"),
                        Resource::CodeSystem(cs),
                    );
                }
                _ => {}
            }
        }
    }

    Ok(Arc::new(resolver_data))
}

fn create_resolver(data: Arc<ResolverData>) -> CanonicalResolution {
    Arc::new(Box::new(
        move |resource_type: ResourceType,
              url: String|
              -> Pin<
            Box<
                dyn std::future::Future<Output = Result<Resource, OperationOutcomeError>>
                    + Send
                    + Sync,
            >,
        > {
            let data = data.clone();
            Box::pin(async move {
                if let Some(resources) = data.clone().get(&resource_type)
                    && let Some(resource) = resources.get(&url)
                {
                    Ok(resource.clone())
                } else {
                    Err(OperationOutcomeError::error(
                        OperationOutcomeCodes::NotFound,
                        format!("Could not resolve canonical url: {}", url),
                    ))
                }
            })
        },
    ))
}

pub async fn generate_fhir_types_from_files(
    file_paths: &Vec<String>,
) -> Result<TokenStream, OperationOutcomeError> {
    let data = load_terminologies(file_paths)?;

    let terminology = FHIRCanonicalTerminology::new(create_resolver(data.clone()));

    let mut codes = Vec::new();

    for resource in data.get(&ResourceType::ValueSet).unwrap().values() {
        match resource {
            Resource::ValueSet(valueset) => {
                let expanded_valueset = terminology
                    .expand(ValueSetExpand::Input {
                        valueSet: Some(valueset.clone()),
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
                    })
                    .await;
                if let Ok(expanded_valueset) = expanded_valueset {
                    codes.push(generate_enum_variants(expanded_valueset.return_));
                }
            }
            _ => panic!("Expected ValueSet resource"),
        }
    }

    Ok(quote! {
        #(#codes)*
    })
}
