use haste_fhir_client::{
    FHIRClient,
    request::{
        DeleteRequest, FHIRCreateRequest, FHIRDeleteInstanceRequest, FHIRDeleteSystemRequest,
        FHIRDeleteTypeRequest, FHIRHistoryInstanceRequest, FHIRHistorySystemRequest,
        FHIRHistoryTypeRequest, FHIRInvokeInstanceRequest, FHIRInvokeSystemRequest,
        FHIRInvokeTypeRequest, FHIRReadRequest, FHIRRequest, FHIRResponse, FHIRTransactionRequest,
        FHIRUpdateInstanceRequest, FHIRVersionReadRequest, HistoryRequest, HistoryResponse,
        InvocationRequest, InvokeResponse, Operation, SearchResponse, UpdateRequest,
    },
    url::ParsedParameters,
};
use haste_fhir_model::r4::generated::{
    resources::{
        Resource, ResourceType, TestReport, TestReportSetup, TestReportSetupAction,
        TestReportSetupActionAssert, TestReportSetupActionOperation, TestReportTeardown,
        TestReportTeardownAction, TestReportTest, TestReportTestAction, TestScript,
        TestScriptFixture, TestScriptSetup, TestScriptSetupAction, TestScriptSetupActionAssert,
        TestScriptSetupActionOperation, TestScriptTeardown, TestScriptTeardownAction,
        TestScriptTest, TestScriptTestAction, TestScriptVariable,
    },
    terminology::{
        AssertDirectionCodes, AssertOperatorCodes, BoundCode, BundleType, IssueType,
        ReportActionResultCodes, ReportResultCodes, ReportStatusCodes, TestscriptOperationCodes,
    },
    types::{FHIRId, FHIRMarkdown, FHIRString, Reference},
};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_pointer::{Key, TypedPointer};
use haste_reflect::MetaValue;
use regex::Regex;
use std::{
    any::Any,
    collections::HashMap,
    sync::{Arc, LazyLock},
    time::Duration,
};
use tokio::sync::Mutex;

use crate::conversion::ConvertedValue;

mod conversion;

#[derive(Debug)]
pub enum TestScriptError {
    ExecutionError(String),
    ValidationError(String),
    FixtureNotFound,
    InvalidFixture,
    OperationError(OperationOutcomeError),
}

#[derive(Debug, Clone)]
enum Response {
    FHIRResponse(Box<FHIRResponse>),
    OperationError(Arc<OperationOutcomeError>),
}

#[derive(Debug)]
enum Fixtures {
    Resource(Resource),
    Request(FHIRRequest),
    Response(Response),
}

// Internal structure to hold current test result and testing fixtures.
struct TestState {
    fp_engine: haste_fhirpath::FPEngine,
    fixtures: HashMap<String, Fixtures>,
    latest_request: Option<FHIRRequest>,
    latest_response: Option<Response>,
    result: BoundCode<ReportResultCodes>,
}

impl TestState {
    fn new() -> Self {
        TestState {
            fp_engine: haste_fhirpath::FPEngine::new(),
            fixtures: HashMap::new(),
            latest_request: None,
            latest_response: None,
            result: ReportResultCodes::pending(),
        }
    }
    fn resolve_fixture<'a>(
        &'a self,
        fixture_id: &str,
    ) -> Result<&'a dyn MetaValue, TestScriptError> {
        let fixture = self
            .fixtures
            .get(fixture_id)
            .ok_or(TestScriptError::FixtureNotFound)?;

        match fixture {
            Fixtures::Resource(res) => Ok(res),
            Fixtures::Request(req) => {
                request_to_meta_value(req).ok_or_else(|| TestScriptError::InvalidFixture)
            }
            Fixtures::Response(response) => {
                response_to_meta_value(response).ok_or_else(|| TestScriptError::InvalidFixture)
            }
        }
    }
}

struct TestResult<T> {
    pub state: Arc<Mutex<TestState>>,
    pub value: T,
}

fn response_to_meta_value(response: &Response) -> Option<&dyn MetaValue> {
    match response {
        Response::FHIRResponse(fhir_response) => match &**fhir_response {
            FHIRResponse::Create(res) => Some(&res.resource),
            FHIRResponse::Read(res) => Some(&res.resource),
            FHIRResponse::VersionRead(res) => Some(&res.resource),
            FHIRResponse::Update(res) => Some(&res.resource),
            FHIRResponse::Patch(res) => Some(&res.resource),
            FHIRResponse::Batch(res) => Some(&res.resource),
            FHIRResponse::Transaction(res) => Some(&res.resource),

            FHIRResponse::Capabilities(res) => Some(&res.capabilities),
            FHIRResponse::Search(res) => match res {
                SearchResponse::Type(res) => Some(&res.bundle),
                SearchResponse::System(res) => Some(&res.bundle),
            },
            FHIRResponse::History(res) => match res {
                HistoryResponse::Instance(res) => Some(&res.bundle),
                HistoryResponse::Type(res) => Some(&res.bundle),
                HistoryResponse::System(res) => Some(&res.bundle),
            },
            FHIRResponse::Invoke(res) => match res {
                InvokeResponse::Instance(res) => Some(&res.resource),
                InvokeResponse::Type(res) => Some(&res.resource),
                InvokeResponse::System(res) => Some(&res.resource),
            },

            FHIRResponse::Delete(_) => None,
        },
        Response::OperationError(op_error) => {
            let outcome = op_error.outcome();
            Some(outcome)
        }
    }
}

fn request_to_meta_value(request: &FHIRRequest) -> Option<&dyn MetaValue> {
    match request {
        FHIRRequest::Create(req) => Some(&req.resource),

        FHIRRequest::Update(update_request) => match update_request {
            UpdateRequest::Conditional(req) => Some(&req.resource),
            UpdateRequest::Instance(req) => Some(&req.resource),
        },

        FHIRRequest::Batch(req) => Some(&req.resource),
        FHIRRequest::Transaction(req) => Some(&req.resource),
        FHIRRequest::Invocation(req) => match req {
            haste_fhir_client::request::InvocationRequest::Instance(req) => Some(&req.parameters),
            haste_fhir_client::request::InvocationRequest::Type(req) => Some(&req.parameters),
            haste_fhir_client::request::InvocationRequest::System(req) => Some(&req.parameters),
        },
        FHIRRequest::Read(_)
        | FHIRRequest::VersionRead(_)
        | FHIRRequest::Compartment(_)
        | FHIRRequest::Patch(_)
        | FHIRRequest::Delete(_)
        | FHIRRequest::Capabilities
        | FHIRRequest::Search(_)
        | FHIRRequest::History(_) => None,
    }
}

