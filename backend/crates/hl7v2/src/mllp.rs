use haste_fhir_model::r4::generated::terminology::IssueType;
use haste_fhir_operation_error::OperationOutcomeError;
use std::io::Read as _;

pub fn tcp_listener(address: &str, port: u16) -> Result<(), OperationOutcomeError> {
    std::net::TcpListener::bind(format!("{}:{}", address, port))
        .map_err(|e| OperationOutcomeError::fatal(IssueType::Exception(None), e.to_string()))?
        .incoming()
        .for_each(|stream| {
            if let Ok(mut stream) = stream {
                let mut buffer = [0; 1024];
                if let Ok(size) = stream.read(&mut buffer) {
                    let message = String::from_utf8_lossy(&buffer[..size]);
                    println!("Received message: {}", message);
                }
            }
        });

    Ok(())
}
