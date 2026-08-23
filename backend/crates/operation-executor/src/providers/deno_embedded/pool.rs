use crate::providers::deno_embedded::{EXECUTION_TIMEOUT, build_deno_runtime, run_code};
use crate::structs::PluginCodeType;
use crate::traits::OperationExecutor;
use crate::validate::validate_parameters;
use crate::{CUSTOM_CODE_EXTENSION_URL, extract_code_from_operation_definition};
use crossbeam_channel::{Receiver, Sender};
use deno_core::serde_json::json;
use deno_core::{error::AnyError, serde_json};
use haste_fhir_client::FHIRClient;
use haste_fhir_client::request::InvocationRequest;
use haste_fhir_model::r4::generated::resources::{OperationDefinition, Parameters};
use haste_fhir_model::r4::generated::terminology::{IssueType, OperationParameterUse};
use haste_fhir_operation_error::OperationOutcomeError;
use std::collections::VecDeque;
use std::io;
use std::sync::Arc;
use std::sync::mpsc;
use std::thread::{self, JoinHandle};
use tokio::runtime::Runtime;
use tokio::sync::oneshot;

type JobResult = Result<Option<serde_json::Value>, AnyError>;

/// How many never-yet-used isolates each worker keeps ready to hand out
/// immediately, instead of building one synchronously on the request path.
///
/// Jobs run strictly one at a time per worker, so a spare of 1 is enough to
/// fully hide isolate-creation latency: each job consumes the spare left
/// over from the previous job, then -- *after* its own response has already
/// been sent -- builds a fresh replacement for whichever job comes next.
/// This does not change how many isolates a call ever gets (still exactly
/// one, used once); it only moves *when* that isolate's construction cost
/// is paid, off the critical path of the request it doesn't belong to.
const WARM_RUNTIME_BUFFER_SIZE: usize = 1;

/// Hard ceiling on the size of a single custom operation's source code.
const MAX_CUSTOM_OPERATION_CODE_BYTES: usize = 256 * 1024;

/// How many jobs may sit queued (in addition to however many are already
/// running) before a `DenoPool` starts rejecting new work instead of
/// accepting it.
const QUEUE_DEPTH_MULTIPLIER: usize = 4;

pub struct DenoPool {
    command_tx: Sender<WorkerCommand>,
    workers: Vec<JoinHandle<()>>,
    max_queue_depth: usize,
}

impl DenoPool {
    /// Creates a new [`DenoPool`] with the specified number of worker threads.
    ///
    /// Each worker is spawned during construction. If any worker fails to start, all workers
    /// that were successfully spawned up to that point are shut down before the error is
    /// returned.
    ///
    /// All workers pull jobs from a single shared queue, so an idle worker always picks up
    /// the next job regardless of which worker happens to be busy -- unlike a fixed
    /// round-robin assignment, a single slow or wedged script can't head-of-line-block jobs
    /// that land on "its" worker while other workers sit idle.
    ///
    /// # Arguments
    ///
    /// * `thread_count` - The number of worker threads to create. Must be greater than zero.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    ///
    /// * `thread_count` is zero.
    /// * A worker thread fails to spawn.
    ///
    /// If worker creation fails partway through initialization, all previously spawned workers
    /// are shut down before the error is returned.
    pub fn new(thread_count: usize) -> Result<Self, AnyError> {
        if thread_count == 0 {
            return Err(io::Error::other("DenoPool requires at least one worker thread").into());
        }

        let (command_tx, command_rx) = crossbeam_channel::unbounded();
        let mut workers = Vec::with_capacity(thread_count);

        for index in 0..thread_count {
            let result = spawn_worker(index, command_rx.clone());

            match result {
                Ok(worker) => workers.push(worker),
                Err(error) => {
                    shutdown_workers(&command_tx, &mut workers);
                    return Err(error);
                }
            }
        }

        Ok(Self {
            command_tx,
            workers,
            max_queue_depth: thread_count * QUEUE_DEPTH_MULTIPLIER,
        })
    }

