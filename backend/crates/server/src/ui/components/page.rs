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
        body class="bg-[#07080b] text-[#f0f2f5] antialiased" {
            section class="min-h-screen flex items-center justify-center bg-[#07080b] p-4" {
                div class="flex flex-col items-center justify-center w-full max-w-md" {
                    (children)
                }
            }
        }
    }
}
