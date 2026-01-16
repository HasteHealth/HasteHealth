use haste_fhir_client::{
    FHIRClient,
    request::{
        DeleteRequest, FHIRCreateRequest, FHIRDeleteInstanceRequest, FHIRDeleteSystemRequest,
        FHIRDeleteTypeRequest, FHIRReadRequest, FHIRRequest, FHIRResponse, FHIRTransactionRequest,
        HistoryResponse, InvokeResponse, SearchResponse, UpdateRequest,
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
        TestScriptTest, TestScriptTestAction,
    },
    terminology::{
        AssertDirectionCodes, AssertOperatorCodes, BundleType, IssueType, ReportActionResultCodes,
        ReportResultCodes, ReportStatusCodes, TestscriptOperationCodes,
    },
    types::{FHIRMarkdown, FHIRString, Reference},
};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_pointer::{Key, Pointer};
use haste_reflect::MetaValue;
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

#[derive(Debug)]
enum Fixtures {
    Resource(Resource),
    Request(FHIRRequest),
    Response(FHIRResponse),
}

// Internal structure to hold current test result and testing fixtures.
struct TestState {
    fp_engine: haste_fhirpath::FPEngine,
    fixtures: HashMap<String, Fixtures>,
    latest_request: Option<FHIRRequest>,
    latest_response: Option<FHIRResponse>,
    result: ReportResultCodes,
}

impl TestState {
    fn new() -> Self {
        TestState {
            fp_engine: haste_fhirpath::FPEngine::new(),
            fixtures: HashMap::new(),
            latest_request: None,
            latest_response: None,
            result: ReportResultCodes::Pending(None),
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

fn response_to_meta_value<'a>(response: &'a FHIRResponse) -> Option<&'a dyn MetaValue> {
    match response {
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
    }
}

fn request_to_meta_value<'a>(request: &'a FHIRRequest) -> Option<&'a dyn MetaValue> {
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
    response: FHIRResponse,
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
        let string_type: Option<String> = operation_resource_type.as_ref().into();
        ResourceType::try_from(string_type.unwrap_or_default()).map_err(|_| {
            TestScriptError::ExecutionError(format!(
                "Unsupported resource type '{:?}' for operation at '{}'.",
                operation_resource_type.as_ref(),
                path
            ))
        })
    } else if let Some(target) = target {
        ResourceType::try_from(target.typename()).map_err(|_| {
            TestScriptError::ExecutionError(format!(
                "Unsupported resource type '{}' for operation at '{}'.",
                target.typename(),
                path
            ))
        })
    } else {
        Err(TestScriptError::ExecutionError(format!(
            "Failed to derive resource type for operation at '{}'.",
            path
        )))
    }
}

