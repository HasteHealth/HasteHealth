use crate::{server::asset_route, ui::components::page_html};
use maud::{Markup, html};
use oxidized_jwt::TenantId;

pub fn error_html(tenant: &TenantId, message: Markup) -> Markup {
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
            }
        }
        div class="w-full bg-white rounded-lg shadow  md:mt-0  xl:p-0  sm:max-w-md" {
            div class="p-6 space-y-4 md:space-y-6 sm:p-8" {
                (message)
            }
        }
    })
}
