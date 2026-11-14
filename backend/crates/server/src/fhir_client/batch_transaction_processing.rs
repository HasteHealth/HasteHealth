use crate::{
    fhir_client::{
        ClientState, ServerCTX,
        middleware::{
            Backend,
            auth_z::{access_control::RequestPolicy, scope_check},
        },
        utilities::request_to_resource_type,
    },
    fhir_http::{self, HTTPRequest},
};
use axum::http::Method;
use futures_util::future::try_join_all;
use haste_fhir_client::{
    request::{
        FHIRRequest, FHIRResponse, FHIRSearchTypeRequest, HistoryResponse, InvokeResponse,
        SearchRequest, SearchResponse,
    },
    url::{ParsedParameter, ParsedParameters},
};
use haste_fhir_model::r4::generated::{
    resources::{Bundle, BundleEntry, BundleEntryResponse, Resource, ResourceType},
    terminology::{BundleType, IssueType},
    types::{FHIRString, FHIRUri, Reference},
};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_fhir_search::{SearchEngine, SearchOptions};
use haste_reflect::MetaValue;
use haste_repository::types::SupportedFHIRVersions;
use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::{algo::toposort, visit::EdgeRef};
use std::{collections::HashMap, pin::Pin, str::FromStr, sync::Arc};

fn convert_bundle_entry(fhir_response: Result<FHIRResponse, OperationOutcomeError>) -> BundleEntry {
    match fhir_response {
        Ok(FHIRResponse::Create(res)) => BundleEntry {
            resource: Some(Box::new(res.resource)),
            ..Default::default()
        },
        Ok(FHIRResponse::Read(res)) => {
            if let Some(resource) = res.resource {
                BundleEntry {
                    resource: Some(Box::new(resource)),
                    ..Default::default()
                }
            } else {
                BundleEntry {
                    response: Some(BundleEntryResponse {
                        outcome: Some(Box::new(Resource::OperationOutcome(
                            OperationOutcomeError::error(
                                IssueType::not_found(),
                                "Resource not found".to_string(),
                            )
                            .outcome()
                            .clone(),
                        ))),
                        ..Default::default()
                    }),
                    ..Default::default()
                }
            }
        }
        Ok(FHIRResponse::Update(res)) => BundleEntry {
            resource: Some(Box::new(res.resource)),
            ..Default::default()
        },
        Ok(FHIRResponse::VersionRead(res)) => BundleEntry {
            resource: Some(Box::new(res.resource)),
            ..Default::default()
        },
        Ok(FHIRResponse::Delete(_res)) => BundleEntry {
            resource: None,
            ..Default::default()
        },
        Ok(FHIRResponse::History(res)) => match res {
            HistoryResponse::Instance(res) => BundleEntry {
                resource: Some(Box::new(Resource::Bundle(res.bundle))),
                ..Default::default()
            },
            HistoryResponse::Type(res) => BundleEntry {
                resource: Some(Box::new(Resource::Bundle(res.bundle))),
                ..Default::default()
            },
            HistoryResponse::System(res) => BundleEntry {
                resource: Some(Box::new(Resource::Bundle(res.bundle))),
                ..Default::default()
            },
        },

        Ok(FHIRResponse::Search(res)) => match res {
            SearchResponse::Type(res) => BundleEntry {
                resource: Some(Box::new(Resource::Bundle(res.bundle))),
                ..Default::default()
            },
            SearchResponse::System(res) => BundleEntry {
                resource: Some(Box::new(Resource::Bundle(res.bundle))),
                ..Default::default()
            },
        },

        Ok(FHIRResponse::Patch(res)) => BundleEntry {
            resource: Some(Box::new(res.resource)),
            ..Default::default()
        },

        Ok(FHIRResponse::Capabilities(res)) => BundleEntry {
            resource: Some(Box::new(Resource::CapabilityStatement(res.capabilities))),
            ..Default::default()
        },

        Ok(FHIRResponse::Invoke(res)) => match res {
            InvokeResponse::Instance(res) => BundleEntry {
                resource: Some(Box::new(res.resource)),
                ..Default::default()
            },
            InvokeResponse::Type(res) => BundleEntry {
                resource: Some(Box::new(res.resource)),
                ..Default::default()
            },
            InvokeResponse::System(res) => BundleEntry {
                resource: Some(Box::new(res.resource)),
                ..Default::default()
            },
        },

        Ok(FHIRResponse::Batch(res)) => BundleEntry {
            resource: Some(Box::new(Resource::Bundle(res.resource))),
            ..Default::default()
        },
        Ok(FHIRResponse::Transaction(res)) => BundleEntry {
            resource: Some(Box::new(Resource::Bundle(res.resource))),
            ..Default::default()
        },
        // An entry queued as an asynchronous job: no resource yet, only the
        // status URL to poll, as the entry's `location`.
        Ok(FHIRResponse::Accepted(res)) => BundleEntry {
            response: Some(BundleEntryResponse {
                status: Box::new(FHIRString::from("202 Accepted".to_string())),
                location: Some(Box::new(FHIRUri::from(res.content_location))),
                ..Default::default()
            }),
            ..Default::default()
        },
        Err(operation_error) => {
            let operation_outcome = operation_error.outcome().clone();

            BundleEntry {
                response: Some(BundleEntryResponse {
                    outcome: Some(Box::new(Resource::OperationOutcome(operation_outcome))),
                    ..Default::default()
                }),
                ..Default::default()
            }
        }
    }
}

