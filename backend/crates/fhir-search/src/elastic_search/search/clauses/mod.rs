use crate::elastic_search::flatten_parameter_field_name;
use haste_fhir_model::r4::generated::resources::SearchParameter;

mod date;
mod number;
mod quantity;
mod reference;
mod string;
mod token;
mod uri;

pub use date::*;
pub use number::*;
pub use quantity::*;
pub use reference::*;
pub use string::*;
pub use token::*;
pub use uri::*;

/// Resolves the Elasticsearch field path a clause builder should query for
/// this parameter.
///
/// - System-level parameters get their own top-level field, named from the
///   parameter's (fixed, known-at-migration-time) URL - `flatten_parameter_field_name`
///   keeps a literal `.` in that URL from being misread as an object-path
///   separator.
/// - Project-level (user-submitted) parameters instead share one generic
///   `dynamic_parameters` nested field (see `migration.rs`), where the URL is
///   a plain value rather than a field name - an arbitrary user-supplied URL
///   can't safely become a new Elasticsearch field, so it's matched via a
///   `term` query on `dynamic_parameters.url` (added by the caller) instead
///   of being part of this path.
pub fn namespace_parameter(namespace: Option<&str>, search_parameter: &SearchParameter) -> String {
    match namespace {
        None => {
            let url = search_parameter.url.value.as_deref().unwrap_or("");
            flatten_parameter_field_name(url)
        }
        Some(ns) => {
            let type_ = search_parameter.type_.as_str().unwrap_or("string");
            format!("{ns}.value.{type_}")
        }
    }
}