fn associate_request_response_variables(
    state: &mut TestState,
    operation: &TestScriptSetupActionOperation,
    request: FHIRRequest,
    response: Response,
) {
    if let Some(request_var) = operation
        .requestId
        .as_ref()
        .and_then(|id| id.value.as_ref())
    {
        // Associate request variable in state
        state
            .fixtures
            .insert(request_var.clone(), Fixtures::Request(request.clone()));
    }

    if let Some(response_var) = operation
        .responseId
        .as_ref()
        .and_then(|id| id.value.as_ref())
    {
        // Associate response variable in state
        state
            .fixtures
            .insert(response_var.clone(), Fixtures::Response(response.clone()));
    }

    state.latest_request = Some(request);
    state.latest_response = Some(response);
}

/// Derive the resource type from operation or from the metavalue if not present on operation.
fn derive_resource_type(
    operation: &TestScriptSetupActionOperation,
    target: Option<&dyn MetaValue>,
    path: &str,
) -> Result<ResourceType, TestScriptError> {
    if let Some(operation_resource_type) = operation.resource.as_ref() {
        let string_type = operation_resource_type.as_str();
        ResourceType::try_from(string_type.unwrap_or_default()).map_err(|_| {
            TestScriptError::ExecutionError(format!(
                "Unsupported resource type '{operation_resource_type:?}' for operation at '{path}'."
            ))
        })
    } else if let Some(target) = target {
        ResourceType::try_from(target.fhir_type()).map_err(|_| {
            TestScriptError::ExecutionError(format!(
                "Unsupported resource type '{}' for operation at '{path}'.",
                target.fhir_type()
            ))
        })
    } else {
        Err(TestScriptError::ExecutionError(format!(
            "Failed to derive resource type for operation at '{path}'.",
        )))
    }
}

static EXPRESSION_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\$\{([^}]*)\}").unwrap());

async fn get_variable(
    state: &TestState,
    variables: &[TestScriptVariable],
    variable_id: &str,
) -> Result<ConvertedValue, TestScriptError> {
    let Some(variable) = variables
        .iter()
        .find(|v| v.name.value.as_deref() == Some(variable_id))
    else {
        return Err(TestScriptError::ExecutionError(format!(
            "Variable with id '{variable_id}' not found."
        )));
    };

    if let Some(expression) = variable
        .expression
        .as_ref()
        .and_then(|exp| exp.value.as_ref())
    {
        let values =
            if let Some(source_id) = variable.sourceId.as_ref().and_then(|id| id.value.as_ref()) {
                let source = state.resolve_fixture(source_id)?;
                vec![source]
            } else {
                vec![]
            };

        let eval_result = state
            .fp_engine
            .evaluate(expression, values)
            .await
            .map_err(|e| {
                TestScriptError::ExecutionError(format!(
                    "Failed to evaluate FHIRPath expression for variable '{variable_id}': {e}"
                ))
            })?;

        let converted_values = eval_result
            .iter()
            .map(conversion::convert_meta_value)
            .collect::<Vec<_>>();

        if converted_values.len() == 1 {
            Ok(converted_values.into_iter().next().unwrap())
        } else {
            Err(TestScriptError::ExecutionError(format!(
                "Variable '{variable_id}' evaluation returned multiple values; only single value supported.",
            )))
        }
    } else {
        Err(TestScriptError::ExecutionError(format!(
            "Only support variable with expression for variable id '{variable_id}'.",
        )))
    }
}

async fn evaluate_variable(
    state: &TestState,
    pointer: TypedPointer<TestScript, TestScript>,
    value: &str,
) -> Result<String, TestScriptError> {
    let mut result = value.to_string();
    let variable_pointer =
        pointer.descend::<Vec<TestScriptVariable>>(&Key::Field("variable".to_string()));
    let default_variables = vec![];

    let variables = if let Some(pointer) = variable_pointer.as_ref() {
        pointer.value().unwrap_or(&default_variables)
    } else {
        &default_variables
    };

    for reg_match in EXPRESSION_REGEX.captures_iter(value) {
        let full_match = reg_match.get(0).map_or("", |m| m.as_str());
        let Some(variable_id) = reg_match.get(1).map(|m| m.as_str()) else {
            return Err(TestScriptError::ExecutionError(format!(
                "Invalid variable expression in '{value}'."
            )));
        };

        let variable = get_variable(state, variables, variable_id).await?;
        result = result.replace(full_match, variable.to_string().as_str());
    }

    Ok(result)
}

async fn testscript_operation_to_fhir_request(
    state: &TestState,
    pointer: &TypedPointer<TestScript, TestScriptSetupActionOperation>,
) -> Result<FHIRRequest, TestScriptError> {
    let operation = get_operation(pointer)?;
    let op = get_operation_type(operation);

    match op {
        Some(op) if Some(op) == TestscriptOperationCodes::read().as_str() => {
            read_request(state, pointer, operation)
        }

        Some(op) if Some(op) == TestscriptOperationCodes::vread().as_str() => {
            version_read_request(state, pointer, operation)
        }

        Some(op) if Some(op) == TestscriptOperationCodes::search().as_str() => {
            search_request(state, pointer, operation).await
        }

        Some(op) if Some(op) == TestscriptOperationCodes::history().as_str() => {
            history_request(state, pointer, operation).await
        }

        Some(op) if Some(op) == TestscriptOperationCodes::transaction().as_str() => {
            transaction_request(state, pointer, operation)
        }

        Some(op) if Some(op) == TestscriptOperationCodes::create().as_str() => {
            create_request(state, pointer, operation)
        }

        Some(op) if Some(op) == TestscriptOperationCodes::update().as_str() => {
            update_request(state, pointer, operation)
        }

        Some(op) if Some(op) == TestscriptOperationCodes::delete().as_str() => {
            delete_request(state, pointer, operation)
        }

        Some(op) if Some(op) == TestscriptOperationCodes::delete_cond_multiple().as_str() => {
            delete_cond_multiple_request(state, pointer, operation)
        }

        Some("invoke") => invoke_request(state, pointer, operation),

        _ => Err(TestScriptError::ExecutionError(format!(
            "Unsupported TestScript operation type: {op:?} at '{}'.",
            pointer.path(),
        ))),
    }
}

fn read_request(
    state: &TestState,
    pointer: &TypedPointer<TestScript, TestScriptSetupActionOperation>,
    operation: &TestScriptSetupActionOperation,
) -> Result<FHIRRequest, TestScriptError> {
    let target_id = require_target_id(operation, pointer.path(), "Read")?;
    let target = state.resolve_fixture(target_id)?;

    Ok(FHIRRequest::Read(FHIRReadRequest {
        resource_type: derive_resource_type(operation, Some(target), pointer.path())?,
        id: fixture_string_field(target, target_id, "id")?,
    }))
}

