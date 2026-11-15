use maud::{Markup, html};

use crate::SERVER_VERSION;

/// Faint server version shown under the login forms.
pub fn version_watermark() -> Markup {
    html! {
        p class="text-center font-mono text-xs text-[#525a66] mt-4" title="Haste Health version" { "v" (SERVER_VERSION) }
    }
}
