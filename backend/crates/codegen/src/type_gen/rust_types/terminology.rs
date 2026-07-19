use crate::utilities::{generate::capitalize, load};
use haste_fhir_client::canonical_resolver::CanonicalResolver;
use haste_fhir_generated_ops::generated::ValueSetExpand;
use haste_fhir_model::r4::generated::{
    resources::{Resource, ResourceType, ValueSet, ValueSetExpansionContains},
    terminology::IssueType,
};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_fhir_terminology::{FHIRTerminology, client::FHIRCanonicalTerminology};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use std::{
    collections::{BTreeMap, HashMap},
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
        .replace(";", "Semicolon")
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
    let terminology_url = value_set
        .url
        .as_ref()
        .and_then(|v| v.value.as_ref())
        .clone()
        .expect("ValueSet must have a url");

    if let Some(expansion) = value_set.expansion {
        let codes_map = expansion
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

        let mut codes = codes_map.values().collect::<Vec<_>>();

        codes.sort_by(|a, b| a.code.cmp(&b.code));

        let code_vec = codes.iter().map(|c| &c.code).collect::<Vec<_>>();

        let const_variants = codes.iter().enumerate().map(|(i, c)| {
            let variant_name = format_ident!("{}", &format_string(&c.code).to_uppercase());
            let display = c.description.as_ref().map(|d| d.as_str()).unwrap_or("");
            let index = i as u16;

            quote! {
                #[doc = #display]
                pub const #variant_name: BoundCode<Self> = BoundCode::from_index(#index);
            }
        });

        if codes.len() > 0 && codes.len() < 400 {
            return Some(quote! {
                #[doc = #terminology_url]
                pub struct #terminology_enum_name;

                impl ValueSetDef for #terminology_enum_name {
                    const URL: &'static str = #terminology_url;
                    const CODES: &'static [&'static str] = &[#(#code_vec),*];
                }

                impl #terminology_enum_name {
                    #(#const_variants)*
                    #[doc = "Element present without a value."]
                    pub const NULL: BoundCode<Self> = BoundCode::null();
                }


            });
        }
    }

    None
}

type ResolverData = BTreeMap<ResourceType, BTreeMap<String, Arc<Resource>>>;

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
            // Filter for json only
            .filter(|e| e.path().extension().map_or(false, |ext| ext == "json"))
        {
            let resource = load::load_from_file(entry.path())
                .map_err(|f| OperationOutcomeError::error(IssueType::EXCEPTION, f))?;

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
                                        Arc::new(Resource::ValueSet(vs)),
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
                                        Arc::new(Resource::CodeSystem(cs)),
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
                        Arc::new(Resource::ValueSet(vs)),
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
                        Arc::new(Resource::CodeSystem(cs)),
                    );
                }
                _ => {}
            }
        }
    }

    Ok(Arc::new(resolver_data))
}

#[derive(Clone)]
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
        url: &str,
    ) -> impl Future<Output = Result<Option<Arc<Resource>>, OperationOutcomeError>> + Send {
        let data = self.data.clone();
        Box::pin(async move {
            if let Some(resources) = data.clone().get(&resource_type)
                && let Some(resource) = resources.get(url)
            {
                Ok(Some(resource.clone()))
            } else {
                Err(OperationOutcomeError::error(
                    IssueType::NOTFOUND,
                    format!("Could not resolve canonical url: {}", url),
                ))
            }
        })
    }
}

pub struct GeneratedTerminologies {
    pub tokens: TokenStream,
    pub inlined_terminologies: HashMap<String, String>,
}

