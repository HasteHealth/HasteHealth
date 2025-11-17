use crate::ui::components::{banner, page_html};
use haste_jwt::{ProjectId, TenantId};
use maud::{Markup, html};
use std::borrow::Cow;

pub fn message_html(
    tenant: &TenantId,
    project: &haste_fhir_model::r4::generated::resources::Project,
    message: Markup,
) -> Markup {
    let project_id = project.id.clone().map(|id| ProjectId::new(id)).unwrap();
    let project_name = project
        .name
        .value
        .as_ref()
        .map(|s| Cow::Borrowed(s.as_str()))
        .unwrap_or_else(|| Cow::Owned(project_id.as_ref().to_string()));

    page_html(html! {
        (banner(tenant, Some(&project_name)))
        div class="w-full bg-white rounded-lg shadow  md:mt-0  xl:p-0  sm:max-w-md" {
            div class="p-6 space-y-4 md:space-y-6 sm:p-8" {
                (message)
            }
        }
    })
}
