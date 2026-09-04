use crate::ui::components::{TenantName, page_banner, page_html};
use haste_jwt::TenantId;
use maud::{Markup, html};

pub fn error_html(tenant: &TenantId, message: &Markup, branding: Option<&TenantName>) -> Markup {
    page_html(&html! {
        (page_banner(tenant.as_ref(), None, branding))
        div class="border border-brand-50 w-full bg-white   bg-white rounded-lg shadow  md:mt-0  xl:p-0 " {
            div class="p-6 space-y-4 md:space-y-6 sm:p-8" {
                (message)
            }
        }
    })
}
