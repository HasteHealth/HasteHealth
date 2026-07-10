use crate::CLIState;
use clap::Subcommand;
use haste_fhir_client::FHIRClient;
use haste_fhir_converter::Input;
use haste_fhir_model::r4::generated::resources::Resource;
use haste_fhir_model::r4::generated::terminology::{BundleType, IssueType};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_hl7v2::mllp::MllpFormatter;
use std::collections::HashMap;
use std::io::Write;
use std::net::TcpListener;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Subcommand)]
pub(crate) enum HL7v2Commands {
    Receiver {
        #[arg(short, long)]
        address: String,
        #[arg(short, long)]
        port: u16,
        #[arg(short, long)]
        main: String,
        #[arg(short, long)]
        template_dir: String,
    },
    Sender {
        #[arg(short, long)]
        address: String,
        #[arg(short, long)]
        port: u16,
    },
}

pub(crate) async fn hl7v2(
    state: Arc<Mutex<CLIState>>,
    command: &HL7v2Commands,
) -> Result<(), OperationOutcomeError> {
    let fhir_client = crate::client::fhir_client(state).await?;

    match command {
        HL7v2Commands::Receiver {
            address,
            port,
            main,
            template_dir,
        } => {
            let listener = TcpListener::bind(format!("{}:{}", address, port)).unwrap();

            let environment = haste_fhir_converter::create_environment(Some(template_dir));

            let template = environment.get_template(&main).map_err(|e| {
                OperationOutcomeError::error(IssueType::Exception(None), e.to_string())
            })?;

            for stream in listener.incoming() {
                let mut stream = match stream {
                    Ok(s) => s,
                    Err(e) => {
                        eprintln!("Failed to accept connection: {}", e);
                        continue;
                    }
                };

                loop {
                    let frame = match MllpFormatter::read_frame(&mut stream) {
                        Ok(f) => f,
                        Err(e) => {
                            eprintln!("Connection ended: {}", e);
                            break;
                        }
                    };

                    let start = std::time::Instant::now();

                    let hl7v2_bytes = match MllpFormatter::decode(frame.as_slice()) {
                        Ok(b) => b.to_vec(),
                        Err(e) => {
                            eprintln!("Failed to decode MLLP frame: {}", e);
                            let _ = stream.write_all(&MllpFormatter::nak());
                            continue;
                        }
                    };

                    let hl7v2_string = String::from_utf8_lossy(&hl7v2_bytes).to_string();

                    let hl7v2 = haste_fhir_converter::convert_input(Input::HL7V2(hl7v2_string))?;

                    let mut ctx = HashMap::new();
                    ctx.insert("hl7v2", hl7v2);

                    let haste_fhir_converter::Output::FHIR(resource) =
                        haste_fhir_converter::transform(
                            &template,
                            ctx,
                            &haste_fhir_converter::OutputFormat::FHIR,
                        )?
                    else {
                        eprintln!("Unexpected output format from template");
                        let _ = stream.write_all(&MllpFormatter::nak());
                        continue;
                    };

                    tracing::info!("total transformation: {:?}", start.elapsed());

                    match resource {
                        Resource::Bundle(bundle) => match bundle.type_.as_ref() {
                            BundleType::Batch(_) => match fhir_client.batch((), bundle).await {
                                Ok(_) => {
                                    if let Err(e) = stream.write_all(&MllpFormatter::ack()) {
                                        eprintln!("Failed to send ACK: {}", e);
                                        break;
                                    }
                                }
                                Err(e) => {
                                    eprintln!("Failed to send batch {}", e);
                                    if let Err(e) = stream.write_all(&MllpFormatter::nak()) {
                                        eprintln!("Failed to send NAK: {}", e);
                                        break;
                                    }
                                }
                            },
                            BundleType::Transaction(_) => {
                                match fhir_client.transaction((), bundle).await {
                                    Ok(_) => {
                                        if let Err(e) = stream.write_all(&MllpFormatter::ack()) {
                                            eprintln!("Failed to send ACK: {}", e);
                                            break;
                                        }
                                    }
                                    Err(e) => {
                                        eprintln!("Failed to send transaction {}", e);
                                        if let Err(e) = stream.write_all(&MllpFormatter::nak()) {
                                            eprintln!("Failed to send NAK: {}", e);
                                            break;
                                        }
                                    }
                                }
                            }
                            _ => {
                                eprintln!("Unsupported Bundle type: {:?}", bundle.type_);
                                let _ = stream.write_all(&MllpFormatter::nak());
                                continue;
                            }
                        },
                        _ => {
                            let resource_type = resource.resource_type();
                            match fhir_client.create((), resource_type, resource).await {
                                Ok(_) => {
                                    if let Err(e) = stream.write_all(&MllpFormatter::ack()) {
                                        eprintln!("Failed to send ACK: {}", e);
                                        break;
                                    }
                                }
                                Err(e) => {
                                    eprintln!("Failed to send resource {}", e);
                                    if let Err(e) = stream.write_all(&MllpFormatter::nak()) {
                                        eprintln!("Failed to send NAK: {}", e);
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }
            }

            Ok(())
        }
        HL7v2Commands::Sender {
            address: _,
            port: _,
        } => {
            todo!("HL7v2 sender not implemented yet");
        }
    }
}
