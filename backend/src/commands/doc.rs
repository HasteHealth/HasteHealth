use haste_fhir_operation_error::OperationOutcomeError;

use crate::Cli;

/// Runs the `doc` command: renders this CLI's `--help` text for every command to Markdown.
pub(crate) async fn run(output: &str) -> Result<(), OperationOutcomeError> {
    let markdown: String = clap_markdown::help_markdown::<Cli>();

    let top_string = "
| Context | Invocation                                                   |
| ------- | ------------------------------------------------------------ |
| npm     | `npm install -g haste-health`, then `haste-health <command>` |
| npx     | `npx haste-health <command>`                                 |
| Source  | `cargo run <command>`                                        |
| Binary  | `./haste-health <command>`                                   |
| Docker  | `docker run <image> <command>`                               |

## Shell completion

With `haste-health` on your `PATH`, add the line for your shell to its startup file.
Tab then completes commands, options, FHIR resource types and saved profile names.
See the [CLI setup guide](https://haste.health/docs/tutorials/cli#shell-completion).

| Shell | Startup file                  | Line                                     |
| ----- | ----------------------------- | ---------------------------------------- |
| zsh   | `~/.zshrc`, after `compinit`  | `source <(COMPLETE=zsh haste-health)`    |
| bash  | `~/.bashrc`                   | `eval \"$(COMPLETE=bash haste-health)\"` |
| fish  | `~/.config/fish/config.fish`  | `COMPLETE=fish haste-health \\| source`  |
";

    let markdown = format!("{top_string}\n\n{markdown}");

    tokio::fs::write(output, markdown).await.map_err(|e| {
        OperationOutcomeError::error(
            haste_fhir_model::r4::generated::terminology::IssueType::exception(),
            e.to_string(),
        )
    })?;

    Ok(())
}
