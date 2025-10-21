use crate::{
    auth_n::oidc::{routes::route_string::oidc_route_string, ui::app_header},
    server::asset_route,
};
use maud::{Markup, html};
use oxidized_fhir_model::r4::generated::resources::ClientApplication;
use oxidized_repository::types::{ProjectId, TenantId};
use std::borrow::Cow;

pub fn login_form_html(
    tenant: &TenantId,
    project: &oxidized_fhir_model::r4::generated::resources::Project,
    client_app: &ClientApplication,
    login_route: &str,
) -> Markup {
    let project_id = project.id.clone().map(|id| ProjectId::new(id)).unwrap();
    let project_name = project
        .name
        .as_ref()
        .and_then(|name| name.value.as_ref())
        .map(|s| Cow::Borrowed(s.as_str()))
        .unwrap_or_else(|| Cow::Owned(project_id.as_ref().to_string()));
    let password_reset_route = oidc_route_string(tenant, &project_id, "password-reset");
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

    html! {
        head {
            meta charset="utf-8" {}
            meta name="viewport" content="width=device-width, initial-scale=1" {}
            link rel="preload" as="image" href=(asset_route("img/logo.svg")) {}
            title { "Oxidized Health" }
            link rel="icon" href=(asset_route("img/logo.svg")) {}
            link rel="stylesheet" href=(asset_route("css/app.css")) {}
        }
        body {
            section class="bg-gray-50  h-screen" {
                div class="flex flex-col items-center justify-center px-6 py-8 space-y-4 mx-auto md:h-screen lg:py-0" {
                    div class="flex flex-col items-center justify-center space-y-1" {
                        a href="#" class="flex items-center text-2xl font-semibold text-gray-900" {
                            img class="w-8 h-8 mr-2" src=(asset_route("img/logo.svg")) alt="logo" {}
                            "Oxidized Health"
                        }
                        div class="flex space-x-1 items-center justify-center text-sm text-slate-400" {
                            div {
                                span class="font-bold" {
                                    (tenant.as_ref())
                                }
                            }
                            div {
                                span {
                                    (project_name)
                                }
                            }
                        }
                    }
                    div class="w-full bg-white rounded-lg shadow md:mt-0 xl:p-0 sm:max-w-md text-slate-700" {
                        div class="p-6 space-y-4 md:space-y-6 sm:p-8" {
                            // div {}
                            // div {}
                            h1 class="text-xl font-bold leading-tight tracking-tight text-gray-900 md:text-2xl " { "Sign in to " span class="underline text-slate-500 " {(client_name)} }
                            form class="space-y-4 md:space-y-6" action=(login_route) method="POST" {
                                div {
                                    label for="email" class="block mb-2 text-sm font-medium text-gray-900 " { "Your email" }
                                    input type="email" id="email" class="bg-gray-50 border border-gray-300 text-gray-900 sm:text-sm rounded-lg focus:ring-teal-600 focus:border-teal-600 block w-full p-2.5 " placeholder="name@company.com" required name="email" value="" {}
                                }
                                div {
                                    label for="password" class="block mb-2 text-sm font-medium text-gray-900" { "Password" }
                                    input type="password" id="password" placeholder="••••••••" class="bg-gray-50 border border-gray-300 text-gray-900 sm:text-sm rounded-lg focus:ring-teal-600 focus:border-teal-600 block w-full p-2.5" required name="password" {}
                                }
                                div class="flex items-center justify-between" {
                                    div class="flex items-start" {
                                        div class="flex items-center h-5" {
                                            input id="remember" aria-describedby="remember" type="checkbox" class="w-4 h-4" required {}
                                        }
                                        div class="ml-3 text-sm" {
                                            label for="remember" class="text-gray-500" { "Remember me" }
                                        }
                                    }
                                    a href=(password_reset_route_str) class="text-sm font-medium text-teal-600 hover:underline " { "Forgot password?" }
                                }
                                button type="submit" class="w-full text-white bg-teal-600 hover:bg-teal-700 focus:ring-4 focus:outline-none focus:ring-teal-300 font-medium rounded-lg text-sm px-5 py-2.5 text-center " { "Sign in" }
                            }
                            div class="mt-4 space-y-2" {}
                        }
                    }
                }
            }
        }

    }
}
