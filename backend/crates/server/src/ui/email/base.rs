use axum::http::Uri;
use maud::{Markup, html};

use crate::static_assets::asset_route;

pub fn base(uri: &Uri, children: Markup) -> Markup {
    let img_url = Uri::builder()
        .scheme(uri.scheme().unwrap().clone())
        .authority(uri.authority().unwrap().clone())
        .path_and_query(asset_route("img/logo.png"))
        .build()
        .unwrap();
    html! {
        div style="color: #441306;" {
            div style="padding:0px 24px 24px"{}
            table width="100%" style="margin:0 auto;max-width:600px;background-color:#ffffff" role="presentation" cellspacing="0" cellpadding="0" border="0" {
                tbody {
                    tr style="width:100%" {
                        td style="padding: 0px 24px; vertical-align: middle; width: 100px;" {
                            img alt="Haste Health Logo" src=(img_url) width="100"  {}
                        }
                        td style="vertical-align: middle;" {
                            span style="color: #ff6900; font-weight:bold; font-size:24px;" { "Haste Health"}
                        }
                    }
                }
            }

            table align="center" width="100%" style="margin:0 auto;max-width:600px;background-color:#ffffff" role="presentation" cellspacing="0" cellpadding="0" border="0" {
                tbody {
                    tr style="width:100%"{
                        td {
                            div style="padding:0px 24px" {
                                (children)
                            }
                        }
                    }
                }
            }
        }
    }
}
