//! `Patient/{id}/$everything`: the patient and every resource in their
//! compartment, in one searchset Bundle.
//!
//! The resource types and the search parameter that ties each one to the
//! patient both come from the R4 Patient `CompartmentDefinition`, the same
//! artifact `/Patient/{id}/{Type}` already reads. Nothing here hard-codes a
//! list of clinical types, so a compartment change is picked up without a code
//! change.
//!
//! One search per (type, parameter) pair runs through the normal client, so
//! access policies, scopes and auditing apply exactly as they do to the
//! equivalent hand-written search.

use crate::fhir_client::{
    ServerCTX,
    middleware::{ServerMiddlewareState, operations::ServerOperationContext},
};
use haste_artifacts::ARTIFACT_RESOURCES;
use haste_fhir_client::{
    FHIRClient,
    request::InvocationRequest,
    url::{Parameter, ParsedParameter, ParsedParameters},
};
use haste_fhir_generated_ops::generated::PatientEverything;
use haste_fhir_model::r4::generated::{
    resources::{
        Bundle, BundleEntry, CompartmentDefinition, CompartmentDefinitionResource, Resource,
        ResourceType,
    },
    terminology::{BundleType, CompartmentType, IssueType},
    types::FHIRUnsignedInt,
};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_fhir_ops::OperationExecutor;
use haste_fhir_search::SearchEngine;
use haste_fhir_terminology::FHIRTerminology;
use haste_jwt::{ProjectId, TenantId};
use haste_repository::Repository;
use std::{
    collections::HashSet,
    sync::{Arc, LazyLock},
};

/// Entries returned when the caller gives no `_count`. A chart pull is a read
/// of a whole record, so the default is generous, but it is still a cap: a
/// patient with a decade of device telemetry must not turn one request into an
/// unbounded result set.
const DEFAULT_MAX_ENTRIES: usize = 1_000;

/// Hard ceiling on entries, whatever `_count` asks for. Past this the honest
/// answer is a bulk export, not a bigger Bundle.
const ABSOLUTE_MAX_ENTRIES: usize = 10_000;

/// Page size for each per-type search. Independent of the entry cap: it only
/// controls how many rows one round trip fetches.
const SEARCH_PAGE_SIZE: usize = 500;

/// The R4 Patient compartment, resolved once.
static PATIENT_COMPARTMENT: LazyLock<Option<&'static CompartmentDefinition>> =
    LazyLock::new(|| {
        ARTIFACT_RESOURCES.iter().find_map(|r| match r {
            Resource::CompartmentDefinition(c) if c.code == CompartmentType::patient() => Some(c),
            _ => None,
        })
    });

pub fn patient_everything<
    Repo: Repository + Send + Sync + 'static,
    Search: SearchEngine + Send + Sync + 'static,
    Terminology: FHIRTerminology + Send + Sync + 'static,
    Client: FHIRClient<Arc<ServerCTX<Client>>, OperationOutcomeError> + 'static,
>() -> OperationExecutor<
    ServerOperationContext<ServerMiddlewareState<Repo, Search, Terminology>, Client>,
    PatientEverything::Input,
    PatientEverything::Output,
> {
    OperationExecutor::new(
        PatientEverything::CODE.to_string(),
        Box::new(
            |context: ServerOperationContext<
                ServerMiddlewareState<Repo, Search, Terminology>,
                Client,
            >,
             _tenant: TenantId,
             _project: ProjectId,
             request: &InvocationRequest,
             input: PatientEverything::Input| {
                // Resolved before the future so it does not borrow `request`.
                let patient_id = instance_id(request);
                Box::pin(async move {
                    let patient_id = patient_id?;
                    let client = context.ctx.client.clone();
                    let bundle =
                        everything(client.as_ref(), context.ctx.clone(), patient_id, &input)
                            .await?;

                    Ok(PatientEverything::Output { return_: bundle })
                })
            },
        ),
    )
}