fn testscript_operation_to_fhir_request(
    state: &TestState,
    operation: &TestScriptSetupActionOperation,
    path: &str,
) -> Result<FHIRRequest, TestScriptError> {
    let operation_type = operation
        .type_
        .as_ref()
        .and_then(|t| t.code.as_ref())
        .and_then(|c| c.value.clone());

    if operation_type == (&TestscriptOperationCodes::Read(None)).into() {
        let Some(target_id) = operation.targetId.as_ref().and_then(|id| id.value.as_ref()) else {
            return Err(TestScriptError::ExecutionError(format!(
                "Read operation requires targetId at '{}'.",
                path
            )));
        };

        let target = state.resolve_fixture(target_id)?;

        Ok(FHIRRequest::Read(FHIRReadRequest {
            resource_type: derive_resource_type(operation, Some(target), path)?,
            id: target
                .get_field("id")
                .ok_or_else(|| {
                    TestScriptError::ExecutionError(format!(
                        "Target fixture '{}' does not have an 'id' field.",
                        target_id
                    ))
                })?
                .as_any()
                .downcast_ref::<String>()
                .cloned()
                .unwrap_or_default(),
        }))
    } else if operation_type == (&TestscriptOperationCodes::Transaction(None)).into() {
        let Some(source_id) = operation.sourceId.as_ref().and_then(|id| id.value.as_ref()) else {
            return Err(TestScriptError::ExecutionError(format!(
                "Transaction operation requires sourceId at '{}'.",
                path
            )));
        };

        let source = state.resolve_fixture(source_id)?;
        let resource = (source as &dyn Any)
            .downcast_ref::<Resource>()
            .cloned()
            .ok_or_else(|| {
                TestScriptError::ExecutionError(format!(
                    "Target fixture '{}' is not a Resource.",
                    source_id
                ))
            })?;

        match resource {
            Resource::Bundle(bundle) => {
                if !matches!(bundle.type_.as_ref(), BundleType::Transaction(_)) {
                    return Err(TestScriptError::ExecutionError(format!(
                        "Fixture must be a transaction bundle for transaction operations for sourceId '{}'.",
                        source_id
                    )));
                }

                Ok(FHIRRequest::Transaction(FHIRTransactionRequest {
                    resource: bundle,
                }))
            }

            _ => Err(TestScriptError::ExecutionError(format!(
                "Fixture '{}' is not a transaction Bundle resource.",
                source_id
            ))),
        }
    } else if operation_type == (&TestscriptOperationCodes::Create(None)).into() {
        let Some(source_id) = operation.sourceId.as_ref().and_then(|id| id.value.as_ref()) else {
            return Err(TestScriptError::ExecutionError(format!(
                "Create operation requires sourceId at '{}'.",
                path
            )));
        };

        let source = state.resolve_fixture(source_id)?;
        let resource = (source as &dyn Any)
            .downcast_ref::<Resource>()
            .cloned()
            .ok_or_else(|| {
                TestScriptError::ExecutionError(format!(
                    "Target fixture '{}' is not a Resource.",
                    source_id
                ))
            })?;

        Ok(FHIRRequest::Create(FHIRCreateRequest {
            resource_type: derive_resource_type(operation, Some(source), path)?,
            resource: resource,
        }))
    } else if operation_type == (&TestscriptOperationCodes::Delete(None)).into() {
        let Some(target_id) = operation.targetId.as_ref().and_then(|id| id.value.as_ref()) else {
            return Err(TestScriptError::ExecutionError(format!(
                "Delete operation requires targetId at '{}'.",
                path
            )));
        };

        let target = state.resolve_fixture(target_id)?;

        Ok(FHIRRequest::Delete(DeleteRequest::Instance(
            FHIRDeleteInstanceRequest {
                resource_type: derive_resource_type(operation, Some(target), path)?,
                id: target
                    .get_field("id")
                    .ok_or_else(|| {
                        TestScriptError::ExecutionError(format!(
                            "Target fixture '{}' does not have an 'id' field.",
                            target_id
                        ))
                    })?
                    .as_any()
                    .downcast_ref::<String>()
                    .cloned()
                    .unwrap_or_default(),
            },
        )))
    } else if operation_type == (&TestscriptOperationCodes::DeleteCondMultiple(None)).into() {
        let delete_parameters = ParsedParameters::try_from(
            operation
                .params
                .as_ref()
                .and_then(|p| p.value.as_ref())
                .cloned()
                .unwrap_or("".to_string())
                .as_str(),
        )
        .map_err(|e| {
            TestScriptError::ExecutionError(format!(
                "Failed to parse parameters for DeleteCondMultiple operation at '{}': {}",
                path, e
            ))
        })?;
        if operation.resource.is_some() {
            Ok(FHIRRequest::Delete(DeleteRequest::Type(
                FHIRDeleteTypeRequest {
                    resource_type: derive_resource_type(operation, None, path)?,
                    parameters: delete_parameters,
                },
            )))
        } else {
            Ok(FHIRRequest::Delete(DeleteRequest::System(
                FHIRDeleteSystemRequest {
                    parameters: delete_parameters,
                },
            )))
        }
    } else {
        Err(TestScriptError::ExecutionError(format!(
            "Unsupported TestScript operation type: {:?} at '{}'.",
            operation_type, path
        )))
    }
}

