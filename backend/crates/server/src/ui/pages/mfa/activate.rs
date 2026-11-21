use crate::ui::{
    components::{TenantContext, form_errors, otp_code_input, page_banner, page_html},
    styles::{BUTTON, CARD, CARD_BODY, HEADING, TEXT},
};
use maud::{Markup, html};

pub fn mfa_activate_html(
    context: &TenantContext,
    csrf_token: &str,
    credential_id: &str,
    qr_code_img_src: &str,
    digits: usize,
    activate_route: &str,
    errors: Option<Vec<String>>,
) -> Markup {
    page_html(&html! {
        (page_banner(context.tenant.as_ref(), None, Some(&context.branding)))
        div class=(CARD) {
            div class=(CARD_BODY) {
                (form_errors(errors.as_deref().unwrap_or_default()))
                h1 class=(HEADING) { "Set up an authenticator app" }
                p class=(TEXT) { "Scan the QR code below with your authenticator app, then enter the code it generates to confirm setup." }
                div class="flex items-center justify-center" {
                    img src=(qr_code_img_src) alt="MFA QR code" class="h-40 w-40 rounded-lg border border-line p-2" {}
                }
                form class="space-y-4 md:space-y-6" id="mfa_create_form" action=(activate_route) method="POST" {
                    input type="hidden" name="csrf_token" value=(csrf_token) {}
                    input type="hidden" name="credential_id" value=(credential_id) {}
                    (otp_code_input(digits))
                    button type="submit" class=(BUTTON) { "Activate" }
                }
            }
        }
    })
}
