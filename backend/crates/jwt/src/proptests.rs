//! Property tests for scope strings: formatting a parsed scope list and
//! parsing the result is a fixed point, whitespace between scopes is not
//! significant, permission letters parse only in `cruds` order, and no input
//! panics the parser.

use proptest::prelude::*;
use proptest::test_runner::TestCaseError;

use crate::scopes::{Scopes, SmartResourceScopePermissions};

const RESOURCE_TYPES: &[&str] = &[
    "Patient",
    "Observation",
    "Encounter",
    "Condition",
    "MedicationRequest",
    "Practitioner",
    "Organization",
    "Bundle",
    "Binary",
    "User",
    "Membership",
    "ClientApplication",
    "HL7V2",
];

const PERMISSION_ORDER: &str = "cruds";

fn permissions() -> impl Strategy<Value = String> {
    prop_oneof![
        prop::sample::select(vec!["*", "read", "write"]).prop_map(String::from),
        prop::sample::subsequence(PERMISSION_ORDER.chars().collect::<Vec<_>>(), 1..=5)
            .prop_map(|letters| letters.into_iter().collect()),
    ]
}

fn resource_scope() -> impl Strategy<Value = String> {
    (
        prop::sample::select(vec!["user", "system", "patient"]),
        prop_oneof![Just("*"), prop::sample::select(RESOURCE_TYPES)],
        permissions(),
    )
        .prop_map(|(user, level, permissions)| format!("{user}/{level}.{permissions}"))
}

fn scope() -> impl Strategy<Value = String> {
    prop_oneof![
        prop::sample::select(vec![
            "openid",
            "profile",
            "email",
            "offline_access",
            "online_access",
            "fhirUser",
            "launch",
            "launch/patient",
            "launch/encounter",
        ])
        .prop_map(String::from),
        resource_scope(),
    ]
}

fn scope_list() -> impl Strategy<Value = Vec<String>> {
    prop::collection::vec(scope(), 0..8)
}

fn parse(text: &str) -> Result<Scopes, TestCaseError> {
    Scopes::try_from(text)
        .map_err(|error| TestCaseError::fail(format!("{text:?} did not parse: {error}")))
}

proptest! {
    /// Formatting a parsed scope list and parsing the result is a fixed
    /// point: the same scopes, and the same text again.
    #[test]
    fn format_then_parse_is_a_fixed_point(scopes in scope_list()) {
        let text = scopes.join(" ");
        let parsed = parse(&text)?;
        let formatted = String::from(parsed.clone());
        let again = parse(&formatted)?;
        prop_assert_eq!(&again, &parsed, "{:?} formatted as {:?}", text, formatted);
        prop_assert_eq!(String::from(again), formatted);
    }

    /// Whitespace between scopes is not significant.
    #[test]
    fn whitespace_between_scopes_is_not_significant(
        scopes in scope_list(),
        separators in prop::collection::vec(
            prop::sample::select(vec![" ", "  ", "\t", "\n", " \t "]), 9
        ),
    ) {
        let mut spaced = String::new();
        for (scope, separator) in scopes.iter().zip(&separators) {
            spaced.push_str(separator);
            spaced.push_str(scope);
        }
        spaced.push_str(separators[8]);
        prop_assert_eq!(parse(&spaced)?, parse(&scopes.join(" "))?);
    }

    /// Permission letters parse only as an in-order subsequence of `cruds`.
    #[test]
    fn permissions_parse_only_in_cruds_order(letters in "[crudsx]{0,6}") {
        let positions: Vec<Option<usize>> =
            letters.chars().map(|letter| PERMISSION_ORDER.find(letter)).collect();
        let ordered = positions.iter().all(Option::is_some)
            && positions.windows(2).all(|pair| pair[0] < pair[1]);
        prop_assert_eq!(
            SmartResourceScopePermissions::try_from(letters.as_str()).is_ok(),
            ordered,
            "{:?}", letters
        );
    }

    /// Parsing never panics, whatever the text.
    #[test]
    fn parsing_never_panics(text in ".*") {
        let _ = Scopes::try_from(text.as_str());
    }

    /// Nor on text shaped like a scope.
    #[test]
    fn parsing_scope_shaped_text_never_panics(
        text in "(user|system|patient|launch)/[A-Za-z*./ ]{0,12}"
    ) {
        let _ = Scopes::try_from(text.as_str());
    }
}