async fn run_operation<CTX, Client: FHIRClient<CTX, OperationOutcomeError>>(
    client: &Client,
    ctx: CTX,
    state: Arc<Mutex<TestState>>,
    pointer: Pointer<TestScript, TestScriptSetupActionOperation>,
    options: Arc<TestRunnerOptions>,
) -> Result<TestResult<TestReportSetupActionOperation>, TestScriptError> {
    let operation = pointer.value().ok_or_else(|| {
        TestScriptError::ExecutionError(format!(
            "Failed to retrieve TestScript operation at '{}'.",
            pointer.path()
        ))
    })?;

    let mut state_guard = state.lock().await;
    let fhir_request =
        testscript_operation_to_fhir_request(&state_guard, operation, pointer.path())?;
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
                fhir_response,
            );

            drop(state_guard);

            Ok(TestResult {
                state: state.clone(),
                value: TestReportSetupActionOperation {
                    result: Box::new(ReportActionResultCodes::Pass(None)),
                    ..Default::default()
                },
            })
        }
        Err(op_error) => {
            tracing::error!("Operation at '{}' failed: {}", pointer.path(), op_error);

            Ok(TestResult {
                state: state.clone(),
                value: TestReportSetupActionOperation {
                    result: Box::new(ReportActionResultCodes::Fail(None)),
                    message: Some(Box::new(FHIRMarkdown {
                        value: Some(format!("Operation failed: {}", op_error)),
                        ..Default::default()
                    })),
                    ..Default::default()
                },
            })
        }
    }
}

static DEFAULT_DIRECTION: LazyLock<Box<AssertDirectionCodes>> =
    LazyLock::new(|| Box::new(AssertDirectionCodes::Response(None)));

async fn get_source<'a>(
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
            .unwrap_or(&DEFAULT_DIRECTION)
            .as_ref()
        {
            AssertDirectionCodes::Request(_) => {
                if let Some(request) = state.latest_request.as_ref() {
                    request_to_meta_value(request)
                        .ok_or_else(|| TestScriptError::InvalidFixture)
                        .map(Some)
                } else {
                    Ok(None)
                }
            }
            AssertDirectionCodes::Response(_) => {
                if let Some(response) = state.latest_response.as_ref() {
                    response_to_meta_value(response)
                        .ok_or_else(|| TestScriptError::InvalidFixture)
                        .map(Some)
                } else {
                    Ok(None)
                }
            }
            AssertDirectionCodes::Null(_) => Err(TestScriptError::ExecutionError(
                "Assert direction cannot be 'null' when sourceId is not provided.".to_string(),
            )),
        }
    }
}

fn evaluate_operator(
    operator: &Box<AssertOperatorCodes>,
    a: &Vec<conversion::ConvertedValue>,
    b: &Vec<conversion::ConvertedValue>,
) -> bool {
    match operator.as_ref() {
        AssertOperatorCodes::Equals(_) | AssertOperatorCodes::Null(_) => a == b,

        AssertOperatorCodes::Contains(_) => {
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
        AssertOperatorCodes::Empty(_) => todo!("Empty operator not implemented"),
        AssertOperatorCodes::Eval(_) => todo!("Eval operator not implemented"),
        AssertOperatorCodes::GreaterThan(_) => todo!("GreaterThan operator not implemented"),
        AssertOperatorCodes::In(_) => todo!("In operator not implemented"),
        AssertOperatorCodes::LessThan(_) => todo!("LessThan operator not implemented"),
        AssertOperatorCodes::NotContains(_) => todo!("NotContains operator not implemented"),
        AssertOperatorCodes::NotEmpty(_) => todo!("NotEmpty operator not implemented"),
        AssertOperatorCodes::NotEquals(_) => todo!("NotEquals operator not implemented"),
        AssertOperatorCodes::NotIn(_) => todo!("NotIn operator not implemented"),
    }
    // a == b
}

static DEFAULT_EQUAL_OPERATOR: LazyLock<Box<AssertOperatorCodes>> =
    LazyLock::new(|| Box::new(AssertOperatorCodes::Equals(None)));

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
                    "FHIRPath evaluation error for comparison fixture '{}': {}",
                    comparision_fixture_id, e
                ))
            })?;

        result
            .iter()
            .map(|d| {
                conversion::convert_meta_value(d).ok_or_else(|| {
                    TestScriptError::ExecutionError(
                        "Failed to convert comparison fixture value.".to_string(),
                    )
                })
            })
            .collect::<Result<Vec<_>, TestScriptError>>()
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

