use haste_fhir_operation_error::OperationOutcomeError;

use crate::Cli;

pub(crate) async fn generate_cli_markdown(output: &str) -> Result<(), OperationOutcomeError> {
    let markdown: String = clap_markdown::help_markdown::<Cli>();

    std::fs::write(output, markdown).map_err(|e| {
        OperationOutcomeError::error(
            haste_fhir_model::r4::generated::terminology::IssueType::exception(),
            e.to_string(),
        )
    })?;

    Ok(())
}