pub async fn bundle_entry_to_fhir_request<B: Backend>(
    entry: BundleEntry,
    ctx: &Arc<ServerCTX<B>>,
) -> Result<FHIRRequest, OperationOutcomeError> {
    if let Some(request) = entry.request.as_ref() {
        let url = request.url.value.as_deref().unwrap_or_default();

        let (path, query) = url.split_once("?").unwrap_or((url, ""));
        let request_method_string = request.method.as_str();
        let Ok(method) = Method::from_str(request_method_string.unwrap_or_default()) else {
            return Err(OperationOutcomeError::error(
                IssueType::invalid(),
                "Invalid HTTP Method".to_string(),
            ));
        };

        let http_request = HTTPRequest::new(
            method,
            path.to_string(),
            if let Some(body) = entry.resource {
                fhir_http::HTTPBody::Resource(*body)
            } else {
                fhir_http::HTTPBody::Bytes(axum::body::Bytes::new())
            },
            query,
        );

        fhir_http::http_request_to_fhir_request(SupportedFHIRVersions::R4, http_request, ctx).await
    } else {
        Err(OperationOutcomeError::error(
            IssueType::invalid(),
            "Bundle entry missing request".to_string(),
        ))
    }
}

pub async fn process_batch_bundle<Client: Backend>(
    fhir_client: &Client,
    ctx: Arc<ServerCTX<Client>>,
    request_bundle_entries: Vec<BundleEntry>,
) -> Result<Bundle, OperationOutcomeError> {
    let mut bundle_response_entries = Vec::with_capacity(request_bundle_entries.len());
    for e in request_bundle_entries.into_iter() {
        let fhir_request = bundle_entry_to_fhir_request(e, &ctx).await?;

        let fhir_response = fhir_client.request(ctx.clone(), fhir_request).await;
        bundle_response_entries.push(convert_bundle_entry(fhir_response));
    }

    Ok(Bundle {
        type_: BundleType::batch_response(),
        entry: Some(bundle_response_entries),
        ..Default::default()
    })
}

fn get_resource_from_response(response: &FHIRResponse) -> Option<&Resource> {
    match response {
        FHIRResponse::Create(res) => Some(&res.resource),
        FHIRResponse::Read(res) => res.resource.as_ref(),
        FHIRResponse::Update(res) => Some(&res.resource),
        FHIRResponse::VersionRead(res) => Some(&res.resource),
        FHIRResponse::Patch(res) => Some(&res.resource),
        _ => None,
    }
}

struct SortedTransactionEntry {
    entry: BundleEntry,
    idx: usize,
}