/// Assertions are what determine the testreports ultimate pass/fail status.
/// So set that within state here depending on assertion success/failure.
async fn run_assertion(
    state: Arc<Mutex<TestState>>,
    pointer: Pointer<TestScript, TestScriptSetupActionAssert>,
) -> Result<TestResult<TestReportSetupActionAssert>, TestScriptError> {
    let assertion = pointer.value().ok_or_else(|| {
        TestScriptError::ExecutionError(format!(
            "Failed to retrieve TestScript assertion at '{}'.",
            pointer.path()
        ))
    })?;

    let mut state_guard = state.lock().await;

    let Some(source) = get_source(&*state_guard, assertion).await? else {
        return Err(TestScriptError::ExecutionError(format!(
            "Failed to resolve source for assertion at '{}'.",
            pointer.path()
        )));
    };

    let operator = assertion
        .operator
        .as_ref()
        .unwrap_or(&*DEFAULT_EQUAL_OPERATOR);

    if assertion.resource.is_some() {
        let resource_string = assertion
            .resource
            .as_ref()
            .and_then(|r| {
                let string_type: Option<String> = r.as_ref().into();
                string_type
            })
            .unwrap_or("".to_string());

        let operation_evaluation_result = evaluate_operator(
            operator,
            &vec![conversion::ConvertedValue::String(resource_string)],
            &vec![conversion::ConvertedValue::String(
                source.typename().to_string(),
            )],
        );
        if !operation_evaluation_result {
            tracing::error!(
                "Assertion at '{}' failed: resource type does not match.",
                pointer.path()
            );

            state_guard.result = ReportResultCodes::Fail(None);
            return Ok(TestResult {
                state: state.clone(),
                value: TestReportSetupActionAssert {
                    result: Box::new(ReportActionResultCodes::Fail(None)),
                    ..Default::default()
                },
            });
        }
    }
    if let Some(expression) = assertion.expression.as_ref().and_then(|e| e.value.as_ref()) {
        let comparison_to = derive_comparison_to(&state_guard, assertion).await;

        let Ok(result) = state_guard
            .fp_engine
            .evaluate(expression, vec![source])
            .await
        else {
            tracing::error!(
                "Assertion at '{}' failed: FHIRPath evaluation error.",
                pointer.path()
            );

            state_guard.result = ReportResultCodes::Fail(None);
            return Err(TestScriptError::ExecutionError(format!(
                "FHIRPath failed to evaluate at '{}' error.",
                pointer.path()
            )));
        };

        let converted_values = result
            .iter()
            .filter_map(|v| conversion::convert_meta_value(v))
            .collect::<Vec<_>>();

        let operation_evaluation_result =
            evaluate_operator(operator, &converted_values, &comparison_to?);

        if !operation_evaluation_result {
            tracing::error!(
                "Assertion at '{}' failed: operator evaluation failed.",
                pointer.path()
            );

            state_guard.result = ReportResultCodes::Fail(None);
            return Ok(TestResult {
                state: state.clone(),
                value: TestReportSetupActionAssert {
                    result: Box::new(ReportActionResultCodes::Fail(None)),
                    ..Default::default()
                },
            });
        }
    }

    return Ok(TestResult {
        state: state.clone(),
        value: TestReportSetupActionAssert {
            result: Box::new(ReportActionResultCodes::Pass(None)),
            ..Default::default()
        },
    });
}

