use axum::http::Uri;
use maud::{Markup, html};

use crate::server::asset_route;

pub fn base(uri: &Uri, children: Markup) -> Markup {
    let img_url = Uri::builder()
        .scheme(uri.scheme().unwrap().clone())
        .authority(uri.authority().unwrap().clone())
        .path_and_query(asset_route("img/logo.svg"))
        .build()
        .unwrap();
    html! {
        div style="background-color:#f2f5f7;color:#242424;font-family:&quot;Helvetica Neue&quot;,&quot;Arial Nova&quot;,&quot;Nimbus Sans&quot;,Arial,sans-serif;font-size:16px;font-weight:400;letter-spacing:0.15008px;line-height:1.5;margin:0;padding:32px 0;min-height:100%;width:100%" {
            table align="center" width="100%" style="margin:0 auto;max-width:600px;background-color:#ffffff" role="presentation" cellspacing="0" cellpadding="0" border="0" {
                tbody {
                    tr style="width:100%"{
                        td {
                            div style="padding:24px 24px 24px 24px" {
                                img alt="OxidizedHealth Logo" src=(img_url) width="100"  {}
                            }
                            div style="font-weight:normal;padding:0px 24px 16px 24px" { "To verify your email and set your password click below." }
                            div style="padding:16px 24px 24px 24px" {
                                (children)
                            }
                        }
                    }
                }
            }
        }
    }
}
