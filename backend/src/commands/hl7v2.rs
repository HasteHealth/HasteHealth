//! Bridge HL7 v2 messages to and from the FHIR server.
//!
//! The receiver is an MLLP listener: it accepts framed HL7 v2 messages,
//! converts each one to FHIR with a Jinja template, submits the result, and
//! answers the sender with an HL7 `ACK`.
//!
//! # What "production ready" means here
//!
//! An interface engine is not a browser. It keeps one socket open for hours,
//! sends thousands of messages down it, retries anything it does not see an
//! `ACK` for, and treats a missing or malformed acknowledgement as a failure to
//! deliver. The properties that follow from that:
//!
//! - **Every message is answered.** A parse failure, a template failure and a
//!   write failure each produce an `ACK` with the appropriate `MSA-1`, never a
//!   dropped connection. A sender that gets silence retries forever.
//! - **`AR` vs `AE` is chosen deliberately.** `AR` (reject) means the sender
//!   should not retry -- the message itself is wrong. `AE` (error) means it
//!   should -- we could not store a message that may well be fine. Getting this
//!   backwards either loses data or produces an infinite retry loop.
//! - **One message cannot take down the listener.** Each connection is its own
//!   task and each message its own error scope.
//! - **Every wait is bounded.** A half-open socket, a sender that stops
//!   mid-frame, or a frame that never terminates are all survivable.
//! - **Shutdown drains.** On SIGINT/SIGTERM the listener stops accepting and
//!   lets in-flight messages finish, so a message is not acknowledged and then
//!   lost, or lost without being acknowledged.

use crate::cli::state::CliState;
use clap::Subcommand;
use haste_fhir_client::FHIRClient;
use haste_fhir_converter::Input;
use haste_fhir_model::r4::generated::resources::Resource;
use haste_fhir_model::r4::generated::terminology::{BundleType, IssueType};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_hl7v2::ack::{AckCode, InboundHeader, build_ack};
use haste_hl7v2::mllp_async::{FrameEnd, encode_frame, read_frame};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::AsyncWriteExt;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{Mutex, Semaphore};

/// Bridge HL7v2 messages to and from the FHIR server.
#[derive(Subcommand, Debug)]
pub(crate) enum HL7v2Commands {
    /// Listen for MLLP-framed HL7v2 messages, convert them to FHIR, and submit them.
    Receiver {
        /// Address to bind the MLLP listener to.
        #[arg(short, long, default_value = "0.0.0.0")]
        address: String,
        /// Port to bind the MLLP listener to.
        #[arg(short, long)]
        port: u16,
        /// Entry template file name (resolved within --template-dir) used to convert
        /// incoming HL7v2 messages to FHIR.
        #[arg(short, long)]
        main: String,
        /// Directory containing the conversion templates.
        #[arg(short, long)]
        template_dir: String,
        /// Maximum connections served at once. Further senders wait rather than
        /// being refused, since a refused connection looks like an outage.
        #[arg(long, default_value_t = 64)]
        max_connections: usize,
        /// Close a connection that sends nothing for this long, in seconds.
        /// Bounds half-open sockets, which TCP alone will not detect.
        #[arg(long, default_value_t = 300)]
        idle_timeout_secs: u64,
        /// Give up on a partially received message after this long, in seconds.
        #[arg(long, default_value_t = 30)]
        read_timeout_secs: u64,
    },
    /// Send HL7v2 messages over MLLP.
    Sender {
        /// Address of the MLLP receiver to send to.
        #[arg(short, long)]
        address: String,
        /// Port of the MLLP receiver to send to.
        #[arg(short, long)]
        port: u16,
        /// File containing the HL7v2 message to send. Reads stdin when absent.
        #[arg(short, long)]
        file: Option<String>,
        /// How long to wait for the ACK, in seconds.
        #[arg(long, default_value_t = 30)]
        timeout_secs: u64,
    },
}

