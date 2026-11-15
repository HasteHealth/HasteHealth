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

const CHECKBOX: &str = "w-4 h-4 rounded-none border border-[#242831] bg-[#14171d] text-[#e5451a] focus:ring-1 focus:ring-[#ff5e26] disabled:opacity-60";

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
    let project_id = project
        .id
        .clone()
        .and_then(|id| ProjectId::new(id).ok())
        .expect("a stored project has a valid id");
    let project_name = project.name.value.as_ref().map_or_else(
        || Cow::Owned(project_id.as_ref().to_string()),
        |s| Cow::Borrowed(s.as_str()),
    );

    let scope_route = oidc_route_string(tenant, &project_id, "auth/scope");
    let scope_route_str = scope_route.to_str().expect("Could not create scope route");

    page_html(&html! {
            (page_banner(tenant.as_ref(), Some(&project_name), branding))
            div class="border border-[#1e2229] w-full bg-[#0d0f12] text-[#f0f2f5] p-6 space-y-6 shadow-2xl" {
                form class="space-y-4" action=(scope_route_str) method="POST" {
                    (client_app_header_html(client_application))
                    div {
                        span class="block text-xs leading-relaxed text-[#8c96a5]" {
                            "The above app is requesting the following permissions. Uncheck any you do not want to grant, then allow or deny access for the app."
                        }
                    }
                    div class="max-h-72 overflow-auto border border-[#1e2229]" {
                        table class="w-full border-collapse font-mono text-xs text-[#f0f2f5]" {
                            tbody {
                                @for (index, scope) in authorization_info.scope.0.iter().enumerate() {
                                    @let value = String::from(scope.clone());
                                    @let id = format!("scope-{index}");
                                    tr class="border-b border-[#1e2229] last:border-b-0" {
                                        td class="w-px p-3" {
                                            @if is_fixed_scope(scope) {
                                                // A disabled checkbox is not submitted, so the scope
                                                // travels in a hidden field.
                                                input type="hidden" name="scope" value=(value) {}
                                                input id=(id) type="checkbox" class=(CHECKBOX) checked disabled {}
                                            } @else {
                                                input id=(id) type="checkbox" class=(CHECKBOX) name="scope" value=(value) checked {}
                                            }
                                        }
                                        td class="py-3 pr-4" {
                                            label for=(id) class="block text-xs text-[#f0f2f5]" { (value) }
                                        }
                                        td class="py-3 pr-4 text-right text-[10px] uppercase tracking-wider text-[#8c96a5]" {
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
                    div class="flex items-center gap-3 pt-2" {
                        button type="submit" name="accept" value="on" class="flex-1 cursor-pointer border border-[#e5451a] bg-[#e5451a] px-4 py-2.5 text-center font-mono text-xs font-semibold uppercase tracking-wider text-white transition-colors hover:bg-[#ff5e26]" {
                            "Allow"
                        }
                        button type="submit" name="accept" value="off" class="flex-1 cursor-pointer border border-[#1e2229] bg-[#14171d] px-4 py-2.5 text-center font-mono text-xs font-semibold uppercase tracking-wider text-[#f0f2f5] transition-colors hover:bg-[#1e2229]" {
                            "Deny"
                        }
                    }
                }
            }
    })
}