// Sets up the main datastructures to be used by generated inline terminologies
fn prebuilt_code() -> TokenStream {
    quote! {
        use crate::r4::generated::types::{Element, Extension};
        use haste_reflect::MetaValue;
        use serde::{Deserialize, Deserializer, Serialize, Serializer};
        use std::{any::Any, fmt, marker::PhantomData, sync::OnceLock};

        pub trait ValueSetDef: 'static + Send + Sync {
            const URL: &'static str;
            const CODES: &'static [&'static str]; // sorted; codegen enforces
        }

        pub struct BoundCode<VS: ValueSetDef> {
            code: Option<u16>, // index into VS::CODES; None = today's `Null` variant
            element: Option<Element>,
            value_cache: OnceLock<String>, // lazily materializes CODES[i] as an owned String, so MetaValue::get_field("value") matches FHIRCode's Option<String> shape
            _vs: PhantomData<VS>,
        }

        impl<VS: ValueSetDef> BoundCode<VS> {
            pub const fn from_index(i: u16) -> Self {
                Self {
                    code: Some(i),
                    element: None,
                    value_cache: OnceLock::new(),
                    _vs: PhantomData,
                }
            }
            pub const fn null() -> Self {
                Self {
                    code: None,
                    element: None,
                    value_cache: OnceLock::new(),
                    _vs: PhantomData,
                }
            }

            pub fn new(s: &str) -> Option<Self> {
                VS::CODES
                    .binary_search(&s)
                    .ok()
                    .map(|i| Self::from_index(i as u16))
            }
            pub fn as_str(&self) -> Option<&'static str> {
                self.code.map(|i| VS::CODES[i as usize])
            }
            pub fn element(&self) -> Option<&Element> {
                self.element.as_ref()
            }
            pub fn element_mut(&mut self) -> &mut Element {
                self.element.get_or_insert_with(Default::default)
            }
            pub fn extension_mut(&mut self) -> &mut Option<Vec<Box<Extension>>> {
                &mut self.element_mut().extension
            }
            pub fn id_mut(&mut self) -> &mut Option<String> {
                &mut self.element_mut().id
            }

            pub fn empty(&self) -> bool {
                self.code.is_none() && self.element.is_none()
            }

            pub fn serialize_as_field<M: serde::ser::SerializeMap>(
                &self,
                field_name: &str,
                serializer: &mut M,
            ) -> Result<(), M::Error> {
                let code = self.as_str();
                let element = self.element();

                if let Some(value) = code {
                    serializer.serialize_entry(field_name, &value)?;
                }

                if let Some(element) = element {
                    let element_key = format!("_{}", field_name);
                    serializer.serialize_entry(&element_key, element)?;
                }

                Ok(())
            }

            pub fn serialize_as_vector<M: serde::ser::SerializeMap>(
                field_name: &str,
                values: &[Self],
                serializer: &mut M,
            ) -> Result<(), M::Error> {
                let value_array: Vec<Option<&'static str>> = values.iter().map(|v| v.as_str()).collect();
                let element_array: Vec<Option<_>> = values.iter().map(|v| v.element()).collect();

                if value_array.iter().any(|v| v.is_some()) {
                    serializer.serialize_entry(field_name, &value_array)?;
                }

                if element_array.iter().any(|e| e.is_some()) {
                    let element_key = format!("_{}", field_name);
                    let element_array: Vec<Option<_>> = values.iter().map(|v| v.element()).collect();
                    serializer.serialize_entry(&element_key, &element_array)?;
                }

                Ok(())
            }
        }

        impl<VS: ValueSetDef> Clone for BoundCode<VS> {
            fn clone(&self) -> Self {
                Self {
                    code: self.code,
                    element: self.element.clone(),
                    value_cache: OnceLock::new(),
                    _vs: PhantomData,
                }
            }
        }

        impl<VS: ValueSetDef> fmt::Debug for BoundCode<VS> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                // e.g. `AdministrativeGender("male")`
                write!(f, "{}({:?})", std::any::type_name::<VS>(), self.as_str())
            }
        }

        // Code-index equality only, per the earlier discussion: `g == AdministrativeGender::MALE`
        // must not go false because the value carries an extension.
        impl<VS: ValueSetDef> PartialEq for BoundCode<VS> {
            fn eq(&self, other: &Self) -> bool {
                self.code == other.code
            }
        }
        impl<VS: ValueSetDef> Eq for BoundCode<VS> {}

        impl<VS: ValueSetDef> MetaValue for BoundCode<VS> {
            fn fields(&self) -> Vec<&'static str> {
                vec!["value", "id", "extension"]
            }

            fn get_field<'a>(&'a self, field: &str) -> Option<&'a dyn MetaValue> {
                match field {
                    "value" => self.code.as_ref().map(|i| {
                        self.value_cache.get_or_init(|| VS::CODES[*i as usize].to_string()) as &dyn MetaValue
                    }),
                    _ => self.element.as_ref().and_then(|e| e.get_field(field)),
                }
            }

            fn get_field_mut<'a>(&'a mut self, field: &str) -> Option<&'a mut dyn MetaValue> {
                match field {
                    "value" => None,
                    _ => self.element.as_mut().and_then(|e| e.get_field_mut(field)),
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

            fn fhir_type(&self) -> &'static str {
                "code"
            }

            fn is_many(&self) -> bool {
                false
            }
        }

        // Non-generic over VS — monomorphizes per Deserializer, i.e. ~once.
        fn parse_code<E: serde::de::Error>(
            codes: &'static [&'static str],
            url: &'static str,
            s: &str,
        ) -> Result<u16, E> {
            codes
                .binary_search(&s)
                .map(|i| i as u16)
                .map_err(|_| E::custom(format_args!("'{s}' is not a valid code in ValueSet {url}")))
        }

        impl<'de, VS: ValueSetDef> Deserialize<'de> for BoundCode<VS> {
            fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
                // Element/_field handling stays at the parent-struct level,
                // exactly as your current derive does it.
                let s = <&str>::deserialize(d)?; // or Cow, matching current behavior
                parse_code::<D::Error>(VS::CODES, VS::URL, s).map(Self::from_index)
            }
        }

        impl<VS: ValueSetDef> Serialize for BoundCode<VS> {
            fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
                match self.as_str() {
                    Some(c) => s.serialize_str(c),
                    None => s.serialize_none(), // Null case; match current `_field`-only semantics
                }
            }
        }

        impl<VS: ValueSetDef> Default for BoundCode<VS> {
            fn default() -> Self {
                Self::null()
            }
        }
    }
}

pub async fn generate(
    file_paths: &Vec<String>,
) -> Result<GeneratedTerminologies, OperationOutcomeError> {
    let data = load_terminologies(file_paths)?;

    let resolver = InlineResolver::new(data.clone());
    let terminology = FHIRCanonicalTerminology::new();

    let mut codes = Vec::new();

    let mut inlined_terminologies = HashMap::new();

    for resource in data.get(&ResourceType::ValueSet).unwrap().values() {
        match &**resource {
            Resource::ValueSet(valueset) => {
                let expanded_valueset = terminology
                    .expand(
                        resolver.clone(),
                        ValueSetExpand::Input {
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
                        },
                    )
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

    let prebuilt_code = prebuilt_code();

    Ok(GeneratedTerminologies {
        inlined_terminologies,
        tokens: quote! {
            #![allow(dead_code)]
            #![allow(non_camel_case_types)]
            /// DO NOT EDIT THIS FILE. It is auto-generated by the FHIR Rust code generator.
            #prebuilt_code

            #(#codes)*
        },
    })
}
