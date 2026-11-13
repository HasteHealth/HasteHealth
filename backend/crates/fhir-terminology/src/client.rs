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

fn get_concepts(codesystem: &CodeSystem) -> Result<&[CodeSystemConcept], OperationOutcomeError> {
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
            Ok(codesystem.concept.as_deref().unwrap_or_default())
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
    codesystem_concept: &[CodeSystemConcept],
) -> Vec<ValueSetExpansionContains> {
    // Borrows the concepts and copies only what the expansion keeps, so a
    // concept's definition and properties are never cloned.
    codesystem_concept
        .iter()
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
            code: Some(c.code.clone()),
            display: c.display.clone(),
            designation: c.designation.as_ref().map(|designations| {
                designations
                    .iter()
                    .map(|d| ValueSetComposeIncludeConceptDesignation {
                        id: d.id.clone(),
                        extension: d.extension.clone(),
                        modifierExtension: d.modifierExtension.clone(),
                        language: d.language.clone(),
                        use_: d.use_.clone(),
                        value: d.value.clone(),
                    })
                    .collect::<Vec<_>>()
            }),
            contains: c
                .concept
                .as_deref()
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
        Ok(code_system_concept_to_valueset_expansion(
            code_system
                .url
                .as_ref()
                .and_then(|url| url.value.as_deref()),
            code_system
                .version
                .as_ref()
                .and_then(|version| version.value.as_deref()),
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
                value: Some(format!(
                    "Code '{}' is not in the ValueSet",
                    code.value.as_deref().unwrap_or_default()
                )),
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

#[cfg(test)]
mod tests {
    use super::*;
    use haste_fhir_model::r4::generated::{
        resources::{
            CodeSystemConceptDesignation, CodeSystemConceptProperty,
            CodeSystemConceptPropertyValueTypeChoice, ValueSetCompose,
        },
        terminology::BoundCode,
        types::FHIRCode,
    };

    #[derive(Clone)]
    struct InMemoryResolver {
        code_system: Arc<Resource>,
        value_set: Arc<Resource>,
    }

    impl CanonicalResolver for InMemoryResolver {
        async fn resolve(
            &self,
            resource_type: ResourceType,
            _canonical_url: &str,
        ) -> Result<Option<Arc<Resource>>, OperationOutcomeError> {
            Ok(Some(if resource_type == ResourceType::CodeSystem {
                self.code_system.clone()
            } else {
                self.value_set.clone()
            }))
        }
    }

    fn code(value: &str) -> Box<FHIRCode> {
        Box::new(FHIRCode {
            value: Some(value.to_string()),
            ..Default::default()
        })
    }

    fn string(value: &str) -> Box<FHIRString> {
        Box::new(FHIRString {
            value: Some(value.to_string()),
            ..Default::default()
        })
    }

    fn uri(value: &str) -> Box<FHIRUri> {
        Box::new(FHIRUri {
            value: Some(value.to_string()),
            ..Default::default()
        })
    }

    fn concept(value: &str, display: &str) -> CodeSystemConcept {
        CodeSystemConcept {
            code: code(value),
            display: Some(string(display)),
            ..Default::default()
        }
    }

    fn resolver(
        content: BoundCode<CodesystemContentMode>,
        concepts: Vec<CodeSystemConcept>,
    ) -> InMemoryResolver {
        InMemoryResolver {
            code_system: Arc::new(Resource::CodeSystem(CodeSystem {
                url: Some(uri("http://example.org/cs")),
                version: Some(string("1.0.0")),
                content,
                concept: Some(concepts),
                ..Default::default()
            })),
            value_set: Arc::new(Resource::ValueSet(ValueSet {
                url: Some(uri("http://example.org/vs")),
                compose: Some(ValueSetCompose {
                    include: vec![ValueSetComposeInclude {
                        system: Some(uri("http://example.org/cs")),
                        ..Default::default()
                    }],
                    ..Default::default()
                }),
                ..Default::default()
            })),
        }
    }

    fn input() -> ValueSetExpand::Input {
        ValueSetExpand::Input {
            url: Some(FHIRUri {
                value: Some("http://example.org/vs".to_string()),
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
        }
    }

    async fn expand(resolver: InMemoryResolver) -> Vec<ValueSetExpansionContains> {
        FHIRCanonicalTerminology::new()
            .expand(resolver, input())
            .await
            .unwrap()
            .return_
            .expansion
            .unwrap()
            .contains
            .unwrap()
    }

    #[tokio::test]
    async fn expands_a_flat_code_system() {
        let mut first = concept("a", "Alpha");
        first.definition = Some(string("The definition of alpha."));
        first.property = Some(vec![CodeSystemConceptProperty {
            code: code("status"),
            value: CodeSystemConceptPropertyValueTypeChoice::Code(code("active")),
            ..Default::default()
        }]);
        first.designation = Some(vec![CodeSystemConceptDesignation {
            language: Some(code("de")),
            value: string("Alfa"),
            ..Default::default()
        }]);

        let contains = expand(resolver(
            CodesystemContentMode::complete(),
            vec![first, concept("b", "Beta")],
        ))
        .await;

        assert_eq!(contains.len(), 2);
        let alpha = &contains[0];
        assert_eq!(
            alpha.system.as_ref().and_then(|s| s.value.as_deref()),
            Some("http://example.org/cs")
        );
        assert_eq!(
            alpha.version.as_ref().and_then(|v| v.value.as_deref()),
            Some("1.0.0")
        );
        assert_eq!(
            alpha.code.as_ref().and_then(|c| c.value.as_deref()),
            Some("a")
        );
        assert_eq!(
            alpha.display.as_ref().and_then(|d| d.value.as_deref()),
            Some("Alpha")
        );
        let designations = alpha.designation.as_ref().unwrap();
        assert_eq!(designations.len(), 1);
        assert_eq!(
            designations[0]
                .language
                .as_ref()
                .and_then(|l| l.value.as_deref()),
            Some("de")
        );
        assert_eq!(designations[0].value.value.as_deref(), Some("Alfa"));
        assert!(alpha.contains.is_none());
        assert!(contains[1].designation.is_none());

        let serialized = serde_json::to_string(&contains).unwrap();
        assert!(!serialized.contains("The definition of alpha."));
        assert!(!serialized.contains("property"));
        assert!(!serialized.contains("status"));
    }

    #[tokio::test]
    async fn expands_a_hierarchical_code_system_into_nested_contains() {
        let mut child = concept("child", "Child");
        child.concept = Some(vec![concept("grandchild", "Grandchild")]);
        let mut parent = concept("parent", "Parent");
        parent.concept = Some(vec![child, concept("sibling", "Sibling")]);

        let contains = expand(resolver(CodesystemContentMode::complete(), vec![parent])).await;

        assert_eq!(contains.len(), 1);
        let children = contains[0].contains.as_ref().unwrap();
        let codes = children
            .iter()
            .map(|c| c.code.as_ref().and_then(|c| c.value.as_deref()))
            .collect::<Vec<_>>();
        assert_eq!(codes, vec![Some("child"), Some("sibling")]);
        let grandchildren = children[0].contains.as_ref().unwrap();
        assert_eq!(grandchildren.len(), 1);
        assert_eq!(
            grandchildren[0]
                .code
                .as_ref()
                .and_then(|c| c.value.as_deref()),
            Some("grandchild")
        );
        assert_eq!(
            grandchildren[0]
                .system
                .as_ref()
                .and_then(|s| s.value.as_deref()),
            Some("http://example.org/cs")
        );
        assert!(children[1].contains.is_none());
    }

    #[tokio::test]
    async fn refuses_a_code_system_whose_content_is_not_present() {
        let error = FHIRCanonicalTerminology::new()
            .expand(
                resolver(
                    CodesystemContentMode::not_present(),
                    vec![concept("a", "Alpha")],
                ),
                input(),
            )
            .await
            .unwrap_err();

        let issue = &error.outcome().issue[0];
        assert_eq!(issue.code, IssueType::not_supported());
        assert_eq!(
            issue.diagnostics.as_ref().and_then(|d| d.value.as_deref()),
            Some("CodeSystem content is 'not-present'")
        );
    }
}
