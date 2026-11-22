use crate::ui::{
    components::{TenantName, page_banner, page_html},
    styles::{BUTTON, CARD, CARD_BODY, HEADING, ROW, ROW_BUTTON, ROW_BUTTON_DANGER, TEXT},
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

pub fn mfa_admin_html(
    tenant: &TenantId,
    csrf_token: &str,
    credentials: &[UserMFACredential],
    create_route: &str,
    delete_route: &str,
    activate_route: &str,
    branding: Option<&TenantName>,
) -> Markup {
    page_html(&html! {
        (page_banner(tenant.as_ref(), None, branding))
        div class=(CARD) {
            div class=(CARD_BODY) {
                h1 class=(HEADING) { "Manage MFA methods" }
                p class=(TEXT) { "Create a new authenticator method or remove old ones." }

                form action=(create_route) method="POST" {
                    input type="hidden" name="csrf_token" value=(csrf_token) {}
                    button type="submit" class=(BUTTON) { "Create new MFA credential" }
                }

                @if credentials.is_empty() {
                    div class="rounded-lg border border-line bg-surface-muted p-4 text-sm text-ink-body" {
                        "No MFA credentials configured yet."
                    }
                } @else {
                    div class="grid grid-cols-1 gap-3" {
                        @for credential in credentials.iter() {
                            div class=(ROW) {
                                div class="min-w-0" {
                                    span class="block text-sm font-medium text-ink" { (credential_type_label(&credential.credential_type)) }
                                    @if let Some(added_label) = credential_added_label(credential) {
                                        span class="block text-xs text-ink-subtle" { (added_label) }
                                    }
                                    @if !credential.is_active {
                                        span class="block text-xs text-warning" { "Not activated yet" }
                                    }
                                }
                                div class="flex items-center gap-2" {
                                    @if !credential.is_active {
                                        a href=(format!("{}/{}", activate_route, credential.id.as_deref().unwrap_or("")))
                                        class=(ROW_BUTTON) {
                                            "Activate"
                                        }
                                    }
                                    form action=(format!("{}/{}", delete_route, credential.id.as_deref().unwrap_or(""))) method="POST" {
                                        input type="hidden" name="csrf_token" value=(csrf_token) {}
                                        input type="hidden" name="credential_id" value=(credential.id.as_deref().unwrap_or("")) {}
                                        button type="submit" class=(ROW_BUTTON_DANGER) {
                                            "Delete"
                                        }
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
