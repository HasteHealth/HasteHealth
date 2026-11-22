use maud::{Markup, html};

use crate::static_assets::asset_route;

pub fn page_html(children: &Markup) -> Markup {
    html! {
        head {
            meta charset="utf-8" {}
            meta name="viewport" content="width=device-width, initial-scale=1" {}
            link rel="preload" as="image" href=(asset_route("img/logo.svg")) {}
            title { "Haste Health" }
            link rel="icon" href=(asset_route("img/logo.svg")) {}
            link rel="stylesheet" href=(asset_route("css/app.css")) {}
        }
        body class="bg-background text-ink-body" {
            section class="flex min-h-screen flex-col items-center px-6 py-8" {
                div class="grow" {}
                div class="w-full max-w-lg space-y-4" {
                    (children)
                }
                div class="grow md:grow-2" {}
            }
        }
    }
}
