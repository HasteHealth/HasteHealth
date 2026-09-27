use clap::Subcommand;
use haste_fhir_model::r4::generated::terminology::IssueType;
use haste_fhir_operation_error::OperationOutcomeError;
use std::path::{Path, PathBuf};

/// Where the website reads the pricing table from, relative to `backend/`.
const DEFAULT_OUTPUT: &str = "../frontend/packages/website/static/pricing/tiers.json";

/// Publish what each subscription tier allows.
#[derive(Subcommand, Debug)]
pub(crate) enum SubscriptionCommands {
    /// Write the subscription tiers to JSON for the website's pricing page.
    Export {
        /// Where to write the tiers.
        #[arg(short, long, default_value = DEFAULT_OUTPUT)]
        output: PathBuf,
        /// Fail if the output is out of date instead of writing it.
        #[arg(long)]
        check: bool,
    },
    /// Print what each tier allows, as a table.
    Show,
}

fn error(message: String) -> OperationOutcomeError {
    OperationOutcomeError::error(IssueType::exception(), message)
}

/// The pricing table as JSON, newline-terminated so it is a well-formed text
/// file and diffs cleanly.
fn rendered() -> Result<String, OperationOutcomeError> {
    let mut json = serde_json::to_string_pretty(&haste_subscription::export::build())
        .map_err(|e| error(format!("Failed to serialize the subscription tiers: {e}")))?;
    json.push('\n');

    Ok(json)
}

async fn export(output: &Path, check: bool) -> Result<(), OperationOutcomeError> {
    let contents = rendered()?;

    if check {
        let current = tokio::fs::read_to_string(output)
            .await
            .map_err(|e| error(format!("{}: {e}", output.display())))?;

        return if current == contents {
            println!("{} is up to date", output.display());
            Ok(())
        } else {
            Err(error(format!(
                "Out of date, run `cargo run subscription export` to regenerate: {}",
                output.display()
            )))
        };
    }

    if let Some(parent) = output.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(|e| error(format!("{}: {e}", parent.display())))?;
    }

    tokio::fs::write(output, contents)
        .await
        .map_err(|e| error(format!("{}: {e}", output.display())))?;
    println!("Wrote {}", output.display());

    Ok(())
}

fn show() {
    let export = haste_subscription::export::build();

    println!(
        "Request costs: read {}, search {}, history {}, invocation {}, write {} points (budget per {})\n",
        export.operation_points.read,
        export.operation_points.search,
        export.operation_points.history,
        export.operation_points.invocation,
        export.operation_points.write,
        export.window,
    );

    for tier in &export.tiers {
        let tier_id: String = tier.limits.tier.clone().into();

        println!("{} ({})", tier.limits.display_name, tier_id);
        println!(
            "  {}{}  -  {}",
            tier.limits.price,
            tier.limits.cadence.unwrap_or(""),
            tier.limits.audience,
        );
        println!("  Request budget    {}", tier.request_budget_label);
        println!("  Total resources   {}", tier.total_resources_label);
        println!("  Support           {}", tier.limits.support);
        println!("  Uptime SLA        {}", tier.limits.uptime_sla);
        println!(
            "  Customization     {}",
            if tier.limits.tenant_customization {
                "allowed"
            } else {
                "not allowed"
            }
        );
        println!(
            "  BAA               {}",
            if tier.limits.baa_available {
                "available"
            } else {
                "not available"
            }
        );
        println!();
    }
}

pub(crate) async fn run(command: &SubscriptionCommands) -> Result<(), OperationOutcomeError> {
    match command {
        SubscriptionCommands::Export { output, check } => export(output, *check).await,
        SubscriptionCommands::Show => {
            show();
            Ok(())
        }
    }
}