/// `$everything` is instance-level: without an id there is no compartment to
/// gather.
fn instance_id(request: &InvocationRequest) -> Result<String, OperationOutcomeError> {
    match request {
        InvocationRequest::Instance(instance) => Ok(instance.id.clone()),
        InvocationRequest::Type(_) | InvocationRequest::System(_) => {
            Err(OperationOutcomeError::error(
                IssueType::invalid(),
                "$everything must be invoked on a Patient instance, as Patient/{id}/$everything."
                    .to_string(),
            ))
        }
    }
}

async fn everything<Client: FHIRClient<Arc<ServerCTX<Client>>, OperationOutcomeError>>(
    fhir_client: &Client,
    ctx: Arc<ServerCTX<Client>>,
    patient_id: String,
    input: &PatientEverything::Input,
) -> Result<Bundle, OperationOutcomeError> {
    let compartment = PATIENT_COMPARTMENT.ok_or_else(|| {
        OperationOutcomeError::fatal(
            IssueType::not_supported(),
            "The Patient CompartmentDefinition is not loaded, so $everything cannot resolve \
             which resource types belong to a patient's compartment."
                .to_string(),
        )
    })?;

    let max_entries = max_entries(input)?;
    let requested_types = requested_types(input);

    // The Patient itself is in its own compartment but has no compartment
    // parameter pointing at it, so it is read directly.
    let mut entries: Vec<BundleEntry> = Vec::new();
    let mut seen: HashSet<(String, String)> = HashSet::new();

    if requested_types
        .as_ref()
        .is_none_or(|types| types.contains(ResourceType::Patient.as_ref()))
    {
        if let Some(patient) = fhir_client
            .read(ctx.clone(), ResourceType::Patient, patient_id.clone())
            .await?
        {
            push_entry(&mut entries, &mut seen, Box::new(patient));
        } else {
            return Err(OperationOutcomeError::error(
                IssueType::not_found(),
                format!("Patient/{patient_id} was not found."),
            ));
        }
    }

    let reference = format!("{}/{patient_id}", ResourceType::Patient.as_ref());

    // A truncated result is still a valid searchset; the Bundle records that it
    // is incomplete rather than silently looking whole.
    let mut truncated = false;

    for (resource_type, parameter_name) in compartment_searches(compartment, &requested_types) {
        if entries.len() >= max_entries {
            truncated = true;
            break;
        }

        let parameters = search_parameters(&parameter_name, &reference, input);

        let bundle = fhir_client
            .search_type(
                ctx.clone(),
                resource_type,
                ParsedParameters::new(parameters),
            )
            .await?;

        for entry in bundle.entry.unwrap_or_default() {
            if entries.len() >= max_entries {
                truncated = true;
                break;
            }
            if let Some(resource) = entry.resource {
                push_entry(&mut entries, &mut seen, resource);
            }
        }
    }

    let total = entries.len() as u64;

    Ok(Bundle {
        type_: BundleType::searchset(),
        // The count of what this Bundle carries. It is not a compartment-wide
        // total: obtaining that would mean counting every type a second time.
        total: (!truncated).then(|| {
            Box::new(FHIRUnsignedInt {
                value: Some(total),
                ..Default::default()
            })
        }),
        entry: Some(entries),
        ..Default::default()
    })
}

/// Adds a resource unless an equal `(type, id)` is already present. A resource
/// can be reachable through more than one compartment parameter — an
/// Observation matches both `subject` and `performer` — and FHIR requires each
/// to appear once.
fn push_entry(
    entries: &mut Vec<BundleEntry>,
    seen: &mut HashSet<(String, String)>,
    resource: Box<Resource>,
) {
    let Some(key) = resource_key(&resource) else {
        return;
    };
    if seen.insert(key) {
        entries.push(BundleEntry {
            resource: Some(resource),
            ..Default::default()
        });
    }
}

fn resource_key(resource: &Resource) -> Option<(String, String)> {
    let id = resource.id().clone()?;
    Some((resource.resource_type().as_ref().to_string(), id))
}

