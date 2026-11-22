use crate::{
    auth_n::oidc::routes::{federated::FederatedInitiate, route_string::oidc_route_string},
    ui::{
        components::{TenantContext, form_errors, page_banner, page_html, version_watermark},
        styles::{BUTTON, BUTTON_SECONDARY, CARD, CARD_BODY, HEADING, INPUT, LABEL, LINK},
    },
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
        div class=(CARD) {
            div class=(CARD_BODY) {
                (form_errors(errors.as_deref().unwrap_or_default()))
                h1 class=(HEADING) { "Sign in to " span class="underline text-ink-muted" { (client_name) } }
                form class="space-y-4 md:space-y-6" action=(login_route) method="POST" {
                    input type="hidden" name="csrf_token" value=(csrf_token) {}
                    div {
                        label for="email" class=(LABEL) { "Your email" }
                        input type="email" id="email" class=(INPUT) placeholder="name@company.com" required name="email" value="" {}
                    }
                    div class="space-y-2" {
                        div {
                            label for="password" class=(LABEL) { "Password" }
                            input type="password" id="password" placeholder="••••••••" class=(INPUT) required name="password" {}
                        }
                        div class="text-sm" {
                            a href=(password_reset_route_str) class=(LINK) { "Forgot password?" }
                        }
                    }
                    button type="submit" class=(BUTTON) { "Sign in" }
                }
                @if let Some(identity_providers) = identity_providers {
                    div class="space-y-2" {
                        @for idp in identity_providers {
                            a href=(login_route.replace("/interactions/login", &FederatedInitiate{identity_provider_id: idp.id.clone().unwrap_or_default()}.to_string())) class=(BUTTON_SECONDARY) {
                                (format!("Sign in with {}", idp.name.value.as_ref().unwrap_or(&"Unknown".to_string())))
                            }
                        }
                    }
                }
            }
        }
        (version_watermark())
    })
}
