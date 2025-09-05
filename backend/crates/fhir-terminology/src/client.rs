use crate::FHIRTerminology;
use oxidized_fhir_client::{
    FHIRClient, // url::{Parameter, ParsedParameter},
    url::{Parameter, ParsedParameter},
};
use oxidized_fhir_generated_ops::generated::{
    CodeSystemLookup, ValueSetExpand, ValueSetValidateCode,
};
use oxidized_fhir_model::r4::types::{
    CodeSystem, CodeSystemConcept, FHIRString, FHIRUri, Resource, ResourceType, ValueSet,
    ValueSetComposeInclude, ValueSetComposeIncludeConceptDesignation, ValueSetExpansion,
    ValueSetExpansionContains,
};
// use oxidized_fhir_model::r4::types::ResourceType;
use oxidized_fhir_operation_error::{OperationOutcomeCodes, OperationOutcomeError};
use std::{borrow::Cow, marker::PhantomData, sync::Arc};

pub struct FHIRClientTerminology<CTX, Error, Client: FHIRClient<CTX, Error>> {
    _ctx: PhantomData<CTX>,
    _error: PhantomData<Error>,
    client: Arc<Box<Client>>,
}

async fn resolve_valueset<CTX, Client: FHIRClient<CTX, OperationOutcomeError>>(
    client: &Arc<Box<Client>>,
    ctx: CTX,
    input: &ValueSetExpand::Input,
) -> Result<Option<ValueSet>, OperationOutcomeError> {
    if let Some(valueset) = input.valueSet.as_ref() {
        return Ok(Some(valueset.clone()));
    } else if let Some(url) = &input.url.as_ref().and_then(|u| u.value.as_ref()) {
        let mut result = client
            .search_type(
                ctx,
                ResourceType::ValueSet,
                vec![ParsedParameter::Resource(Parameter {
                    name: "url".to_string(),
                    value: vec![url.to_string()],
                    modifier: None,
                    chains: None,
                })],
            )
            .await?;
        if result.len() > 1 {
            return Err(OperationOutcomeError::error(
                OperationOutcomeCodes::Duplicate,
                format!("Multiple ValueSet resources found for url {}", url),
            ));
        } else if let Some(resource) = result.pop() {
            return match resource {
                Resource::ValueSet(vs) => Ok(Some(vs)),
                _ => Ok(None),
            };
        }
    }
    Ok(None)
}

fn are_codes_inline(include: &ValueSetComposeInclude) -> bool {
    include.concept.is_some()
}

fn codes_inline_to_expansion(include: &ValueSetComposeInclude) -> Vec<ValueSetExpansionContains> {
    include
        .concept
        .as_ref()
        .map(|v| Cow::Borrowed(v))
        .unwrap_or(Cow::Owned(vec![]))
        .iter()
        .map(|c| ValueSetExpansionContains {
            system: include.system.clone(),
            code: Some(c.code.clone()),
            display: c.display.clone(),
            ..Default::default()
        })
        .collect()
}

async fn resolve_codesystem<CTX, Client: FHIRClient<CTX, OperationOutcomeError>>(
    client: &Arc<Box<Client>>,
    ctx: CTX,
    url: &str,
) -> Result<Option<CodeSystem>, OperationOutcomeError> {
    let mut result = client
        .search_type(
            ctx,
            ResourceType::CodeSystem,
            vec![ParsedParameter::Resource(Parameter {
                name: "url".to_string(),
                value: vec![url.to_string()],
                modifier: None,
                chains: None,
            })],
        )
        .await?;

    if result.len() > 1 {
        return Err(OperationOutcomeError::error(
            OperationOutcomeCodes::Duplicate,
            format!("Multiple ValueSet resources found for url {}", url),
        ));
    } else if let Some(resource) = result.pop() {
        return match resource {
            Resource::CodeSystem(code_system) => Ok(Some(code_system)),
            _ => Ok(None),
        };
    }

    Ok(None)
}

async fn get_concepts(
    codesystem: CodeSystem,
) -> Result<Vec<CodeSystemConcept>, OperationOutcomeError> {
    match codesystem.content.value.as_ref().map(|s| s.as_str()) {
        Some("not-present") => Err(OperationOutcomeError::error(
            OperationOutcomeCodes::NotSupported,
            "CodeSystem content is 'not-present'".to_string(),
        )),
        Some("fragment") | Some("complete") | Some("supplement") => {
            Ok(codesystem.concept.clone().unwrap_or_default())
        }
        Some(_) | None => Err(OperationOutcomeError::error(
            OperationOutcomeCodes::Invalid,
            "CodeSystem content has invalid value".to_string(),
        )),
    }
}

