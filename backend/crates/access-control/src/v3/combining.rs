//! The combining algorithms of FHIR R6 Permission (XACML's), written as a fold:
//! [`step`] takes in one more rule decision and [`finish`] settles the result.

use super::Decision;
use haste_fhir_model::r4::generated::terminology::{Accesspolicyv3combining, BoundCode};
use std::{cmp::max_by_key, ops::ControlFlow};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Algorithm {
    DenyOverrides,
    PermitOverrides,
    DenyUnlessPermit,
    PermitUnlessDeny,
    FirstApplicable,
}

/// The algorithm a `combining` code names; absent or unknown is deny-overrides.
#[must_use]
pub fn algorithm(code: Option<&BoundCode<Accesspolicyv3combining>>) -> Algorithm {
    match code.and_then(BoundCode::as_str) {
        Some("permit-overrides") => Algorithm::PermitOverrides,
        Some("deny-unless-permit") => Algorithm::DenyUnlessPermit,
        Some("permit-unless-deny") => Algorithm::PermitUnlessDeny,
        Some("first-applicable") => Algorithm::FirstApplicable,
        _ => Algorithm::DenyOverrides,
    }
}

/// Rank when a deny wins. An indeterminate rule counts as a deny, so it outranks a permit.
const fn deny_wins(decision: Decision) -> u8 {
    match decision {
        Decision::NotApplicable => 0,
        Decision::Permit => 1,
        Decision::Indeterminate => 2,
        Decision::Deny => 3,
    }
}

/// Rank when a permit wins. An indeterminate rule counts as a deny, so a deny outranks it.
const fn permit_wins(decision: Decision) -> u8 {
    match decision {
        Decision::NotApplicable => 0,
        Decision::Indeterminate => 1,
        Decision::Deny => 2,
        Decision::Permit => 3,
    }
}

/// Folds the next rule's decision into the running `result`. `Break` settles the result
/// early: a deny under a deny-wins algorithm, or the first applicable rule. A permit never
/// settles it, so every permitting rule gets to contribute its obligations.
pub fn step(
    algorithm: Algorithm,
    result: Decision,
    next: Decision,
) -> ControlFlow<Decision, Decision> {
    match algorithm {
        Algorithm::FirstApplicable if next == Decision::NotApplicable => {
            ControlFlow::Continue(result)
        }
        Algorithm::FirstApplicable => ControlFlow::Break(next),
        Algorithm::DenyOverrides | Algorithm::PermitUnlessDeny if next == Decision::Deny => {
            ControlFlow::Break(next)
        }
        Algorithm::DenyOverrides | Algorithm::PermitUnlessDeny => {
            ControlFlow::Continue(max_by_key(result, next, |d| deny_wins(*d)))
        }
        Algorithm::PermitOverrides | Algorithm::DenyUnlessPermit => {
            ControlFlow::Continue(max_by_key(result, next, |d| permit_wins(*d)))
        }
    }
}

/// The decision once every rule is folded in: the `unless` algorithms turn "no rule
/// applied" into their default.
#[must_use]
pub const fn finish(algorithm: Algorithm, result: Decision) -> Decision {
    match (algorithm, result) {
        (Algorithm::DenyUnlessPermit, Decision::NotApplicable) => Decision::Deny,
        (Algorithm::PermitUnlessDeny, Decision::NotApplicable) => Decision::Permit,
        _ => result,
    }
}
