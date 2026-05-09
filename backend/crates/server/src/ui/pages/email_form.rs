use crate::ui::components::{banner, page_html};
use haste_jwt::TenantId;
use maud::{Markup, html};

pub struct EmailInformation {
    pub continue_url: String,
}

pub fn email_form_html(
    tenant: &TenantId,
    project: Option<&haste_fhir_model::r4::generated::resources::Project>,
    email_information: &EmailInformation,
) -> Markup {
    let project_name = project
        .and_then(|p| p.name.value.as_ref())
        .or_else(|| project.and_then(|p| p.id.as_ref()))
        .map(|s| s.as_str());

    page_html(html! {
        (banner(tenant.as_ref(), project_name))
        div class="border border-brand-50 w-full bg-white   bg-white rounded-lg shadow  md:mt-0  xl:p-0 " {
            form class="space-y-4 md:space-y-6" action=(email_information.continue_url) method="POST" {
                div class="p-6 space-y-4 md:space-y-6 sm:p-8" {
                    div {
                        label for="email" class="block mb-2 text-sm font-medium text-slate-600 dark:text-white" {
                            "Enter your email"
                        }
                        input type="email" id="email" class="bg-gray-50 border border-gray-300 text-slate-900 sm:text-sm rounded-lg focus:ring-blue-600 focus:border-blue-600 block w-full p-2.5" placeholder="name@company.com" required="" name="email" {}
                    }
                    button type="submit" class="cursor-pointer w-full text-white bg-brand-600 hover:bg-brand-500 focus:ring-4 focus:outline-none focus:ring-brand-300 font-medium rounded-lg text-sm px-5 py-2.5 text-center" {
                        "Continue"
                    }
                }
            }
        }
    })
}