fn collect_references<'a>(value: &'a dyn MetaValue, out: &mut Vec<&'a Reference>) {
    for field in value.fields() {
        let Some(child_field) = value.get_field(field) else {
            continue;
        };

        for child in child_field.flatten() {
            if let Some(reference) = child.as_any().downcast_ref::<Reference>() {
                out.push(reference);
            }

            collect_references(child, out);
        }
    }
}

/// The resource type a conditional reference (`Practitioner?identifier=...`)
/// searches, or `None` when `reference` is not one. Per
/// [https://hl7.org/fhir/R4/http.html#trules] it is relative to the server's
/// base and starts with a resource type.
fn conditional_reference_type(reference: &str) -> Option<ResourceType> {
    let (resource_type, _) = reference.split_once('?')?;
    ResourceType::try_from(resource_type).ok()
}

/// The search a conditional reference describes.
///
/// # Errors
///
/// Invalid when `reference` is not a conditional reference, or its query does
/// not parse or holds anything but filtering parameters (none of `_count`,
/// `_sort` and the like apply), or none at all.
fn conditional_reference_search(
    reference: &str,
) -> Result<FHIRSearchTypeRequest, OperationOutcomeError> {
    let invalid = |reason: &str| {
        OperationOutcomeError::error(
            IssueType::invalid(),
            format!("Conditional reference '{reference}' {reason}."),
        )
    };
    let (Some(resource_type), Some((_, query))) = (
        conditional_reference_type(reference),
        reference.split_once('?'),
    ) else {
        return Err(invalid("is not a resource type followed by a search"));
    };
    let pairs: Vec<(String, String)> = url::form_urlencoded::parse(query.as_bytes())
        .into_owned()
        .collect();
    let parameters = ParsedParameters::try_from(pairs.as_slice())
        .map_err(|_| invalid("has a query that does not parse"))?;
    if parameters.parameters().is_empty() {
        return Err(invalid("has no search parameters"));
    }
    if parameters
        .parameters()
        .iter()
        .any(|parameter| matches!(parameter, ParsedParameter::Result(_)))
    {
        return Err(invalid("may only use filtering parameters"));
    }

    Ok(FHIRSearchTypeRequest {
        resource_type,
        parameters,
    })
}

/// A mutable handle on a reference inside a bundle entry the graph owns.
///
/// # Safety
///
/// The entry must outlive the handle and stay put: each reference sits in a
/// resource boxed on the heap, so moving the graph's nodes does not move it.
/// No two handles may point at the same reference.
unsafe fn mutable_reference<'a>(reference: &Reference) -> Pin<&'a mut Reference> {
    let mut_ptr = reference as *const Reference as *mut Reference;
    Pin::new(unsafe { mut_ptr.as_mut().unwrap() })
}

pub struct SortedTransaction<'a> {
    graph: DiGraph<Option<SortedTransactionEntry>, Option<Pin<&'a mut Reference>>>,
    topo_sort_ordering: Vec<NodeIndex>,
}

/// Every conditional reference in a bundle, by its search URI, so each
/// distinct one is looked up once however many entries use it.
type ConditionalReferences<'a> = HashMap<String, Vec<Pin<&'a mut Reference>>>;

/// Points each of `references` at `literal` (`Type/id`), keeping any
/// extensions on the element.
fn point_at(references: Vec<Pin<&mut Reference>>, literal: &str) {
    for reference in references {
        if let Some(value) = Pin::into_inner(reference).reference.as_mut() {
            value.value = Some(literal.to_string());
        }
    }
}

/// A search the engine could not build, naming a parameter the type does not
/// define ([`IssueType::not_found`]) or a modifier it does not support
/// ([`IssueType::not_supported`]), makes the conditional reference invalid:
/// the transaction is refused as a bad request, not as not found or not
/// implemented. Any other failure is the server's and passes through.
fn invalid_search(search_uri: &str, error: OperationOutcomeError) -> OperationOutcomeError {
    let issues = &error.outcome().issue;
    if !issues.iter().any(|issue| {
        issue.code == IssueType::not_found() || issue.code == IssueType::not_supported()
    }) {
        return error;
    }

    let reason = issues
        .iter()
        .filter_map(|issue| issue.diagnostics.as_ref()?.value.clone())
        .collect::<Vec<_>>()
        .join(" ");
    OperationOutcomeError::error(
        IssueType::invalid(),
        format!("Conditional reference '{search_uri}' is not a valid search: {reason}"),
    )
    .with_source(error)
}

