use maud::{Markup, html};

use crate::static_assets::asset_route;

pub fn banner(header: &str, subheader: Option<&str>) -> Markup {
    html! {
        div class="flex flex-col items-center justify-center space-y-1" {
            a href="#" class="relative flex items-center text-2xl font-semibold text-gray-900" {
                img class="absolute w-8 h-8 mr-2" src=(asset_route("img/logo.svg")) alt="logo" style="left:-34px;" {}
                "Haste Health"
            }
            div class="flex space-x-1 items-center justify-center text-sm text-slate-400" {
                div {
                    span class="font-bold" {
                        (header)
                    }
                }
                @if let Some(project_name) = subheader {
                    div {
                        span {
                            (project_name)
                        }
                    }
                }
            }
        }
    }
}