fn version_read_request(
    state: &TestState,
    pointer: &TypedPointer<TestScript, TestScriptSetupActionOperation>,
    operation: &TestScriptSetupActionOperation,
) -> Result<FHIRRequest, TestScriptError> {
    let target_id = require_target_id(operation, pointer.path(), "Version Read")?;
    let target = state.resolve_fixture(target_id)?;

    Ok(FHIRRequest::VersionRead(FHIRVersionReadRequest {
        resource_type: derive_resource_type(operation, Some(target), pointer.path())?,
        id: fixture_string_field(target, target_id, "id")?,
        version_id: fixture_version_id(target, target_id)?.into(),
    }))
}

async fn search_request(
    state: &TestState,
    pointer: &TypedPointer<TestScript, TestScriptSetupActionOperation>,
    operation: &TestScriptSetupActionOperation,
) -> Result<FHIRRequest, TestScriptError> {
    let query = operation
        .params
        .as_ref()
        .and_then(|p| p.value.as_deref())
        .unwrap_or_default();

    let parameters =
        parsed_parameters(state, pointer.root(), query, pointer.path(), "Search").await?;

    if let Ok(resource_type) = derive_resource_type(operation, None, pointer.path()) {
        Ok(FHIRRequest::Search(
            haste_fhir_client::request::SearchRequest::Type(
                haste_fhir_client::request::FHIRSearchTypeRequest {
                    resource_type,
                    parameters,
                },
            ),
        ))
    } else {
        Ok(FHIRRequest::Search(
            haste_fhir_client::request::SearchRequest::System(
                haste_fhir_client::request::FHIRSearchSystemRequest { parameters },
            ),
        ))
    }
}

async fn history_request(
    state: &TestState,
    pointer: &TypedPointer<TestScript, TestScriptSetupActionOperation>,
    operation: &TestScriptSetupActionOperation,
) -> Result<FHIRRequest, TestScriptError> {
    let parameters = parsed_parameters(
        state,
        pointer.root(),
        operation
            .params
            .as_ref()
            .and_then(|p| p.value.as_deref())
            .unwrap_or_default(),
        pointer.path(),
        "History",
    )
    .await?;

    if let Some(target_id) = operation
        .targetId
        .as_ref()
        .and_then(|id| id.value.as_deref())
    {
        let target = state.resolve_fixture(target_id)?;

        Ok(FHIRRequest::History(HistoryRequest::Instance(
            FHIRHistoryInstanceRequest {
                resource_type: derive_resource_type(operation, Some(target), pointer.path())?,
                id: fixture_string_field(target, target_id, "id")?,
                parameters,
            },
        )))
    } else if operation.resource.is_some() {
        Ok(FHIRRequest::History(HistoryRequest::Type(
            FHIRHistoryTypeRequest {
                resource_type: derive_resource_type(operation, None, pointer.path())?,
                parameters,
            },
        )))
    } else {
        Ok(FHIRRequest::History(HistoryRequest::System(
            FHIRHistorySystemRequest { parameters },
        )))
    }
}

fn transaction_request(
    state: &TestState,
    pointer: &TypedPointer<TestScript, TestScriptSetupActionOperation>,
    operation: &TestScriptSetupActionOperation,
) -> Result<FHIRRequest, TestScriptError> {
    let source_id = require_source_id(operation, pointer.path(), "Transaction")?;
    let source = state.resolve_fixture(source_id)?;
    let resource = fixture_resource(source, source_id)?;

    match resource {
        Resource::Bundle(bundle) => {
            if bundle.type_ != BundleType::transaction() {
                return Err(TestScriptError::ExecutionError(format!(
                    "Fixture must be a transaction bundle for transaction operations for sourceId '{source_id}'."
                )));
            }

            Ok(FHIRRequest::Transaction(FHIRTransactionRequest {
                resource: bundle,
            }))
        }
        _ => Err(TestScriptError::ExecutionError(format!(
            "Fixture '{source_id}' is not a transaction Bundle resource."
        ))),
    }
}

fn create_request(
    state: &TestState,
    pointer: &TypedPointer<TestScript, TestScriptSetupActionOperation>,
    operation: &TestScriptSetupActionOperation,
) -> Result<FHIRRequest, TestScriptError> {
    let source_id = require_source_id(operation, pointer.path(), "Create")?;
    let source = state.resolve_fixture(source_id)?;
    let resource = fixture_resource(source, source_id)?;

    Ok(FHIRRequest::Create(FHIRCreateRequest {
        resource_type: derive_resource_type(operation, Some(source), pointer.path())?,
        resource,
    }))
}

fn update_request(
    state: &TestState,
    pointer: &TypedPointer<TestScript, TestScriptSetupActionOperation>,
    operation: &TestScriptSetupActionOperation,
) -> Result<FHIRRequest, TestScriptError> {
    let source_id = require_source_id(operation, pointer.path(), "Update")?;
    let source = state.resolve_fixture(source_id)?;
    let resource = fixture_resource(source, source_id)?;

    let target_id = require_target_id(operation, pointer.path(), "Update")?;
    let target = state.resolve_fixture(target_id)?;
    let target_resource = fixture_resource(target, target_id)?;

    Ok(FHIRRequest::Update(UpdateRequest::Instance(
        FHIRUpdateInstanceRequest {
            resource_type: derive_resource_type(operation, Some(target), pointer.path())?,
            id: fixture_string_field(&target_resource, target_id, "id")?,
            resource,
        },
    )))
}

fn delete_request(
    state: &TestState,
    pointer: &TypedPointer<TestScript, TestScriptSetupActionOperation>,
    operation: &TestScriptSetupActionOperation,
) -> Result<FHIRRequest, TestScriptError> {
    let target_id = require_target_id(operation, pointer.path(), "Delete")?;
    let target = state.resolve_fixture(target_id)?;

    Ok(FHIRRequest::Delete(DeleteRequest::Instance(
        FHIRDeleteInstanceRequest {
            resource_type: derive_resource_type(operation, Some(target), pointer.path())?,
            id: fixture_string_field(target, target_id, "id")?,
        },
    )))
}

fn delete_cond_multiple_request(
    _state: &TestState,
    pointer: &TypedPointer<TestScript, TestScriptSetupActionOperation>,
    operation: &TestScriptSetupActionOperation,
) -> Result<FHIRRequest, TestScriptError> {
    let parameters = ParsedParameters::try_from(
        operation
            .params
            .as_ref()
            .and_then(|p| p.value.as_deref())
            .unwrap_or_default(),
    )
    .map_err(|e| {
        TestScriptError::ExecutionError(format!(
            "Failed to parse parameters for DeleteCondMultiple operation at '{}': {}",
            pointer.path(),
            e
        ))
    })?;

    if operation.resource.is_some() {
        Ok(FHIRRequest::Delete(DeleteRequest::Type(
            FHIRDeleteTypeRequest {
                resource_type: derive_resource_type(operation, None, pointer.path())?,
                parameters,
            },
        )))
    } else {
        Ok(FHIRRequest::Delete(DeleteRequest::System(
            FHIRDeleteSystemRequest { parameters },
        )))
    }
}

