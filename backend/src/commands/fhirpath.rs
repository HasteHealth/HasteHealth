use haste_fhir_model::r4::generated::{resources::Resource, terminology::IssueType};
use haste_fhir_operation_error::OperationOutcomeError;

use crate::utilities;

fn parse_fhir_data() -> Result<Resource, OperationOutcomeError> {
    let mut buffer = String::new();
    utilities::read_stdin(&mut buffer)?;

    let resource = serde_json::from_str::<Resource>(&buffer).map_err(|e| {
        OperationOutcomeError::error(
            IssueType::exception(),
            format!(
                "Failed to parse FHIR data must be a FHIR R4 Resource: {}",
                e
            ),
        )
    })?;

    Ok(resource)
}

/// Runs the `fhir-path` command: evaluates `fhirpath` against a FHIR resource read from stdin.
pub(crate) async fn run(fhirpath: &str) -> Result<(), OperationOutcomeError> {
    let data = parse_fhir_data()?;
    let engine = haste_fhirpath::FPEngine::new();

    let result = engine.evaluate(fhirpath, vec![&data]).await.map_err(|e| {
        OperationOutcomeError::error(
            IssueType::exception(),
            format!("Failed to evaluate FHIRPath: {}", e),
        )
    })?;

    println!("{:#?}", result.iter().collect::<Vec<_>>());

    Ok(())
}
