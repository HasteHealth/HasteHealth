use crate::ui::components::{banner, page_html};
use haste_jwt::{ProjectId, TenantId};
use maud::{Markup, html};
use std::borrow::Cow;

pub struct EmailInformation {
    pub continue_url: String,
}

pub fn email_form_html(
    tenant: &TenantId,
    project: &haste_fhir_model::r4::generated::resources::Project,
    email_information: &EmailInformation,
) -> Markup {
    let project_id = project.id.clone().map(|id| ProjectId::new(id)).unwrap();
    let project_name = project
        .name
        .as_ref()
        .and_then(|name| name.value.as_ref())
        .map(|s| Cow::Borrowed(s.as_str()))
        .unwrap_or_else(|| Cow::Owned(project_id.as_ref().to_string()));

    page_html(html! {
        (banner(tenant, Some(&project_name)))
        div class="w-full bg-white rounded-lg shadow  md:mt-0  xl:p-0  sm:max-w-md" {
            form class="space-y-4 md:space-y-6" action=(email_information.continue_url) method="POST" {
                div class="p-6 space-y-4 md:space-y-6 sm:p-8" {
                    div {
                        label for="email" class="block mb-2 text-sm font-medium text-gray-900 dark:text-white" {
                            "Enter your email"
                        }
                        input type="email" id="email" class="bg-gray-50 border border-gray-300 text-gray-900 sm:text-sm rounded-lg focus:ring-blue-600 focus:border-blue-600 block w-full p-2.5" placeholder="name@company.com" required="" name="email" {}
                    }
                    button type="submit" class="w-full text-white bg-orange-600 hover:bg-orange-700 focus:ring-4 focus:outline-none focus:ring-orange-300 font-medium rounded-lg text-sm px-5 py-2.5 text-center" {
                        "Continue"
                    }
                }
            }
        }
    })
}