fn invoke_request(
    state: &TestState,
    pointer: &TypedPointer<TestScript, TestScriptSetupActionOperation>,
    operation: &TestScriptSetupActionOperation,
) -> Result<FHIRRequest, TestScriptError> {
    let op_code = operation
        .url
        .as_ref()
        .and_then(|u| u.value.as_deref())
        .ok_or_else(|| {
            TestScriptError::ExecutionError(format!(
                "Invoke operation requires url at '{}' which is used for the operation code.",
                pointer.path()
            ))
        })?;

    let fhir_operation = Operation::new(op_code);

    let source_id = require_source_id(operation, pointer.path(), "Invoke")?;
    let source = state.resolve_fixture(source_id)?;

    let Resource::Parameters(parameters) = fixture_resource(source, source_id)? else {
        return Err(TestScriptError::ExecutionError(format!(
            "Source fixture '{source_id}' is not a Parameters resource."
        )));
    };

    if let Some(target_id) = operation
        .targetId
        .as_ref()
        .and_then(|id| id.value.as_deref())
    {
        let target = state.resolve_fixture(target_id)?;

        Ok(FHIRRequest::Invocation(InvocationRequest::Instance(
            FHIRInvokeInstanceRequest {
                operation: fhir_operation,
                resource_type: derive_resource_type(operation, Some(target), pointer.path())?,
                id: fixture_string_field(target, target_id, "id")?,
                parameters,
            },
        )))
    } else if let Ok(resource_type) = derive_resource_type(operation, None, pointer.path()) {
        Ok(FHIRRequest::Invocation(InvocationRequest::Type(
            FHIRInvokeTypeRequest {
                operation: fhir_operation,
                resource_type,
                parameters,
            },
        )))
    } else {
        Ok(FHIRRequest::Invocation(InvocationRequest::System(
            FHIRInvokeSystemRequest {
                operation: fhir_operation,
                parameters,
            },
        )))
    }
}

fn get_operation(
    pointer: &TypedPointer<TestScript, TestScriptSetupActionOperation>,
) -> Result<&TestScriptSetupActionOperation, TestScriptError> {
    pointer.value().ok_or_else(|| {
        TestScriptError::ExecutionError(format!(
            "Failed to retrieve TestScript operation at '{}'.",
            pointer.path()
        ))
    })
}

fn get_operation_type(operation: &TestScriptSetupActionOperation) -> Option<&str> {
    operation
        .type_
        .as_ref()
        .and_then(|t| t.code.as_ref())
        .and_then(|c| c.value.as_deref())
}

fn require_target_id<'a>(
    operation: &'a TestScriptSetupActionOperation,
    path: &str,
    operation_name: &str,
) -> Result<&'a str, TestScriptError> {
    operation
        .targetId
        .as_ref()
        .and_then(|id| id.value.as_deref())
        .ok_or_else(|| {
            TestScriptError::ExecutionError(format!(
                "{operation_name} operation requires targetId at '{path}'."
            ))
        })
}

fn require_source_id<'a>(
    operation: &'a TestScriptSetupActionOperation,
    path: &str,
    operation_name: &str,
) -> Result<&'a str, TestScriptError> {
    operation
        .sourceId
        .as_ref()
        .and_then(|id| id.value.as_deref())
        .ok_or_else(|| {
            TestScriptError::ExecutionError(format!(
                "{operation_name} operation requires sourceId at '{path}'."
            ))
        })
}

fn fixture_string_field(
    fixture: &dyn MetaValue,
    fixture_name: &str,
    field: &str,
) -> Result<String, TestScriptError> {
    fixture
        .get_field(field)
        .ok_or_else(|| {
            TestScriptError::ExecutionError(format!(
                "Fixture '{fixture_name}' does not have '{field}' field."
            ))
        })?
        .as_any()
        .downcast_ref::<String>()
        .cloned()
        .ok_or_else(|| {
            TestScriptError::ExecutionError(format!(
                "Field '{field}' on fixture '{fixture_name}' is not a String."
            ))
        })
}

fn fixture_version_id(
    fixture: &dyn MetaValue,
    fixture_name: &str,
) -> Result<String, TestScriptError> {
    fixture
        .get_field("meta")
        .and_then(|meta| meta.get_field("versionId"))
        .ok_or_else(|| {
            TestScriptError::ExecutionError(format!(
                "Fixture '{fixture_name}' does not have a 'versionId' field."
            ))
        })?
        .as_any()
        .downcast_ref::<Box<FHIRId>>()
        .cloned()
        .and_then(|v| v.value)
        .ok_or_else(|| {
            TestScriptError::ExecutionError(format!(
                "Fixture '{fixture_name}' does not have a valid 'versionId' field."
            ))
        })
}

fn fixture_resource(
    fixture: &dyn MetaValue,
    fixture_name: &str,
) -> Result<Resource, TestScriptError> {
    (fixture as &dyn Any)
        .downcast_ref::<Resource>()
        .cloned()
        .ok_or_else(|| {
            TestScriptError::ExecutionError(format!("Fixture '{fixture_name}' is not a Resource."))
        })
}

async fn parsed_parameters(
    state: &TestState,
    root: TypedPointer<TestScript, TestScript>,
    raw: &str,
    path: &str,
    operation: &str,
) -> Result<ParsedParameters, TestScriptError> {
    let evaluated = evaluate_variable(state, root, raw).await?;

    ParsedParameters::try_from(evaluated.as_str()).map_err(|e| {
        TestScriptError::ExecutionError(format!(
            "Failed to parse parameters for {operation} operation at '{path}': {e}"
        ))
    })
}