async fn run_action<CTX, Client: FHIRClient<CTX, OperationOutcomeError>>(
    client: &Client,
    ctx: CTX,
    state: Arc<Mutex<TestState>>,
    pointer: Pointer<TestScript, TestScriptTestAction>,
    options: Arc<TestRunnerOptions>,
) -> Result<TestResult<TestReportSetupAction>, TestScriptError> {
    tracing::info!("Running TestScript action at path: {}", pointer.path());
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

async fn run_setup_action<CTX, Client: FHIRClient<CTX, OperationOutcomeError>>(
    client: &Client,
    ctx: CTX,
    state: Arc<Mutex<TestState>>,
    pointer: Pointer<TestScript, TestScriptSetupAction>,
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
    pointer: Pointer<TestScript, TestScript>,
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

    for fixture in fixtures.iter() {
        if let Some(reference_string) = fixture
            .resource
            .as_ref()
            .and_then(|r| r.reference.as_ref())
            .and_then(|refe| refe.value.as_ref())
        {
            let resolved_resource = if reference_string.starts_with('#')
                && let Some(contained) =
                    pointer.descend::<Vec<Box<Resource>>>(&Key::Field("contained".to_string()))
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
                        IssueType::NotFound(None),
                        format!("Contained resource with id '{}' not found.", local_id),
                    ));
                };

                resource.as_ref().clone()
            } else {
                let parts = reference_string.split("/").collect::<Vec<&str>>();
                if parts.len() != 2 {
                    return Err(OperationOutcomeError::error(
                        IssueType::Invalid(None),
                        format!("Invalid fixture reference: {}", reference_string),
                    ));
                }

                let resource_type = parts[0];
                let id = parts[1];

                let Some(remote_resource) = client
                    .read(
                        ctx.clone(),
                        ResourceType::try_from(resource_type).map_err(|_| {
                            OperationOutcomeError::error(
                                IssueType::Invalid(None),
                                format!(
                                    "Invalid resource type in fixture reference: '{}'",
                                    resource_type
                                ),
                            )
                        })?,
                        id.to_string(),
                    )
                    .await?
                else {
                    return Err(OperationOutcomeError::error(
                        IssueType::NotFound(None),
                        format!("Resource '{}' with id '{}' not found.", resource_type, id),
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
    pointer: Pointer<TestScript, TestScriptSetup>,
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
    pointer: Pointer<TestScript, TestScriptTeardown>,
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
    pointer: Pointer<TestScript, TestScriptTest>,
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
    pointer: Pointer<TestScript, Vec<TestScriptTest>>,
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

pub async fn run<CTX: Clone, Client: FHIRClient<CTX, OperationOutcomeError>>(
    client: &Client,
    ctx: CTX,
    test_script: Arc<TestScript>,
    options: Arc<TestRunnerOptions>,
) -> Result<TestReport, TestScriptError> {
    // Placeholder implementation
    tracing::info!("Running TestScript Runner with FHIR Client");

    let mut test_report = TestReport {
        status: Box::new(ReportStatusCodes::Completed(None)),
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
    let pointer = Pointer::<TestScript, TestScript>::new(test_script);

    state = setup_fixtures(client, ctx.clone(), state, pointer.clone(), options.clone())
        .await
        .map_err(|e| TestScriptError::OperationError(e))?;

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
        ReportResultCodes::Pending(_) => {
            test_report.result = Box::new(ReportResultCodes::Pass(None))
        }
        status => test_report.result = Box::new(status.clone()),
    }

    Ok(test_report)
}
