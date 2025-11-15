use crate::utilities::{generate::capitalize, load};
use haste_fhir_generated_ops::generated::ValueSetExpand;
use haste_fhir_model::r4::generated::{
    resources::{Resource, ResourceType, ValueSet, ValueSetExpansionContains},
    terminology::IssueType,
};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_fhir_terminology::{
    FHIRTerminology, client::FHIRCanonicalTerminology, resolvers::CanonicalResolver,
};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use std::{
    collections::{BTreeMap, HashMap},
    pin::Pin,
    sync::Arc,
};
use walkdir::WalkDir;

#[derive(Hash, Ord, PartialOrd, Eq, PartialEq)]
struct Code {
    description: Option<String>,
    code: String,
}

fn flatten_concepts(contains: ValueSetExpansionContains) -> BTreeMap<String, Code> {
    let mut codes = BTreeMap::new();

    if let Some(code) = contains.code
        && let Some(code_string) = code.value.as_ref()
    {
        codes.insert(
            code_string.to_string(),
            Code {
                description: contains.display.and_then(|d| d.value),
                code: code_string.to_string(),
            },
        );
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
        .split(':')
        .map(|id| capitalize(id))
        .collect::<Vec<_>>()
        .join("_")
        .split('/')
        .map(|id| capitalize(id))
        .collect::<Vec<_>>()
        .join("_")
        // Replacements
        .replace(" ", "")
        .replace("<", "Greater")
        .replace(">", "Less")
        .replace("=", "Equal")
        .replace("[", "LeftSquareBracket")
        .replace("]", "RightSquareBracket")
        .replace("*", "Star")
        .replace("%", "Percent")
        .replace("!", "Not")
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
    } else if safe_string == "Null" {
        format!("_Null")
    } else {
        safe_string
    }
}

fn generate_enum_variants(value_set: ValueSet) -> Option<TokenStream> {
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
            .map(|concept| flatten_concepts(concept))
            .reduce(|mut codes, cur| {
                codes.extend(cur);
                codes
            })
            .unwrap_or_default();

        if codes.len() > 0 && codes.len() < 100 {
            let enum_variants = codes.iter().map(|(_code, code)| {
                let code_string = &code.code;
                let code_ident = format_ident!("{}", format_string(code_string));
                let doc_attribute = code.description.as_ref().map_or(quote! {}, |d| {
                    quote! {
                        #[doc = #d]
                    }
                });

                let code_attribute = quote! {
                    #[code = #code_string]
                };

                quote! {
                    #doc_attribute
                    #code_attribute
                    #code_ident(Option<Element>)
                }
            });
            let try_from_value_variants = codes.iter().map(|(_code, code)| {
                let code_string = &code.code;
                let code_ident = format_ident!("{}", format_string(code_string));
                quote! {
                    #code_string => Ok(#terminology_enum_name::#code_ident(None))
                }
            });
            let into_string_variants = codes.iter().map(|(_code, code)| {
                let code_string = &code.code;
                let code_ident = format_ident!("{}", format_string(code_string));
                quote! {
                    #terminology_enum_name::#code_ident(_) => Some(#code_string.to_string())
                }
            });

            let get_field_variants = codes.iter().map(|(_code, code)| {
                let code_string = &code.code;
                let code_ident = format_ident!("{}", format_string(code_string));

                quote! {
                    #terminology_enum_name::#code_ident(Some(e)) => e.get_field(field)
                }
            });

            let get_field_mut_variant = codes.iter().map(|(_code, code)| {
                let code_string = &code.code;
                let code_ident = format_ident!("{}", format_string(code_string));

                quote! {
                    #terminology_enum_name::#code_ident(Some(e)) => e.get_field_mut(field)
                }
            });

            return Some(quote! {
                #[derive(Debug, Clone, FHIRJSONSerialize, FHIRJSONDeserialize)]
                #[fhir_serialize_type = "valueset"]
                pub enum #terminology_enum_name {
                    #(#enum_variants),*,
                    #[doc = "If value is missing and just the element is present."]
                    Null(Option<Element>),
                }

                impl Default for #terminology_enum_name {
                    fn default() -> Self {
                        #terminology_enum_name::Null(None)
                    }
                }

                impl TryFrom<String> for #terminology_enum_name {
                    type Error = String;
                    fn try_from(value: String) -> Result<Self, String> {
                        match value.as_str() {
                            #(#try_from_value_variants),*,
                            _ => Err(format!("Unknown code '{}'", value)),
                        }
                    }
                }

                impl Into<Option<String>> for &#terminology_enum_name {
                     fn into(self) -> Option<String> {
                        match self {
                            #(#into_string_variants),*,
                            #terminology_enum_name::Null(_) => None,
                        }
                    }
                }

                impl MetaValue for #terminology_enum_name {
                    fn fields(&self) -> Vec<&'static str> {
                        vec!["value", "id", "extension"]
                    }

                    fn get_field<'a>(&'a self, field: &str) -> Option<&'a dyn MetaValue> {
                        match field {
                            "value" => {
                                let code_value: Option<String> = self.into();
                                if let Some(code_value) = code_value {
                                    let v = Box::new(code_value);
                                    let code_ref: &'a String = Box::leak(v);
                                    Some(code_ref)
                                } else {
                                    None
                                }
                            },
                            _ => match self {
                                #(#get_field_variants),*,
                                #terminology_enum_name::Null(Some(e)) => e.get_field(field),
                                _ => None,
                            }
                        }
                    }

                    fn get_field_mut<'a>(&'a mut self, field: &str) -> Option<&'a mut dyn MetaValue> {
                        match field {
                            "value" => None,
                            _ => match self {
                                #(#get_field_mut_variant),*,
                                #terminology_enum_name::Null(Some(e)) => e.get_field_mut(field),
                                _ => None,
                            }
                        }
                    }

                    fn get_index<'a>(&'a self, _index: usize) -> Option<&'a dyn MetaValue> {
                        None
                    }

                    fn get_index_mut<'a>(&'a mut self, _index: usize) -> Option<&'a mut dyn MetaValue> {
                        None
                    }

                    fn flatten(&self) -> Vec<&dyn MetaValue> {
                        vec![self]
                    }

                    fn as_any(&self) -> &dyn Any {
                        self
                    }

                    fn typename(&self) -> &'static str {
                        "FHIRCode"
                    }
                }
            });
        }
    }

    None
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
                .map_err(|f| OperationOutcomeError::error(IssueType::Exception(None), f))?;

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

