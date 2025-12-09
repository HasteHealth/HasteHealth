use crate::ui::components::{banner, page_html};
use haste_jwt::TenantId;
use maud::{Markup, html};

pub fn error_html(tenant: &TenantId, message: Markup) -> Markup {
    page_html(html! {
        (banner(tenant.as_ref(), None))
        div class="w-full bg-white rounded-lg shadow  md:mt-0  xl:p-0  sm:max-w-md" {
            div class="p-6 space-y-4 md:space-y-6 sm:p-8" {
                (message)
            }
        }
    })
}
