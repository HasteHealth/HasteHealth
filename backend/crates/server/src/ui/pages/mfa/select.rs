use crate::ui::components::{banner, page_html};
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
) -> Markup {
    page_html(html! {
        (banner(tenant.as_ref(), None))
        div class="border border-brand-50 w-full bg-white   bg-white rounded-lg shadow md:mt-0 xl:p-0 text-slate-700" {
            @if credentials.is_empty() {
                div class="p-6 space-y-4 md:space-y-6 sm:p-8" {
                    span class="font-semibold leading-tight text-red-600 text-md " { "No MFA methods found. Please contact your administrator." }
                }
            } @else {
                div class="p-6 space-y-4 md:space-y-6 sm:p-8" {
                    h1 class="text-xl font-bold leading-tight tracking-tight text-slate-900 md:text-2xl " { "Choose a verification method" }
                    div class="space-y-4 md:space-y-6" {
                        div class="grid grid-cols-1 gap-3" {
                            @for credential in credentials.iter() {
                                form action=(mfa_select_route) method="POST" {
                                    input type="hidden" name="csrf_token" value=(csrf_token) {}
                                    input type="hidden" name="credential_id" value=(credential.id.as_deref().unwrap_or("")) {}
                                    button type="submit"
                                    class="cursor-pointer flex w-full flex-col items-start rounded-lg border border-gray-200 bg-white px-4 py-2.5 text-left text-sm font-medium text-slate-900 transition-colors hover:bg-brand-50 hover:border-brand-200" {
                                        span { (credential_type_label(&credential.credential_type)) }
                                        @if let Some(added_label) = credential_added_label(credential) {
                                            span class="text-xs font-normal text-slate-400" { (added_label) }
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