async fn run_operation<CTX, Client: FHIRClient<CTX, OperationOutcomeError>>(
    client: &Client,
    ctx: CTX,
    state: Arc<Mutex<TestState>>,
    pointer: TypedPointer<TestScript, TestScriptSetupActionOperation>,
    options: Arc<TestRunnerOptions>,
) -> Result<TestResult<TestReportSetupActionOperation>, TestScriptError> {
    let operation = pointer.value().ok_or_else(|| {
        TestScriptError::ExecutionError(format!(
            "Failed to retrieve TestScript operation at '{}'.",
            pointer.path()
        ))
    })?;

    let mut state_guard = state.lock().await;
    let fhir_request = testscript_operation_to_fhir_request(&state_guard, &pointer).await?;
    let fhir_response = client.request(ctx, fhir_request.clone()).await;
    if let Some(wait_duration) = options.wait_between_operations {
        tokio::time::sleep(wait_duration).await;
    }

    match fhir_response {
        Ok(fhir_response) => {
            associate_request_response_variables(
                &mut state_guard,
                operation,
                fhir_request,
                Response::FHIRResponse(Box::new(fhir_response)),
            );

            drop(state_guard);

            Ok(TestResult {
                state: state.clone(),
                value: TestReportSetupActionOperation {
                    result: ReportActionResultCodes::pass(),
                    ..Default::default()
                },
            })
        }
        Err(op_error) => {
            let op_error = Arc::new(op_error);
            tracing::warn!(
                path = pointer.path(),
                operation.label = operation
                    .label
                    .as_ref()
                    .and_then(|l| l.value.as_deref())
                    .unwrap_or("<no-label>"),
                operation.operation_type = get_operation_type(operation).unwrap_or("<unknown>"),
                error = %op_error,
                "TestScript operation failed"
            );
            associate_request_response_variables(
                &mut state_guard,
                operation,
                fhir_request,
                Response::OperationError(op_error.clone()),
            );

            Ok(TestResult {
                state: state.clone(),
                value: TestReportSetupActionOperation {
                    result: ReportActionResultCodes::warning(),
                    message: Some(Box::new(FHIRMarkdown {
                        value: Some(format!("Operation failed: {op_error}")),
                        ..Default::default()
                    })),
                    ..Default::default()
                },
            })
        }
    }
}

fn get_source<'a>(
    state: &'a TestState,
    assertion: &TestScriptSetupActionAssert,
) -> Result<Option<&'a dyn MetaValue>, TestScriptError> {
    if let Some(source_id) = assertion.sourceId.as_ref().and_then(|id| id.value.as_ref()) {
        let source = state.resolve_fixture(source_id)?;
        Ok(Some(source))
    } else {
        match assertion
            .direction
            .as_ref()
            .unwrap_or(&AssertDirectionCodes::response())
        {
            assertion if assertion == &AssertDirectionCodes::request() => {
                if let Some(request) = state.latest_request.as_ref() {
                    request_to_meta_value(request)
                        .ok_or_else(|| TestScriptError::InvalidFixture)
                        .map(Some)
                } else {
                    Ok(None)
                }
            }
            assertion if assertion == &AssertDirectionCodes::response() => {
                if let Some(response) = state.latest_response.as_ref() {
                    response_to_meta_value(response)
                        .ok_or_else(|| TestScriptError::InvalidFixture)
                        .map(Some)
                } else {
                    Ok(None)
                }
            }
            _ => Err(TestScriptError::ExecutionError(
                "Assert direction cannot be 'null' when sourceId is not provided.".to_string(),
            )),
        }
    }
}

fn evaluate_operator(
    operator: &BoundCode<AssertOperatorCodes>,
    a: &Vec<conversion::ConvertedValue>,
    b: &Vec<conversion::ConvertedValue>,
) -> bool {
    match operator {
        operator
            if operator == &AssertOperatorCodes::equals()
                || operator == &AssertOperatorCodes::null() =>
        {
            a == b
        }
        operator if operator == &AssertOperatorCodes::not_equals() => !(a == b),

        operator if operator == &AssertOperatorCodes::contains() => {
            if a.len() != 1 || b.len() != 1 {
                return false;
            }

            match (&a[0], &b[0]) {
                (ConvertedValue::String(a_str), ConvertedValue::String(b_str)) => {
                    a_str.contains(b_str)
                }
                _ => false,
            }
        }
        operator if operator == &AssertOperatorCodes::empty() => {
            todo!("Empty operator not implemented")
        }
        operator if operator == &AssertOperatorCodes::eval() => {
            todo!("Eval operator not implemented")
        }
        operator if operator == &AssertOperatorCodes::greater_than() => {
            todo!("GreaterThan operator not implemented")
        }
        operator if operator == &AssertOperatorCodes::in_() => todo!("In operator not implemented"),
        operator if operator == &AssertOperatorCodes::less_than() => {
            todo!("LessThan operator not implemented")
        }
        operator if operator == &AssertOperatorCodes::not_contains() => {
            todo!("NotContains operator not implemented")
        }
        operator if operator == &AssertOperatorCodes::not_empty() => {
            todo!("NotEmpty operator not implemented")
        }
        operator if operator == &AssertOperatorCodes::not_in() => {
            todo!("NotIn operator not implemented")
        }
        _ => {
            todo!("Operator '{:?}' not implemented", operator)
        }
    }
    // a == b
}

async fn derive_comparison_to(
    state: &TestState,
    assertion: &TestScriptSetupActionAssert,
) -> Result<Vec<ConvertedValue>, TestScriptError> {
    if let Some(comparision_fixture_id) = assertion
        .compareToSourceId
        .as_ref()
        .and_then(|c| c.value.as_ref())
    {
        let comparison_fixture = state.resolve_fixture(comparision_fixture_id)?;

        let Some(comparison_expression) = assertion
            .compareToSourceExpression
            .as_ref()
            .and_then(|exp| exp.value.as_ref())
        else {
            return Err(TestScriptError::ExecutionError(
                "compareToSourceExpression is required when compareToSourceId is provided."
                    .to_string(),
            ));
        };

        let result = state
            .fp_engine
            .evaluate(comparison_expression, vec![comparison_fixture])
            .await
            .map_err(|e| {
                TestScriptError::ExecutionError(format!(
                    "FHIRPath evaluation error for comparison fixture '{comparision_fixture_id}': {e}"
                ))
            })?;

        Ok(result
            .iter()
            .map(conversion::convert_meta_value)
            .collect::<Vec<_>>())
    } else if let Some(value) = assertion.value.as_ref().and_then(|v| v.value.as_ref())
        && let Some(converted_value) = conversion::convert_string_value(value.as_ref())
    {
        Ok(vec![converted_value])
    } else {
        Err(TestScriptError::ExecutionError(
            "Failed to derive comparison value for assertion.".to_string(),
        ))
    }
}

/// Assertions carry a `label`/`description` intended by the FHIR spec for
/// exactly this purpose: identifying an assertion in test engine output.
fn assert_label(assertion: &TestScriptSetupActionAssert) -> &str {
    assertion
        .label
        .as_ref()
        .and_then(|l| l.value.as_deref())
        .or_else(|| {
            assertion
                .description
                .as_ref()
                .and_then(|d| d.value.as_deref())
        })
        .unwrap_or("<no-label>")
}

