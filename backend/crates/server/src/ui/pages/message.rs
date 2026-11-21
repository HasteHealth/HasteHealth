use crate::ui::{
    components::{TenantName, page_banner, page_html},
    styles::{CARD, CARD_BODY},
};
use haste_jwt::{ProjectId, TenantId};
use maud::{Markup, html};
use std::borrow::Cow;

pub fn message_html(
    tenant: Option<&TenantId>,
    project: Option<&haste_fhir_model::r4::generated::resources::Project>,
    message: &Markup,
    branding: Option<&TenantName>,
) -> Markup {
    let project_id = project.map(|project| {
        project
            .id
            .clone()
            .and_then(|id| ProjectId::new(id).ok())
            .expect("a stored project has a valid id")
    });
    let project_name = project
        .and_then(|project| project.name.value.as_ref())
        .map(|s| Cow::Borrowed(s.as_str()))
        .or_else(|| project_id.map(|p_id| Cow::Owned(p_id.as_ref().to_string())));

    page_html(&html! {
        (page_banner(tenant.map_or("", |t| t.as_ref()), project_name.as_deref(), branding))
        div class=(CARD) {
            div class=(CARD_BODY) {
                (message)
            }
        }
    })
}
