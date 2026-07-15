use crate::ui::components::{banner, otp_code_input, page_html};
use haste_jwt::TenantId;
use maud::{Markup, html};

#[allow(dead_code)]
pub fn totp_entry_html(
    tenant: &TenantId,
    csrf_token: &str,
    digits: usize,
    mfa_entry_route: &str,
    errors: Option<Vec<String>>,
) -> Markup {
    page_html(html! {
        (banner(tenant.as_ref(), None))
        div class="border border-brand-50 w-full bg-white   bg-white rounded-lg shadow md:mt-0 xl:p-0 text-slate-700" {
            div class="p-6 space-y-4 md:space-y-6 sm:p-8" {
                @if let Some(errors) = errors {
                    div class="mb-4" {
                        @for error in errors {
                            div class="text-red-600 text-sm" { (error) }
                        }
                    }
                }
                h1 class="text-xl font-bold leading-tight tracking-tight text-slate-900 md:text-2xl " { "Enter verification code" }
                p class="text-sm text-slate-500" { "Enter the code from your authenticator app." }
                form class="space-y-4 md:space-y-6" id="mfa-entry-form" action=(mfa_entry_route) method="POST" {
                    input type="hidden" name="csrf_token" value=(csrf_token) {}
                    (otp_code_input(digits))
                    button type="submit" class="cursor-pointer w-full text-white bg-brand-600 hover:bg-brand-500 focus:ring-4 focus:outline-none focus:ring-brand-300 font-medium rounded-lg text-sm px-5 py-2.5 text-center " { "Verify" }
                }
            }
        }
    })
}
