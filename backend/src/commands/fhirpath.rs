use haste_fhir_model::r4::generated::{resources::Resource, terminology::IssueType};
use haste_fhir_operation_error::OperationOutcomeError;

fn parse_fhir_data() -> Result<Resource, OperationOutcomeError> {
    let mut buffer = String::new();
    std::io::stdin().read_line(&mut buffer).map_err(|_| {
        OperationOutcomeError::fatal(
            IssueType::Exception(None),
            "Failed to read from stdin.".into(),
        )
    })?;
    let resource = haste_fhir_serialization_json::from_str::<Resource>(&buffer).map_err(|e| {
        OperationOutcomeError::error(
            IssueType::Exception(None),
            format!(
                "Failed to parse FHIR data must be a FHIR R4 Resource: {}",
                e
            ),
        )
    })?;

    Ok(resource)
}

pub fn fhirpath(fhirpath: &str) -> Result<(), OperationOutcomeError> {
    let data = parse_fhir_data()?;
    let engine = haste_fhirpath::FPEngine::new();

    let result = engine.evaluate(fhirpath, vec![&data]).map_err(|e| {
        OperationOutcomeError::error(
            IssueType::Exception(None),
            format!("Failed to evaluate FHIRPath: {}", e),
        )
    })?;

    println!("{:#?}", result.iter().collect::<Vec<_>>());

    Ok(())
}
