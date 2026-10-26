use haste_fhir_model::r4::generated::{resources::ViewDefinitionSelect, terminology::IssueType};
use haste_fhir_operation_error::OperationOutcomeError;

/// A column's name and, when the view gives one, its type.
type Column<'a> = (&'a str, Option<&'a str>);

/// The columns a select gives: its own, its nested selects', then its `unionAll`'s first branch.
fn select_columns<'a>(select: &'a ViewDefinitionSelect, columns: &mut Vec<Column<'a>>) {
    for column in select.column.iter().flatten() {
        let type_ = column.type_.as_ref().and_then(|t| t.value.as_deref());
        columns.push((column.name.value.as_deref().unwrap_or_default(), type_));
    }

    for nested in select.select.iter().flatten() {
        select_columns(nested, columns);
    }

    if let Some(first) = select.unionAll.iter().flatten().next() {
        select_columns(first, columns);
    }
}

fn describe(columns: &[Column<'_>]) -> String {
    columns
        .iter()
        .map(|(name, type_)| type_.map_or_else(|| (*name).to_string(), |t| format!("{name}:{t}")))
        .collect::<Vec<_>>()
        .join(", ")
}

/// Refuses a `unionAll` whose branches differ in column names, their order, or (where both
/// branches give one) type. Nested selects are checked too.
pub(crate) fn check_union_columns(
    selects: &[ViewDefinitionSelect],
) -> Result<(), OperationOutcomeError> {
    for select in selects {
        check_union_columns(select.select.as_deref().unwrap_or_default())?;

        let branches = select.unionAll.as_deref().unwrap_or_default();
        check_union_columns(branches)?;

        let mut first = Vec::new();
        let Some((head, rest)) = branches.split_first() else {
            continue;
        };
        select_columns(head, &mut first);

        for branch in rest {
            let mut columns = Vec::new();
            select_columns(branch, &mut columns);

            let same = columns.len() == first.len()
                && columns.iter().zip(&first).all(|((name, type_), (n, t))| {
                    name == n && (type_.is_none() || t.is_none() || type_ == t)
                });

            if !same {
                return Err(OperationOutcomeError::error(
                    IssueType::invalid(),
                    format!(
                        "unionAll branches must have the same columns: [{}] differs from [{}]",
                        describe(&first),
                        describe(&columns)
                    ),
                ));
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::check_union_columns;
    use haste_fhir_model::r4::generated::resources::ViewDefinitionSelect;

    fn select(json: &serde_json::Value) -> Vec<ViewDefinitionSelect> {
        vec![serde_json::from_value(json.clone()).unwrap()]
    }

    fn branch(columns: &[(&str, &str)]) -> serde_json::Value {
        let columns: Vec<_> = columns
            .iter()
            .map(|(name, type_)| serde_json::json!({"name": name, "path": "id", "type": type_}))
            .collect();
        serde_json::json!({"column": columns})
    }

    fn union(branches: &[serde_json::Value]) -> Vec<ViewDefinitionSelect> {
        select(&serde_json::json!({"unionAll": branches}))
    }

    #[test]
    fn matching_branches_pass() {
        let a = branch(&[("a", "id"), ("b", "id")]);
        assert!(check_union_columns(&union(&[a.clone(), a])).is_ok());
    }

    #[test]
    fn different_names_fail() {
        let views = union(&[
            branch(&[("a", "id"), ("b", "id")]),
            branch(&[("a", "id"), ("c", "id")]),
        ]);
        assert!(check_union_columns(&views).is_err());
    }

    #[test]
    fn different_order_fails() {
        let views = union(&[
            branch(&[("a", "id"), ("b", "id")]),
            branch(&[("b", "id"), ("a", "id")]),
        ]);
        assert!(check_union_columns(&views).is_err());
    }

    #[test]
    fn different_types_fail() {
        let views = union(&[branch(&[("a", "id")]), branch(&[("a", "string")])]);
        assert!(check_union_columns(&views).is_err());
    }

    #[test]
    fn nested_union_is_checked() {
        let inner =
            serde_json::json!({"unionAll": [branch(&[("a", "id")]), branch(&[("b", "id")])]});
        let views = select(&serde_json::json!({"select": [inner]}));
        assert!(check_union_columns(&views).is_err());
    }
}
