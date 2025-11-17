use crate::{
    auth_n::oidc::routes::{federated::FederatedInitiate, route_string::oidc_route_string},
    ui::components::{banner, page_html},
};
use haste_fhir_model::r4::generated::resources::{ClientApplication, IdentityProvider};
use haste_jwt::{ProjectId, TenantId};
use maud::{Markup, html};
use std::borrow::Cow;

pub fn login_form_html(
    tenant: &TenantId,
    project: &haste_fhir_model::r4::generated::resources::Project,
    identity_providers: Option<&Vec<IdentityProvider>>,
    client_app: &ClientApplication,
    login_route: &str,
    errors: Option<Vec<String>>,
) -> Markup {
    let project_id = project.id.clone().map(|id| ProjectId::new(id)).unwrap();
    let project_name = project
        .name
        .value
        .as_ref()
        .map(|s| Cow::Borrowed(s.as_str()))
        .unwrap_or_else(|| Cow::Owned(project_id.as_ref().to_string()));
    let password_reset_route =
        oidc_route_string(tenant, &project_id, "interactions/password-reset");
    let password_reset_route_str = password_reset_route
        .to_str()
        .expect("Could not create password reset route.");
    let client_name = client_app
        .name
        .value
        .as_ref()
        .map(|s| Cow::Borrowed(s))
        .unwrap_or_else(|| {
            Cow::Owned(
                client_app
                    .id
                    .clone()
                    .unwrap_or_else(|| "unknown client".to_string()),
            )
        });

    page_html(html! {
        (banner(tenant, Some(&project_name)))
        div class="w-full bg-white rounded-lg shadow md:mt-0 xl:p-0 sm:max-w-md text-slate-700" {
            div class="p-6 space-y-4 md:space-y-6 sm:p-8" {
                // div {}
                // div {}
                @if let Some(errors) = errors {
                    div class="mb-4" {
                        @for error in errors {
                            div class="text-red-600 text-sm" { (error) }
                        }
                    }
                }
                h1 class="text-xl font-bold leading-tight tracking-tight text-gray-900 md:text-2xl " { "Sign in to " span class="underline text-slate-500 " {(client_name)} }
                form class="space-y-4 md:space-y-6" action=(login_route) method="POST" {
                    div {
                        label for="email" class="block mb-2 text-sm font-medium text-gray-900 " { "Your email" }
                        input type="email" id="email" class="bg-gray-50 border border-gray-300 text-gray-900 sm:text-sm rounded-lg focus:ring-orange-600 focus:border-orange-600 block w-full p-2.5 " placeholder="name@company.com" required name="email" value="" {}
                    }
                    div class="space-y-2" {
                        div {
                            label for="password" class="block mb-2 text-sm font-medium text-gray-900" { "Password" }
                            input type="password" id="password" placeholder="••••••••" class="bg-gray-50 border border-gray-300 text-gray-900 sm:text-sm rounded-lg focus:ring-orange-600 focus:border-orange-600 block w-full p-2.5" required name="password" {}
                        }
                        div class="flex items-center justify-between" {
                            a href=(password_reset_route_str) class="text-sm font-medium text-orange-600 hover:underline " { "Forgot password?" }
                        }
                    }
                    button type="submit" class="w-full text-white bg-orange-600 hover:bg-orange-700 focus:ring-4 focus:outline-none focus:ring-orange-300 font-medium rounded-lg text-sm px-5 py-2.5 text-center " { "Sign in" }
                }

                @if let Some(identity_providers) = identity_providers {
                    div class="mb-4 space-y-2" {
                        @for idp in identity_providers {
                            a href=(login_route.replace("/interactions/login", &FederatedInitiate{identity_provider_id: idp.id.clone().unwrap_or_default()}.to_string())) class="space-x-2 flex content-center justify-center text-white bg-slate-600 hover:bg-slate-700 focus:ring-4 focus:outline-none focus:ring-slate-300 font-medium rounded-lg text-sm px-5 py-2.5 text-center " {
                                div { (format!("Sign in with {}", idp.name.value.as_ref().unwrap_or(&"Unknown".to_string()))) }
                            }
                        }
                    }
                }


            }
        }
    })
}