    async fn execute<
        CTX: Clone + Send + 'static,
        Client: FHIRClient<CTX, OperationOutcomeError> + 'static,
    >(
        &self,
        ctx: CTX,
        client: Arc<Client>,
        media_type: PluginCodeType,
        code: impl Into<String>,
        input: serde_json::Value,
    ) -> JobResult {
        let (response_tx, response_rx) = oneshot::channel();
        let code = code.into();

        let task = Box::new(
            move |runtime: &Runtime, warm_pool: &mut VecDeque<deno_core::JsRuntime>| {
                let prewarmed = warm_pool.pop_front();

                let result = runtime.block_on(async move {
                    let deno_runtime = prewarmed.unwrap_or_else(build_deno_runtime::<CTX, Client>);

                    let output = run_code(
                        deno_runtime,
                        ctx,
                        client,
                        media_type,
                        &code,
                        input,
                        EXECUTION_TIMEOUT,
                    )
                    .await?;

                    output
                        .map(serde_json::from_value)
                        .transpose()
                        .map_err(AnyError::from)
                });

                let _ = response_tx.send(result);

                // Refill *after* the response has already been sent, so this
                // build only delays the worker picking up its next job it
                // never adds to the latency of the job that just finished.
                if warm_pool.len() < WARM_RUNTIME_BUFFER_SIZE {
                    warm_pool.push_back(build_deno_runtime::<CTX, Client>());
                }
            },
        ) as Box<dyn WorkerTask>;

        self.command_tx
            .send(WorkerCommand::Run(task))
            .map_err(|_| io::Error::other("DenoPool has no workers accepting jobs"))?;

        response_rx
            .await
            .map_err(|_| io::Error::other("DenoPool worker dropped the response channel"))?
    }
}

impl Drop for DenoPool {
    fn drop(&mut self) {
        shutdown_workers(&self.command_tx, &mut self.workers);
    }
}

fn get_parameters(input: &InvocationRequest) -> &Parameters {
    match input {
        InvocationRequest::Instance(instance_request) => &instance_request.parameters,
        InvocationRequest::Type(type_request) => &type_request.parameters,
        InvocationRequest::System(system_request) => &system_request.parameters,
    }
}

fn request_to_json(input: &InvocationRequest) -> Result<serde_json::Value, OperationOutcomeError> {
    let parameter_json: serde_json::Value =
        serde_json::to_value(get_parameters(input)).map_err(|_| {
            OperationOutcomeError::error(
                IssueType::invalid(),
                "Failed to convert operation input parameters to JSON value".to_string(),
            )
        })?;

    match input {
        InvocationRequest::Instance(instance_request) => Ok(json!({
            "id": &instance_request.id,
            "resource": instance_request.resource_type.as_ref(),
            "parameters": parameter_json,

        })),
        InvocationRequest::Type(type_request) => Ok(json!({
            "resource": type_request.resource_type.as_ref(),
            "parameters": parameter_json,
        })),
        InvocationRequest::System(_system_request) => Ok(json!({
            "parameters": parameter_json,
        })),
    }
}

impl OperationExecutor for DenoPool {
    async fn execute_operation<
        CTX: Clone + Send + 'static,
        Client: FHIRClient<CTX, OperationOutcomeError> + 'static,
    >(
        &self,
        context: CTX,
        client: Arc<Client>,
        operation: &OperationDefinition,
        input: &InvocationRequest,
    ) -> Result<Parameters, OperationOutcomeError> {
        validate_parameters(
            get_parameters(input),
            operation.parameter.as_deref().unwrap_or_default(),
            &OperationParameterUse::in_(),
        )?;

        if self.command_tx.len() >= self.max_queue_depth {
            return Err(OperationOutcomeError::error(
                IssueType::throttled(),
                "Too many custom operations are already queued; try again shortly".to_string(),
            ));
        }

        let (code, media_type) =
            extract_code_from_operation_definition(operation).ok_or_else(|| {
                OperationOutcomeError::error(
                    IssueType::invalid(),
                    format!(
                        "OperationDefinition missing custom code extension metadata '{CUSTOM_CODE_EXTENSION_URL}'"
                    ),
                )
            })?;

        if code.len() > MAX_CUSTOM_OPERATION_CODE_BYTES {
            return Err(OperationOutcomeError::error(
                IssueType::invalid(),
                format!(
                    "Custom operation source code exceeds the maximum allowed size of {MAX_CUSTOM_OPERATION_CODE_BYTES} bytes"
                ),
            ));
        }

        let media_type = PluginCodeType::try_from(media_type)?;

        let output = self
            .execute(
                context,
                client,
                media_type,
                code.to_string(),
                request_to_json(input)?,
            )
            .await
            .map_err(|error| {
                OperationOutcomeError::error(
                    IssueType::processing(),
                    format!("Failed to execute operation custom code: {error}"),
                )
            })?
            .ok_or_else(|| {
                OperationOutcomeError::error(
                    IssueType::processing(),
                    "Operation custom code returned no output".to_string(),
                )
            })?;

        let output = serde_json::from_value::<Parameters>(output).map_err(|error| {
            OperationOutcomeError::error(
                IssueType::invalid(),
                format!("Operation custom code returned invalid Parameters payload: {error}"),
            )
        })?;

        validate_parameters(
            &output,
            operation.parameter.as_deref().unwrap_or_default(),
            &OperationParameterUse::out(),
        )?;

        Ok(output)
    }
}

enum WorkerCommand {
    Run(Box<dyn WorkerTask>),
    Shutdown,
}

trait WorkerTask: Send + 'static {
    fn run(self: Box<Self>, runtime: &Runtime, warm_pool: &mut VecDeque<deno_core::JsRuntime>);
}