/// Points every conditional reference at the one resource its search matches.
///
/// Answered by the search index alone: the id it already holds is all a
/// reference needs, so no resource is read. The lookups run at once, each
/// authorized as the caller's own search of the type would be: their access
/// policies must grant it, and it carries the filters their scopes set.
///
/// # Errors
///
/// Forbidden when the caller may not run a reference's search, invalid when
/// it is not a search the type supports. Per
/// [https://hl7.org/fhir/R4/http.html#trules] the transaction fails when a
/// reference matches no resource or more than one.
async fn resolve_conditional_references<B: Backend>(
    state: &ClientState<B>,
    ctx: &ServerCTX<B>,
    conditional_references: ConditionalReferences<'_>,
) -> Result<(), OperationOutcomeError> {
    if conditional_references.is_empty() {
        return Ok(());
    }

    let policy = &RequestPolicy::of_caller(state, ctx).await?;
    let lookups = conditional_references
        .into_iter()
        .map(|(search_uri, references)| async move {
            let search = conditional_reference_search(&search_uri)?;
            let filters = scope_check::search_filters(&ctx.user.claims, &search.resource_type)?;
            let search = SearchRequest::Type(search);
            if let Some(policy) = policy {
                policy.check(&FHIRRequest::Search(search.clone())).await?;
            }

            let found = state
                .search
                .search(
                    &ctx.fhir_version,
                    &ctx.tenant,
                    &ctx.project,
                    &search,
                    Some(SearchOptions {
                        // Two tell one match from several.
                        count_limit: Some(2),
                        filters,
                    }),
                )
                .await
                .map_err(|error| invalid_search(&search_uri, error))?;

            let mut entries = found.entries.into_iter();
            match (entries.next(), entries.next()) {
                (Some(entry), None) => {
                    let literal = format!("{}/{}", entry.resource_type.as_ref(), entry.id.as_ref());
                    point_at(references, &literal);
                    Ok(())
                }
                (None, _) => Err(OperationOutcomeError::error(
                    IssueType::invalid(),
                    format!("Conditional reference '{search_uri}' matched no resources."),
                )),
                (Some(_), Some(_)) => Err(OperationOutcomeError::error(
                    IssueType::conflict(),
                    format!("Conditional reference '{search_uri}' matched multiple resources."),
                )),
            }
        });

    try_join_all(lookups).await?;
    Ok(())
}

/// Prepares a transaction bundle before its transaction opens, to keep that
/// short: orders the entries so each comes after those it references by
/// `fullUrl`, and points its conditional references (`Type?search`) at the
/// resources they match.
pub async fn build_sorted_transaction_graph<'a, B: Backend>(
    state: &ClientState<B>,
    ctx: &ServerCTX<B>,
    request_bundle_entries: Vec<BundleEntry>,
) -> Result<SortedTransaction<'a>, OperationOutcomeError> {
    let (sorted_transaction, conditional_references) = sort_entries(request_bundle_entries)?;
    resolve_conditional_references(state, ctx, conditional_references).await?;

    Ok(sorted_transaction)
}

