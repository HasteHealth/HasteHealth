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

pub fn namespace_parameter(namespace: Option<&str>, search_parameter: &SearchParameter) -> String {
    namespace
        .map(|ns| {
            ns.to_string()
                + "."
                + search_parameter
                    .url
                    .value
                    .as_ref()
                    .map(|s| s.as_str())
                    .unwrap_or("")
        })
        .unwrap_or_else(|| {
            search_parameter
                .url
                .value
                .as_ref()
                .map(|s| s.as_str())
                .unwrap_or("")
                .to_string()
        })
}
