use crate::FHIRTerminology;
use haste_fhir_client::canonical_resolver::CanonicalResolver;
use haste_fhir_generated_ops::generated::{CodeSystemLookup, ValueSetExpand, ValueSetValidateCode};
use haste_fhir_model::r4::{
    datetime::DateTime,
    generated::{
        resources::{
            CodeSystem, CodeSystemConcept, Resource, ResourceType, ValueSet,
            ValueSetComposeInclude, ValueSetComposeIncludeConceptDesignation, ValueSetExpansion,
            ValueSetExpansionContains,
        },
        terminology::{CodesystemContentMode, IssueType},
        types::{FHIRBoolean, FHIRDateTime, FHIRString, FHIRUri},
    },
};
use haste_fhir_operation_error::OperationOutcomeError;
use std::{borrow::Cow, pin::Pin, sync::Arc};

pub struct FHIRCanonicalTerminology;

impl Default for FHIRCanonicalTerminology {
    fn default() -> Self {
        Self::new()
    }
}

impl FHIRCanonicalTerminology {
    #[must_use]
    pub fn new() -> Self {
        FHIRCanonicalTerminology
    }
}

async fn resolve_valueset<Resolver: CanonicalResolver>(
    canonical_resolution: Resolver,
    mut input: ValueSetExpand::Input,
) -> Result<Option<Arc<Resource>>, OperationOutcomeError> {
    if input.valueSet.is_some() {
        let mut valueset: Option<ValueSet> = None;
        std::mem::swap(&mut input.valueSet, &mut valueset);
        Ok(valueset.map(|v| Arc::new(Resource::ValueSet(v))))
    } else if let Some(url) = &input.url.as_ref().and_then(|u| u.value.as_ref()) {
        let resolved_resource = canonical_resolution
            .resolve(ResourceType::ValueSet, url)
            .await?;

        Ok(resolved_resource)
    } else {
        Ok(None)
    }
}

fn are_codes_inline(include: &ValueSetComposeInclude) -> bool {
    include.concept.is_some()
}

fn codes_inline_to_expansion(include: &ValueSetComposeInclude) -> Vec<ValueSetExpansionContains> {
    include
        .concept
        .as_ref()
        .map_or(Cow::Owned(vec![]), Cow::Borrowed)
        .iter()
        .map(|c| ValueSetExpansionContains {
            system: include.system.clone(),
            code: Some(c.code.clone()),
            display: c.display.clone(),
            ..Default::default()
        })
        .collect()
}

async fn resolve_codesystem<Resolver: CanonicalResolver>(
    canonical_resolution: Resolver,
    url: &str,
) -> Result<Option<Arc<Resource>>, OperationOutcomeError> {
    let code_system = canonical_resolution
        .resolve(ResourceType::CodeSystem, url)
        .await?;

    Ok(code_system)
}

