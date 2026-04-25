use crate::{
    ServerEnvironmentVariables,
    auth_n::oidc::hardcoded_clients::admin_app,
    ui::components::{banner, page_html},
};
use haste_config::Config;
use haste_fhir_model::r4::generated::{resources::Project, terminology::IssueType};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_jwt::{ProjectId, TenantId};
use maud::{Markup, html};

fn get_project_id(project: &Project) -> Result<ProjectId, OperationOutcomeError> {
    project
        .id
        .clone()
        .map(|id| ProjectId::new(id))
        .ok_or_else(|| {
            OperationOutcomeError::error(
                IssueType::Exception(None),
                "Project ID not found".to_string(),
            )
        })
}

pub fn project_select_html(
    config: &dyn Config<ServerEnvironmentVariables>,
    tenant: &TenantId,
    projects: &[Project],
) -> Result<Markup, OperationOutcomeError> {
    Ok(page_html(html! {
        (banner(tenant.as_ref(), None))
        div class="w-full bg-white rounded-lg shadow md:mt-0 xl:p-0 w-md sm:max-w-md text-slate-700" {
            @if projects.is_empty() {
                div class="p-6 space-y-4 md:space-y-6 sm:p-8" {
                    span class="font-semibold leading-tight text-red-600 text-md " { "No projects found. Please contact your administrator." }
                }
            } @else {
                div class="p-6 space-y-4 md:space-y-6 sm:p-8" {
                    div class="space-y-4 md:space-y-6" {
                        div class="grid grid-cols-1 gap-3" {
                            @for project
                            in projects.iter() {
                                div {
                                    a href=(admin_app::redirect_url(config, tenant, &get_project_id(project)?).unwrap_or("".to_string()))
                                    class="block w-full rounded-lg border border-gray-200 bg-white px-4 py-2.5 text-center text-sm font-medium text-slate-900 transition-colors hover:bg-orange-50 hover:border-orange-200" {
                                        (project.name.value.as_ref().unwrap_or(&project.id.clone().unwrap_or_else(|| "Unnamed Project".to_string())))
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }))
}
