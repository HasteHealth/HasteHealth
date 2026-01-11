use haste_fhir_client::request::{DeleteRequest, FHIRRequest, InvocationRequest, UpdateRequest};
use haste_reflect::MetaValue;

// Use Internal Hashmap for storing created Values.
#[derive(Debug)]
struct RequestReflection(FHIRRequest, String, String);

impl From<FHIRRequest> for RequestReflection {
    fn from(request: FHIRRequest) -> Self {
        let request_type = request_type_string(&request).to_string();
        let request_level = request_to_level(&request).to_string();
        RequestReflection(request, request_type, request_level)
    }
}

impl From<RequestReflection> for FHIRRequest {
    fn from(reflection: RequestReflection) -> Self {
        reflection.0
    }
}

static SHARED_FIELDS: &[&str] = &["type", "level"];

// Type codes pulled from https://hl7.org/fhir/R4/http.html
///
/// Instance Level Interactions
// read	Read the current state of the resource
// vread	Read the state of a specific version of the resource
// update	Update an existing resource by its id (or create it if it is new)
// patch	Update an existing resource by posting a set of changes to it
// delete	Delete a resource
// history	Retrieve the change history for a particular resource
// Type Level Interactions
// create	Create a new resource with a server assigned id
// search	Search the resource type based on some filter criteria
// history	Retrieve the change history for a particular resource type
// Whole System Interactions
// capabilities	Get a capability statement for the system
// batch/transaction	Update, create or delete a set of resources in a single interaction
// history	Retrieve the change history for all resources
// search	Search across all resource types based on some filter criteria

fn request_type_string(request: &FHIRRequest) -> &'static str {
    match request {
        FHIRRequest::Create(_) => "create",
        FHIRRequest::Read(_) => "read",
        FHIRRequest::VersionRead(_) => "vread",
        FHIRRequest::Update(_) => "update",
        FHIRRequest::Patch(_) => "patch",
        FHIRRequest::Delete(_) => "delete",
        FHIRRequest::Capabilities => "capabilities",
        FHIRRequest::Search(_) => "search",
        FHIRRequest::History(_) => "history",
        // Not on the main http going to set as "invoke".
        FHIRRequest::Invocation(_) => "invoke",
        FHIRRequest::Batch(_) => "batch",
        FHIRRequest::Transaction(_) => "transaction",
    }
}

// fn request_resource_type() -> &'static str {}

fn request_to_level(request: &FHIRRequest) -> &'static str {
    match request {
        FHIRRequest::Read(_)
        | FHIRRequest::VersionRead(_)
        | FHIRRequest::Update(_)
        | FHIRRequest::Patch(_)
        | FHIRRequest::Delete(_) => "instance",
        FHIRRequest::History(hl) => match hl {
            haste_fhir_client::request::HistoryRequest::Instance(_) => "instance",
            haste_fhir_client::request::HistoryRequest::Type(_) => "type",
            haste_fhir_client::request::HistoryRequest::System(_) => "system",
        },
        FHIRRequest::Create(_) | FHIRRequest::Search(_) => "type",
        FHIRRequest::Capabilities
        | FHIRRequest::Batch(_)
        | FHIRRequest::Transaction(_)
        | FHIRRequest::Invocation(_) => "system",
    }
}

impl MetaValue for RequestReflection {
    fn fields(&self) -> Vec<&'static str> {
        SHARED_FIELDS.to_vec()
    }

    fn get_field<'a>(&'a self, field: &str) -> Option<&'a dyn MetaValue> {
        match field {
            "type" => Some(&self.1),
            "level" => Some(&self.2),
            "resource" => match &self.0 {
                FHIRRequest::Create(fhircreate_request) => Some(&fhircreate_request.resource),
                FHIRRequest::Batch(fhirbatch_request) => Some(&fhirbatch_request.resource),
                FHIRRequest::Transaction(fhirtransaction_request) => {
                    Some(&fhirtransaction_request.resource)
                }
                FHIRRequest::Read(fhirread_request) => None,
                FHIRRequest::VersionRead(fhirversion_read_request) => None,
                FHIRRequest::Update(update_request) => match update_request {
                    UpdateRequest::Conditional(conditional_update) => {
                        Some(&conditional_update.resource)
                    }
                    UpdateRequest::Instance(instance_update) => Some(&instance_update.resource),
                },
                FHIRRequest::Patch(fhirpatch_request) => None,
                FHIRRequest::Delete(delete_request) => None,
                FHIRRequest::Capabilities => None,
                FHIRRequest::Search(search_request) => None,
                FHIRRequest::History(history_request) => None,

                FHIRRequest::Invocation(invocation_request) => match invocation_request {
                    InvocationRequest::Instance(invocation_request) => {
                        Some(&invocation_request.parameters)
                    }
                    InvocationRequest::Type(invocation_request) => {
                        Some(&invocation_request.parameters)
                    }
                    InvocationRequest::System(invocation_request) => {
                        Some(&invocation_request.parameters)
                    }
                },
            },
            "id" => match &self.0 {
                FHIRRequest::Read(fhirread_request) => Some(&fhirread_request.id),
                FHIRRequest::VersionRead(fhirversion_read_request) => {
                    Some(&fhirversion_read_request.id)
                }
                FHIRRequest::Update(update_request) => match update_request {
                    UpdateRequest::Instance(instance) => Some(&instance.id),
                    _ => None,
                },
                FHIRRequest::Patch(fhirpatch_request) => Some(&fhirpatch_request.id),
                FHIRRequest::Delete(delete_request) => match delete_request {
                    DeleteRequest::Instance(instance) => Some(&instance.id),
                    _ => None,
                },
                _ => None,
            },
            "resource_type" => match &self.0 {
                FHIRRequest::Create(fhircreate_request) => Some(
                    &(<&'static str>::from(fhircreate_request.resource_type)) as &dyn MetaValue,
                ),
                // FHIRRequest::Read(fhirread_request) => {
                //     Some(&fhirread_request.resource_type.as_ref())
                // }
                // FHIRRequest::VersionRead(fhirversion_read_request) => {
                //     Some(&fhirversion_read_request.resource_type.as_ref())
                // }
                // FHIRRequest::Update(update_request) => Some(&update_request.resource_type),
                // FHIRRequest::Patch(fhirpatch_request) => Some(&fhirpatch_request.resource_type),
                // FHIRRequest::Delete(delete_request) => Some(&delete_request.resource_type),
                // FHIRRequest::Search(search_request) => Some(&search_request.resource_type),
                _ => None,
            },
            _ => None,
        }
    }

    fn get_field_mut<'a>(&'a mut self, _field: &str) -> Option<&'a mut dyn MetaValue> {
        None
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

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn typename(&self) -> &'static str {
        "FHIRRequest"
    }
}
