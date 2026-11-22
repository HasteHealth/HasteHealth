use maud::{Markup, html};

/// The reasons a form was rejected, in a panel. Renders nothing when there
/// are none.
pub fn form_errors<S: AsRef<str>>(errors: &[S]) -> Markup {
    html! {
        @if !errors.is_empty() {
            div class="rounded-lg border border-error-line bg-error-soft p-3 text-sm text-error" role="alert" {
                @for error in errors {
                    div { (error.as_ref()) }
                }
            }
        }
    }
}