/// Assertions are what determine the testreports ultimate pass/fail status.
/// So set that within state here depending on assertion success/failure.
async fn run_assertion(
    state: Arc<Mutex<TestState>>,
    pointer: TypedPointer<TestScript, TestScriptSetupActionAssert>,
) -> Result<TestResult<TestReportSetupActionAssert>, TestScriptError> {
    let assertion = pointer.value().ok_or_else(|| {
        TestScriptError::ExecutionError(format!(
            "Failed to retrieve TestScript assertion at '{}'.",
            pointer.path()
        ))
    })?;

    let mut state_guard = state.lock().await;

    let Some(source) = get_source(&state_guard, assertion)? else {
        return Err(TestScriptError::ExecutionError(format!(
            "Failed to resolve source for assertion at '{}'.",
            pointer.path()
        )));
    };
    let default = AssertOperatorCodes::equals();
    let operator = assertion.operator.as_ref().unwrap_or(&default);

    if assertion.resource.is_some() {
        let resource_string = assertion
            .resource
            .as_ref()
            .and_then(haste_fhir_model::r4::generated::terminology::BoundCode::as_str)
            .unwrap_or("");

        let operation_evaluation_result = evaluate_operator(
            operator,
            &vec![conversion::ConvertedValue::String(
                resource_string.to_string(),
            )],
            &vec![conversion::ConvertedValue::String(
                source.fhir_type().to_string(),
            )],
        );
        if !operation_evaluation_result {
            let message = format!(
                "Assertion '{}' at '{}' failed: resource type '{resource_string}' does not match '{}'.",
                assert_label(assertion),
                pointer.path(),
                source.fhir_type()
            );
            tracing::error!(
                assert.label = assert_label(assertion),
                path = pointer.path(),
                "{message}"
            );

            state_guard.result = ReportResultCodes::fail();
            return Ok(TestResult {
                state: state.clone(),
                value: TestReportSetupActionAssert {
                    result: ReportActionResultCodes::fail(),
                    message: Some(Box::new(FHIRMarkdown {
                        value: Some(message),
                        ..Default::default()
                    })),
                    ..Default::default()
                },
            });
        }
    }
    if let Some(expression) = assertion.expression.as_ref().and_then(|e| e.value.as_ref()) {
        let comparison_to = derive_comparison_to(&state_guard, assertion).await?;

        let Ok(result) = state_guard
            .fp_engine
            .evaluate(expression, vec![source])
            .await
        else {
            tracing::error!(
                assert.label = assert_label(assertion),
                path = pointer.path(),
                expression = expression.as_str(),
                "Assertion '{}' at '{}' failed: FHIRPath expression '{expression}' failed to evaluate.",
                assert_label(assertion),
                pointer.path(),
            );

            state_guard.result = ReportResultCodes::fail();
            return Err(TestScriptError::ExecutionError(format!(
                "Assertion '{}' at '{}': FHIRPath expression '{expression}' failed to evaluate.",
                assert_label(assertion),
                pointer.path(),
            )));
        };

        let converted_values = result
            .iter()
            .map(conversion::convert_meta_value)
            .collect::<Vec<_>>();

        let operation_evaluation_result =
            evaluate_operator(operator, &converted_values, &comparison_to);

        if !operation_evaluation_result {
            let message = format!(
                "Assertion '{}' at '{}' failed: '{converted_values:?}' {operator:?} '{comparison_to:?}'.",
                assert_label(assertion),
                pointer.path(),
            );
            tracing::error!(
                assert.label = assert_label(assertion),
                path = pointer.path(),
                "{message}"
            );

            state_guard.result = ReportResultCodes::fail();
            return Ok(TestResult {
                state: state.clone(),
                value: TestReportSetupActionAssert {
                    result: ReportActionResultCodes::fail(),
                    message: Some(Box::new(FHIRMarkdown {
                        value: Some(message),
                        ..Default::default()
                    })),
                    ..Default::default()
                },
            });
        }
    }

    Ok(TestResult {
        state: state.clone(),
        value: TestReportSetupActionAssert {
            result: ReportActionResultCodes::pass(),
            ..Default::default()
        },
    })
}

async fn run_action<CTX, Client: FHIRClient<CTX, OperationOutcomeError>>(
    client: &Client,
    ctx: CTX,
    state: Arc<Mutex<TestState>>,
    pointer: TypedPointer<TestScript, TestScriptTestAction>,
    options: Arc<TestRunnerOptions>,
) -> Result<TestResult<TestReportSetupAction>, TestScriptError> {
    tracing::info!("Running TestScript action at path: {}", pointer.path());
    let action = pointer.value().ok_or_else(|| {
        TestScriptError::ExecutionError(format!(
            "Failed to retrieve TestScript action at '{}'.",
            pointer.path()
        ))
    })?;

    // Should be either an operation or an assert.
    // Both should not exist at the same time.
    if action.operation.is_some() {
        let Some(operation_pointer) =
            pointer.descend::<TestScriptSetupActionOperation>(&Key::Field("operation".to_string()))
        else {
            return Err(TestScriptError::ExecutionError(format!(
                "Failed to retrieve TestScript operation at '{}'.",
                pointer.path()
            )));
        };

        let result = run_operation(client, ctx, state, operation_pointer, options).await?;

        Ok(TestResult {
            state: result.state,
            value: TestReportSetupAction {
                operation: Some(result.value),
                ..Default::default()
            },
        })
    } else if action.assert.is_some() {
        let Some(assertion_pointer) =
            pointer.descend::<TestScriptSetupActionAssert>(&Key::Field("assert".to_string()))
        else {
            return Err(TestScriptError::ExecutionError(format!(
                "Failed to retrieve TestScript assertion at '{}'.",
                pointer.path()
            )));
        };

        let assertion = run_assertion(state, assertion_pointer).await?;

        Ok(TestResult {
            state: assertion.state,
            value: TestReportSetupAction {
                assert: Some(assertion.value),
                ..Default::default()
            },
        })
    } else {
        Err(TestScriptError::ExecutionError(format!(
            "TestScript action must have either an operation or an assert at '{}'.",
            pointer.path()
        )))
    }
}

async fn run_setup_action<CTX, Client: FHIRClient<CTX, OperationOutcomeError>>(
    client: &Client,
    ctx: CTX,
    state: Arc<Mutex<TestState>>,
    pointer: TypedPointer<TestScript, TestScriptSetupAction>,
    options: Arc<TestRunnerOptions>,
) -> Result<TestResult<TestReportSetupAction>, TestScriptError> {
    let action = pointer.value().ok_or_else(|| {
        TestScriptError::ExecutionError(format!(
            "Failed to retrieve TestScript action at '{}'.",
            pointer.path()
        ))
    })?;

    tracing::info!("Running TestScript action at path: {}", pointer.path());

    // Should be either an operation or an assert.
    // Both should not exist at the same time.
    if action.operation.is_some() {
        let Some(operation_pointer) =
            pointer.descend::<TestScriptSetupActionOperation>(&Key::Field("operation".to_string()))
        else {
            return Err(TestScriptError::ExecutionError(format!(
                "Failed to retrieve TestScript operation at '{}'.",
                pointer.path()
            )));
        };

        let result = run_operation(client, ctx, state, operation_pointer, options).await?;

        Ok(TestResult {
            state: result.state,
            value: TestReportSetupAction {
                operation: Some(result.value),
                ..Default::default()
            },
        })
    } else if action.assert.is_some() {
        let Some(assertion_pointer) =
            pointer.descend::<TestScriptSetupActionAssert>(&Key::Field("assert".to_string()))
        else {
            return Err(TestScriptError::ExecutionError(format!(
                "Failed to retrieve TestScript assertion at '{}'.",
                pointer.path()
            )));
        };

        let assertion = run_assertion(state, assertion_pointer).await?;

        Ok(TestResult {
            state: assertion.state,
            value: TestReportSetupAction {
                assert: Some(assertion.value),
                ..Default::default()
            },
        })
    } else {
        Err(TestScriptError::ExecutionError(format!(
            "TestScript action must have either an operation or an assert at '{}'.",
            pointer.path()
        )))
    }
}

