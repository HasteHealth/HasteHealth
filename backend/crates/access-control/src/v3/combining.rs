//! The combining algorithms of FHIR R6 Permission (XACML's), written as a fold:
//! [`step`] takes in one more rule decision and [`finish`] settles the result.

use super::Decision;
use haste_fhir_model::r4::generated::terminology::{Accesspolicyv3combining, BoundCode};
use std::{cmp::max_by_key, ops::ControlFlow};

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
    combining: &BoundCode<Accesspolicyv3combining>,
    result: Decision,
    next: Decision,
) -> ControlFlow<Decision, Decision> {
    match combining {
        c if *c == Accesspolicyv3combining::first_applicable() => {
            if next == Decision::NotApplicable {
                ControlFlow::Continue(result)
            } else {
                ControlFlow::Break(next)
            }
        }
        c if *c == Accesspolicyv3combining::permit_overrides()
            || *c == Accesspolicyv3combining::deny_unless_permit() =>
        {
            ControlFlow::Continue(max_by_key(result, next, |d| permit_wins(*d)))
        }
        // deny-overrides and permit-unless-deny; an absent or unknown code is deny-overrides.
        _ if next == Decision::Deny => ControlFlow::Break(next),
        _ => ControlFlow::Continue(max_by_key(result, next, |d| deny_wins(*d))),
    }
}

/// The decision once every rule is folded in: the `unless` algorithms turn "no rule
/// applied" into their default.
#[must_use]
pub fn finish(combining: &BoundCode<Accesspolicyv3combining>, result: Decision) -> Decision {
    match combining {
        c if result == Decision::NotApplicable
            && *c == Accesspolicyv3combining::deny_unless_permit() =>
        {
            Decision::Deny
        }
        c if result == Decision::NotApplicable
            && *c == Accesspolicyv3combining::permit_unless_deny() =>
        {
            Decision::Permit
        }
        _ => result,
    }
}
