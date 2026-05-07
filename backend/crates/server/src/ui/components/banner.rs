use maud::{Markup, html};

use crate::static_assets::asset_route;

pub fn banner(header: &str, subheader: Option<&str>) -> Markup {
    html! {
        div class="flex flex-col items-center justify-center space-y-1" {
            a href="#" class="h-20 w-50 relative flex justify-center items-center text-2xl font-semibold text-gray-900" {
                img src=(asset_route("img/logo_text.svg")) alt="logo"  {}
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
