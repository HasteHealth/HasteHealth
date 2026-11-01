//! Install FHIR packages (implementation guides) into the active profile's
//! project, through `ImplementationGuide/$install` and `$uninstall`.

use crate::cli::state::CliState;
use base64::{Engine as _, engine::general_purpose::STANDARD};
use clap::Subcommand;
use haste_fhir_client::{FHIRClient, url::ParsedParameters};
use haste_fhir_generated_ops::generated::{
    HasteHealthImplementationguideInstall as Install,
    HasteHealthImplementationguideUninstall as Uninstall,
};
use haste_fhir_model::r4::generated::{
    resources::{Parameters, Resource, ResourceType},
    terminology::IssueType,
    types::{FHIRBase64Binary, FHIRBoolean, FHIRString},
};
use haste_fhir_operation_error::OperationOutcomeError;
use std::{path::Path, sync::Arc};
use tokio::sync::Mutex;

/// Install FHIR packages (implementation guides) into the active profile's
/// project: their profiles, extensions, ValueSets, CodeSystems, ConceptMaps,
/// NamingSystems and SearchParameters.
#[derive(Subcommand, Debug)]
pub(crate) enum PackageCommands {
    /// Install packages, with the packages they depend on.
    Install {
        /// What to install: `name@version` from the package registry (for
        /// example `hl7.fhir.us.core@6.1.0`, `hl7.fhir.us.core@6.1.x`, or a
        /// name alone for the latest), a package tarball (`.tgz`), or a
        /// package folder (one holding `package.json`).
        #[arg(required = true)]
        packages: Vec<String>,
        /// Install only the packages given, not the packages they depend on.
        #[arg(long)]
        no_dependencies: bool,
        /// Also install the example resources of the packages given.
        #[arg(long)]
        examples: bool,
        /// Print the server's response (a Parameters resource) instead of a
        /// summary.
        #[arg(long)]
        json: bool,
    },
    /// Uninstall a package: delete what it installed. Packages it depends on
    /// stay installed.
    Uninstall {
        /// The package name, for example `hl7.fhir.us.core`.
        name: String,
        /// Only uninstall if this version is the one installed.
        #[arg(long)]
        version: Option<String>,
    },
    /// List the installed packages.
    List,
}

fn fhir_string(value: impl Into<String>) -> FHIRString {
    FHIRString {
        value: Some(value.into()),
        ..Default::default()
    }
}

fn value(string: Option<&FHIRString>) -> &str {
    string.and_then(|s| s.value.as_deref()).unwrap_or_default()
}

/// A package to upload, read from `argument` when it names a file or folder.
fn upload(argument: &str) -> Result<Option<FHIRBase64Binary>, OperationOutcomeError> {
    let path = Path::new(argument);
    let bytes = if path.is_dir() {
        haste_fhir_package::pack(path)?
    } else if path.is_file() {
        std::fs::read(path).map_err(|e| {
            OperationOutcomeError::error(
                IssueType::exception(),
                format!("Failed to read '{argument}': {e}"),
            )
        })?
    } else {
        return Ok(None);
    };
    Ok(Some(FHIRBase64Binary {
        value: Some(STANDARD.encode(bytes)),
        ..Default::default()
    }))
}

fn parameters(
    input: impl Into<Vec<haste_fhir_model::r4::generated::resources::ParametersParameter>>,
) -> Parameters {
    Parameters {
        parameter: Some(input.into()),
        ..Default::default()
    }
}

fn print_install_summary(output: &Install::Output) {
    for package in output.package.iter().flatten() {
        let name = value(Some(&package.name));
        let version = value(Some(&package.version));
        let status = package.status.value.as_deref().unwrap_or_default();
        let resources = package
            .resources
            .as_ref()
            .and_then(|count| count.value)
            .unwrap_or_default();
        match status {
            "skipped" => println!(
                "skipped    {name}@{version}: {}",
                value(package.reason.as_ref())
            ),
            "updated" => println!(
                "updated    {name}@{version} (was {}): {resources} resources",
                value(package.previousVersion.as_ref())
            ),
            _ => println!("{status:<10} {name}@{version}: {resources} resources"),
        }
    }
    let warnings = output
        .outcome
        .iter()
        .flat_map(|outcome| &outcome.issue)
        .filter_map(|issue| issue.diagnostics.as_ref()?.value.as_deref())
        .collect::<Vec<_>>();
    if !warnings.is_empty() {
        println!("\n{} warning(s):", warnings.len());
        for warning in warnings {
            println!("  {warning}");
        }
    }
}

/// Runs the `package` command group.
pub(crate) async fn run(
    state: Arc<Mutex<CliState>>,
    command: &PackageCommands,
) -> Result<(), OperationOutcomeError> {
    let fhir_client = crate::cli::client::fhir_client(state).await?;

    match command {
        PackageCommands::Install {
            packages,
            no_dependencies,
            examples,
            json,
        } => {
            let mut named = Vec::new();
            let mut uploads = Vec::new();
            for argument in packages {
                match upload(argument)? {
                    Some(content) => uploads.push(content),
                    None => named.push(fhir_string(argument)),
                }
            }
            let input = Install::Input {
                package: Some(named).filter(|named| !named.is_empty()),
                npmContent: Some(uploads).filter(|uploads| !uploads.is_empty()),
                dependencies: Some(FHIRBoolean {
                    value: Some(!no_dependencies),
                    ..Default::default()
                }),
                examples: Some(FHIRBoolean {
                    value: Some(*examples),
                    ..Default::default()
                }),
            };

            let result = fhir_client
                .invoke_type(
                    (),
                    ResourceType::ImplementationGuide,
                    "install".to_string(),
                    parameters(input),
                )
                .await?;

            if *json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&result).expect("Failed to serialize response")
                );
                return Ok(());
            }
            let Resource::Parameters(result) = result else {
                return Err(OperationOutcomeError::error(
                    IssueType::exception(),
                    "The server answered $install with something other than Parameters."
                        .to_string(),
                ));
            };
            print_install_summary(&Install::Output::try_from(
                result.parameter.unwrap_or_default(),
            )?);
            Ok(())
        }
        PackageCommands::Uninstall { name, version } => {
            let input = Uninstall::Input {
                name: fhir_string(name),
                version: version.as_ref().map(fhir_string),
            };
            let result = fhir_client
                .invoke_type(
                    (),
                    ResourceType::ImplementationGuide,
                    "uninstall".to_string(),
                    parameters(input),
                )
                .await?;
            match result {
                Resource::OperationOutcome(outcome) => {
                    for issue in outcome.issue {
                        println!("{}", value(issue.diagnostics.as_deref()));
                    }
                }
                other => println!(
                    "{}",
                    serde_json::to_string_pretty(&other).expect("Failed to serialize response")
                ),
            }
            Ok(())
        }
        PackageCommands::List => {
            let bundle = fhir_client
                .search_type(
                    (),
                    ResourceType::ImplementationGuide,
                    ParsedParameters::try_from("_count=1000")?,
                )
                .await?;
            for entry in bundle.entry.unwrap_or_default() {
                if let Some(Resource::ImplementationGuide(guide)) = entry.resource.as_deref() {
                    let title = guide
                        .title
                        .as_deref()
                        .and_then(|title| title.value.as_deref())
                        .unwrap_or_default();
                    println!(
                        "{}@{}  {title}",
                        guide.packageId.value.as_deref().unwrap_or_default(),
                        value(guide.version.as_deref()),
                    );
                }
            }
            Ok(())
        }
    }
}
