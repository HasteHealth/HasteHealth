//! Serializes [`crate::TIERS`] for the website's pricing page.
//!
//! The pricing page renders whatever this produces, so a published price or
//! limit cannot disagree with what the server enforces. Written by
//! `cargo run subscription export` and committed; CI re-runs it with `--check`.

use crate::{OPERATION_POINTS, OperationPoints, RequestBudget, ResourceLimit, TIERS, TierLimits};
use serde::Serialize;

/// What a tier's daily budget buys, in whole requests of one kind.
///
/// Derived rather than written down: a budget is only meaningful next to what it
/// affords, and stating it in requests is how buyers think about it.
#[derive(Debug, Serialize)]
pub struct BudgetExamples {
    pub reads_per_day: Option<u64>,
    pub writes_per_day: Option<u64>,
    pub searches_per_day: Option<u64>,
}

impl BudgetExamples {
    fn for_budget(budget: &RequestBudget, points: &OperationPoints) -> Self {
        let per_day = |cost: u32| match budget {
            // Unmetered has no meaningful example count.
            RequestBudget::Unmetered => None,
            RequestBudget::Points(available) if cost > 0 => {
                Some(*available as u64 / u64::from(cost))
            }
            RequestBudget::Points(_) => None,
        };

        Self {
            reads_per_day: per_day(points.read),
            writes_per_day: per_day(points.write),
            searches_per_day: per_day(points.search),
        }
    }
}

/// One tier as the pricing page consumes it: the enforced limits, plus the
/// derived figures and prose the page would otherwise have to compute itself.
#[derive(Debug, Serialize)]
pub struct ExportedTier<'a> {
    /// Every field of the tier definition, flattened so the page reads
    /// `tier.price` rather than `tier.limits.price`.
    #[serde(flatten)]
    pub limits: &'a TierLimits,
    /// The request budget rendered for display, e.g. `"250,000 points / day"`.
    pub request_budget_label: String,
    /// The total resource cap rendered for display, e.g. `"250,000"`.
    pub total_resources_label: String,
    /// What the budget buys, for the explanatory line under it.
    pub budget_examples: BudgetExamples,
}

/// The whole pricing table, as written to the website.
#[derive(Debug, Serialize)]
pub struct Export<'a> {
    /// Warns anyone who opens the committed file not to edit it.
    #[serde(rename = "$comment")]
    pub comment: &'static str,
    /// The cost of each kind of request, so the page can state it once.
    pub operation_points: &'a OperationPoints,
    /// How long a request budget lasts before it resets.
    pub window: &'static str,
    pub tiers: Vec<ExportedTier<'a>>,
}

/// Formats a number with thousands separators: `25000` becomes `"25,000"`.
fn with_separators(value: u64) -> String {
    let digits = value.to_string();
    let mut out = String::with_capacity(digits.len() + digits.len() / 3);

    for (index, digit) in digits.chars().enumerate() {
        if index > 0 && (digits.len() - index) % 3 == 0 {
            out.push(',');
        }
        out.push(digit);
    }

    out
}

fn budget_label(budget: &RequestBudget) -> String {
    match budget {
        RequestBudget::Points(points) => {
            format!("{} points / day", with_separators(*points as u64))
        }
        RequestBudget::Unmetered => "Unmetered".to_string(),
    }
}

fn resource_label(limit: &ResourceLimit) -> String {
    match limit {
        ResourceLimit::Count(count) => with_separators(*count),
        ResourceLimit::Unlimited => "Unlimited".to_string(),
    }
}

/// Builds the export from [`TIERS`].
pub fn build() -> Export<'static> {
    Export {
        comment: "Generated from backend/crates/subscription by `cargo run subscription export`. Do not edit by hand.",
        operation_points: &OPERATION_POINTS,
        window: "day",
        tiers: TIERS
            .iter()
            .map(|limits| ExportedTier {
                limits,
                request_budget_label: budget_label(&limits.request_budget),
                total_resources_label: resource_label(&limits.total_resources),
                budget_examples: BudgetExamples::for_budget(
                    &limits.request_budget,
                    &OPERATION_POINTS,
                ),
            })
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use haste_jwt::claims::SubscriptionTier;

    #[test]
    fn formats_thousands_separators() {
        assert_eq!(with_separators(0), "0");
        assert_eq!(with_separators(50), "50");
        assert_eq!(with_separators(999), "999");
        assert_eq!(with_separators(1_000), "1,000");
        assert_eq!(with_separators(25_000), "25,000");
        assert_eq!(with_separators(5_000_000), "5,000,000");
    }

    #[test]
    fn labels_the_enforced_numbers() {
        let export = build();
        let free = &export.tiers[0];

        assert_eq!(free.limits.tier, SubscriptionTier::Free);
        assert_eq!(free.request_budget_label, "250,000 points / day");
        assert_eq!(free.total_resources_label, "250,000");

        let self_hosted = export.tiers.last().expect("tiers are not empty");
        assert_eq!(self_hosted.request_budget_label, "Unmetered");
        assert_eq!(self_hosted.total_resources_label, "Unlimited");
    }

    #[test]
    fn derives_what_the_budget_buys() {
        let export = build();
        let free = &export.tiers[0];

        // 250,000 points at 1 per read, 6 per search and 25 per write.
        assert_eq!(free.budget_examples.reads_per_day, Some(250_000));
        assert_eq!(free.budget_examples.searches_per_day, Some(41_666));
        assert_eq!(free.budget_examples.writes_per_day, Some(10_000));

        let self_hosted = export.tiers.last().expect("tiers are not empty");
        assert_eq!(self_hosted.budget_examples.reads_per_day, None);
    }

    /// The page keys its cards off the claim value, so each tier must serialize
    /// to the string the JWT carries.
    #[test]
    fn serializes_the_tier_claim_value() {
        let json = serde_json::to_value(build()).expect("export serializes");
        let tiers = json["tiers"].as_array().expect("tiers is an array");

        let ids: Vec<&str> = tiers
            .iter()
            .map(|tier| tier["tier"].as_str().expect("tier is a string"))
            .collect();

        assert_eq!(ids, ["free", "professional", "team", "unlimited"]);
    }

    /// The website's `ResourceLimit`/`RequestBudget` types narrow on these exact
    /// shapes: an externally tagged object for a variant with a value, a bare
    /// tag string for a unit variant. Changing either breaks the pricing page's
    /// static render, so pin both here.
    #[test]
    fn limits_keep_the_shape_the_website_narrows_on() {
        let json = serde_json::to_value(build()).expect("export serializes");
        let tiers = &json["tiers"];

        // Free is capped, so every limit is an object carrying its count.
        assert_eq!(
            tiers[0]["total_resources"],
            serde_json::json!({"count": 250_000})
        );
        assert_eq!(
            tiers[0]["request_budget"],
            serde_json::json!({"points": 250_000})
        );

        // Uncapped limits are the bare tag string, not `{"unlimited": null}`.
        assert_eq!(tiers[1]["total_resources"], serde_json::json!("unlimited"));
        assert_eq!(
            tiers[3]["request_budget"],
            serde_json::json!("unmetered")
        );
    }

    /// `flatten` puts the tier's own fields at the top level; the page reads
    /// them there.
    #[test]
    fn flattens_tier_fields_to_the_top_level() {
        let json = serde_json::to_value(build()).expect("export serializes");
        let free = &json["tiers"][0];

        assert_eq!(free["display_name"], "Developer");
        assert_eq!(free["price"], "$0");
        assert_eq!(free["support"], "Community");
        assert_eq!(free["tenant_customization"], false);
        assert_eq!(free["projects"]["count"], 2);
    }
}
