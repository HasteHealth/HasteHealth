use crate::ui::components::{TenantName, page_banner, page_html};
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
        div class="border border-[var(--auth-line)] w-full bg-[var(--auth-surface)] rounded-none shadow-2xl md:mt-0 xl:p-0 text-[var(--auth-ink)]" {
            form class="space-y-4 md:space-y-6" action=(email_information.continue_url) method="POST" {
                input type="hidden" name="csrf_token" value=(csrf_token) {}
                div class="p-6 space-y-4 md:space-y-6 sm:p-8" {
                    div {
                        label for="email" class="block mb-2 font-mono text-xs uppercase tracking-wider text-[var(--auth-ink-muted)]" {
                            "Enter your email"
                        }
                        input type="email" id="email" class="bg-[var(--auth-surface-raised)] border border-[var(--auth-line-strong)] text-[var(--auth-ink)] text-sm px-3 py-2 rounded-none outline-none focus:border-[var(--auth-signal)] block w-full transition-colors" placeholder="name@company.com" required="" name="email" {}
                    }
                    button type="submit" class="cursor-pointer w-full text-[var(--auth-on-signal)] bg-[var(--auth-signal)] hover:bg-[var(--auth-signal-hover)] focus:ring-2 focus:outline-none focus:ring-[var(--auth-signal)] font-mono text-xs uppercase tracking-wider font-semibold rounded-none px-5 py-2.5 text-center transition-colors" {
                        "Continue"
                    }
                }
            }
        }
    })
}
