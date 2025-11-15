use haste_fhir_model::r4::generated::resources::ClientApplication;
use maud::{Markup, html};
use std::borrow::Cow;

pub fn client_app_header_html(client_app: &ClientApplication) -> Markup {
    let client_name = client_app
        .name
        .value
        .as_ref()
        .map(|s| Cow::Borrowed(s))
        .unwrap_or(Cow::Owned("Unnamed Client".to_string()));

    html! {
        div class="flex flex-col justify-center items-center text-2xl font-semibold text-gray-900  space-y-2" {
            div {
                div class="flex  justify-center items-center w-12 h-12 rounded-full bg-green-100 text-green-800 " {
                    div {(client_name.chars().next().unwrap_or('U').to_uppercase())}
                }
            }
            div {(client_name)}
        }
    }
}