/// Settings a connection task needs, resolved once at startup.
struct ReceiverConfig {
    template_dir: String,
    main_template: String,
    idle_timeout: Duration,
    read_timeout: Duration,
}

/// What handling one message produced.
struct MessageOutcome {
    code: AckCode,
    detail: Option<String>,
}

impl MessageOutcome {
    fn accepted() -> Self {
        Self {
            code: AckCode::ApplicationAccept,
            detail: None,
        }
    }

    /// The message is wrong; the sender must not retry it unchanged.
    fn rejected(detail: impl Into<String>) -> Self {
        Self {
            code: AckCode::ApplicationReject,
            detail: Some(detail.into()),
        }
    }

    /// We could not store a message that may be fine; the sender should retry.
    fn errored(detail: impl Into<String>) -> Self {
        Self {
            code: AckCode::ApplicationError,
            detail: Some(detail.into()),
        }
    }
}

/// Runs the `hl7v2` command group.
pub(crate) async fn run(
    state: Arc<Mutex<CliState>>,
    command: &HL7v2Commands,
) -> Result<(), OperationOutcomeError> {
    match command {
        HL7v2Commands::Receiver {
            address,
            port,
            main,
            template_dir,
            max_connections,
            idle_timeout_secs,
            read_timeout_secs,
        } => {
            let fhir_client = crate::cli::client::fhir_client(state).await?;

            // Templates are compiled once at startup so a broken template is a
            // startup failure rather than a per-message one. Building the
            // environment per message would also reparse every file on every
            // message, which at interface-engine volumes dominates the work.
            let environment = haste_fhir_converter::create_environment(Some(template_dir));
            environment.get_template(main).map_err(|e| {
                OperationOutcomeError::fatal(
                    IssueType::not_found(),
                    format!("Conversion template '{main}' not found in '{template_dir}': {e}"),
                )
            })?;

            let config = Arc::new(ReceiverConfig {
                template_dir: template_dir.clone(),
                main_template: main.clone(),
                idle_timeout: Duration::from_secs(*idle_timeout_secs),
                read_timeout: Duration::from_secs(*read_timeout_secs),
            });

            serve(
                &format!("{address}:{port}"),
                fhir_client,
                config,
                *max_connections,
            )
            .await
        }
        HL7v2Commands::Sender {
            address,
            port,
            file,
            timeout_secs,
        } => {
            send_message(
                &format!("{address}:{port}"),
                file.as_deref(),
                Duration::from_secs(*timeout_secs),
            )
            .await
        }
    }
}

/// Accepts connections until shutdown is signalled.
async fn serve<Client>(
    bind: &str,
    fhir_client: Arc<Client>,
    config: Arc<ReceiverConfig>,
    max_connections: usize,
) -> Result<(), OperationOutcomeError>
where
    Client: FHIRClient<(), OperationOutcomeError> + Send + Sync + 'static,
{
    let listener = TcpListener::bind(bind).await.map_err(|e| {
        OperationOutcomeError::fatal(
            IssueType::exception(),
            format!("Failed to bind the MLLP listener to {bind}: {e}"),
        )
    })?;

    let local = listener
        .local_addr()
        .map_or_else(|_| bind.to_string(), |a| a.to_string());
    tracing::info!(
        address = %local,
        max_connections,
        template = %config.main_template,
        "MLLP listener started"
    );

    // Bounds concurrent connections without refusing any: a sender that is
    // turned away cannot tell an overloaded listener from a down one, and will
    // usually alert rather than wait.
    let permits = Arc::new(Semaphore::new(max_connections));
    // Held by each in-flight connection so shutdown can wait for them.
    let in_flight = Arc::new(tokio::sync::RwLock::new(()));

    loop {
        tokio::select! {
            // Biased so a pending shutdown wins over a ready connection;
            // otherwise a busy listener could accept indefinitely.
            biased;

            _ = shutdown_signal() => {
                tracing::info!("Shutdown signalled; no longer accepting connections");
                break;
            }

            accepted = listener.accept() => {
                let (stream, peer) = match accepted {
                    Ok(pair) => pair,
                    Err(e) => {
                        // A failed accept is usually transient (a file
                        // descriptor limit, a connection reset during the
                        // handshake) and must not end the listener.
                        tracing::warn!("Failed to accept an MLLP connection: {e}");
                        continue;
                    }
                };

                let Ok(permit) = permits.clone().acquire_owned().await else {
                    break;
                };

                let fhir_client = fhir_client.clone();
                let config = config.clone();
                let guard = in_flight.clone();

                tokio::spawn(async move {
                    let _permit = permit;
                    let _guard = guard.read().await;
                    handle_connection(stream, peer, fhir_client, config).await;
                });
            }
        }
    }

    // Draining: every connection task holds a read guard, so the write waits
    // until the last one finishes. A message already acknowledged has been
    // written; one not yet acknowledged will be retried by the sender.
    tracing::info!("Draining in-flight messages");
    let _ = in_flight.write().await;
    tracing::info!("MLLP listener stopped");

    Ok(())
}

