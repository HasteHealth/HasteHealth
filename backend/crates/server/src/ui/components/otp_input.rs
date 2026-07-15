use maud::{Markup, html};

use crate::static_assets::asset_route;

/// Renders a hidden `code` field backed by a row of single-digit boxes, plus
/// the script that keeps them in sync. `form_id` must be the id of the
/// enclosing `<form>` and must be unique on the page.
pub fn otp_code_input(digits: usize) -> Markup {
    let digit_class = format!("digit_input");

    html! {
        input type="hidden" name="otp_code" id="otp_code" {}
        div class="flex items-center justify-center gap-2" {
            @for i in 0..digits {
                input
                type="text"
                inputmode="numeric"
                pattern="[0-9]*"
                maxlength="1"
                autocomplete="one-time-code"
                data-mfa-digit=(i)
                class=(format!("{digit_class} h-12 w-10 rounded-lg border border-gray-300 bg-gray-50 text-center text-xl font-semibold text-slate-900 focus:border-brand-600 focus:outline-none focus:ring-brand-600")) {}
            }
        }
        script src=(asset_route("js/otp_code.js")) {}
    }
}
