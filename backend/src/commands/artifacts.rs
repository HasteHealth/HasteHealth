use clap::Subcommand;
use haste_artifact_patcher::{Build, Change};
use haste_fhir_model::r4::generated::terminology::IssueType;
use haste_fhir_operation_error::OperationOutcomeError;
use serde_json::Value;
use similar::{ChangeTag, TextDiff};
use std::{
    fmt::Write,
    path::{Path, PathBuf},
};

/// Patch externally provided (HL7) artifacts without editing the upstream files.
#[derive(Subcommand, Debug)]
pub(crate) enum ArtifactCommands {
    /// Apply a package's patches and rules to its upstream files and write the `.min.json` outputs.
    Build {
        /// Patch manifest(s), e.g. `../artifacts/r4/hl7-core/patches/manifest.toml`.
        #[arg(required = true)]
        manifest: Vec<PathBuf>,
        /// Fail if any output is out of date instead of writing it.
        #[arg(long)]
        check: bool,
    },
    /// List every change the patches and rules make to the upstream resources.
    Diff {
        /// Patch manifest(s), e.g. `../artifacts/r4/hl7-core/patches/manifest.toml`.
        #[arg(required = true)]
        manifest: Vec<PathBuf>,
        /// Only show resources whose `ResourceType/id` contains this text.
        #[arg(short, long)]
        resource: Option<String>,
        /// Only show changes whose origin (`patch <file>` or `rule <name>`) contains this text.
        #[arg(short, long)]
        origin: Option<String>,
        /// Print a count of changes per origin instead of the changes.
        #[arg(long)]
        summary: bool,
    },
}

fn error(message: String) -> OperationOutcomeError {
    OperationOutcomeError::error(IssueType::exception(), message)
}

fn build(manifest: &Path) -> Result<Build, OperationOutcomeError> {
    haste_artifact_patcher::build(manifest).map_err(|e| error(e.to_string()))
}

fn render(value: Option<&Value>) -> String {
    match value {
        None => "(absent)".to_string(),
        Some(Value::String(text)) => text.clone(),
        Some(other) => other.to_string(),
    }
}

fn print_value(prefix: char, value: Option<&Value>) {
    for line in render(value).lines() {
        println!("    {prefix} {line}");
    }
}

/// Word diff of two strings in `git diff --word-diff` style
/// (`[-removed-]{+added+}`), printing only the lines that changed.
fn print_text_diff(before: &str, after: &str) {
    let mut marked = String::new();
    for change in TextDiff::from_words(before, after).iter_all_changes() {
        match change.tag() {
            ChangeTag::Equal => marked.push_str(change.value()),
            ChangeTag::Delete => {
                let _ = write!(marked, "[-{}-]", change.value());
            }
            ChangeTag::Insert => {
                let _ = write!(marked, "{{+{}+}}", change.value());
            }
        }
    }
    for line in marked
        .lines()
        .filter(|l| l.contains("[-") || l.contains("{+"))
    {
        println!("    ~ {line}");
    }
}

fn print_changes(changes: &[&Change]) {
    let mut current = None;
    for change in changes {
        let heading = (&change.resource, &change.origin);
        if current != Some(heading) {
            println!(
                "\n{}  [{}]  ({})",
                change.resource,
                change.origin,
                change.source.display()
            );
            current = Some(heading);
        }
        println!("  {}", change.pointer);
        if let (Some(Value::String(before)), Some(Value::String(after))) =
            (&change.before, &change.after)
        {
            print_text_diff(before, after);
        } else {
            print_value('-', change.before.as_ref());
            print_value('+', change.after.as_ref());
        }
    }
}

fn print_summary(changes: &[&Change]) {
    let mut counts: Vec<(String, usize)> = Vec::new();
    for change in changes {
        let origin = change.origin.to_string();
        match counts.iter_mut().find(|(name, _)| *name == origin) {
            Some((_, count)) => *count += 1,
            None => counts.push((origin, 1)),
        }
    }
    for (origin, count) in counts {
        println!("{count:>8}  {origin}");
    }
}

/// Runs the `artifacts` command group.
pub(crate) async fn run(command: &ArtifactCommands) -> Result<(), OperationOutcomeError> {
    match command {
        ArtifactCommands::Build { manifest, check } => {
            let mut stale = Vec::new();
            for manifest in manifest {
                let result = build(manifest)?;
                for output in result.outputs {
                    let current = tokio::fs::read_to_string(&output.path).await.ok();
                    if current.as_deref() == Some(output.contents.as_str()) {
                        continue;
                    }
                    if *check {
                        stale.push(output.path.display().to_string());
                    } else {
                        tokio::fs::write(&output.path, output.contents)
                            .await
                            .map_err(|e| error(format!("{}: {e}", output.path.display())))?;
                        println!("Wrote {}", output.path.display());
                    }
                }
                println!(
                    "{}: {} change(s) from patches and rules",
                    manifest.display(),
                    result.changes.len()
                );
            }

            if stale.is_empty() {
                Ok(())
            } else {
                Err(error(format!(
                    "Out of date, run `cargo run artifacts build` to regenerate: {}",
                    stale.join(", ")
                )))
            }
        }
        ArtifactCommands::Diff {
            manifest,
            resource,
            origin,
            summary,
        } => {
            for manifest in manifest {
                let result = build(manifest)?;
                let changes = result
                    .changes
                    .iter()
                    .filter(|c| {
                        resource
                            .as_ref()
                            .is_none_or(|r| c.resource.contains(r.as_str()))
                    })
                    .filter(|c| {
                        origin
                            .as_ref()
                            .is_none_or(|o| c.origin.to_string().contains(o.as_str()))
                    })
                    .collect::<Vec<_>>();

                println!("# {} ({} change(s))", manifest.display(), changes.len());
                if *summary {
                    print_summary(&changes);
                } else {
                    print_changes(&changes);
                }
            }
            Ok(())
        }
    }
}
