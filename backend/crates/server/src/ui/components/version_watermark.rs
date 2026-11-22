use maud::{Markup, html};

use crate::SERVER_VERSION;

/// Faint server version shown under the login forms.
pub fn version_watermark() -> Markup {
    html! {
        p class="text-center text-xs text-ink-muted" title="Haste Health version" { "v" (SERVER_VERSION) }
    }
}
