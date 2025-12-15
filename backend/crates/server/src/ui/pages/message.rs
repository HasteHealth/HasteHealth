use crate::ui::components::{banner, page_html};
use haste_jwt::{ProjectId, TenantId};
use maud::{Markup, html};
use std::borrow::Cow;

pub fn message_html(
    tenant: &TenantId,
    project: Option<&haste_fhir_model::r4::generated::resources::Project>,
    message: Markup,
) -> Markup {
    let project_id =
        project.map(|project| project.id.clone().map(|id| ProjectId::new(id)).unwrap());
    let project_name = project
        .and_then(|project| project.name.value.as_ref())
        .map(|s| Cow::Borrowed(s.as_str()))
        .or_else(|| project_id.map(|p_id| Cow::Owned(p_id.as_ref().to_string())));

    page_html(html! {
        (banner(tenant.as_ref(), project_name.as_ref().map(|p| p.as_ref())))
        div class="w-full bg-white rounded-lg shadow  md:mt-0  xl:p-0  sm:max-w-md" {
            div class="p-6 space-y-4 md:space-y-6 sm:p-8" {
                (message)
            }
        }
    })
}
