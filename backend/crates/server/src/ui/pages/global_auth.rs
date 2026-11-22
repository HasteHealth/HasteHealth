//! Pages for sign-up and login under `/auth`, before a tenant is known.

use crate::{
    auth_n::global::{
        bot_check::{HONEYPOT_FIELD, TURNSTILE_FIELD},
        email_code::{CODE_DIGITS, Purpose},
    },
    middleware::security_headers::TURNSTILE_ORIGIN,
    tenants::{TENANT_ID_MAX_LEN, TENANT_ID_MIN_LEN},
    ui::{
        components::{banner, form_errors, otp_code_input, page_html, version_watermark},
        styles::{
            BUTTON, BUTTON_SECONDARY, CARD, CARD_BODY, CHOICE, EMPHASIS, HEADING, INPUT, LABEL,
            LINK, ROW, ROW_BUTTON, ROW_BUTTON_SECONDARY, TEXT,
        },
    },
};
use haste_jwt::HOSTNAME_ID_PATTERN;
use maud::{Markup, html};

pub const SIGNUP_ROUTE: &str = "/auth/signup";
pub const LOGIN_ROUTE: &str = "/auth/login";
pub const VERIFY_ROUTE: &str = "/auth/verify";
pub const RESEND_ROUTE: &str = "/auth/verify/resend";
pub const TENANT_ROUTE: &str = "/auth/verify/tenant";
pub const DECIDE_ROUTE: &str = "/auth/verify/decide";
pub const CREATE_ROUTE: &str = "/auth/verify/create";
pub const INVITATION_ROUTE: &str = "/auth/invitation";

