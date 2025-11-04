use crate::{static_assets::asset_route, ui::components::page_html};
use maud::{Markup, html};
use oxidized_jwt::{ProjectId, TenantId};
use std::borrow::Cow;

pub fn message_html(
    tenant: &TenantId,
    project: &oxidized_fhir_model::r4::generated::resources::Project,
    message: Markup,
) -> Markup {
    let project_id = project.id.clone().map(|id| ProjectId::new(id)).unwrap();
    let project_name = project
        .name
        .as_ref()
        .and_then(|name| name.value.as_ref())
        .map(|s| Cow::Borrowed(s.as_str()))
        .unwrap_or_else(|| Cow::Owned(project_id.as_ref().to_string()));

    page_html(html! {
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
                (message)
            }
        }
    })
}
