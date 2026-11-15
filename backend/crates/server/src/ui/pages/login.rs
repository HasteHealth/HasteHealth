use crate::{
    auth_n::oidc::routes::{federated::FederatedInitiate, route_string::oidc_route_string},
    ui::components::{TenantContext, page_banner, page_html, version_watermark},
};
use haste_fhir_model::r4::generated::resources::{ClientApplication, IdentityProvider};
use haste_jwt::ProjectId;
use maud::{Markup, html};
use std::borrow::Cow;

pub fn login_form_html(
    context: &TenantContext,
    project: &haste_fhir_model::r4::generated::resources::Project,
    csrf_token: &str,
    identity_providers: Option<&Vec<IdentityProvider>>,
    client_app: &ClientApplication,
    login_route: &str,
    errors: Option<Vec<String>>,
) -> Markup {
    let project_id = project
        .id
        .clone()
        .and_then(|id| ProjectId::new(id).ok())
        .expect("a stored project has a valid id");
    let project_name = project.name.value.as_ref().map_or_else(
        || Cow::Owned(project_id.as_ref().to_string()),
        |s| Cow::Borrowed(s.as_str()),
    );
    let password_reset_route =
        oidc_route_string(&context.tenant, &project_id, "interactions/password-reset");
    let password_reset_route_str = password_reset_route
        .to_str()
        .expect("Could not create password reset route.");
    let client_name = client_app.name.value.as_ref().map_or_else(
        || {
            Cow::Owned(
                client_app
                    .id
                    .clone()
                    .unwrap_or_else(|| "unknown client".to_string()),
            )
        },
        Cow::Borrowed,
    );

    page_html(&html! {
        (page_banner(context.tenant.as_ref(), Some(&project_name), Some(&context.branding)))
        div class="border border-[var(--auth-line)] w-full bg-[var(--auth-surface)] text-[var(--auth-ink)] p-6 space-y-6 shadow-2xl" {
            @if let Some(errors) = errors {
                div class="p-3 bg-[var(--auth-error-soft)] border border-[var(--auth-error-line)] text-[var(--auth-error)] text-xs font-mono" {
                    @for error in errors {
                        div { (error) }
                    }
                }
            }
            div class="space-y-1.5" {
                h1 class="text-base font-mono font-semibold tracking-wide text-[var(--auth-ink)]" {
                    "Sign in to " span class="text-[var(--auth-signal-ink)]" { (client_name) }
                }
                p class="text-xs text-[var(--auth-ink-muted)]" { "Enter your clinical platform credentials to access the workspace." }
            }
            form class="space-y-4" action=(login_route) method="POST" {
                input type="hidden" name="csrf_token" value=(csrf_token) {}
                div class="space-y-1.5" {
                    label for="email" class="block font-mono text-xs uppercase tracking-wider text-[var(--auth-ink-muted)]" { "Your email" }
                    input type="email" id="email" class="w-full bg-[var(--auth-surface-raised)] border border-[var(--auth-line-strong)] text-[var(--auth-ink)] text-sm px-3 py-2 outline-none focus:border-[var(--auth-signal)] transition-colors" placeholder="name@company.com" required name="email" value="" {}
                }
                div class="space-y-1.5" {
                    div class="flex items-center justify-between" {
                        label for="password" class="block font-mono text-xs uppercase tracking-wider text-[var(--auth-ink-muted)]" { "Password" }
                        a href=(password_reset_route_str) class="text-xs font-mono text-[var(--auth-signal-ink)] hover:underline" { "Forgot password?" }
                    }
                    input type="password" id="password" placeholder="••••••••" class="w-full bg-[var(--auth-surface-raised)] border border-[var(--auth-line-strong)] text-[var(--auth-ink)] text-sm px-3 py-2 outline-none focus:border-[var(--auth-signal)] transition-colors" required name="password" {}
                }
                button type="submit" class="w-full mt-2 cursor-pointer text-[var(--auth-on-signal)] bg-[var(--auth-signal)] hover:bg-[var(--auth-signal-hover)] font-mono text-xs uppercase tracking-wider font-semibold py-2.5 px-4 transition-colors text-center" { "Sign In" }
            }

            @if let Some(identity_providers) = identity_providers {
                div class="pt-4 border-t border-[var(--auth-line)] space-y-2" {
                    @for idp in identity_providers {
                        a href=(login_route.replace("/interactions/login", &FederatedInitiate{identity_provider_id: idp.id.clone().unwrap_or_default()}.to_string())) class="space-x-2 flex items-center justify-center text-[var(--auth-ink)] bg-[var(--auth-surface-raised)] border border-[var(--auth-line-strong)] hover:border-[var(--auth-signal)] font-mono text-xs py-2 px-4 transition-colors" {
                            div { (format!("Sign in with {}", idp.name.value.as_ref().unwrap_or(&"Unknown".to_string()))) }
                        }
                    }
                }
            }
        }
        (version_watermark())
    })
}
