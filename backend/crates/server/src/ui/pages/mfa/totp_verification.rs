use crate::ui::{
    components::{TenantName, form_errors, otp_code_input, page_banner, page_html},
    styles::{BUTTON, CARD, CARD_BODY, HEADING, TEXT},
};
use haste_jwt::TenantId;
use maud::{Markup, html};

pub fn totp_entry_html(
    tenant: &TenantId,
    csrf_token: &str,
    digits: usize,
    mfa_entry_route: &str,
    errors: Option<Vec<String>>,
    branding: Option<&TenantName>,
) -> Markup {
    page_html(&html! {
        (page_banner(tenant.as_ref(), None, branding))
        div class=(CARD) {
            div class=(CARD_BODY) {
                (form_errors(errors.as_deref().unwrap_or_default()))
                h1 class=(HEADING) { "Enter verification code" }
                p class=(TEXT) { "Enter the code from your authenticator app." }
                form class="space-y-4 md:space-y-6" id="mfa-entry-form" action=(mfa_entry_route) method="POST" {
                    input type="hidden" name="csrf_token" value=(csrf_token) {}
                    (otp_code_input(digits))
                    button type="submit" class=(BUTTON) { "Verify" }
                }
            }
        }
    })
}
