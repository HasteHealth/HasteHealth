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

/// Longest chain of `compose.include.valueSet` references an expansion follows.
const MAX_VALUE_SET_DEPTH: usize = 32;
/// Most value sets an expansion resolves through `compose.include.valueSet`.
const MAX_INCLUDED_VALUE_SETS: usize = 1_000;
/// Most concepts an expansion holds, nested `contains` counted.
const MAX_EXPANSION_CONCEPTS: usize = 100_000;

/// Past a limit of the expansion: well formed but more than the server
/// expands, so 422 like other limit refusals.
fn too_costly(diagnostic: String) -> OperationOutcomeError {
    let error = OperationOutcomeError::error(IssueType::too_costly(), diagnostic);
    OperationOutcomeError::unprocessable(error.outcome().clone())
}

/// What one expansion has used of its limits. Value sets include each other
/// by canonical, so nothing else bounds how deep or how wide an expansion gets.
#[derive(Default)]
struct ExpansionState {
    /// Canonical of the value set the expansion was asked for.
    root: Option<String>,
    /// Canonicals of the included value sets being expanded, outermost first.
    path: Vec<String>,
    included_value_sets: usize,
    concepts: usize,
}

impl ExpansionState {
    /// Records that the value set at `canonical` is about to be expanded
    /// inside the ones already on the path.
    fn enter(&mut self, canonical: &str) -> Result<(), OperationOutcomeError> {
        if self.root.as_deref() == Some(canonical)
            || self.path.iter().any(|entered| entered == canonical)
        {
            let error = OperationOutcomeError::error(
                IssueType::invalid(),
                format!("ValueSet '{canonical}' includes itself through compose.include.valueSet"),
            );
            return Err(OperationOutcomeError::unprocessable(
                error.outcome().clone(),
            ));
        }
        if self.path.len() >= MAX_VALUE_SET_DEPTH {
            return Err(too_costly(format!(
                "ValueSet includes other value sets more than {MAX_VALUE_SET_DEPTH} levels deep"
            )));
        }
        self.included_value_sets += 1;
        if self.included_value_sets > MAX_INCLUDED_VALUE_SETS {
            return Err(too_costly(format!(
                "ValueSet includes more than {MAX_INCLUDED_VALUE_SETS} other value sets"
            )));
        }
        self.path.push(canonical.to_string());
        Ok(())
    }

    fn leave(&mut self) {
        self.path.pop();
    }

