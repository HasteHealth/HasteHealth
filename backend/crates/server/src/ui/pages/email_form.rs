use crate::ui::{
    components::{TenantName, page_banner, page_html},
    styles::{BUTTON, CARD, CARD_BODY, INPUT, LABEL},
};
use haste_jwt::TenantId;
use maud::{Markup, html};

pub struct EmailInformation {
    pub continue_url: String,
}

pub fn email_form_html(
    tenant: &TenantId,
    project: Option<&haste_fhir_model::r4::generated::resources::Project>,
    csrf_token: &str,
    email_information: &EmailInformation,
    branding: Option<&TenantName>,
) -> Markup {
    let project_name = project
        .and_then(|p| p.name.value.as_ref())
        .or_else(|| project.and_then(|p| p.id.as_ref()))
        .map(std::string::String::as_str);

    page_html(&html! {
        (page_banner(tenant.as_ref(), project_name, branding))
        div class=(CARD) {
            form class=(CARD_BODY) action=(email_information.continue_url) method="POST" {
                input type="hidden" name="csrf_token" value=(csrf_token) {}
                div {
                    label for="email" class=(LABEL) { "Enter your email" }
                    input type="email" id="email" class=(INPUT) placeholder="name@company.com" required="" name="email" {}
                }
                button type="submit" class=(BUTTON) { "Continue" }
            }
        }
    })
}
