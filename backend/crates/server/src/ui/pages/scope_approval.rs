use crate::{
    auth_n::oidc::routes::{
        route_string::oidc_route_string,
        scope::{ScopeForm, is_fixed_scope},
    },
    ui::components::{TenantName, client_app_header_html, page_banner, page_html},
};
use haste_fhir_model::r4::generated::resources::ClientApplication;
use haste_jwt::{ProjectId, TenantId};
use maud::{Markup, html};
use std::borrow::Cow;

const CHECKBOX: &str =
    "w-4 h-4 rounded border-gray-300 text-brand-600 focus:ring-brand-500 disabled:opacity-60";

/// The consent page: one checkbox per requested scope, all checked. The user
/// unchecks what the app should not get, then allows or denies. The scopes
/// [`is_fixed_scope`] names stay checked.
pub fn scope_approval_html(
    tenant: &TenantId,
    project: &haste_fhir_model::r4::generated::resources::Project,
    client_application: &ClientApplication,
    authorization_info: &ScopeForm,
    branding: Option<&TenantName>,
) -> Markup {
    let project_id = project.id.clone().map(ProjectId::new).unwrap();
    let project_name = project.name.value.as_ref().map_or_else(
        || Cow::Owned(project_id.as_ref().to_string()),
        |s| Cow::Borrowed(s.as_str()),
    );

    let scope_route = oidc_route_string(tenant, &project_id, "auth/scope");
    let scope_route_str = scope_route.to_str().expect("Could not create scope route");

    page_html(&html! {
            (page_banner(tenant.as_ref(), Some(&project_name), branding))
            div class="border border-brand-50 w-full bg-white   bg-white rounded-lg shadow  md:mt-0  xl:p-0" {
                form class="p-6 space-y-4 md:space-y-6 sm:p-8" action=(scope_route_str) method="POST" {
                    (client_app_header_html(client_application))
                    div {
                        span class="text-sm text-slate-500" {
                            "The above app is requesting the following permissions. Uncheck any you do not want to grant, then allow or deny access for the app."
                        }
                    }
                    div class="max-h-72 overflow-auto" {
                        table class="border-collapse w-full" {
                            tbody {
                                @for (index, scope) in authorization_info.scope.0.iter().enumerate() {
                                    @let value = String::from(scope.clone());
                                    @let id = format!("scope-{index}");
                                    tr class="border border-gray-200" {
                                        td class="p-4 w-px" {
                                            @if is_fixed_scope(scope) {
                                                // A disabled checkbox is not submitted, so the scope
                                                // travels in a hidden field.
                                                input type="hidden" name="scope" value=(value) {}
                                                input id=(id) type="checkbox" class=(CHECKBOX) checked disabled {}
                                            } @else {
                                                input id=(id) type="checkbox" class=(CHECKBOX) name="scope" value=(value) checked {}
                                            }
                                        }
                                        td class="py-4 pr-4" {
                                            label for=(id) class="block" { (value) }
                                        }
                                        td class="py-4 pr-4 text-right text-xs text-slate-400" {
                                            @if is_fixed_scope(scope) { "Required" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    input readonly="" class="hidden" type="text" name="csrf_token" value=(authorization_info.csrf_token) {}
                    input readonly="" class="hidden" type="text" name="client_id" value=(authorization_info.client_id) {}
                    input readonly="" class="hidden" type="text" name="response_type" value=(authorization_info.response_type) {}
                    input readonly="" class="hidden" type="text" name="state" value=(authorization_info.state)  {}
                    input readonly="" class="hidden" type="text" name="code_challenge" value=(authorization_info.code_challenge)  {}
                    input readonly="" class="hidden" type="text" name="code_challenge_method" value=(authorization_info.code_challenge_method) {}
                    input readonly="" class="hidden" type="text" name="redirect_uri" value=(authorization_info.redirect_uri) {}
                    @if let Some(launch) = &authorization_info.launch {
                        input readonly="" class="hidden" type="text" name="launch" value=(launch) {}
                    }
                    @if let Some(aud) = &authorization_info.aud {
                        input readonly="" class="hidden" type="text" name="aud" value=(aud) {}
                    }
                    div class="justify-center items-center flex space-x-4" {
                        button type="submit" name="accept" value="on" class="cursor-pointer text-white bg-brand-600 hover:bg-brand-500 focus:ring-4 focus:outline-none focus:ring-brand-300 font-medium rounded-lg text-sm px-5 py-2.5 text-center dark:bg-brand-500 dark:hover:bg-brand-500 dark:focus:ring-brand-800" {
                            "Allow"
                        }
                        button type="submit" name="accept" value="off" class="cursor-pointer text-slate-900 bg-gray-100 hover:bg-gray-200 focus:ring-4 focus:outline-none  font-medium rounded-lg text-sm px-5 py-2.5 text-center dark:text-white dark:bg-gray-600 dark:hover:bg-gray-700 dark:focus:ring-gray-800" {
                            "Deny"
                        }
                    }
            }
        }
    })
}