fn get_concepts(codesystem: &CodeSystem) -> Result<Vec<CodeSystemConcept>, OperationOutcomeError> {
    match &codesystem.content {
        content_type if content_type == &CodesystemContentMode::not_present() => {
            Err(OperationOutcomeError::error(
                IssueType::not_supported(),
                "CodeSystem content is 'not-present'".to_string(),
            ))
        }
        content_type
            if content_type == &CodesystemContentMode::fragment()
                || content_type == &CodesystemContentMode::complete()
                || content_type == &CodesystemContentMode::supplement() =>
        {
            Ok(codesystem.concept.clone().unwrap_or_default())
        }
        _ => Err(OperationOutcomeError::error(
            IssueType::invalid(),
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
            contains: c
                .concept
                .map(|concepts| code_system_concept_to_valueset_expansion(url, version, concepts)),
            ..Default::default()
        })
        .collect()
}

async fn get_valueset_expansion_contains<
    Resolver: CanonicalResolver + Send + Clone + Sync + 'static,
>(
    canonical_resolution: Resolver,
    include: &ValueSetComposeInclude,
) -> Result<Vec<ValueSetExpansionContains>, OperationOutcomeError> {
    if are_codes_inline(include) {
        Ok(codes_inline_to_expansion(include))
    } else if let Some(valueset_uris) = include.valueSet.as_ref() {
        let mut contains = vec![];
        for valueset_uri in valueset_uris {
            if let Some(valueset_uri) = valueset_uri.value.as_ref() {
                let output = expand_valueset(
                    canonical_resolution.clone(),
                    ValueSetExpand::Input {
                        url: Some(FHIRUri {
                            value: Some(valueset_uri.clone()),
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
                );
            }
        }
        Ok(contains)
    } else if let Some(system) = include.system.as_ref()
        && let Some(uri) = system.value.as_ref()
        && let Some(resource) =
            resolve_codesystem(canonical_resolution.clone(), uri.as_str()).await?
        && let Resource::CodeSystem(code_system) = &*resource
    {
        let url = code_system.url.clone();
        let version = code_system.version.clone();

        Ok(code_system_concept_to_valueset_expansion(
            url.and_then(|v| v.value).as_deref(),
            version.and_then(|v| v.value).as_deref(),
            get_concepts(code_system)?,
        ))
    } else {
        Ok(vec![])
    }
}

async fn get_valueset_expansion<Resolver: CanonicalResolver + Sync + Send + Clone + 'static>(
    canonical_resolution: Resolver,
    value_set: &ValueSet,
) -> Result<Vec<ValueSetExpansionContains>, OperationOutcomeError> {
    let mut result = Vec::new();
    if let Some(compose) = value_set.compose.as_ref() {
        for include in &compose.include {
            result.extend(
                get_valueset_expansion_contains(canonical_resolution.clone(), include).await?,
            );
        }
    }
    Ok(result)
}

fn expand_valueset<Resolver: CanonicalResolver + Sync + Send + Clone + 'static>(
    canonical_resolution: Resolver,
    input: ValueSetExpand::Input,
) -> Pin<Box<dyn Future<Output = Result<ValueSetExpand::Output, OperationOutcomeError>> + Send>> {
    // Implementation would go here
    Box::pin(async move {
        let resolved = resolve_valueset(canonical_resolution.clone(), input).await?;

        if let Some(resource) = resolved
            && let Resource::ValueSet(value_set) = &*resource
        {
            let contains = get_valueset_expansion(canonical_resolution.clone(), value_set).await?;
            let mut expanded_valueset = value_set.clone();

            expanded_valueset.expansion = Some(ValueSetExpansion {
                contains: Some(contains),
                timestamp: Box::new(FHIRDateTime {
                    value: Some(DateTime::Iso8601(chrono::Utc::now())),
                    ..Default::default()
                }),
                ..Default::default()
            });

            Ok(ValueSetExpand::Output {
                return_: expanded_valueset,
            })
        } else {
            Err(OperationOutcomeError::error(
                IssueType::not_found(),
                "ValueSet could not be resolved".to_string(),
            ))
        }
    })
}

impl FHIRTerminology for FHIRCanonicalTerminology {
    async fn expand<Resolver: CanonicalResolver + Send + Clone + Sync + 'static>(
        &self,
        resolver: Resolver,
        input: ValueSetExpand::Input,
    ) -> Result<ValueSetExpand::Output, OperationOutcomeError> {
        expand_valueset(resolver, input).await
    }
    async fn validate<Resolver: CanonicalResolver + Send + Clone + Sync + 'static>(
        &self,
        resolver: Resolver,
        input: ValueSetValidateCode::Input,
    ) -> Result<ValueSetValidateCode::Output, OperationOutcomeError> {
        let Some(code) = input.code else {
            return Err(OperationOutcomeError::error(
                IssueType::invalid(),
                "No code provided for validation only support 'code' field validation".to_string(),
            ));
        };

        // Implementation would go here
        let expansion = self
            .expand(
                resolver,
                ValueSetExpand::Input {
                    url: input.url,
                    valueSet: input.valueSet,
                    valueSetVersion: input.valueSetVersion,
                    context: input.context,
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

        let valueset = expansion.return_;

        if let Some(expansion) = valueset.expansion
            && let Some(contains) = expansion.contains
        {
            for contain in contains {
                if contain.code.as_ref().is_some_and(|c| c.value == code.value) {
                    return Ok(ValueSetValidateCode::Output {
                        result: FHIRBoolean {
                            value: Some(true),
                            ..Default::default()
                        },
                        display: None,
                        message: Some(FHIRString {
                            value: Some("Code is valid in the ValueSet".to_string()),
                            ..Default::default()
                        }),
                    });
                }
            }
        }

        Ok(ValueSetValidateCode::Output {
            result: FHIRBoolean {
                value: Some(false),
                ..Default::default()
            },
            display: None,
            message: Some(FHIRString {
                value: Some("Code is valid in the ValueSet".to_string()),
                ..Default::default()
            }),
        })
    }
    async fn lookup<Resolver: CanonicalResolver + Send + Clone + Sync + 'static>(
        &self,
        _resolver: Resolver,
        _input: CodeSystemLookup::Input,
    ) -> Result<CodeSystemLookup::Output, OperationOutcomeError> {
        // Implementation would go here
        unimplemented!()
    }
}
