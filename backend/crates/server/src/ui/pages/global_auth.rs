//! Pages for sign-up and login under `/auth`, before a tenant is known.

use crate::{
    auth_n::global::{
        bot_check::{HONEYPOT_FIELD, TURNSTILE_FIELD},
        email_code::{CODE_DIGITS, Purpose},
    },
    middleware::security_headers::TURNSTILE_ORIGIN,
    tenants::{TENANT_ID_MAX_LEN, TENANT_ID_MIN_LEN},
    ui::components::{banner, otp_code_input, page_html, version_watermark},
};
use maud::{Markup, html};

const CARD: &str =
    "border border-brand-50 w-full bg-white rounded-lg shadow md:mt-0 xl:p-0 text-slate-700";
const INPUT: &str = "bg-gray-50 border border-gray-300 text-slate-900 sm:text-sm rounded-lg focus:ring-brand-600 focus:border-brand-600 block w-full p-2.5";
const BUTTON: &str = "cursor-pointer w-full text-white bg-brand-600 hover:bg-brand-500 focus:ring-4 focus:outline-none focus:ring-brand-300 font-medium rounded-lg text-sm px-5 py-2.5 text-center";
const LINK: &str = "text-sm font-medium text-brand-600 hover:underline";

pub const SIGNUP_ROUTE: &str = "/auth/signup";
pub const LOGIN_ROUTE: &str = "/auth/login";
pub const VERIFY_ROUTE: &str = "/auth/verify";
pub const RESEND_ROUTE: &str = "/auth/verify/resend";
pub const TENANT_ROUTE: &str = "/auth/verify/tenant";
pub const CREATE_ROUTE: &str = "/auth/verify/create";

fn error_html(error: Option<&str>) -> Markup {
    html! {
        @if let Some(error) = error {
            div class="text-red-600 text-sm" { (error) }
        }
    }
}

/// The one-field form that starts either flow. `turnstile_site_key` adds the
/// Cloudflare widget.
pub fn email_form_html(
    purpose: Purpose,
    csrf_token: &str,
    turnstile_site_key: Option<&str>,
    error: Option<&str>,
) -> Markup {
    let (title, heading, action, switch) = match purpose {
        Purpose::Signup => (
            "Start for free",
            "Create your workspace",
            SIGNUP_ROUTE,
            html! { "Already have a workspace? " a href=(LOGIN_ROUTE) class=(LINK) { "Log in" } },
        ),
        Purpose::Login => (
            "Log in",
            "Welcome back",
            LOGIN_ROUTE,
            html! { "New to Haste Health? " a href=(SIGNUP_ROUTE) class=(LINK) { "Start for free" } },
        ),
    };

    page_html(&html! {
        (banner(title, None))
        div class=(CARD) {
            form class="p-6 space-y-4 sm:p-8" action=(action) method="POST" {
                input type="hidden" name="csrf_token" value=(csrf_token) {}
                h1 class="text-xl font-bold leading-tight tracking-tight text-slate-900 md:text-2xl" { (heading) }
                p class="text-sm text-slate-500" {
                    "Enter your email and we'll send you a " (CODE_DIGITS) "-digit code."
                }
                div {
                    label for="email" class="block mb-2 text-sm font-medium text-slate-600" { "Email address" }
                    input type="email" id="email" name="email" class=(INPUT) placeholder="name@company.com" required autocomplete="email" autofocus {}
                }
                // Honeypot: hidden from people, and labelled so that only
                // indiscriminate form fillers complete it.
                div class="hidden" aria-hidden="true" {
                    label for=(HONEYPOT_FIELD) { "Leave this field blank" }
                    input type="text" id=(HONEYPOT_FIELD) name=(HONEYPOT_FIELD) tabindex="-1" autocomplete="off" placeholder="Leave blank" {}
                }
                @if let Some(site_key) = turnstile_site_key {
                    div class="cf-turnstile" data-sitekey=(site_key) data-response-field-name=(TURNSTILE_FIELD) {}
                    script src=(format!("{TURNSTILE_ORIGIN}/turnstile/v0/api.js")) async defer {}
                }
                (error_html(error))
                button type="submit" class=(BUTTON) { "Send me a code" }
                p class="text-sm text-slate-500" { (switch) }
            }
        }
        (version_watermark())
    })
}

