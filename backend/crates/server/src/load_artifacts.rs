use oxidized_artifacts::ARTIFACT_RESOURCES;
use oxidized_fhir_operation_error::OperationOutcomeError;

pub async fn load_artifacts() -> Result<(), OperationOutcomeError> {
    ARTIFACT_RESOURCES.iter().for_each(|res| {
        println!("Loaded resource: {:?}", res);
    });

    Ok(())
}
