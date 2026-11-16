use haste_fhir_model::r4::generated::resources::ClientApplication;
use maud::{Markup, html};
use std::borrow::Cow;

pub fn client_app_header_html(client_app: &ClientApplication) -> Markup {
    let client_name = client_app
        .name
        .value
        .as_ref()
        .map(Cow::Borrowed)
        .unwrap_or(Cow::Owned("Unnamed Client".to_string()));

    html! {
        div class="flex flex-col justify-center items-center gap-3 mb-2" {
            div class="flex justify-center items-center w-12 h-12 rounded-none border border-[var(--auth-line)] bg-[var(--auth-surface-raised)] font-mono text-base font-semibold text-[var(--auth-signal-ink)]" {
                (client_name.chars().next().unwrap_or('U').to_uppercase())
            }
            div class="font-mono text-base font-semibold tracking-wide text-[var(--auth-ink)]" {
                (client_name)
            }
        }
    }
}
