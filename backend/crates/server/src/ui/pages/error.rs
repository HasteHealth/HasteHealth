use crate::ui::{
    components::{TenantName, page_banner, page_html},
    styles::{CARD, CARD_BODY},
};
use haste_jwt::TenantId;
use maud::{Markup, html};

pub fn error_html(tenant: &TenantId, message: &Markup, branding: Option<&TenantName>) -> Markup {
    page_html(&html! {
        (page_banner(tenant.as_ref(), None, branding))
        div class=(CARD) {
            div class=(CARD_BODY) {
                (message)
            }
        }
    })
}