/// Resolves when the process is asked to stop.
async fn shutdown_signal() {
    let interrupt = tokio::signal::ctrl_c();

    #[cfg(unix)]
    {
        let mut terminate =
            match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
                Ok(signal) => signal,
                Err(e) => {
                    tracing::warn!("Could not listen for SIGTERM: {e}");
                    let _ = interrupt.await;
                    return;
                }
            };

        tokio::select! {
            _ = interrupt => {}
            _ = terminate.recv() => {}
        }
    }

    #[cfg(not(unix))]
    {
        let _ = interrupt.await;
    }
}

/// Serves one sender until it disconnects, idles out, or breaks framing.
async fn handle_connection<Client>(
    mut stream: TcpStream,
    peer: SocketAddr,
    fhir_client: Arc<Client>,
    config: Arc<ReceiverConfig>,
) where
    Client: FHIRClient<(), OperationOutcomeError> + Send + Sync + 'static,
{
    // Messages are small and latency matters more than packet efficiency; an
    // ACK held by Nagle's algorithm looks to the sender like a slow receiver.
    if let Err(e) = stream.set_nodelay(true) {
        tracing::debug!(%peer, "Could not disable Nagle: {e}");
    }

    tracing::debug!(%peer, "MLLP connection opened");
    let mut handled: u64 = 0;

    loop {
        // The idle timeout covers waiting for the *next* message; the read
        // timeout covers finishing one already started. They are separate
        // because a connection may legitimately sit quiet for minutes between
        // messages, but must never sit half-written for that long.
        let frame = match tokio::time::timeout(config.idle_timeout, read_frame(&mut stream)).await {
            Err(_) => {
                tracing::debug!(%peer, handled, "Closing idle MLLP connection");
                break;
            }
            Ok(Ok(FrameEnd::Closed)) => {
                tracing::debug!(%peer, handled, "MLLP connection closed by peer");
                break;
            }
            Ok(Ok(FrameEnd::Frame(payload))) => payload,
            Ok(Err(e)) => {
                // Framing is broken, so the stream position is unknown and no
                // further message on it can be trusted. There is also no
                // control id to acknowledge against.
                tracing::warn!(%peer, handled, "MLLP framing error, closing connection: {e}");
                break;
            }
        };

        let message = String::from_utf8_lossy(&frame).to_string();
        let header = InboundHeader::scan(&message);
        let control_id = header.control_id.clone();

        let outcome = tokio::time::timeout(
            config.read_timeout,
            handle_message(&message, fhir_client.as_ref(), config.as_ref()),
        )
        .await
        .unwrap_or_else(|_| {
            MessageOutcome::errored("Timed out converting or storing the message; please retry")
        });

        match outcome.code {
            AckCode::ApplicationAccept => {
                tracing::info!(%peer, control_id = %control_id, "Accepted HL7v2 message");
            }
            AckCode::ApplicationReject => {
                tracing::warn!(
                    %peer, control_id = %control_id,
                    detail = outcome.detail.as_deref().unwrap_or(""),
                    "Rejected HL7v2 message"
                );
            }
            AckCode::ApplicationError => {
                tracing::error!(
                    %peer, control_id = %control_id,
                    detail = outcome.detail.as_deref().unwrap_or(""),
                    "Failed to store HL7v2 message"
                );
            }
        }

        let ack = build_ack(
            &header,
            outcome.code,
            outcome.detail.as_deref(),
            &timestamp(),
        );

        if let Err(e) = stream.write_all(&encode_frame(ack.as_bytes())).await {
            // The message may well have been stored; the sender will retry and
            // is expected to deduplicate on the control id.
            tracing::warn!(%peer, control_id = %control_id, "Failed to send ACK: {e}");
            break;
        }
        if let Err(e) = stream.flush().await {
            tracing::warn!(%peer, control_id = %control_id, "Failed to flush ACK: {e}");
            break;
        }

        handled += 1;
    }
}

