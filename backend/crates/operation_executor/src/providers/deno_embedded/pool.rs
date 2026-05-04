use crate::{PluginCodeType, run_code};
use deno_core::{error::AnyError, serde_json};
use haste_fhir_client::FHIRClient;
use haste_fhir_operation_error::OperationOutcomeError;
use std::io;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc;
use std::thread::{self, JoinHandle};
use tokio::runtime::Runtime;
use tokio::sync::oneshot;

type JobResult = Result<Option<serde_json::Value>, AnyError>;

pub struct DenoPool {
    next_worker: AtomicUsize,
    workers: Vec<WorkerHandle>,
}

impl DenoPool {
    pub fn new(thread_count: usize) -> Result<Self, AnyError> {
        if thread_count == 0 {
            return Err(io::Error::other("DenoPool requires at least one worker thread").into());
        }

        let mut workers = Vec::with_capacity(thread_count);

        for index in 0..thread_count {
            let result = spawn_worker(index);

            match result {
                Ok(worker) => workers.push(worker),
                Err(error) => {
                    shutdown_workers(&mut workers);
                    return Err(error);
                }
            }
        }

        Ok(Self {
            next_worker: AtomicUsize::new(0),
            workers,
        })
    }

    pub async fn execute<
        CTX: Clone + Send + 'static,
        Client: FHIRClient<CTX, OperationOutcomeError> + 'static,
    >(
        &self,
        ctx: CTX,
        client: Arc<Client>,
        media_type: PluginCodeType,
        code: impl Into<String>,
    ) -> JobResult {
        let worker_index = self.next_worker.fetch_add(1, Ordering::Relaxed) % self.workers.len();
        let worker = &self.workers[worker_index];
        let (response_tx, response_rx) = oneshot::channel();
        let code = code.into();

        let task = Box::new(move |runtime: &Runtime| {
            let result = runtime.block_on(async move {
                let output = run_code(ctx, client, media_type, code).await?;

                output
                    .map(serde_json::from_value)
                    .transpose()
                    .map_err(AnyError::from)
            });

            let _ = response_tx.send(result);
        }) as Box<dyn WorkerTask>;

        worker
            .command_tx
            .send(WorkerCommand::Run(task))
            .map_err(|_| io::Error::other("DenoPool worker is not accepting jobs"))?;

        response_rx
            .await
            .map_err(|_| io::Error::other("DenoPool worker dropped the response channel"))?
    }
}

impl Drop for DenoPool {
    fn drop(&mut self) {
        shutdown_workers(&mut self.workers);
    }
}

struct WorkerHandle {
    command_tx: mpsc::Sender<WorkerCommand>,
    join_handle: Option<JoinHandle<()>>,
}

enum WorkerCommand {
    Run(Box<dyn WorkerTask>),
    Shutdown,
}

trait WorkerTask: Send + 'static {
    fn run(self: Box<Self>, runtime: &Runtime);
}

impl<Function> WorkerTask for Function
where
    Function: FnOnce(&Runtime) + Send + 'static,
{
    fn run(self: Box<Self>, runtime: &Runtime) {
        (*self)(runtime);
    }
}

fn spawn_worker(index: usize) -> Result<WorkerHandle, AnyError> {
    let (command_tx, command_rx) = mpsc::channel();
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

            while let Ok(command) = command_rx.recv() {
                match command {
                    WorkerCommand::Run(task) => task.run(&runtime),
                    WorkerCommand::Shutdown => break,
                }
            }
        })?;

    startup_rx
        .recv()
        .map_err(|_| io::Error::other("DenoPool worker failed during startup"))??;

    Ok(WorkerHandle {
        command_tx,
        join_handle: Some(join_handle),
    })
}

fn shutdown_workers(workers: &mut [WorkerHandle]) {
    for worker in workers.iter() {
        let _ = worker.command_tx.send(WorkerCommand::Shutdown);
    }

    for worker in workers.iter_mut() {
        if let Some(join_handle) = worker.join_handle.take() {
            let _ = join_handle.join();
        }
    }
}
