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

pub fn mfa_admin_html(
    tenant: &TenantId,
    csrf_token: &str,
    credentials: &[UserMFACredential],
    create_route: &str,
    delete_route: &str,
    activate_route: &str,
) -> Markup {
    page_html(html! {
        (banner(tenant.as_ref(), None))
        div class="border border-brand-50 w-full bg-white   bg-white rounded-lg shadow md:mt-0 xl:p-0 text-slate-700" {
            div class="p-6 space-y-4 md:space-y-6 sm:p-8" {
                h1 class="text-xl font-bold leading-tight tracking-tight text-slate-900 md:text-2xl " { "Manage MFA methods" }
                p class="text-sm text-slate-500" { "Create a new authenticator method or remove old ones." }

                form action=(create_route) method="POST" {
                    input type="hidden" name="csrf_token" value=(csrf_token) {}
                    button type="submit"
                    class="cursor-pointer w-full text-white bg-brand-600 hover:bg-brand-500 focus:ring-4 focus:outline-none focus:ring-brand-300 font-medium rounded-lg text-sm px-5 py-2.5 text-center" {
                        "Create new MFA credential"
                    }
                }

                @if credentials.is_empty() {
                    div class="rounded-lg border border-gray-200 bg-gray-50 p-4" {
                        span class="text-sm text-slate-600" { "No MFA credentials configured yet." }
                    }
                } @else {
                    div class="grid grid-cols-1 gap-3" {
                        @for credential in credentials.iter() {
                            div class="flex items-center justify-between gap-3 rounded-lg border border-gray-200 bg-white px-4 py-2.5" {
                                div class="min-w-0" {
                                    span class="block text-sm font-medium text-slate-900" { (credential_type_label(&credential.credential_type)) }
                                    @if let Some(added_label) = credential_added_label(credential) {
                                        span class="block text-xs text-slate-400" { (added_label) }
                                    }
                                    @if !credential.is_active {
                                        span class="block text-xs text-amber-600" { "Not activated yet" }
                                    }
                                }
                                div class="flex items-center gap-2" {
                                    @if !credential.is_active {
                                        a href=(format!("{}/{}", activate_route, credential.id.as_deref().unwrap_or("")))
                                        class="rounded-lg border border-brand-200 px-3 py-1.5 text-sm font-medium text-brand-600 transition-colors hover:bg-brand-50" {
                                            "Activate"
                                        }
                                    }
                                    form action=(format!("{}/{}", delete_route, credential.id.as_deref().unwrap_or(""))) method="POST" {
                                        input type="hidden" name="csrf_token" value=(csrf_token) {}
                                        input type="hidden" name="credential_id" value=(credential.id.as_deref().unwrap_or("")) {}
                                        button type="submit"
                                        class="cursor-pointer rounded-lg border border-red-200 px-3 py-1.5 text-sm font-medium text-red-600 transition-colors hover:bg-red-50" {
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
