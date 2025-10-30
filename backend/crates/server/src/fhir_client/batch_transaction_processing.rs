use crate::{
    fhir_client::{FHIRServerClient, ServerCTX},
    fhir_http::{self, HTTPRequest},
};
use axum::http::Method;
use oxidized_fhir_client::{
    FHIRClient,
    request::{FHIRRequest, FHIRResponse},
};
use oxidized_fhir_model::r4::generated::{
    resources::{Bundle, BundleEntry, BundleEntryResponse, Resource, ResourceType},
    terminology::{BundleType, IssueType},
    types::Reference,
};
use oxidized_fhir_operation_error::OperationOutcomeError;
use oxidized_fhir_search::SearchEngine;
use oxidized_fhir_terminology::FHIRTerminology;
use oxidized_reflect::MetaValue;
use oxidized_repository::{Repository, types::SupportedFHIRVersions};
use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::{algo::toposort, visit::EdgeRef};
use std::{pin::Pin, str::FromStr, sync::Arc};

fn convert_bundle_entry(fhir_response: Result<FHIRResponse, OperationOutcomeError>) -> BundleEntry {
    match fhir_response {
        Ok(FHIRResponse::Create(res)) => BundleEntry {
            resource: Some(Box::new(res.resource)),
            ..Default::default()
        },
        Ok(FHIRResponse::Read(res)) => BundleEntry {
            resource: Some(Box::new(res.resource)),
            ..Default::default()
        },
        Ok(FHIRResponse::Update(res)) => BundleEntry {
            resource: Some(Box::new(res.resource)),
            ..Default::default()
        },
        Ok(FHIRResponse::VersionRead(res)) => BundleEntry {
            resource: Some(Box::new(res.resource)),
            ..Default::default()
        },
        Ok(FHIRResponse::DeleteInstance(_res)) => BundleEntry {
            resource: None,
            ..Default::default()
        },
        Ok(FHIRResponse::DeleteType(_res)) => BundleEntry {
            resource: None,
            ..Default::default()
        },
        Ok(FHIRResponse::DeleteSystem(_res)) => BundleEntry {
            resource: None,
            ..Default::default()
        },
        Ok(FHIRResponse::HistoryInstance(res)) => {
            let bundle = oxidized_fhir_client::axum::to_bundle(
                BundleType::History(None),
                None,
                res.resources,
            );
            BundleEntry {
                resource: Some(Box::new(Resource::Bundle(bundle))),
                ..Default::default()
            }
        }
        Ok(FHIRResponse::HistoryType(res)) => {
            let bundle = oxidized_fhir_client::axum::to_bundle(
                BundleType::History(None),
                None,
                res.resources,
            );
            BundleEntry {
                resource: Some(Box::new(Resource::Bundle(bundle))),
                ..Default::default()
            }
        }
        Ok(FHIRResponse::HistorySystem(res)) => {
            let bundle = oxidized_fhir_client::axum::to_bundle(
                BundleType::History(None),
                None,
                res.resources,
            );
            BundleEntry {
                resource: Some(Box::new(Resource::Bundle(bundle))),
                ..Default::default()
            }
        }
        Ok(FHIRResponse::SearchSystem(res)) => {
            let bundle = oxidized_fhir_client::axum::to_bundle(
                BundleType::Searchset(None),
                res.total,
                res.resources,
            );
            BundleEntry {
                resource: Some(Box::new(Resource::Bundle(bundle))),
                ..Default::default()
            }
        }
        Ok(FHIRResponse::SearchType(res)) => {
            let bundle = oxidized_fhir_client::axum::to_bundle(
                BundleType::Searchset(None),
                res.total,
                res.resources,
            );
            BundleEntry {
                resource: Some(Box::new(Resource::Bundle(bundle))),
                ..Default::default()
            }
        }
        Ok(FHIRResponse::Patch(res)) => BundleEntry {
            resource: Some(Box::new(res.resource)),
            ..Default::default()
        },

        Ok(FHIRResponse::Capabilities(res)) => BundleEntry {
            resource: Some(Box::new(Resource::CapabilityStatement(res.capabilities))),
            ..Default::default()
        },

        Ok(FHIRResponse::InvokeInstance(res)) => BundleEntry {
            resource: Some(Box::new(res.resource)),
            ..Default::default()
        },
        Ok(FHIRResponse::InvokeType(res)) => BundleEntry {
            resource: Some(Box::new(res.resource)),
            ..Default::default()
        },
        Ok(FHIRResponse::InvokeSystem(res)) => BundleEntry {
            resource: Some(Box::new(res.resource)),
            ..Default::default()
        },
        Ok(FHIRResponse::Batch(res)) => BundleEntry {
            resource: Some(Box::new(Resource::Bundle(res.resource))),
            ..Default::default()
        },
        Ok(FHIRResponse::Transaction(res)) => BundleEntry {
            resource: Some(Box::new(Resource::Bundle(res.resource))),
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

fn bundle_entry_to_fhir_request(entry: BundleEntry) -> Result<FHIRRequest, OperationOutcomeError> {
    if let Some(request) = entry.request.as_ref() {
        let url = request
            .url
            .value
            .as_ref()
            .map(|s| s.as_str())
            .unwrap_or_default();

        let (path, query) = url.split_once("?").unwrap_or((url, ""));
        let request_method_string: Option<String> = request.method.as_ref().into();
        let Ok(method) = Method::from_str(&request_method_string.unwrap_or_default()) else {
            return Err(OperationOutcomeError::error(
                IssueType::Invalid(None),
                "Invalid HTTP Method".to_string(),
            ));
        };

        let http_request = HTTPRequest::new(
            method,
            path.to_string(),
            if let Some(body) = entry.resource {
                fhir_http::HTTPBody::Resource(*body)
            } else {
                fhir_http::HTTPBody::String("".to_string())
            },
            query.to_string(),
        );

        let Ok(fhir_request) =
            fhir_http::http_request_to_fhir_request(SupportedFHIRVersions::R4, http_request)
        else {
            return Err(OperationOutcomeError::error(
                IssueType::Invalid(None),
                "Invalid Bundle entry".to_string(),
            ));
        };

        Ok(fhir_request)
    } else {
        Err(OperationOutcomeError::error(
            IssueType::Invalid(None),
            "Bundle entry missing request".to_string(),
        ))
    }
}

pub async fn process_batch_bundle<
    Repo: Repository + Send + Sync,
    Search: SearchEngine + Send + Sync,
    Terminology: FHIRTerminology + Send + Sync,
>(
    fhir_client: &FHIRServerClient<Repo, Search, Terminology>,
    ctx: Arc<ServerCTX>,
    request_bundle_entries: Vec<BundleEntry>,
) -> Result<Bundle, OperationOutcomeError> {
    let mut bundle_response_entries = Vec::with_capacity(request_bundle_entries.len());
    for e in request_bundle_entries.into_iter() {
        let fhir_request = bundle_entry_to_fhir_request(e)?;

        let fhir_response = fhir_client.request(ctx.clone(), fhir_request).await;
        bundle_response_entries.push(convert_bundle_entry(fhir_response));
    }

    Ok(Bundle {
        type_: Box::new(BundleType::BatchResponse(None)),
        entry: Some(bundle_response_entries),
        ..Default::default()
    })
}

fn get_resource_from_response<'a>(response: &'a FHIRResponse) -> Option<&'a Resource> {
    match response {
        FHIRResponse::Create(res) => Some(&res.resource),
        FHIRResponse::Read(res) => Some(&res.resource),
        FHIRResponse::Update(res) => Some(&res.resource),
        FHIRResponse::VersionRead(res) => Some(&res.resource),
        FHIRResponse::Patch(res) => Some(&res.resource),
        _ => None,
    }
}

fn get_resource_type_from_fhir_request(request: &FHIRRequest) -> Option<ResourceType> {
    match request {
        FHIRRequest::Create(req) => Some(req.resource_type.clone()),
        FHIRRequest::Read(req) => Some(req.resource_type.clone()),
        FHIRRequest::UpdateInstance(req) => Some(req.resource_type.clone()),
        FHIRRequest::ConditionalUpdate(req) => Some(req.resource_type.clone()),
        FHIRRequest::DeleteInstance(req) => Some(req.resource_type.clone()),
        FHIRRequest::SearchType(req) => Some(req.resource_type.clone()),
        FHIRRequest::VersionRead(req) => Some(req.resource_type.clone()),
        FHIRRequest::Patch(req) => Some(req.resource_type.clone()),
        FHIRRequest::DeleteType(req) => Some(req.resource_type.clone()),

        FHIRRequest::HistoryInstance(req) => Some(req.resource_type.clone()),
        FHIRRequest::HistoryType(req) => Some(req.resource_type.clone()),
        FHIRRequest::InvokeInstance(req) => Some(req.resource_type.clone()),
        FHIRRequest::InvokeType(req) => Some(req.resource_type.clone()),

        FHIRRequest::SearchSystem(_)
        | FHIRRequest::DeleteSystem(_)
        | FHIRRequest::Capabilities
        | FHIRRequest::HistorySystem(_)
        | FHIRRequest::InvokeSystem(_)
        | FHIRRequest::Batch(_)
        | FHIRRequest::Transaction(_) => None,
    }
}

pub async fn process_transaction_bundle<
    Repo: Repository + Send + Sync + 'static,
    Search: SearchEngine + Send + Sync + 'static,
    Terminology: FHIRTerminology + Send + Sync + 'static,
>(
    fhir_client: &FHIRServerClient<Repo, Search, Terminology>,
    ctx: Arc<ServerCTX>,
    request_bundle_entries: Vec<BundleEntry>,
) -> Result<Bundle, OperationOutcomeError> {
    let fp_engine = oxidized_fhirpath::FPEngine::new();

    let mut graph = DiGraph::<Option<BundleEntry>, Pin<&mut Reference>>::new();
    // Used for index lookup when mutating.
    let mut indices_map = std::collections::HashMap::<String, NodeIndex>::new();

    // Instantiate the nodes. See [https://hl7.org/fhir/R4/bundle.html#references] for handling of refernces in bundle.
    // Currently we will resolve only internal references (i.e. those that reference other entries in the bundle via fullUrl).
    request_bundle_entries.into_iter().for_each(|entry| {
        let full_url = entry
            .fullUrl
            .as_ref()
            .and_then(|fu| fu.value.as_ref())
            .map(|s| s.to_string());
        let node_index = graph.add_node(Some(entry));
        if let Some(full_url) = full_url {
            indices_map.insert(full_url, node_index);
        }
    });

    // Avoid borrow issue process as tupple collection than add to graph.
    let edges = graph
        .node_indices()
        .flat_map(|cur_index| {
            let fp_result = fp_engine
                .evaluate(
                    "$this.descendants().ofType(Reference)",
                    vec![&graph[cur_index]],
                )
                .unwrap();

            fp_result
                .iter()
                .filter_map(|mv| mv.as_any().downcast_ref::<Reference>())
                .filter_map(|reference| {
                    if let Some(reference_string) =
                        reference.reference.as_ref().and_then(|r| r.value.as_ref())
                        && let Some(reference_index) = indices_map.get(reference_string.as_str())
                    {
                        // Convert because need to mutate it.
                        let r = reference as *const Reference;
                        let mut_ptr = r as *mut Reference;
                        let mutable_reference = unsafe { mut_ptr.as_mut().unwrap() };
                        Some((*reference_index, cur_index, Pin::new(mutable_reference)))
                    } else {
                        None
                    }
                })
                .collect::<Vec<(NodeIndex, NodeIndex, Pin<&mut Reference>)>>()
        })
        .collect::<Vec<_>>();

    for edge in edges {
        graph.add_edge(edge.0, edge.1, edge.2);
    }

    let topo_sort_ordering = toposort(&graph, None).map_err(|e| {
        OperationOutcomeError::fatal(
            IssueType::Exception(None),
            format!(
                "Cyclic dependency detected in transaction bundle at node {:?}",
                e.node_id()
            ),
        )
    })?;

    let mut response_entries = vec![];

    for index in topo_sort_ordering.iter() {
        let targets = graph.edges(*index).map(|e| e.id()).collect::<Vec<_>>();
        let edges = targets
            .into_iter()
            .filter_map(|i| graph.remove_edge(i))
            .collect::<Vec<_>>();

        let mut entry = None;
        std::mem::swap(&mut graph[*index], &mut entry);

        let entry = entry.ok_or_else(|| {
            OperationOutcomeError::fatal(
                IssueType::Exception(None),
                "Failed to get node from graph".to_string(),
            )
        })?;

        let fhir_request = bundle_entry_to_fhir_request(entry)?;
        let resource_type = get_resource_type_from_fhir_request(&fhir_request);

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
                    let z = Pin::into_inner(reference_pointer);
                    z.reference = Some(Box::new(
                        oxidized_fhir_model::r4::generated::types::FHIRString {
                            value: Some(ref_string.clone()),
                            ..Default::default()
                        },
                    ));
                }
            } else {
                return Err(OperationOutcomeError::fatal(
                    IssueType::Exception(None),
                    "Failed to update reference - response did not return valid resource with an id."
                        .to_string(),
                ));
            }
        }

        response_entries.push(convert_bundle_entry(Ok(fhir_response)));
    }

    Ok(Bundle {
        type_: Box::new(BundleType::TransactionResponse(None)),
        entry: Some(response_entries),
        ..Default::default()
    })
}

#[cfg(test)]
mod tests {}