async fn setup_fixtures<CTX: Clone, Client: FHIRClient<CTX, OperationOutcomeError>>(
    client: &Client,
    ctx: CTX,
    state: Arc<Mutex<TestState>>,
    pointer: TypedPointer<TestScript, TestScript>,
    _options: Arc<TestRunnerOptions>,
) -> Result<Arc<Mutex<TestState>>, OperationOutcomeError> {
    let mut state_lock = state.lock().await;

    let Some(fixtures_pointer) =
        pointer.descend::<Vec<TestScriptFixture>>(&Key::Field("fixture".to_string()))
    else {
        return Ok(state.clone());
    };

    let Some(fixtures) = fixtures_pointer.value() else {
        return Ok(state.clone());
    };

    for fixture in fixtures {
        if let Some(reference_string) = fixture
            .resource
            .as_ref()
            .and_then(|r| r.reference.as_ref())
            .and_then(|refe| refe.value.as_ref())
        {
            let resolved_resource = if reference_string.starts_with('#')
                && let Some(contained) =
                    pointer.descend::<Vec<Resource>>(&Key::Field("contained".to_string()))
                && let Some(contained) = contained.value()
            {
                let local_id = &reference_string[1..];
                let Some(resource) = contained.iter().find(|res| {
                    if let Some(id) = res.get_field("id")
                        && let Some(id) = id.as_any().downcast_ref::<String>()
                    {
                        id.as_str() == local_id
                    } else {
                        false
                    }
                }) else {
                    return Err(OperationOutcomeError::error(
                        IssueType::not_found(),
                        format!("Contained resource with id '{local_id}' not found."),
                    ));
                };

                resource.clone()
            } else {
                let parts = reference_string.split('/').collect::<Vec<&str>>();
                if parts.len() != 2 {
                    return Err(OperationOutcomeError::error(
                        IssueType::invalid(),
                        format!("Invalid fixture reference: {reference_string}"),
                    ));
                }

                let resource_type = parts[0];
                let id = parts[1];

                let Some(remote_resource) = client
                    .read(
                        ctx.clone(),
                        ResourceType::try_from(resource_type).map_err(|_| {
                            OperationOutcomeError::error(
                                IssueType::invalid(),
                                format!(
                                    "Invalid resource type in fixture reference: '{resource_type}'"
                                ),
                            )
                        })?,
                        id.to_string(),
                    )
                    .await?
                else {
                    return Err(OperationOutcomeError::error(
                        IssueType::not_found(),
                        format!("Resource '{resource_type}' with id '{id}' not found."),
                    ));
                };

                remote_resource
            };

            state_lock.fixtures.insert(
                fixture.id.clone().unwrap_or_default(),
                Fixtures::Resource(resolved_resource),
            );
        }
    }

    drop(state_lock);

    Ok(state)
}

async fn run_setup<CTX: Clone, Client: FHIRClient<CTX, OperationOutcomeError>>(
    client: &Client,
    ctx: CTX,
    state: Arc<Mutex<TestState>>,
    pointer: TypedPointer<TestScript, TestScriptSetup>,
    options: Arc<TestRunnerOptions>,
) -> Result<TestResult<TestReportSetup>, TestScriptError> {
    let mut cur_state = state;

    let mut setup_results = TestReportSetup {
        action: vec![],
        ..Default::default()
    };

    let Some(setup) = pointer.value() else {
        return Ok(TestResult {
            state: cur_state,
            value: setup_results,
        });
    };

    for action in setup.action.iter().enumerate() {
        let action_pointer = pointer
            .descend::<Vec<TestScriptSetupAction>>(&Key::Field("action".to_string()))
            .and_then(|p| p.descend::<TestScriptSetupAction>(&Key::Index(action.0)));

        let action_pointer = action_pointer.ok_or_else(|| {
            TestScriptError::ExecutionError(format!(
                "Failed to retrieve TestScript action at index {}.",
                action.0
            ))
        })?;

        let result = run_setup_action(
            client,
            ctx.clone(),
            cur_state,
            action_pointer,
            options.clone(),
        )
        .await?;
        cur_state = result.state;

        setup_results.action.push(result.value);
    }

    Ok(TestResult {
        state: cur_state,
        value: setup_results,
    })
}

async fn run_teardown<CTX: Clone, Client: FHIRClient<CTX, OperationOutcomeError>>(
    client: &Client,
    ctx: CTX,
    state: Arc<Mutex<TestState>>,
    pointer: TypedPointer<TestScript, TestScriptTeardown>,
    options: Arc<TestRunnerOptions>,
) -> Result<TestResult<TestReportTeardown>, TestScriptError> {
    let mut cur_state = state;

    let mut teardown_results = TestReportTeardown {
        action: vec![],
        ..Default::default()
    };

    let Some(actions) = pointer.value() else {
        return Ok(TestResult {
            state: cur_state,
            value: teardown_results,
        });
    };

    for action in actions.action.iter().enumerate() {
        let action_pointer = pointer
            .descend::<Vec<TestScriptTeardownAction>>(&Key::Field("action".to_string()))
            .and_then(|p| p.descend::<TestScriptTeardownAction>(&Key::Index(action.0)));

        let action_pointer = action_pointer.ok_or_else(|| {
            TestScriptError::ExecutionError(format!(
                "Failed to retrieve TestScript teardown action at index {}.",
                action.0
            ))
        })?;

        let operation_pointer = action_pointer
            .descend::<TestScriptSetupActionOperation>(&Key::Field("operation".to_string()))
            .ok_or_else(|| {
                TestScriptError::ExecutionError(format!(
                    "Failed to retrieve TestScript teardown operation at index {}.",
                    action.0
                ))
            })?;

        let result = run_operation(
            client,
            ctx.clone(),
            cur_state,
            operation_pointer,
            options.clone(),
        )
        .await?;
        cur_state = result.state;

        teardown_results.action.push(TestReportTeardownAction {
            operation: result.value,
            ..Default::default()
        });
    }

    Ok(TestResult {
        state: cur_state,
        value: teardown_results,
    })
}

