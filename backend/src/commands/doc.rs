use haste_fhir_operation_error::OperationOutcomeError;

use crate::Cli;

/// Runs the `doc` command: renders this CLI's `--help` text for every command to Markdown.
pub(crate) async fn run(output: &str) -> Result<(), OperationOutcomeError> {
    let markdown: String = clap_markdown::help_markdown::<Cli>();

    let top_string = "
| Context | Invocation                     |
| ------- | ------------------------------ |
| Source  | `cargo run <command>`          |
| Binary  | `./haste-health <command>`     |
| Docker  | `docker run <image> <command>` |
";

    let markdown = format!("{top_string}\n\n{markdown}");

    std::fs::write(output, markdown).map_err(|e| {
        OperationOutcomeError::error(
            haste_fhir_model::r4::generated::terminology::IssueType::exception(),
            e.to_string(),
        )
    })?;

    Ok(())
}
