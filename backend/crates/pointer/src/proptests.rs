//! Property tests for paths: escaping is reversible, a path ascends through
//! the keys it descended, and a path reads what it names.

use std::sync::Arc;

use haste_fhir_model::r4::generated::{
    resources::Patient,
    types::{FHIRString, HumanName},
};
use proptest::prelude::*;
use proptest::test_runner::TestCaseError;

use crate::{
    Key, Path, TypedPointer,
    escape::{escape_field, unescape_field},
};

fn patient_named(families: &[String]) -> Patient {
    Patient {
        name: Some(
            families
                .iter()
                .map(|family| HumanName {
                    family: Some(Box::new(FHIRString {
                        value: Some(family.clone()),
                        ..Default::default()
                    })),
                    ..Default::default()
                })
                .collect(),
        ),
        ..Default::default()
    }
}

proptest! {
    /// Unescaping undoes escaping, whatever the field holds.
    #[test]
    fn escape_round_trips(field in ".*") {
        prop_assert_eq!(unescape_field(&escape_field(&field)), field);
    }

    /// An escaped field never holds the separator, so a path splits
    /// unambiguously.
    #[test]
    fn escaped_field_has_no_separator(field in ".*") {
        prop_assert!(!escape_field(&field).contains('/'));
    }

    /// Ascending yields the keys the path descended through, last first,
    /// with the parent at each step, until the root.
    #[test]
    fn ascend_undoes_descend(fields in prop::collection::vec(".*", 0..6)) {
        let mut path = Path::new();
        let mut parents = Vec::new();
        for field in &fields {
            parents.push(path.to_string());
            path = path.descend(field);
        }
        for (field, parent) in fields.iter().zip(&parents).rev() {
            let Some((above, key)) = path.ascend() else {
                return Err(TestCaseError::fail(format!("{path} does not ascend")));
            };
            prop_assert_eq!(format!("{:?}", key), format!("{:?}", Key::parse(field)));
            prop_assert_eq!(above.to_string(), parent.clone());
            path = above;
        }
        prop_assert!(path.ascend().is_none(), "the root {} ascends", path);
    }

    /// A path reads the value it names, directly and through a typed
    /// pointer, and an index past the end reads nothing.
    #[test]
    fn path_reads_what_it_names(families in prop::collection::vec(".*", 1..5)) {
        let patient = Arc::new(patient_named(&families));
        for (index, family) in families.iter().enumerate() {
            let path = Path::new()
                .descend("name")
                .descend(&index.to_string())
                .descend("family")
                .descend("value");
            prop_assert_eq!(path.get_typed::<String>(patient.as_ref()), Some(family));

            let pointer = TypedPointer::<Patient, Patient>::new(patient.clone())
                .descend::<Vec<HumanName>>(&Key::Field("name".to_string()))
                .and_then(|pointer| pointer.descend::<HumanName>(&Key::Index(index)))
                .and_then(|pointer| pointer.descend::<Box<FHIRString>>(&Key::Field("family".to_string())))
                .and_then(|pointer| pointer.descend::<String>(&Key::Field("value".to_string())));
            let Some(pointer) = pointer else {
                return Err(TestCaseError::fail(format!("no pointer to {path}")));
            };
            prop_assert_eq!(pointer.path(), path.to_string());
            prop_assert_eq!(pointer.value(), Some(family));
        }

        let past_the_end = Path::new().descend("name").descend(&families.len().to_string());
        prop_assert!(past_the_end.get(patient.as_ref()).is_none());
        let pointer = TypedPointer::<Patient, Patient>::new(patient.clone())
            .descend::<Vec<HumanName>>(&Key::Field("name".to_string()))
            .and_then(|pointer| pointer.descend::<HumanName>(&Key::Index(families.len())));
        prop_assert!(pointer.is_none());
    }
}