/// Where the code is typed in. `notice` confirms a resend.
pub fn code_entry_html(
    csrf_token: &str,
    email: &str,
    start_over_route: &str,
    notice: Option<&str>,
    error: Option<&str>,
) -> Markup {
    page_html(&html! {
        (banner("Check your email", None))
        div class=(CARD) {
            div class="p-6 space-y-4 sm:p-8" {
                h1 class="text-xl font-bold leading-tight tracking-tight text-slate-900 md:text-2xl" { "Enter your code" }
                p class="text-sm text-slate-500" {
                    "We sent a " (CODE_DIGITS) "-digit code to " span class="font-semibold text-slate-700" { (email) } ". It expires in 10 minutes."
                }
                @if let Some(notice) = notice {
                    div class="text-sm text-brand-600" { (notice) }
                }
                (error_html(error))
                form class="space-y-4" id="email-code-form" action=(VERIFY_ROUTE) method="POST" {
                    input type="hidden" name="csrf_token" value=(csrf_token) {}
                    (otp_code_input(CODE_DIGITS))
                    button type="submit" class=(BUTTON) { "Continue" }
                }
                div class="flex items-center justify-between text-sm" {
                    form action=(RESEND_ROUTE) method="POST" {
                        input type="hidden" name="csrf_token" value=(csrf_token) {}
                        button type="submit" class=(format!("cursor-pointer {LINK}")) { "Send a new code" }
                    }
                    a href=(start_over_route) class=(LINK) { "Use a different email" }
                }
            }
        }
    })
}

pub struct TenantChoice {
    pub id: String,
    pub name: Option<String>,
    pub href: String,
}

/// For a verified address with users in more than one tenant.
pub fn tenant_chooser_html(email: &str, tenants: &[TenantChoice]) -> Markup {
    page_html(&html! {
        (banner("Choose a tenant", None))
        div class=(CARD) {
            div class="p-6 space-y-4 sm:p-8" {
                p class="text-sm text-slate-500" {
                    span class="font-semibold text-slate-700" { (email) } " belongs to more than one tenant. Pick the one to open."
                }
                div class="grid grid-cols-1 gap-3" {
                    @for tenant in tenants {
                        a href=(tenant.href) class="block w-full rounded-lg border border-gray-200 bg-white px-4 py-2.5 text-center text-sm font-medium text-slate-900 transition-colors hover:bg-brand-50 hover:border-brand-200" {
                            (tenant.name.as_deref().unwrap_or(&tenant.id))
                            @if tenant.name.is_some() {
                                span class="block text-xs text-slate-400" { (tenant.id) }
                            }
                        }
                    }
                }
            }
        }
    })
}

/// For a verified address with no account: a tenant id, prefilled with a
/// suggestion, and a password.
pub fn create_workspace_html(
    csrf_token: &str,
    email: &str,
    purpose: Purpose,
    tenant_id: &str,
    error: Option<&str>,
) -> Markup {
    let (title, heading, intro) = match purpose {
        Purpose::Signup => (
            "Start for free",
            "Name your workspace",
            html! {
                span class="font-semibold text-slate-700" { (email) } " is verified. Keep the id we picked or choose your own."
            },
        ),
        Purpose::Login => (
            "Log in",
            "No workspace yet",
            html! {
                "There is no account for " span class="font-semibold text-slate-700" { (email) } ". Your email is verified, so you can create a free workspace right now."
            },
        ),
    };

    page_html(&html! {
        (banner(title, None))
        div class=(CARD) {
            form class="p-6 space-y-4 sm:p-8" action=(CREATE_ROUTE) method="POST" {
                input type="hidden" name="csrf_token" value=(csrf_token) {}
                h1 class="text-xl font-bold leading-tight tracking-tight text-slate-900 md:text-2xl" { (heading) }
                p class="text-sm text-slate-500" { (intro) }
                div {
                    label for="tenant" class="block mb-2 text-sm font-medium text-slate-600" { "Tenant id" }
                    input type="text" id="tenant" name="tenant" class=(INPUT) value=(tenant_id) required
                        minlength=(TENANT_ID_MIN_LEN) maxlength=(TENANT_ID_MAX_LEN) pattern="(?!.*--)[a-z0-9-]+"
                        autocomplete="off" autocapitalize="none" spellcheck="false" {}
                    p class="mt-1 text-xs text-slate-400" {
                        (TENANT_ID_MIN_LEN) " to " (TENANT_ID_MAX_LEN) " lowercase letters, digits and hyphens, never two hyphens in a row. It becomes part of your workspace address and cannot be changed later."
                    }
                }
                div {
                    label for="password" class="block mb-2 text-sm font-medium text-slate-600" { "Password" }
                    input type="password" id="password" name="password" class=(INPUT) placeholder="••••••••" required autocomplete="new-password" {}
                    p class="mt-1 text-xs text-slate-400" {
                        "For signing in at your workspace's login page. You can also log in with an emailed code."
                    }
                }
                (error_html(error))
                button type="submit" class=(BUTTON) { "Create my workspace" }
                p class="text-sm text-slate-500" {
                    a href=(LOGIN_ROUTE) class=(LINK) { "Use a different email" }
                }
            }
        }
    })
}