/// The one-field form that starts either flow. `turnstile_site_key` adds the
/// Cloudflare widget. Without `signup_enabled` the login form does not point
/// to sign-up.
pub fn email_form_html(
    purpose: Purpose,
    csrf_token: &str,
    turnstile_site_key: Option<&str>,
    signup_enabled: bool,
    error: Option<&str>,
) -> Markup {
    let (title, heading, action, switch) = match purpose {
        Purpose::Signup => (
            "Start for free",
            "Create your workspace",
            SIGNUP_ROUTE,
            Some(
                html! { "Already have a workspace? " a href=(LOGIN_ROUTE) class=(LINK) { "Log in" } },
            ),
        ),
        Purpose::Login => (
            "Log in",
            "Welcome back",
            LOGIN_ROUTE,
            signup_enabled.then(|| {
                html! { "New to Haste Health? " a href=(SIGNUP_ROUTE) class=(LINK) { "Start for free" } }
            }),
        ),
    };

    page_html(&html! {
        (banner(title, None))
        div class=(CARD) {
            form class=(CARD_BODY) action=(action) method="POST" {
                input type="hidden" name="csrf_token" value=(csrf_token) {}
                h1 class=(HEADING) { (heading) }
                p class=(TEXT) {
                    "Enter your email and we'll send you a " (CODE_DIGITS) "-digit code."
                }
                div {
                    label for="email" class=(LABEL) { "Email address" }
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
                (form_errors(error.as_slice()))
                button type="submit" class=(BUTTON) { "Send me a code" }
                @if let Some(switch) = switch {
                    p class=(TEXT) { (switch) }
                }
            }
        }
    })
}

/// `/auth/signup` on a server with sign-up turned off.
pub fn signup_closed_html() -> Markup {
    page_html(&html! {
        (banner("Sign-up is closed", None))
        div class=(CARD) {
            div class=(CARD_BODY) {
                h1 class=(HEADING) { "Sign-up is not available" }
                p class=(TEXT) {
                    "This server does not let you create a workspace yourself. Ask its administrator for an account."
                }
                p class=(TEXT) {
                    "Already have a workspace? " a href=(LOGIN_ROUTE) class=(LINK) { "Log in" }
                }
            }
        }
    })
}

/// For a verified address with no account, on a server with sign-up turned
/// off.
pub fn no_account_html(email: &str) -> Markup {
    page_html(&html! {
        (banner("Log in", None))
        div class=(CARD) {
            div class=(CARD_BODY) {
                h1 class=(HEADING) { "No account" }
                p class=(TEXT) {
                    "There is no account for " span class=(EMPHASIS) { (email) } ", and this server does not let you create a workspace yourself. Ask its administrator for an account."
                }
                p class=(TEXT) {
                    a href=(LOGIN_ROUTE) class=(LINK) { "Use a different email" }
                }
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
            div class=(CARD_BODY) {
                h1 class=(HEADING) { "Enter your code" }
                p class=(TEXT) {
                    "We sent a " (CODE_DIGITS) "-digit code to " span class=(EMPHASIS) { (email) } ". It expires in 10 minutes."
                }
                @if let Some(notice) = notice {
                    div class="text-sm text-primary" { (notice) }
                }
                (form_errors(error.as_slice()))
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
    /// Not accepted yet.
    pub invited: bool,
}

fn tenant_label(tenant: &TenantChoice) -> Markup {
    html! {
        (tenant.name.as_deref().unwrap_or(&tenant.id))
        @if tenant.name.is_some() {
            span class="block text-xs font-normal text-ink-subtle" { (tenant.id) }
        }
    }
}

/// For a verified address with several tenants or any invitations: open a
/// tenant, accept or decline an invitation, or create a tenant if it has none.
pub fn tenant_chooser_html(csrf_token: &str, email: &str, tenants: &[TenantChoice]) -> Markup {
    let (invited, accepted): (Vec<_>, Vec<_>) = tenants.iter().partition(|tenant| tenant.invited);

    page_html(&html! {
        (banner("Choose a workspace", None))
        div class=(CARD) {
            div class=(CARD_BODY) {
                @if !accepted.is_empty() {
                    div class="space-y-3" {
                        p class=(TEXT) {
                            "Pick the workspace to open as " span class=(EMPHASIS) { (email) } "."
                        }
                        @for tenant in &accepted {
                            a href=(tenant.href) class=(CHOICE) {
                                (tenant_label(tenant))
                            }
                        }
                    }
                }
                @if !invited.is_empty() {
                    div class="space-y-3" {
                        h2 class="text-base font-semibold text-ink" { "Invitations" }
                        p class=(TEXT) {
                            "These workspaces created an account for " span class=(EMPHASIS) { (email) } ". Nothing is shared with them until you accept."
                        }
                        @for tenant in &invited {
                            div class=(ROW) {
                                div class="text-sm font-medium text-ink" { (tenant_label(tenant)) }
                                form class="flex gap-2" action=(DECIDE_ROUTE) method="POST" {
                                    input type="hidden" name="csrf_token" value=(csrf_token) {}
                                    input type="hidden" name="tenant" value=(tenant.id) {}
                                    button type="submit" name="decision" value="accept" class=(ROW_BUTTON) { "Accept" }
                                    button type="submit" name="decision" value="decline" class=(ROW_BUTTON_SECONDARY) { "Decline" }
                                }
                            }
                        }
                    }
                }
                @if accepted.is_empty() {
                    p class=(TEXT) {
                        "Not here to join anyone? " a href=(CREATE_ROUTE) class=(LINK) { "Create your own workspace" }
                    }
                }
            }
        }
    })
}

/// The page behind an emailed invitation link.
pub fn invitation_html(
    csrf_token: &str,
    tenant_id: &str,
    tenant_name: &str,
    email: &str,
    code: &str,
) -> Markup {
    page_html(&html! {
        (banner("You're invited", None))
        div class=(CARD) {
            form class=(CARD_BODY) action=(INVITATION_ROUTE) method="POST" {
                input type="hidden" name="csrf_token" value=(csrf_token) {}
                input type="hidden" name="tenant" value=(tenant_id) {}
                input type="hidden" name="code" value=(code) {}
                h1 class=(HEADING) { "Join " (tenant_name) "?" }
                p class=(TEXT) {
                    span class=(EMPHASIS) { (tenant_name) }
                    @if tenant_name != tenant_id {
                        " (" (tenant_id) ")"
                    }
                    " created an account for " span class=(EMPHASIS) { (email) } " on Haste Health. Accepting logs you in there; nothing is shared with the workspace until you do."
                }
                button type="submit" name="decision" value="accept" class=(BUTTON) { "Accept and continue" }
                button type="submit" name="decision" value="decline" class=(BUTTON_SECONDARY) { "Decline" }
                p class="text-xs text-ink-subtle" {
                    "Declining removes the account. If you don't know this workspace, decline or just close this page."
                }
            }
        }
    })
}

fn notice_html(title: &str, heading: &str, text: Markup) -> Markup {
    page_html(&html! {
        (banner(title, None))
        div class=(CARD) {
            div class=(CARD_BODY) {
                h1 class=(HEADING) { (heading) }
                p class=(TEXT) { (text) }
            }
        }
    })
}

pub fn invitation_invalid_html() -> Markup {
    notice_html(
        "Invitation",
        "This link no longer works",
        html! {
            "It may have expired or been used already. "
            a href=(LOGIN_ROUTE) class=(LINK) { "Log in with your email" }
            " to see the invitations waiting for you."
        },
    )
}

pub fn invitation_accepted_already_html() -> Markup {
    notice_html(
        "Invitation",
        "Already accepted",
        html! {
            "This invitation was accepted before. "
            a href=(LOGIN_ROUTE) class=(LINK) { "Log in" }
            " to open the workspace."
        },
    )
}

pub fn invitation_declined_html(tenant_name: &str) -> Markup {
    notice_html(
        "Invitation",
        "Invitation declined",
        html! {
            span class=(EMPHASIS) { (tenant_name) } " no longer has an account for you. You can close this page."
        },
    )
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
                span class=(EMPHASIS) { (email) } " is verified. Keep the id we picked or choose your own."
            },
        ),
        Purpose::Login => (
            "Log in",
            "No workspace yet",
            html! {
                "There is no account for " span class=(EMPHASIS) { (email) } ". Your email is verified, so you can create a free workspace right now."
            },
        ),
    };

    page_html(&html! {
        (banner(title, None))
        div class=(CARD) {
            form class=(CARD_BODY) action=(CREATE_ROUTE) method="POST" {
                input type="hidden" name="csrf_token" value=(csrf_token) {}
                h1 class=(HEADING) { (heading) }
                p class=(TEXT) { (intro) }
                div {
                    label for="tenant" class=(LABEL) { "Tenant id" }
                    input type="text" id="tenant" name="tenant" class=(INPUT) value=(tenant_id) required
                        minlength=(TENANT_ID_MIN_LEN) maxlength=(TENANT_ID_MAX_LEN) pattern=(HOSTNAME_ID_PATTERN)
                        autocomplete="off" autocapitalize="none" spellcheck="false" {}
                    p class="mt-1 text-xs text-ink-subtle" {
                        (TENANT_ID_MIN_LEN) " to " (TENANT_ID_MAX_LEN) " lowercase letters, digits and single hyphens between them. It becomes part of your workspace address and cannot be changed later."
                    }
                }
                div {
                    label for="password" class=(LABEL) { "Password" }
                    input type="password" id="password" name="password" class=(INPUT) placeholder="••••••••" required autocomplete="new-password" {}
                    p class="mt-1 text-xs text-ink-subtle" {
                        "For signing in at your workspace's login page. You can also log in with an emailed code."
                    }
                }
                (form_errors(error.as_slice()))
                button type="submit" class=(BUTTON) { "Create my workspace" }
                p class=(TEXT) {
                    a href=(LOGIN_ROUTE) class=(LINK) { "Use a different email" }
                }
            }
        }
    })
}
