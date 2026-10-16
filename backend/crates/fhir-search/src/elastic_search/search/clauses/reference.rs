use crate::{
    elastic_search::search::clauses::{all_of, any_of, match_query, nested},
    query::{QueryBuildError, value::ReferenceValues},
};

/// Matches any of the references: by id, and by type when one is named.
pub fn reference(
    field: &str,
    values: &ReferenceValues<'_>,
) -> Result<serde_json::Value, QueryBuildError> {
    let references = match values {
        ReferenceValues::References(references) => references,
        // A hierarchy of references isn't followed, and a target's
        // identifier isn't indexed with the reference.
        ReferenceValues::Above(_) | ReferenceValues::Below(_) | ReferenceValues::Identifier(_) => {
            return Err(QueryBuildError::unsupported_modifier(values.modifier()));
        }
    };

    Ok(any_of(
        references
            .iter()
            .map(|reference| {
                let mut filter = Vec::new();
                if let Some(resource_type) = reference.resource_type {
                    filter.push(match_query(field, "resource_type", resource_type));
                }
                filter.push(match_query(field, "id", reference.id));
                nested(field, all_of(filter, Vec::new()))
            })
            .collect(),
    ))
}