    /// Counts concepts about to join the expansion, before they are copied.
    fn add_concepts(&mut self, count: usize) -> Result<(), OperationOutcomeError> {
        self.concepts = self.concepts.saturating_add(count);
        if self.concepts > MAX_EXPANSION_CONCEPTS {
            return Err(too_costly(format!(
                "ValueSet expands to more than {MAX_EXPANSION_CONCEPTS} concepts"
            )));
        }
        Ok(())
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

fn count_concepts(concepts: &[CodeSystemConcept]) -> usize {
    concepts
        .iter()
        .map(|c| 1 + c.concept.as_deref().map_or(0, count_concepts))
        .sum()
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
    state: &mut ExpansionState,
) -> Result<Vec<ValueSetExpansionContains>, OperationOutcomeError> {
    if are_codes_inline(include) {
        state.add_concepts(include.concept.as_ref().map_or(0, Vec::len))?;
        Ok(codes_inline_to_expansion(include))
    } else if let Some(valueset_uris) = include.valueSet.as_ref() {
        let mut contains = vec![];
        for valueset_uri in valueset_uris {
            if let Some(valueset_uri) = valueset_uri.value.as_ref() {
                contains.extend(
                    expand_included_valueset(canonical_resolution.clone(), valueset_uri, state)
                        .await?,
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
        let concepts = get_concepts(code_system)?;
        state.add_concepts(count_concepts(concepts))?;
        Ok(code_system_concept_to_valueset_expansion(
            code_system
                .url
                .as_ref()
                .and_then(|url| url.value.as_deref()),
            code_system
                .version
                .as_ref()
                .and_then(|version| version.value.as_deref()),
            concepts,
        ))
    } else {
        Ok(vec![])
    }
}

async fn get_valueset_expansion<Resolver: CanonicalResolver + Sync + Send + Clone + 'static>(
    canonical_resolution: Resolver,
    value_set: &ValueSet,
    state: &mut ExpansionState,
) -> Result<Vec<ValueSetExpansionContains>, OperationOutcomeError> {
    let mut result = Vec::new();
    if let Some(compose) = value_set.compose.as_ref() {
        for include in &compose.include {
            result.extend(
                get_valueset_expansion_contains(canonical_resolution.clone(), include, state)
                    .await?,
            );
        }
    }
    Ok(result)
}

fn valueset_not_resolved() -> OperationOutcomeError {
    OperationOutcomeError::error(
        IssueType::not_found(),
        "ValueSet could not be resolved".to_string(),
    )
}

/// Expands a value set another one includes by canonical. This is where an
/// expansion recurses, so the future is boxed and `state` is checked first.
fn expand_included_valueset<'a, Resolver: CanonicalResolver + Sync + Send + Clone + 'static>(
    canonical_resolution: Resolver,
    canonical: &'a str,
    state: &'a mut ExpansionState,
) -> Pin<
    Box<
        dyn Future<Output = Result<Vec<ValueSetExpansionContains>, OperationOutcomeError>>
            + Send
            + 'a,
    >,
> {
    Box::pin(async move {
        state.enter(canonical)?;

        let resolved = canonical_resolution
            .resolve(ResourceType::ValueSet, canonical)
            .await?;
        let Some(Resource::ValueSet(value_set)) = resolved.as_deref() else {
            return Err(valueset_not_resolved());
        };
        let contains = get_valueset_expansion(canonical_resolution, value_set, state).await?;

        state.leave();
        Ok(contains)
    })
}

async fn expand_valueset<Resolver: CanonicalResolver + Sync + Send + Clone + 'static>(
    canonical_resolution: Resolver,
    input: ValueSetExpand::Input,
) -> Result<ValueSetExpand::Output, OperationOutcomeError> {
    let resolved = resolve_valueset(canonical_resolution.clone(), input).await?;

    if let Some(resource) = resolved
        && let Resource::ValueSet(value_set) = &*resource
    {
        let mut state = ExpansionState {
            root: value_set.url.as_ref().and_then(|url| url.value.clone()),
            ..Default::default()
        };

        let contains = get_valueset_expansion(canonical_resolution, value_set, &mut state).await?;
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
        Err(valueset_not_resolved())
    }
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
            ValueSetComposeIncludeConcept,
        },
        terminology::BoundCode,
        types::{FHIRCanonical, FHIRCode},
    };
    use std::collections::HashMap;

    #[derive(Clone)]
    struct InMemoryResolver {
        code_system: Arc<Resource>,
        value_set: Arc<Resource>,
    }

    impl CanonicalResolver for InMemoryResolver {
        fn resolve(
            &self,
            resource_type: ResourceType,
            _canonical_url: &str,
        ) -> impl Future<Output = Result<Option<Arc<Resource>>, OperationOutcomeError>> + Send
        {
            std::future::ready(Ok(Some(if resource_type == ResourceType::CodeSystem {
                self.code_system.clone()
            } else {
                self.value_set.clone()
            })))
        }
    }

    fn code(value: &str) -> FHIRCode {
        FHIRCode {
            value: Some(value.to_string()),
            ..Default::default()
        }
    }

    fn string(value: &str) -> FHIRString {
        FHIRString {
            value: Some(value.to_string()),
            ..Default::default()
        }
    }

    fn uri(value: &str) -> FHIRUri {
        FHIRUri {
            value: Some(value.to_string()),
            ..Default::default()
        }
    }

    fn concept(value: &str, display: &str) -> CodeSystemConcept {
        CodeSystemConcept {
            code: Box::new(code(value)),
            display: Some(Box::new(string(display))),
            ..Default::default()
        }
    }

    fn resolver(
        content: BoundCode<CodesystemContentMode>,
        concepts: Vec<CodeSystemConcept>,
    ) -> InMemoryResolver {
        InMemoryResolver {
            code_system: Arc::new(Resource::CodeSystem(CodeSystem {
                url: Some(Box::new(uri("http://example.org/cs"))),
                version: Some(Box::new(string("1.0.0"))),
                content,
                concept: Some(concepts),
                ..Default::default()
            })),
            value_set: Arc::new(Resource::ValueSet(ValueSet {
                url: Some(Box::new(uri("http://example.org/vs"))),
                compose: Some(ValueSetCompose {
                    include: vec![ValueSetComposeInclude {
                        system: Some(Box::new(uri("http://example.org/cs"))),
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
            url: Some(uri("http://example.org/vs")),
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
        first.definition = Some(Box::new(string("The definition of alpha.")));
        first.property = Some(vec![CodeSystemConceptProperty {
            code: Box::new(code("status")),
            value: CodeSystemConceptPropertyValueTypeChoice::Code(Box::new(code("active"))),
            ..Default::default()
        }]);
        first.designation = Some(vec![CodeSystemConceptDesignation {
            language: Some(Box::new(code("de"))),
            value: Box::new(string("Alfa")),
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

    /// Resolves each canonical to the resource stored under it.
    #[derive(Clone, Default)]
    struct Canonicals(Arc<HashMap<String, Arc<Resource>>>);

    impl Canonicals {
        fn new(resources: Vec<Resource>) -> Self {
            Canonicals(Arc::new(
                resources
                    .into_iter()
                    .map(|resource| {
                        let url = match &resource {
                            Resource::ValueSet(value_set) => value_set.url.clone(),
                            Resource::CodeSystem(code_system) => code_system.url.clone(),
                            _ => None,
                        };
                        (url.unwrap().value.unwrap(), Arc::new(resource))
                    })
                    .collect(),
            ))
        }
    }

    impl CanonicalResolver for Canonicals {
        fn resolve(
            &self,
            _resource_type: ResourceType,
            canonical_url: &str,
        ) -> impl Future<Output = Result<Option<Arc<Resource>>, OperationOutcomeError>> + Send
        {
            std::future::ready(Ok(self.0.get(canonical_url).cloned()))
        }
    }

    fn include_value_sets(urls: &[&str]) -> ValueSetComposeInclude {
        ValueSetComposeInclude {
            valueSet: Some(
                urls.iter()
                    .map(|url| FHIRCanonical {
                        value: Some((*url).to_string()),
                        ..Default::default()
                    })
                    .collect(),
            ),
            ..Default::default()
        }
    }

    fn include_codes(codes: &[&str]) -> ValueSetComposeInclude {
        ValueSetComposeInclude {
            system: Some(Box::new(uri("http://example.org/cs"))),
            concept: Some(
                codes
                    .iter()
                    .map(|value| ValueSetComposeIncludeConcept {
                        code: Box::new(code(value)),
                        ..Default::default()
                    })
                    .collect(),
            ),
            ..Default::default()
        }
    }

    fn value_set(url: &str, include: Vec<ValueSetComposeInclude>) -> Resource {
        Resource::ValueSet(ValueSet {
            url: Some(Box::new(uri(url))),
            compose: Some(ValueSetCompose {
                include,
                ..Default::default()
            }),
            ..Default::default()
        })
    }

    fn input_for(url: &str) -> ValueSetExpand::Input {
        ValueSetExpand::Input {
            url: Some(uri(url)),
            ..input()
        }
    }

    async fn expand_error(resolver: Canonicals, url: &str) -> OperationOutcomeError {
        FHIRCanonicalTerminology::new()
            .expand(resolver, input_for(url))
            .await
            .unwrap_err()
    }

    fn diagnostics(error: &OperationOutcomeError) -> &str {
        error.outcome().issue[0]
            .diagnostics
            .as_ref()
            .and_then(|d| d.value.as_deref())
            .unwrap()
    }

    #[tokio::test]
    async fn refuses_a_value_set_that_includes_itself() {
        let resolver = Canonicals::new(vec![value_set(
            "http://example.org/vs/loop",
            vec![include_value_sets(&["http://example.org/vs/loop"])],
        )]);

        let error = expand_error(resolver, "http://example.org/vs/loop").await;

        assert_eq!(error.outcome().issue[0].code, IssueType::invalid());
        assert_eq!(
            diagnostics(&error),
            "ValueSet 'http://example.org/vs/loop' includes itself through compose.include.valueSet"
        );
    }

    #[tokio::test]
    async fn refuses_value_sets_that_include_each_other() {
        let resolver = Canonicals::new(vec![
            value_set(
                "http://example.org/vs/a",
                vec![include_value_sets(&["http://example.org/vs/b"])],
            ),
            value_set(
                "http://example.org/vs/b",
                vec![include_value_sets(&["http://example.org/vs/a"])],
            ),
        ]);

        let error = expand_error(resolver, "http://example.org/vs/a").await;

        assert_eq!(error.outcome().issue[0].code, IssueType::invalid());
        assert_eq!(
            diagnostics(&error),
            "ValueSet 'http://example.org/vs/a' includes itself through compose.include.valueSet"
        );
    }

    #[tokio::test]
    async fn refuses_to_validate_against_a_value_set_that_includes_itself() {
        let resolver = Canonicals::new(vec![value_set(
            "http://example.org/vs/loop",
            vec![include_value_sets(&["http://example.org/vs/loop"])],
        )]);

        let error = FHIRCanonicalTerminology::new()
            .validate(
                resolver,
                ValueSetValidateCode::Input {
                    url: Some(uri("http://example.org/vs/loop")),
                    context: None,
                    valueSet: None,
                    valueSetVersion: None,
                    code: Some(code("a")),
                    system: None,
                    systemVersion: None,
                    display: None,
                    coding: None,
                    codeableConcept: None,
                    date: None,
                    abstract_: None,
                    displayLanguage: None,
                },
            )
            .await
            .unwrap_err();

        assert_eq!(error.outcome().issue[0].code, IssueType::invalid());
    }

    #[tokio::test]
    async fn expands_a_value_set_reached_through_two_includes() {
        let resolver = Canonicals::new(vec![
            value_set(
                "http://example.org/vs/a",
                vec![include_value_sets(&[
                    "http://example.org/vs/b",
                    "http://example.org/vs/c",
                ])],
            ),
            value_set(
                "http://example.org/vs/b",
                vec![include_value_sets(&["http://example.org/vs/d"])],
            ),
            value_set(
                "http://example.org/vs/c",
                vec![include_value_sets(&["http://example.org/vs/d"])],
            ),
            value_set("http://example.org/vs/d", vec![include_codes(&["x", "y"])]),
        ]);

        let contains = FHIRCanonicalTerminology::new()
            .expand(resolver, input_for("http://example.org/vs/a"))
            .await
            .unwrap()
            .return_
            .expansion
            .unwrap()
            .contains
            .unwrap();

        let codes = contains
            .iter()
            .map(|c| c.code.as_ref().and_then(|c| c.value.as_deref()))
            .collect::<Vec<_>>();
        assert_eq!(codes, vec![Some("x"), Some("y"), Some("x"), Some("y")]);
    }

    /// `http://example.org/vs/0` including `/1` including `/2` and so on, the
    /// last of `included` value sets holding one code.
    fn chain(included: usize) -> Canonicals {
        Canonicals::new(
            (0..=included)
                .map(|level| {
                    let include = if level == included {
                        include_codes(&["a"])
                    } else {
                        include_value_sets(&[&format!("http://example.org/vs/{}", level + 1)])
                    };
                    value_set(&format!("http://example.org/vs/{level}"), vec![include])
                })
                .collect(),
        )
    }

    #[tokio::test]
    async fn follows_included_value_sets_up_to_the_depth_limit() {
        let output = FHIRCanonicalTerminology::new()
            .expand(
                chain(MAX_VALUE_SET_DEPTH),
                input_for("http://example.org/vs/0"),
            )
            .await
            .unwrap();
        assert_eq!(output.return_.expansion.unwrap().contains.unwrap().len(), 1);

        let error = expand_error(chain(MAX_VALUE_SET_DEPTH + 1), "http://example.org/vs/0").await;

        assert_eq!(error.outcome().issue[0].code, IssueType::too_costly());
        assert_eq!(
            diagnostics(&error),
            format!(
                "ValueSet includes other value sets more than {MAX_VALUE_SET_DEPTH} levels deep"
            )
        );
    }

    #[tokio::test]
    async fn refuses_more_included_value_sets_than_the_limit() {
        let includes = vec!["http://example.org/vs/empty"; MAX_INCLUDED_VALUE_SETS + 1];
        let resolver = Canonicals::new(vec![
            value_set(
                "http://example.org/vs/wide",
                vec![include_value_sets(&includes)],
            ),
            value_set("http://example.org/vs/empty", vec![]),
        ]);

        let error = expand_error(resolver, "http://example.org/vs/wide").await;

        assert_eq!(error.outcome().issue[0].code, IssueType::too_costly());
        assert_eq!(
            diagnostics(&error),
            format!("ValueSet includes more than {MAX_INCLUDED_VALUE_SETS} other value sets")
        );
    }

    #[tokio::test]
    async fn refuses_an_expansion_with_more_concepts_than_the_limit() {
        // Each include of `codes` adds 1,000 concepts, half of them nested.
        let concepts = (0..500)
            .map(|i| {
                let mut parent = concept(&format!("p{i}"), "Parent");
                parent.concept = Some(vec![concept(&format!("c{i}"), "Child")]);
                parent
            })
            .collect();
        let includes = vec!["http://example.org/vs/codes"; MAX_EXPANSION_CONCEPTS / 1_000 + 1];
        let resolver = Canonicals::new(vec![
            Resource::CodeSystem(CodeSystem {
                url: Some(Box::new(uri("http://example.org/cs"))),
                content: CodesystemContentMode::complete(),
                concept: Some(concepts),
                ..Default::default()
            }),
            value_set(
                "http://example.org/vs/codes",
                vec![ValueSetComposeInclude {
                    system: Some(Box::new(uri("http://example.org/cs"))),
                    ..Default::default()
                }],
            ),
            value_set(
                "http://example.org/vs/wide",
                vec![include_value_sets(&includes)],
            ),
        ]);

        let error = expand_error(resolver, "http://example.org/vs/wide").await;

        assert_eq!(error.outcome().issue[0].code, IssueType::too_costly());
        assert_eq!(
            diagnostics(&error),
            format!("ValueSet expands to more than {MAX_EXPANSION_CONCEPTS} concepts")
        );
    }
}