impl<Function> WorkerTask for Function
where
    Function: FnOnce(&Runtime, &mut VecDeque<deno_core::JsRuntime>) + Send + 'static,
{
    fn run(self: Box<Self>, runtime: &Runtime, warm_pool: &mut VecDeque<deno_core::JsRuntime>) {
        (*self)(runtime, warm_pool);
    }
}

fn spawn_worker(
    index: usize,
    command_rx: Receiver<WorkerCommand>,
) -> Result<JoinHandle<()>, AnyError> {
    let (startup_tx, startup_rx) = mpsc::sync_channel(1);

    let join_handle = thread::Builder::new()
        .name(format!("deno-pool-{index}"))
        .spawn(move || {
            let runtime = match tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
            {
                Ok(runtime) => {
                    let _ = startup_tx.send(Ok(()));
                    runtime
                }
                Err(error) => {
                    let _ = startup_tx.send(Err::<(), AnyError>(error.into()));
                    return;
                }
            };

            // Local to this OS thread: `deno_core::JsRuntime` wraps a V8
            // isolate that is not `Send`, so a spare can never be built on
            // one thread and handed to another -- each worker must warm its
            // own buffer.
            let mut warm_pool: VecDeque<deno_core::JsRuntime> =
                VecDeque::with_capacity(WARM_RUNTIME_BUFFER_SIZE);

            while let Ok(command) = command_rx.recv() {
                match command {
                    WorkerCommand::Run(task) => task.run(&runtime, &mut warm_pool),
                    WorkerCommand::Shutdown => break,
                }
            }
        })?;

    startup_rx
        .recv()
        .map_err(|_| io::Error::other("DenoPool worker failed during startup"))??;

    Ok(join_handle)
}

fn shutdown_workers(command_tx: &Sender<WorkerCommand>, workers: &mut Vec<JoinHandle<()>>) {
    for _ in 0..workers.len() {
        let _ = command_tx.send(WorkerCommand::Shutdown);
    }

    for join_handle in workers.drain(..) {
        let _ = join_handle.join();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::CUSTOM_CODE_TYPE_EXTENSION_URL;
    use crate::providers::deno_embedded::tests::{MockClient, TestCtx};
    use haste_fhir_client::request::{FHIRInvokeSystemRequest, Operation};
    use haste_fhir_model::r4::generated::types::{Extension, ExtensionValueTypeChoice, FHIRString};

    /// Runs several jobs back-to-back on a single-worker pool -- forcing
    /// every job after the first to consume a runtime the *previous* job's
    /// tail-end pre-warmed -- and checks each one still gets the correct,
    /// independent result. This is the property that actually matters here:
    /// pre-warming must never let one call observe another's state.
    #[tokio::test]
    async fn sequential_jobs_on_one_worker_each_get_independent_results() {
        let pool = DenoPool::new(1).expect("pool should start");

        for i in 0..5 {
            let result = pool
                .execute(
                    TestCtx,
                    Arc::new(MockClient),
                    PluginCodeType::JavaScript,
                    format!("export default async function () {{ return {{ n: {i} }}; }}"),
                    json!({}),
                )
                .await
                .expect("job should succeed")
                .expect("job should return a value");

            assert_eq!(result, json!({ "n": i }));
        }
    }

    fn operation_definition_with_code(code: &str) -> OperationDefinition {
        let type_extension = Extension {
            url: CUSTOM_CODE_TYPE_EXTENSION_URL.to_string(),
            value: Some(ExtensionValueTypeChoice::String(Box::new(FHIRString {
                value: Some("javascript".to_string()),
                ..Default::default()
            }))),
            ..Default::default()
        };

        let code_extension = Extension {
            url: CUSTOM_CODE_EXTENSION_URL.to_string(),
            value: Some(ExtensionValueTypeChoice::String(Box::new(FHIRString {
                value: Some(code.to_string()),
                ..Default::default()
            }))),
            extension: Some(vec![type_extension]),
            ..Default::default()
        };

        OperationDefinition {
            extension: Some(vec![code_extension]),
            ..Default::default()
        }
    }

    #[tokio::test]
    async fn oversized_custom_operation_code_is_rejected() {
        let pool = DenoPool::new(1).expect("pool should start");
        let oversized_code = "a".repeat(MAX_CUSTOM_OPERATION_CODE_BYTES + 1);
        let operation = operation_definition_with_code(&oversized_code);
        let request = InvocationRequest::System(FHIRInvokeSystemRequest {
            operation: Operation::new("test-op"),
            parameters: Parameters::default(),
        });

        let result = pool
            .execute_operation(TestCtx, Arc::new(MockClient), &operation, &request)
            .await;

        assert!(
            result.is_err(),
            "oversized custom operation code must be rejected before execution"
        );
    }
}