struct InlineResolver {
    data: Arc<ResolverData>,
}

impl InlineResolver {
    pub fn new(data: Arc<ResolverData>) -> Self {
        InlineResolver { data }
    }
}

impl CanonicalResolver for InlineResolver {
    fn resolve(
        &self,
        resource_type: ResourceType,
        url: String,
    ) -> Pin<Box<dyn Future<Output = Result<Resource, OperationOutcomeError>> + Send>> {
        let data = self.data.clone();
        Box::pin(async move {
            if let Some(resources) = data.clone().get(&resource_type)
                && let Some(resource) = resources.get(&url)
            {
                Ok(resource.clone())
            } else {
                Err(OperationOutcomeError::error(
                    IssueType::NotFound(None),
                    format!("Could not resolve canonical url: {}", url),
                ))
            }
        })
    }
}

pub struct GeneratedTerminologies {
    pub tokens: TokenStream,
    #[allow(dead_code)]
    pub inlined_terminologies: HashMap<String, String>,
}

pub async fn generate(
    file_paths: &Vec<String>,
) -> Result<GeneratedTerminologies, OperationOutcomeError> {
    let data = load_terminologies(file_paths)?;

    let terminology = FHIRCanonicalTerminology::new(InlineResolver::new(data.clone()));

    let mut codes = Vec::new();

    let mut inlined_terminologies = HashMap::new();

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
                if let Ok(expanded_valueset) = expanded_valueset
                    && let Some(code_enum_code) = generate_enum_variants(expanded_valueset.return_)
                {
                    inlined_terminologies.insert(
                        valueset
                            .url
                            .clone()
                            .expect("VS must have url")
                            .value
                            .clone()
                            .expect("VS must have url"),
                        format_string(&valueset.id.clone().expect("ValueSet must have an id")),
                    );
                    codes.push(code_enum_code);
                }
            }
            _ => panic!("Expected ValueSet resource"),
        }
    }

    Ok(GeneratedTerminologies {
        inlined_terminologies,
        tokens: quote! {
            #![allow(non_camel_case_types)]
            /// DO NOT EDIT THIS FILE. It is auto-generated by the FHIR Rust code generator.
            use self::super::types::Element;
            use std::any::Any;
            use haste_reflect::MetaValue;
            use haste_fhir_serialization_json::derive::{FHIRJSONSerialize, FHIRJSONDeserialize};
            use std::io::Write;
            #(#codes)*
        },
    })
}
