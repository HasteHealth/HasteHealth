//! The `unionAll` rule: every branch gives the same columns.
//!
//! Checked once per run, on the view alone, so a mismatch is refused before any resource is
//! read and the rows themselves need no checking.

use haste_fhir_model::r4::generated::resources::ViewDefinitionSelect;
use haste_fhir_operation_error::OperationOutcomeError;

/// A column's name and, when the view gives one, its type.
type Column<'a> = (&'a str, Option<&'a str>);

/// The columns a select gives, in order: its own, its nested selects', then its `unionAll`'s.
/// The first branch stands for the `unionAll`, since every branch gives the same.
fn columns(select: &ViewDefinitionSelect) -> Vec<Column<'_>> {
    let own = select.column.iter().flatten().map(|column| {
        (
            column.name.value.as_deref().unwrap_or_default(),
            column.type_.as_ref().and_then(|t| t.value.as_deref()),
        )
    });
    let nested = select.select.iter().flatten().flat_map(columns);
    let union = select.unionAll.iter().flatten().take(1).flat_map(columns);
    own.chain(nested).chain(union).collect()
}

/// Whether two column lists agree: the same names in the same order, and the same type
/// wherever both give one.
fn agree(a: &[Column<'_>], b: &[Column<'_>]) -> bool {
    a.len() == b.len()
        && a.iter().zip(b).all(|((name, type_), (other, other_type))| {
            name == other && type_.zip(*other_type).is_none_or(|(t, u)| t == u)
        })
}

fn describe(columns: &[Column<'_>]) -> String {
    columns
        .iter()
        .map(|(name, type_)| type_.map_or_else(|| (*name).to_string(), |t| format!("{name}:{t}")))
        .collect::<Vec<_>>()
        .join(", ")
}

/// Refuses a `unionAll` whose branches differ in their columns: the names, their order, or a
/// type both give. Every select of the view is checked, nested ones and branches included.
pub(crate) fn check_unions(selects: &[ViewDefinitionSelect]) -> Result<(), OperationOutcomeError> {
    selects.iter().try_for_each(|select| {
        let branches = select.unionAll.as_deref().unwrap_or_default();
        let expected = branches.first().map(columns).unwrap_or_default();
        if let Some(found) = branches
            .iter()
            .skip(1)
            .map(columns)
            .find(|found| !agree(&expected, found))
        {
            return Err(super::invalid(format!(
                "unionAll branches must give the same columns: [{}] differs from [{}]",
                describe(&expected),
                describe(&found)
            )));
        }
        check_unions(select.select.as_deref().unwrap_or_default())?;
        check_unions(branches)
    })
}