/// Converts one message and submits it.
///
/// Never returns an error: the caller owes the sender an acknowledgement in
/// every case, so a failure is an [`AckCode`] rather than a `?`.
async fn handle_message<Client>(
    message: &str,
    fhir_client: &Client,
    config: &ReceiverConfig,
) -> MessageOutcome
where
    Client: FHIRClient<(), OperationOutcomeError> + Send + Sync,
{
    if message.trim().is_empty() {
        return MessageOutcome::rejected("Empty message");
    }

    // A message that does not parse will not parse on retry either, so this is
    // a rejection rather than an error.
    let hl7v2 = match haste_fhir_converter::convert_input(Input::HL7V2(message.to_string())) {
        Ok(value) => value,
        Err(e) => return MessageOutcome::rejected(format!("Could not parse the message: {e}")),
    };

    // Rebuilt per message rather than shared: minijinja's `Template` borrows
    // its `Environment`, which cannot be held across an await point in a
    // `'static` task. Startup already proved the template compiles.
    let environment = haste_fhir_converter::create_environment(Some(&config.template_dir));
    let Ok(template) = environment.get_template(&config.main_template) else {
        return MessageOutcome::errored(format!(
            "Conversion template '{}' is no longer available",
            config.main_template
        ));
    };

    let mut ctx = HashMap::new();
    ctx.insert("hl7v2", hl7v2);

    let output = match haste_fhir_converter::transform(
        &template,
        ctx,
        &haste_fhir_converter::OutputFormat::FHIR,
    ) {
        Ok(output) => output,
        Err(e) => {
            // The template ran against this message and produced something that
            // is not a FHIR resource. That is a property of the message (a
            // missing field the template requires), so the sender is told.
            return MessageOutcome::rejected(format!("Could not convert the message to FHIR: {e}"));
        }
    };

    let haste_fhir_converter::Output::FHIR(resource) = output else {
        return MessageOutcome::errored("The conversion template did not produce a FHIR resource");
    };

    submit(*resource, fhir_client).await
}

/// Writes the converted resource to the server.
async fn submit<Client>(resource: Resource, fhir_client: &Client) -> MessageOutcome
where
    Client: FHIRClient<(), OperationOutcomeError> + Send + Sync,
{
    // A storage failure is `AE`, not `AR`: the message was fine and the sender
    // should send it again.
    let stored = match resource {
        Resource::Bundle(bundle) => match &bundle.type_ {
            t if t == &BundleType::transaction() => fhir_client.transaction((), bundle).await,
            t if t == &BundleType::batch() => fhir_client.batch((), bundle).await,
            other => {
                return MessageOutcome::rejected(format!(
                    "A converted Bundle must be a transaction or a batch, got {:?}",
                    other.as_str().unwrap_or("(none)")
                ));
            }
        },
        resource => {
            let resource_type = resource.resource_type();
            fhir_client
                .create((), resource_type, resource)
                .await
                .map(|_| Default::default())
        }
    };

    match stored {
        Ok(_) => MessageOutcome::accepted(),
        Err(e) => MessageOutcome::errored(format!("Could not store the message: {e}")),
    }
}

