use haste_fhir_client::{
    FHIRClient,
    request::{
        FHIRReadRequest, FHIRRequest, FHIRResponse, HistoryResponse, InvokeResponse,
        SearchResponse, UpdateRequest,
    },
};
use haste_fhir_model::r4::generated::{
    resources::{
        Resource, ResourceType, TestReport, TestReportSetup, TestReportSetupAction, TestReportTest,
        TestReportTestAction, TestScript, TestScriptFixture, TestScriptSetup,
        TestScriptSetupActionAssert, TestScriptSetupActionOperation, TestScriptTest,
        TestScriptTestAction,
    },
    terminology::{IssueType, ReportResultCodes, TestscriptOperationCodes},
    types::{FHIRString, Reference},
};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_pointer::{Key, Pointer};
use haste_reflect::MetaValue;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;
use tracing::info;

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
#[derive(Debug)]
struct TestState {
    #[allow(dead_code)]
    result: ReportResultCodes,
    fixtures: HashMap<String, Fixtures>,
    latest_request: Option<FHIRRequest>,
    latest_response: Option<FHIRResponse>,
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

impl TestState {
    fn new() -> Self {
        TestState {
            result: ReportResultCodes::Pending(None),
            fixtures: HashMap::new(),
            latest_request: None,
            latest_response: None,
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
            .insert(request_var.clone(), Fixtures::Request(request));
    }

    if let Some(response_var) = operation
        .responseId
        .as_ref()
        .and_then(|id| id.value.as_ref())
    {
        // Associate response variable in state
        state
            .fixtures
            .insert(response_var.clone(), Fixtures::Response(response));
    }
}

fn testscript_operation_to_fhir_request(
    state: &TestState,
    operation: &TestScriptSetupActionOperation,
) -> Result<FHIRRequest, TestScriptError> {
    let operation_type = operation
        .type_
        .as_ref()
        .and_then(|t| t.code.as_ref())
        .and_then(|c| c.value.clone());

    if operation_type == (&TestscriptOperationCodes::Read(None)).into() {
        let Some(target_id) = operation.targetId.as_ref().and_then(|id| id.value.as_ref()) else {
            return Err(TestScriptError::ExecutionError(
                "Read operation requires targetId at.".to_string(),
            ));
        };

        let target = state.resolve_fixture(target_id)?;

        Ok(FHIRRequest::Read(FHIRReadRequest {
            resource_type: if let Some(operation_resource_type) = operation.resource.as_ref() {
                let string_type: Option<String> = operation_resource_type.as_ref().into();
                ResourceType::try_from(string_type.unwrap_or_default()).map_err(|_| {
                    TestScriptError::ExecutionError(format!(
                        "Unsupported resource type '{:?}' for Read operation.",
                        operation_resource_type.as_ref()
                    ))
                })?
            } else {
                let target_resource_type =
                    ResourceType::try_from(target.typename()).map_err(|_| {
                        TestScriptError::ExecutionError(format!(
                            "Unsupported resource type '{}' for Read operation.",
                            target.typename()
                        ))
                    })?;
                target_resource_type
            },
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
    } else if operation_type == (&TestscriptOperationCodes::Create(None)).into() {
        // Handle Create operation
        todo!("Handle Create operation");
    } else {
        Err(TestScriptError::ExecutionError(format!(
            "Unsupported TestScript operation type: {:?}",
            operation_type
        )))
    }
}

async fn run_operation<CTX, Client: FHIRClient<CTX, OperationOutcomeError>>(
    client: &Client,
    ctx: CTX,
    state: Arc<Mutex<TestState>>,
    pointer: Pointer<TestScript, TestScriptSetupActionOperation>,
) -> Result<Arc<Mutex<TestState>>, TestScriptError> {
    let operation = pointer.value().ok_or_else(|| {
        TestScriptError::ExecutionError(format!(
            "Failed to retrieve TestScript operation at '{}'.",
            pointer.path()
        ))
    })?;

    let mut state_guard = state.lock().await;
    let fhir_request = testscript_operation_to_fhir_request(&state_guard, operation)?;

    let fhir_response = client
        .request(ctx, fhir_request.clone())
        .await
        .map_err(|e| TestScriptError::OperationError(e))?;

    associate_request_response_variables(&mut state_guard, operation, fhir_request, fhir_response);

    drop(state_guard);

    Ok(state)
}

async fn get_source<'a>(
    state: &'a TestState,
    assertion: &TestScriptSetupActionAssert,
) -> Result<Option<&'a dyn MetaValue>, TestScriptError> {
    if let Some(source_id) = assertion.sourceId.as_ref().and_then(|id| id.value.as_ref()) {
        let source = state.resolve_fixture(source_id)?;
        Ok(Some(source))
    } else if let Some(direction) = assertion.direction.as_ref() {
        match direction.as_ref() {
            haste_fhir_model::r4::generated::terminology::AssertDirectionCodes::Request(_) => {
                if let Some(request) = state.latest_request.as_ref() {
                    request_to_meta_value(request)
                        .ok_or_else(|| TestScriptError::InvalidFixture)
                        .map(Some)
                } else {
                    Ok(None)
                }
            }
            haste_fhir_model::r4::generated::terminology::AssertDirectionCodes::Response(_) => {
                if let Some(request) = state.latest_response.as_ref() {
                    response_to_meta_value(request)
                        .ok_or_else(|| TestScriptError::InvalidFixture)
                        .map(Some)
                } else {
                    Ok(None)
                }
            }
            haste_fhir_model::r4::generated::terminology::AssertDirectionCodes::Null(_) => {
                todo!()
            }
        }
    } else {
        todo!();
    }
}

async fn run_assertion(
    state: Arc<Mutex<TestState>>,
    pointer: Pointer<TestScript, TestScriptSetupActionAssert>,
) -> Result<Arc<Mutex<TestState>>, TestScriptError> {
    let assertion = pointer.value().ok_or_else(|| {
        TestScriptError::ExecutionError(format!(
            "Failed to retrieve TestScript assertion at '{}'.",
            pointer.path()
        ))
    })?;

    let state_guard = state.lock().await;

    let Some(_source) = get_source(&*state_guard, assertion).await? else {
        return Err(TestScriptError::ExecutionError(
            "Failed to resolve source for assertion.".to_string(),
        ));
    };

    let _engine = haste_fhirpath::FPEngine::new();
    // let result = engine.evaluate("$this", vec![source]).await;

    if assertion.resource.is_some() {}
    if assertion.expression.is_some() {}

    todo!();
}

async fn run_action<CTX, Client: FHIRClient<CTX, OperationOutcomeError>>(
    client: &Client,
    ctx: CTX,
    state: Arc<Mutex<TestState>>,
    pointer: Pointer<TestScript, TestScriptTestAction>,
) -> Result<TestResult<TestReportSetupAction>, TestScriptError> {
    let action = pointer.value().ok_or_else(|| {
        TestScriptError::ExecutionError(format!(
            "Failed to retrieve TestScript action at '{}'.",
            pointer.path()
        ))
    })?;

    info!("Running TestScript action at path: {}", pointer.path());

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

        let state = run_operation(client, ctx, state, operation_pointer).await?;

        Ok(TestResult {
            state,
            value: TestReportSetupAction {
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

        let state = run_assertion(state, assertion_pointer).await?;

        Ok(TestResult {
            state,
            value: TestReportSetupAction {
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
) -> Result<TestResult<TestReportSetup>, TestScriptError> {
    let mut cur_state = state;

    let mut setup_results = TestReportSetup {
        action: vec![],
        ..Default::default()
    };

    let Some(actions) = pointer.value() else {
        return Ok(TestResult {
            state: cur_state,
            value: setup_results,
        });
    };

    for action in actions.action.iter().enumerate() {
        let action_pointer = pointer
            .descend::<TestScriptTestAction>(&Key::Field("action".to_string()))
            .and_then(|p| p.descend(&Key::Index(action.0)));

        let action_pointer = action_pointer.ok_or_else(|| {
            TestScriptError::ExecutionError(format!(
                "Failed to retrieve TestScript action at index {}.",
                action.0
            ))
        })?;

        let result = run_action(client, ctx.clone(), cur_state, action_pointer).await?;
        cur_state = result.state;

        setup_results.action.push(result.value);
    }

    Ok(TestResult {
        state: cur_state,
        value: setup_results,
    })
}

async fn run_test<CTX: Clone, Client: FHIRClient<CTX, OperationOutcomeError>>(
    client: &Client,
    ctx: CTX,
    state: Arc<Mutex<TestState>>,
    pointer: Pointer<TestScript, TestScriptTest>,
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
            .descend::<TestScriptTestAction>(&Key::Field("action".to_string()))
            .and_then(|p| p.descend(&Key::Index(action.0)))
        else {
            return Err(TestScriptError::ExecutionError(format!(
                "Failed to retrieve TestScript action at index {}.",
                action.0
            )));
        };
        let result = run_action(client, ctx.clone(), cur_state, action_pointer).await?;
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
        let test_result = run_test(client, ctx.clone(), cur_state, test_pointer).await?;
        cur_state = test_result.state;
        test_results.push(test_result.value);
    }

    Ok(TestResult {
        state: cur_state,
        value: test_results,
    })
}

pub async fn run<CTX: Clone, Client: FHIRClient<CTX, OperationOutcomeError>>(
    client: &Client,
    ctx: CTX,
    test_script: Arc<TestScript>,
) -> Result<TestReport, TestScriptError> {
    // Placeholder implementation
    println!("Running TestScript Runner with FHIR Client");

    let mut test_report = TestReport {
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

    state = setup_fixtures(client, ctx.clone(), state, pointer.clone())
        .await
        .map_err(|e| TestScriptError::OperationError(e))?;

    // Run setup actions
    if let Some(setup_pointer) =
        pointer.descend::<TestScriptSetup>(&Key::Field("setup".to_string()))
    {
        let setup_result = run_setup(client, ctx.clone(), state, setup_pointer).await?;
        state = setup_result.state;
        test_report.setup = Some(setup_result.value);
    }

    // Run Test actions
    if let Some(test_pointer) =
        pointer.descend::<Vec<TestScriptTest>>(&Key::Field("test".to_string()))
    {
        let test_result = run_tests(client, ctx.clone(), state, test_pointer).await?;
        state = test_result.state;
        test_report.test = Some(test_result.value);
    }

    println!("State {:?}", state);

    Ok(test_report)
}
