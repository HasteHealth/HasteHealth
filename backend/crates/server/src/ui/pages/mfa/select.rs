use crate::ui::{
    components::{TenantName, page_banner, page_html},
    styles::{CARD, CARD_BODY, CHOICE, HEADING},
};
use haste_jwt::TenantId;
use haste_repository::types::mfa::UserMFACredential;
use maud::{Markup, html};
use tower_sessions::cookie::time::format_description;

fn credential_type_label(credential_type: &str) -> &str {
    match credential_type {
        "totp" => "Authenticator app",
        other => other,
    }
}

fn credential_added_label(credential: &UserMFACredential) -> Option<String> {
    let format = format_description::parse_borrowed::<2>("[year]-[month]-[day]").ok()?;
    let formatted = credential.created_at.format(&format).ok()?;

    Some(format!("Added {formatted}"))
}

#[allow(dead_code)]
pub fn mfa_select_html(
    tenant: &TenantId,
    csrf_token: &str,
    credentials: &[UserMFACredential],
    mfa_select_route: &str,
    branding: Option<&TenantName>,
) -> Markup {
    page_html(&html! {
        (page_banner(tenant.as_ref(), None, branding))
        div class=(CARD) {
            div class=(CARD_BODY) {
                @if credentials.is_empty() {
                    span class="font-semibold leading-tight text-error" { "No MFA methods found. Please contact your administrator." }
                } @else {
                    h1 class=(HEADING) { "Choose a verification method" }
                    div class="grid grid-cols-1 gap-3" {
                        @for credential in credentials.iter() {
                            form action=(mfa_select_route) method="POST" {
                                input type="hidden" name="csrf_token" value=(csrf_token) {}
                                input type="hidden" name="credential_id" value=(credential.id.as_deref().unwrap_or("")) {}
                                button type="submit" class=(CHOICE) {
                                    (credential_type_label(&credential.credential_type))
                                    @if let Some(added_label) = credential_added_label(credential) {
                                        span class="block text-xs font-normal text-ink-subtle" { (added_label) }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    })
}
