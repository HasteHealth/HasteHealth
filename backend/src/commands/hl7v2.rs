use crate::CLIState;
use clap::Subcommand;
use haste_fhir_client::FHIRClient;
use haste_fhir_model::r4::generated::resources::{Resource, ResourceType};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_hl7v2::mllp::MllpFormatter;
use haste_hl7v2::parser::ParsedHL7V2Message;
use std::io::Write;
use std::net::TcpListener;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Subcommand)]
pub enum HL7v2Commands {
    Receiver { address: String, port: u16 },
    Sender { address: String, port: u16 },
}

pub async fn hl7v2(
    state: Arc<Mutex<CLIState>>,
    command: &HL7v2Commands,
) -> Result<(), OperationOutcomeError> {
    let fhir_client = crate::client::fhir_client(state).await?;

    match command {
        HL7v2Commands::Receiver { address, port } => {
            let listener = TcpListener::bind(format!("{}:{}", address, port)).unwrap();

            for stream in listener.incoming() {
                let mut stream = match stream {
                    Ok(s) => s,
                    Err(e) => {
                        eprintln!("Failed to accept connection: {}", e);
                        continue;
                    }
                };

                println!("Received connection from {:?}", stream.peer_addr());

                loop {
                    let frame = match MllpFormatter::read_frame(&mut stream) {
                        Ok(f) => f,
                        Err(e) => {
                            eprintln!("Connection ended: {}", e);
                            break;
                        }
                    };

                    let hl7v2_bytes = match MllpFormatter::decode(frame.as_slice()) {
                        Ok(b) => b.to_vec(),
                        Err(e) => {
                            eprintln!("Failed to decode MLLP frame: {}", e);
                            let _ = stream.write_all(&MllpFormatter::nak());
                            continue;
                        }
                    };

                    let hl7v2_string = String::from_utf8_lossy(&hl7v2_bytes).to_string();

                    let parsed = match ParsedHL7V2Message::try_from(hl7v2_string.as_str()) {
                        Ok(result) => result.0,
                        Err(e) => {
                            eprintln!("Failed to parse HL7v2 message: {}", e);
                            let _ = stream.write_all(&MllpFormatter::nak());
                            continue;
                        }
                    };

                    match fhir_client
                        .create((), ResourceType::HL7V2, Resource::HL7V2(parsed))
                        .await
                    {
                        Ok(_) => {
                            if let Err(e) = stream.write_all(&MllpFormatter::ack()) {
                                eprintln!("Failed to send ACK: {}", e);
                                break;
                            }
                        }
                        Err(e) => {
                            eprintln!("Failed to store HL7v2 message: {}", e);
                            if let Err(e) = stream.write_all(&MllpFormatter::nak()) {
                                eprintln!("Failed to send NAK: {}", e);
                                break;
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