async fn run_test<CTX: Clone, Client: FHIRClient<CTX, OperationOutcomeError>>(
    client: &Client,
    ctx: CTX,
    state: Arc<Mutex<TestState>>,
    pointer: TypedPointer<TestScript, TestScriptTest>,
    options: Arc<TestRunnerOptions>,
) -> Result<TestResult<TestReportTest>, TestScriptError> {
    let mut cur_state = state;
    let mut test_report_test = TestReportTest {
        action: vec![],
        ..Default::default()
    };

    let test = pointer.value().ok_or_else(|| {
        TestScriptError::ExecutionError(format!(
            "Failed to retrieve TestScript test at '{}'.",
            pointer.path()
        ))
    })?;

    for action in test.action.iter().enumerate() {
        let Some(action_pointer) = pointer
            .descend::<Vec<TestScriptTestAction>>(&Key::Field("action".to_string()))
            .and_then(|p| p.descend(&Key::Index(action.0)))
        else {
            return Err(TestScriptError::ExecutionError(format!(
                "Failed to retrieve TestScript test action at index {}.",
                action.0
            )));
        };
        let result = run_action(
            client,
            ctx.clone(),
            cur_state,
            action_pointer,
            options.clone(),
        )
        .await?;
        cur_state = result.state;
        test_report_test.action.push(TestReportTestAction {
            operation: result.value.operation,
            assert: result.value.assert,
            ..Default::default()
        });
    }

    Ok(TestResult {
        state: cur_state,
        value: test_report_test,
    })
}

async fn run_tests<CTX: Clone, Client: FHIRClient<CTX, OperationOutcomeError>>(
    client: &Client,
    ctx: CTX,
    state: Arc<Mutex<TestState>>,
    pointer: TypedPointer<TestScript, Vec<TestScriptTest>>,
    options: Arc<TestRunnerOptions>,
) -> Result<TestResult<Vec<TestReportTest>>, TestScriptError> {
    let mut test_results = vec![];
    let mut cur_state = state;

    let Some(tests) = pointer.value() else {
        return Ok(TestResult {
            state: cur_state,
            value: test_results,
        });
    };

    for test in tests.iter().enumerate() {
        let Some(test_pointer) = pointer.descend(&Key::Index(test.0)) else {
            return Err(TestScriptError::ExecutionError(format!(
                "Failed to retrieve TestScript test at index {}.",
                test.0
            )));
        };
        let test_result = run_test(
            client,
            ctx.clone(),
            cur_state,
            test_pointer,
            options.clone(),
        )
        .await?;
        cur_state = test_result.state;
        test_results.push(test_result.value);
    }

    Ok(TestResult {
        state: cur_state,
        value: test_results,
    })
}

pub struct TestRunnerOptions {
    pub wait_between_operations: Option<Duration>,
}

/// Runs a FHIR `TestScript` using the provided client and execution context.
///
/// This executes the `TestScript` lifecycle:
/// - fixture setup
/// - setup actions
/// - test actions
/// - teardown actions
///
/// # Errors
///
/// Returns [`TestScriptError`] if:
/// - fixture setup fails
/// - setup actions fail
/// - test execution fails
/// - teardown execution fails
/// - an operation performed by the FHIR client fails
#[tracing::instrument(
    name = "testscript_run",
    skip_all,
    fields(
        testscript.id = test_script.id.as_deref().unwrap_or("<no-id>"),
        testscript.name = test_script.name.value.as_deref().unwrap_or("<unnamed>"),
        testscript.url = test_script.url.value.as_deref().unwrap_or("<no-url>"),
    )
)]
pub async fn run<CTX: Clone, Client: FHIRClient<CTX, OperationOutcomeError>>(
    client: &Client,
    ctx: CTX,
    test_script: Arc<TestScript>,
    options: Arc<TestRunnerOptions>,
) -> Result<TestReport, TestScriptError> {
    tracing::info!("Starting TestScript run");

    let mut test_report = TestReport {
        status: ReportStatusCodes::completed(),
        testScript: Box::new(Reference {
            reference: Some(Box::new(FHIRString {
                value: Some(format!(
                    "Testscript/{}",
                    test_script.id.clone().unwrap_or_default()
                )),
                ..Default::default()
            })),
            ..Default::default()
        }),
        ..Default::default()
    };

    let mut state = Arc::new(Mutex::new(TestState::new()));
    let pointer = TypedPointer::<TestScript, TestScript>::new(test_script);

    state = setup_fixtures(client, ctx.clone(), state, pointer.clone(), options.clone())
        .await
        .map_err(TestScriptError::OperationError)?;

    let mut running_state = Ok(());

    // Run setup actions
    if let Some(setup_pointer) =
        pointer.descend::<TestScriptSetup>(&Key::Field("setup".to_string()))
    {
        tracing::info!("Running TestScript setup...");
        let setup_result = run_setup(
            client,
            ctx.clone(),
            state.clone(),
            setup_pointer,
            options.clone(),
        )
        .await;
        match setup_result {
            Ok(res) => {
                state = res.state;
                test_report.setup = Some(res.value);
            }
            Err(e) => {
                tracing::error!(phase = "setup", error = ?e, "TestScript run failed during setup");
                running_state = Err(e);
            }
        }
    }

    // Run Test actions
    if running_state.is_ok()
        && let Some(test_pointer) =
            pointer.descend::<Vec<TestScriptTest>>(&Key::Field("test".to_string()))
    {
        tracing::info!("Running TestScript tests...");
        let test_result = run_tests(
            client,
            ctx.clone(),
            state.clone(),
            test_pointer,
            options.clone(),
        )
        .await;

        match test_result {
            Ok(res) => {
                state = res.state;
                test_report.test = Some(res.value);
            }

            Err(e) => {
                tracing::error!(phase = "test", error = ?e, "TestScript run failed during test execution");
                running_state = Err(e);
            }
        }
    }

    if let Some(teardown_pointer) =
        pointer.descend::<TestScriptTeardown>(&Key::Field("teardown".to_string()))
    {
        tracing::info!("Running TestScript teardown...");

        let result = run_teardown(
            client,
            ctx.clone(),
            state.clone(),
            teardown_pointer,
            options.clone(),
        )
        .await?;

        // state = result.state;
        test_report.teardown = Some(result.value);
    }

    running_state?;

    let state_guard = state.lock().await;
    // Only set result to fail so if still pending can assume pass.
    // Flip to fail in assertion tests if any fail.
    match &state_guard.result {
        state if state == &ReportResultCodes::pending() => {
            test_report.result = ReportResultCodes::pass();
        }
        status => test_report.result = status.clone(),
    }

    if test_report.result == ReportResultCodes::fail() {
        tracing::error!(result = ?test_report.result, "TestScript run finished with failures");
    } else {
        tracing::info!(result = ?test_report.result, "TestScript run finished");
    }

    Ok(test_report)
}