fn code_system_concept_to_valueset_expansion(
    url: Option<&str>,
    version: Option<&str>,
    codesystem_concept: Vec<CodeSystemConcept>,
) -> Vec<ValueSetExpansionContains> {
    codesystem_concept
        .into_iter()
        .map(|c| ValueSetExpansionContains {
            system: url.map(|url| {
                Box::new(FHIRUri {
                    value: Some(url.to_string()),
                    ..Default::default()
                })
            }),
            version: version.map(|v| {
                Box::new(FHIRString {
                    value: Some(v.to_string()),
                    ..Default::default()
                })
            }),
            code: Some(c.code),
            display: c.display,
            designation: c.designation.map(|designations| {
                designations
                    .into_iter()
                    .map(|d| ValueSetComposeIncludeConceptDesignation {
                        id: d.id,
                        extension: d.extension,
                        modifierExtension: d.modifierExtension,
                        language: d.language,
                        use_: d.use_,
                        value: d.value,
                    })
                    .collect::<Vec<_>>()
            }),
            contains: if let Some(concepts) = c.concept {
                Some(code_system_concept_to_valueset_expansion(
                    url, version, concepts,
                ))
            } else {
                None
            },
            ..Default::default()
        })
        .collect()
}

async fn get_valueset_expansion_contains<
    CTX: Clone,
    Client: FHIRClient<CTX, OperationOutcomeError>,
>(
    client: &Arc<Box<Client>>,
    ctx: CTX,
    include: &ValueSetComposeInclude,
) -> Result<Vec<ValueSetExpansionContains>, OperationOutcomeError> {
    if are_codes_inline(include) {
        Ok(codes_inline_to_expansion(include))
    } else if let Some(valueset_uris) = include.valueSet.as_ref() {
        let mut contains = vec![];
        for valueset_uri in valueset_uris {
            if let Some(valueset_uri) = valueset_uri.value.as_ref() {
                let output = expand_valueset(
                    client,
                    ctx.clone(),
                    &ValueSetExpand::Input {
                        url: Some(FHIRUri {
                            value: Some(valueset_uri.to_string()),
                            ..Default::default()
                        }),
                        valueSet: None,
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
                .await?;

                contains.extend(
                    output
                        .return_
                        .expansion
                        .unwrap_or_default()
                        .contains
                        .unwrap_or_default(),
                )
            }
        }
        Ok(contains)
    } else if let Some(system) = include.system.as_ref()
        && let Some(uri) = system.value.as_ref()
        && let Some(code_system) = resolve_codesystem(client, ctx, uri.as_str()).await?
    {
        let url = code_system.url.clone();
        let version = code_system.version.clone();

        return Ok(code_system_concept_to_valueset_expansion(
            url.and_then(|v| v.value).as_ref().map(|url| url.as_str()),
            version.and_then(|v| v.value).as_ref().map(|v| v.as_str()),
            get_concepts(code_system).await?,
        ));
    } else {
        Ok(vec![])
    }
}

async fn get_valueset_expansion<CTX: Clone, Client: FHIRClient<CTX, OperationOutcomeError>>(
    client: &Arc<Box<Client>>,
    ctx: CTX,
    value_set: &ValueSet,
) -> Result<Vec<ValueSetExpansionContains>, OperationOutcomeError> {
    let mut result = Vec::new();
    if let Some(compose) = value_set.compose.as_ref() {
        for include in compose.include.iter() {
            result.extend(get_valueset_expansion_contains(client, ctx.clone(), include).await?);
        }
    }
    Ok(result)
}

async fn expand_valueset<CTX: Clone, Client: FHIRClient<CTX, OperationOutcomeError>>(
    client: &Arc<Box<Client>>,
    ctx: CTX,
    input: &ValueSetExpand::Input,
) -> Result<ValueSetExpand::Output, OperationOutcomeError> {
    // Implementation would go here
    let value_set = resolve_valueset(client, ctx.clone(), input).await?;

    if let Some(mut value_set) = value_set {
        let contains = get_valueset_expansion(client, ctx.clone(), &value_set).await?;
        value_set.expansion = Some(ValueSetExpansion {
            contains: Some(contains),
            ..Default::default()
        });

        Ok(ValueSetExpand::Output { return_: value_set })
    } else {
        return Err(OperationOutcomeError::error(
            OperationOutcomeCodes::NotFound,
            "ValueSet could not be resolved".to_string(),
        ));
    }
}

impl<CTX: Send + Sync + Clone, Client: FHIRClient<CTX, OperationOutcomeError>> FHIRTerminology<CTX>
    for FHIRClientTerminology<CTX, OperationOutcomeError, Client>
{
    async fn expand(
        &self,
        ctx: CTX,
        input: &ValueSetExpand::Input,
    ) -> Result<ValueSetExpand::Output, OperationOutcomeError> {
        expand_valueset(&self.client, ctx.clone(), input).await
    }
    async fn validate(
        &self,
        _ctx: CTX,
        _input: &ValueSetValidateCode::Input,
    ) -> Result<ValueSetValidateCode::Output, OperationOutcomeError> {
        // Implementation would go here
        unimplemented!()
    }
    async fn lookup(
        &self,
        _ctx: CTX,
        _input: &CodeSystemLookup::Input,
    ) -> Result<CodeSystemLookup::Output, OperationOutcomeError> {
        // Implementation would go here
        unimplemented!()
    }
}