/// The entries in dependency order, and the conditional references among them.
fn sort_entries<'a>(
    request_bundle_entries: Vec<BundleEntry>,
) -> Result<(SortedTransaction<'a>, ConditionalReferences<'a>), OperationOutcomeError> {
    let mut graph =
        DiGraph::<Option<SortedTransactionEntry>, Option<Pin<&'a mut Reference>>>::new();
    // Used for index lookup when mutating.
    let mut indices_map = std::collections::HashMap::<String, NodeIndex>::new();

    // Instantiate the nodes. See [https://hl7.org/fhir/R4/bundle.html#references] for handling of refernces in bundle.
    // References to other entries (by fullUrl) become edges. Conditional references
    // (`Type?search`) are collected to be resolved against the server.
    request_bundle_entries
        .into_iter()
        .enumerate()
        .for_each(|(idx, entry)| {
            let full_url = entry
                .fullUrl
                .as_ref()
                .and_then(|fu| fu.value.as_ref())
                .map(|s| s.to_string());
            let node_index = graph.add_node(Some(SortedTransactionEntry { entry, idx }));
            if let Some(full_url) = full_url {
                indices_map.insert(full_url, node_index);
            }
        });

    let mut edges = vec![];
    let mut conditional_references = ConditionalReferences::new();
    for cur_index in graph.node_indices() {
        if let Some(sorted_transaction_entry) = &graph[cur_index] {
            let mut references = Vec::new();
            collect_references(
                &sorted_transaction_entry.entry as &dyn MetaValue,
                &mut references,
            );

            for reference in references {
                let Some(reference_string) =
                    reference.reference.as_ref().and_then(|r| r.value.as_ref())
                else {
                    continue;
                };
                // Converted because they need to be mutated. Each reference is
                // either an edge or conditional, never both, and the entries
                // live in the graph for as long as the transaction does.
                if let Some(reference_index) = indices_map.get(reference_string.as_str()) {
                    edges.push((
                        *reference_index,
                        cur_index,
                        Some(unsafe { mutable_reference(reference) }),
                    ));
                } else if conditional_reference_type(reference_string).is_some() {
                    conditional_references
                        .entry(reference_string.clone())
                        .or_default()
                        .push(unsafe { mutable_reference(reference) });
                }
            }
        }
    }

    for edge in edges {
        graph.add_edge(edge.0, edge.1, edge.2);
    }

    let topo_sort_ordering = toposort(&graph, None).map_err(|e| {
        OperationOutcomeError::fatal(
            IssueType::exception(),
            format!(
                "Cyclic dependency detected in transaction bundle at node {:?}",
                e.node_id()
            ),
        )
    })?;

    Ok((
        SortedTransaction {
            graph,
            topo_sort_ordering,
        },
        conditional_references,
    ))
}