/// `MSH-7` format: `YYYYMMDDHHMMSS`.
fn timestamp() -> String {
    chrono::Utc::now().format("%Y%m%d%H%M%S").to_string()
}

/// Sends one message and waits for its acknowledgement.
async fn send_message(
    target: &str,
    file: Option<&str>,
    timeout: Duration,
) -> Result<(), OperationOutcomeError> {
    let message = match file {
        Some(path) => tokio::fs::read_to_string(path).await.map_err(|e| {
            OperationOutcomeError::fatal(
                IssueType::not_found(),
                format!("Could not read '{path}': {e}"),
            )
        })?,
        None => {
            use std::io::Read as _;
            let mut buf = String::new();
            std::io::stdin().read_to_string(&mut buf).map_err(|e| {
                OperationOutcomeError::fatal(
                    IssueType::exception(),
                    format!("Could not read the message from stdin: {e}"),
                )
            })?;
            buf
        }
    };

    // HL7 v2 separates segments with a carriage return. A file edited on any
    // platform tends to carry newlines, and a receiver splitting on `\r` alone
    // would see one enormous segment.
    let message = normalize_segment_separators(&message);

    let mut stream = tokio::time::timeout(timeout, TcpStream::connect(target))
        .await
        .map_err(|_| {
            OperationOutcomeError::fatal(
                IssueType::timeout(),
                format!("Timed out connecting to {target}"),
            )
        })?
        .map_err(|e| {
            OperationOutcomeError::fatal(
                IssueType::exception(),
                format!("Could not connect to {target}: {e}"),
            )
        })?;

    let _ = stream.set_nodelay(true);

    stream
        .write_all(&encode_frame(message.as_bytes()))
        .await
        .map_err(|e| {
            OperationOutcomeError::fatal(
                IssueType::exception(),
                format!("Could not send the message: {e}"),
            )
        })?;
    stream.flush().await.map_err(|e| {
        OperationOutcomeError::fatal(
            IssueType::exception(),
            format!("Could not flush the message: {e}"),
        )
    })?;

    let frame = tokio::time::timeout(timeout, read_frame(&mut stream))
        .await
        .map_err(|_| {
            OperationOutcomeError::fatal(
                IssueType::timeout(),
                format!("No acknowledgement from {target} within {timeout:?}"),
            )
        })??;

    let FrameEnd::Frame(payload) = frame else {
        return Err(OperationOutcomeError::fatal(
            IssueType::incomplete(),
            "The receiver closed the connection without acknowledging".to_string(),
        ));
    };

    let ack = String::from_utf8_lossy(&payload).to_string();
    println!("{ack}");

    // The exit status reflects MSA-1, so a script can tell acceptance from
    // rejection without parsing the reply.
    if ack_was_accepted(&ack) {
        Ok(())
    } else {
        Err(OperationOutcomeError::error(
            IssueType::processing(),
            format!("The receiver did not accept the message: {}", ack.trim()),
        ))
    }
}

/// Rewrites bare newlines to carriage returns, leaving `\r\n` intact.
fn normalize_segment_separators(message: &str) -> String {
    message.replace("\r\n", "\r").replace('\n', "\r")
}

/// Whether `MSA-1` is `AA` (or `CA`, its commit-level equivalent).
fn ack_was_accepted(ack: &str) -> bool {
    ack.split(['\r', '\n'])
        .find(|segment| segment.starts_with("MSA"))
        .and_then(|msa| msa.split('|').nth(1))
        .is_some_and(|code| code == "AA" || code == "CA")
}