/// `_count` bounded by [`ABSOLUTE_MAX_ENTRIES`].
fn max_entries(input: &PatientEverything::Input) -> Result<usize, OperationOutcomeError> {
    let Some(count) = input.count.as_ref().and_then(|c| c.value) else {
        return Ok(DEFAULT_MAX_ENTRIES);
    };

    let count = usize::try_from(count).map_err(|_| {
        OperationOutcomeError::fatal(
            IssueType::invalid(),
            format!("Invalid _count value: '{count}'."),
        )
    })?;

    if count == 0 {
        return Err(OperationOutcomeError::fatal(
            IssueType::invalid(),
            "_count must be greater than zero.".to_string(),
        ));
    }

    Ok(count.min(ABSOLUTE_MAX_ENTRIES))
}

/// The `_type` filter, lowercased-insensitively by exact code match, or `None`
/// for every type in the compartment.
fn requested_types(input: &PatientEverything::Input) -> Option<HashSet<String>> {
    let types: HashSet<String> = input
        .type_
        .as_ref()?
        .iter()
        .filter_map(|code| code.value.clone())
        // `_type` is documented as comma-delimited within one value.
        .flat_map(|value| {
            value
                .split(',')
                .map(|t| t.trim().to_string())
                .filter(|t| !t.is_empty())
                .collect::<Vec<_>>()
        })
        .collect();

    (!types.is_empty()).then_some(types)
}

/// Every `(resource type, compartment parameter)` pair to search, skipping
/// Patient (read directly) and any type `_type` excludes.
///
/// A type with several compartment parameters yields one pair per parameter:
/// the searches are OR-ed by union, since a single search cannot express
/// `subject OR performer`.
fn compartment_searches(
    compartment: &CompartmentDefinition,
    requested_types: &Option<HashSet<String>>,
) -> Vec<(ResourceType, String)> {
    compartment
        .resource
        .as_deref()
        .unwrap_or_default()
        .iter()
        .filter_map(|resource: &CompartmentDefinitionResource| {
            let code = resource.code.as_str()?;
            // Read directly, and it has no parameter pointing at itself.
            if code == ResourceType::Patient.as_ref() {
                return None;
            }
            if requested_types
                .as_ref()
                .is_some_and(|types| !types.contains(code))
            {
                return None;
            }
            let resource_type: ResourceType = code.try_into().ok()?;

            // No param means the type is named in the compartment but has no
            // link to it, so nothing can be searched for.
            let params = resource.param.as_deref()?;
            Some(
                params
                    .iter()
                    .filter_map(|p| p.value.clone())
                    .map(move |param| (resource_type.clone(), param))
                    .collect::<Vec<_>>(),
            )
        })
        .flatten()
        .collect()
}

/// One per-type search: the compartment link, the page size, and whichever of
/// `_since`, `start` and `end` the caller supplied.
fn search_parameters(
    parameter_name: &str,
    reference: &str,
    input: &PatientEverything::Input,
) -> Vec<ParsedParameter> {
    let plain = |name: &str, value: String| {
        ParsedParameter::from(Parameter {
            name: name.to_string(),
            value: vec![value],
            modifier: None,
            chains: None,
        })
    };

    let mut parameters = vec![
        plain(parameter_name, reference.to_string()),
        plain("_count", SEARCH_PAGE_SIZE.to_string()),
    ];

    // Record currency, so it applies to every type uniformly.
    if let Some(since) = input.since.as_ref().and_then(|s| s.value.as_ref()) {
        parameters.push(plain("_lastUpdated", format!("ge{}", since.to_string())));
    }

    // Care dates. `date` is the clinical date on most compartment types; a type
    // without it is left unfiltered rather than dropped, because excluding a
    // patient's allergies from their chart because they carry no date would
    // lose data the caller asked for.
    if let Some(start) = input.start.as_ref().and_then(|s| s.value.as_ref()) {
        parameters.push(plain("date", format!("ge{}", start.to_string())));
    }
    if let Some(end) = input.end.as_ref().and_then(|e| e.value.as_ref()) {
        parameters.push(plain("date", format!("le{}", end.to_string())));
    }

    parameters
}
