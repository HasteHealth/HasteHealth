use oxidized_fhir_operation_error::OperationOutcomeError;
use oxidized_server::server;

#[tokio::main]
async fn main() -> Result<(), OperationOutcomeError> {
    let server = server().await?;
    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    tracing::info!("Server started");
    axum::serve(listener, server).await.unwrap();

    Ok(())
}