/// Process a transaction bundle, ensuring that references between entries are resolved correctly.
/// Sorts transactions using topological sort to ensure that dependencies are processed first.
pub async fn process_transaction_bundle<'a, Client: Backend>(
    fhir_client: &Client,
    ctx: Arc<ServerCTX<Client>>,
    mut sorted_transaction: SortedTransaction<'a>,
) -> Result<Bundle, OperationOutcomeError> {
    // Performance improvement over .push as we know exact size of vector based on transaction request bundle.
    // Use spare_capacity_mut to get mutable access to uninitialized memory.
    let mut response_entries = Vec::with_capacity(sorted_transaction.topo_sort_ordering.len());
    let mutable_access = response_entries.spare_capacity_mut();

    for index in sorted_transaction.topo_sort_ordering.iter() {
        let targets = sorted_transaction
            .graph
            .edges(*index)
            .map(|e| e.id())
            .collect::<Vec<_>>();
        let edges = targets
            .into_iter()
            // Do memswap as actual removal alters edge locations of graph.
            // So next set of edges would be invalid (although could possibly go in reverse order).
            .filter_map(|i| {
                if let Some(edge_weight) = sorted_transaction.graph.edge_weight_mut(i) {
                    let mut placeholder = None;
                    std::mem::swap(&mut placeholder, edge_weight);

                    placeholder
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        let mut entry = None;
        std::mem::swap(&mut sorted_transaction.graph[*index], &mut entry);

        let sorted_transaction_entry = entry.ok_or_else(|| {
            OperationOutcomeError::fatal(
                IssueType::exception(),
                "Failed to get node from graph".to_string(),
            )
        })?;

        let fhir_request =
            bundle_entry_to_fhir_request(sorted_transaction_entry.entry, &ctx).await?;
        let resource_type = request_to_resource_type(&fhir_request).cloned();

        let fhir_response = fhir_client.request(ctx.clone(), fhir_request).await?;
        let resource = get_resource_from_response(&fhir_response);

        if !edges.is_empty() {
            if let Some(resource_type) = resource_type
                && let Some(resource) = resource
                && let Some(id) = resource
                    .get_field("id")
                    .and_then(|mv| mv.as_any().downcast_ref::<String>())
            {
                let ref_string = format!("{}/{}", resource_type.as_ref(), id);
                for reference_pointer in edges.into_iter() {
                    let reference_pointing_entry = Pin::into_inner(reference_pointer);
                    reference_pointing_entry.reference = Some(Box::new(
                        haste_fhir_model::r4::generated::types::FHIRString {
                            value: Some(ref_string.clone()),
                            ..Default::default()
                        },
                    ));
                }
            } else {
                return Err(OperationOutcomeError::fatal(
                    IssueType::exception(),
                    "Failed to update reference - response did not return valid resource with an id."
                        .to_string(),
                ));
            }
        }

        mutable_access[sorted_transaction_entry.idx].write(convert_bundle_entry(Ok(fhir_response)));
    }

    // Should be safe as we set length to exact number of entries we wrote to.
    unsafe {
        response_entries.set_len(sorted_transaction.topo_sort_ordering.len());
    };

    Ok(Bundle {
        type_: BundleType::transaction_response(),
        entry: Some(response_entries),
        ..Default::default()
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use haste_fhir_model::r4::generated::{
        resources::{Patient, PatientContact},
        types::FHIRString,
    };

    #[test]
    fn collect_references_finds_direct_and_nested_references() {
        let entry = BundleEntry {
            resource: Some(Box::new(Resource::Patient(Patient {
                generalPractitioner: Some(vec![Reference {
                    reference: Some(Box::new(FHIRString {
                        value: Some("Practitioner/123".to_string()),
                        ..Default::default()
                    })),
                    ..Default::default()
                }]),
                contact: Some(vec![PatientContact {
                    organization: Some(Box::new(Reference {
                        reference: Some(Box::new(FHIRString {
                            value: Some("Organization/456".to_string()),
                            ..Default::default()
                        })),
                        ..Default::default()
                    })),
                    ..Default::default()
                }]),
                ..Default::default()
            }))),
            ..Default::default()
        };

        let mut references = Vec::new();
        collect_references(&entry as &dyn MetaValue, &mut references);

        let mut found = references
            .iter()
            .filter_map(|r| r.reference.as_ref().and_then(|s| s.value.clone()))
            .collect::<Vec<_>>();
        found.sort();

        assert_eq!(found, vec!["Organization/456", "Practitioner/123"]);
    }

    fn reference(value: &str) -> Reference {
        Reference {
            reference: Some(Box::new(FHIRString {
                value: Some(value.to_string()),
                ..Default::default()
            })),
            ..Default::default()
        }
    }

    fn patient_entry(full_url: Option<&str>, practitioners: &[&str]) -> BundleEntry {
        BundleEntry {
            fullUrl: full_url.map(|url| Box::new(FHIRUri::from(url.to_string()))),
            resource: Some(Box::new(Resource::Patient(Patient {
                generalPractitioner: Some(practitioners.iter().map(|p| reference(p)).collect()),
                ..Default::default()
            }))),
            ..Default::default()
        }
    }

    fn general_practitioners(entry: &BundleEntry) -> Vec<String> {
        match entry.resource.as_deref() {
            Some(Resource::Patient(patient)) => patient
                .generalPractitioner
                .iter()
                .flatten()
                .filter_map(|r| r.reference.as_ref().and_then(|s| s.value.clone()))
                .collect(),
            _ => vec![],
        }
    }

    #[test]
    fn conditional_reference_type_needs_a_resource_type_before_the_query() {
        assert_eq!(
            conditional_reference_type("Practitioner?identifier=http://hl7.org/fhir/sid/us-npi|1"),
            Some(ResourceType::Practitioner)
        );
        assert_eq!(conditional_reference_type("Practitioner/123"), None);
        assert_eq!(conditional_reference_type("Practitioner/123?x=y"), None);
        assert_eq!(conditional_reference_type("NotAType?identifier=1"), None);
        assert_eq!(
            conditional_reference_type("https://example.com/fhir/Practitioner?identifier=1"),
            None
        );
    }

    #[test]
    fn conditional_reference_search_parses_the_filter() {
        let search = conditional_reference_search(
            "Organization?identifier=https://github.com/synthetichealth/synthea|be96eec8",
        )
        .unwrap();

        assert_eq!(search.resource_type, ResourceType::Organization);
        let parameters = search.parameters.parameters();
        assert_eq!(parameters.len(), 1);
        let ParsedParameter::Resource(parameter) = &parameters[0] else {
            panic!("expected a filtering parameter");
        };
        assert_eq!(parameter.name, "identifier");
        assert_eq!(
            parameter.value,
            vec!["https://github.com/synthetichealth/synthea|be96eec8".to_string()]
        );
    }

    #[test]
    fn conditional_reference_search_rejects_result_parameters_and_empty_queries() {
        assert!(conditional_reference_search("Practitioner?identifier=1&_count=1").is_err());
        assert!(conditional_reference_search("Practitioner?").is_err());
        assert!(conditional_reference_search("Practitioner/123").is_err());
    }

    #[test]
    fn conditional_references_are_collected_once_and_pointed_at_the_match() {
        let npi = "Practitioner?identifier=http://hl7.org/fhir/sid/us-npi|9999999499";
        let entries = vec![
            patient_entry(None, &[npi, "Practitioner/existing"]),
            patient_entry(None, &[npi]),
        ];

        let (sorted, mut conditional_references) = sort_entries(entries).unwrap();
        assert_eq!(
            conditional_references.keys().collect::<Vec<_>>(),
            vec![&npi.to_string()]
        );
        assert_eq!(conditional_references[npi].len(), 2);

        point_at(
            conditional_references.remove(npi).unwrap(),
            "Practitioner/abc",
        );

        let mut found = sorted
            .graph
            .node_weights()
            .flatten()
            .map(|node| general_practitioners(&node.entry))
            .collect::<Vec<_>>();
        found.sort();
        assert_eq!(
            found,
            vec![
                vec!["Practitioner/abc".to_string()],
                vec![
                    "Practitioner/abc".to_string(),
                    "Practitioner/existing".to_string()
                ],
            ]
        );
    }

    #[test]
    fn a_search_the_engine_cannot_build_makes_the_reference_invalid() {
        let unknown_parameter = OperationOutcomeError::error(
            IssueType::not_found(),
            "Search parameter with name 'not-a-parameter' not found.".to_string(),
        );
        let invalid = invalid_search("Practitioner?not-a-parameter=1", unknown_parameter);
        assert_eq!(invalid.http_status(), 400);
        assert!(
            invalid.to_string().contains(
                "Conditional reference 'Practitioner?not-a-parameter=1' is not a valid search: \
                 Search parameter with name 'not-a-parameter' not found."
            ),
            "{invalid}"
        );

        let unsupported_modifier = OperationOutcomeError::error(
            IssueType::not_supported(),
            "Unsupported modifier parameter: 'above'".to_string(),
        );
        assert_eq!(
            invalid_search("Practitioner?identifier:above=1", unsupported_modifier).http_status(),
            400
        );

        let server_failure =
            OperationOutcomeError::fatal(IssueType::exception(), "Database error".to_string());
        assert_eq!(
            invalid_search("Practitioner?identifier=1", server_failure).http_status(),
            500
        );
    }

    #[test]
    fn full_url_references_are_edges_not_conditional() {
        let entries = vec![
            patient_entry(Some("urn:uuid:1"), &[]),
            patient_entry(None, &["urn:uuid:1"]),
        ];

        let (sorted, conditional_references) = sort_entries(entries).unwrap();

        assert!(conditional_references.is_empty());
        assert_eq!(sorted.graph.edge_count(), 1);
    }

    #[test]
    fn collect_references_returns_empty_when_no_references_present() {
        let entry = BundleEntry {
            resource: Some(Box::new(Resource::Patient(Patient::default()))),
            ..Default::default()
        };

        let mut references = Vec::new();
        collect_references(&entry as &dyn MetaValue, &mut references);

        assert!(references.is_empty());
    }
}
