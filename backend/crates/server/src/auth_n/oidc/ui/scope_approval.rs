use crate::{
    auth_n::oidc::{
        routes::{route_string::oidc_route_string, scope::ScopeForm},
        ui::app_header,
    },
    server::asset_route,
};
use maud::{Markup, html};
use oxidized_fhir_model::r4::generated::resources::ClientApplication;
use oxidized_repository::types::{ProjectId, TenantId};
use std::borrow::Cow;

fn exclamation_point() -> Markup {
    html! {
        svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20" fill="currentColor" aria-hidden="true" data-slot="icon" class="w-6 h-6 text-gray-300" {
            path fill-rule="evenodd" d="M18 10a8 8 0 1 1-16 0 8 8 0 0 1 16 0Zm-8-5a.75.75 0 0 1 .75.75v4.5a.75.75 0 0 1-1.5 0v-4.5A.75.75 0 0 1 10 5Zm0 10a1 1 0 1 0 0-2 1 1 0 0 0 0 2Z" clip-rule="evenodd" {}
        }
    }
}

pub fn scope_approval_html(
    tenant: &TenantId,
    project: &oxidized_fhir_model::r4::generated::resources::Project,
    client_application: &ClientApplication,
    authorization_info: &ScopeForm,
) -> Markup {
    let project_id = project.id.clone().map(|id| ProjectId::new(id)).unwrap();
    let project_name = project
        .name
        .as_ref()
        .and_then(|name| name.value.as_ref())
        .map(|s| Cow::Borrowed(s.as_str()))
        .unwrap_or_else(|| Cow::Owned(project_id.as_ref().to_string()));

    let scope_route = oidc_route_string(tenant, &project_id, "auth/scope");
    let scope_route_str = scope_route.to_str().expect("Could not create scope route");

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
                        div class="w-full bg-white rounded-lg shadow  md:mt-0  xl:p-0  sm:max-w-md" {
                            div class="p-6 space-y-4 md:space-y-6 sm:p-8" {
                                (app_header::app_header_html(client_application))
                                div {
                                    span class="text-sm text-gray-500" {
                                        "The above app is requesting the following permissions. Please review and either consent or deny access for the app."
                                    }
                                }
                                div class="max-h-72 overflow-auto" {
                                    table class="border-collapse  list-inside list-disc w-full" {
                                        tbody {
                                            @for s in authorization_info.scope.0.iter() {
                                                tr class="border border-gray-200"{
                                                    td class="p-4" {
                                                        (String::from(s.clone()))
                                                    }
                                                    td {
                                                        div class="items-center justify-center flex" {
                                                            (exclamation_point())
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                                div class="justify-center items-center flex space-x-4" {
                                    form action=(scope_route_str) method="POST" {
                                        input readonly="" class="hidden" type="text" name="client_id" value=(authorization_info.client_id) {}
                                        input readonly="" class="hidden" type="text" name="response_type" value=(authorization_info.response_type) {}
                                        input readonly="" class="hidden" type="text" name="state" value=(authorization_info.state)  {}
                                        input readonly="" class="hidden" type="text" name="code_challenge" value=(authorization_info.code_challenge)  {}
                                        input readonly="" class="hidden" type="text" name="code_challenge_method" value=(authorization_info.code_challenge_method) {}
                                        input readonly="" class="hidden" type="text" name="scope" value=(String::from(authorization_info.scope.clone())) {}
                                        input readonly="" class="hidden" type="text" name="redirect_uri" value=(authorization_info.redirect_uri) {}
                                        input readonly="" class="hidden" type="checkbox" name="accept" checked {}
                                        button type="submit" class="cursor-pointer w-full text-white bg-teal-600 hover:bg-teal-700 focus:ring-4 focus:outline-none focus:ring-teal-300 font-medium rounded-lg text-sm px-5 py-2.5 text-center dark:bg-teal-600 dark:hover:bg-teal-700 dark:focus:ring-teal-800" {
                                            "Allow"
                                        }
                                    }

                                    form action=(scope_route_str) method="POST" {
                                        input readonly="" class="hidden" type="text" name="client_id" value=(authorization_info.client_id) {}
                                        input readonly="" class="hidden" type="text" name="response_type" value=(authorization_info.response_type) {}
                                        input readonly="" class="hidden" type="text" name="state" value=(authorization_info.state)  {}
                                        input readonly="" class="hidden" type="text" name="code_challenge" value=(authorization_info.code_challenge)  {}
                                        input readonly="" class="hidden" type="text" name="code_challenge_method" value=(authorization_info.code_challenge_method) {}
                                        input readonly="" class="hidden" type="text" name="scope" value=(String::from(authorization_info.scope.clone())) {}
                                        input readonly="" class="hidden" type="text" name="redirect_uri" value=(authorization_info.redirect_uri) {}
                                        input readonly="" class="hidden" type="checkbox" name="accept" {}
                                        button type="submit" class="cursor-pointer w-full text-gray-900 bg-gray-100 hover:bg-gray-200 focus:ring-4 focus:outline-none  font-medium rounded-lg text-sm px-5 py-2.5 text-center dark:text-white dark:bg-gray-600 dark:hover:bg-gray-700 dark:focus:ring-gray-800" {
                                            "Deny"
                                        }
                                    }
                                }
                        }
                    }
                }
            }
        }
    }
}
