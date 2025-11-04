use maud::{Markup, html};

use crate::static_assets::asset_route;

pub fn page_html(children: Markup) -> Markup {
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
            section class="bg-gray-50 h-screen" {
                div class="flex flex-col items-center justify-center md:h-screen" {
                    div class="px-6 py-8 lg:py-0 space-y-4 mx-auto md:-mt-32" {
                     (children)
                    }
                }
            }
        }
    }
}
