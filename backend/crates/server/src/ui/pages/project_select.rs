use crate::{
    auth_n::oidc::hardcoded_clients::admin_app,
    config::ServerConfig,
    ui::{
        components::{TenantName, page_banner, page_html},
        styles::{CARD, CARD_BODY, CHOICE},
    },
};
use haste_fhir_model::r4::generated::{resources::Project, terminology::IssueType};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_jwt::{ProjectId, TenantId};
use maud::{Markup, html};

fn get_project_id(project: &Project) -> Result<ProjectId, OperationOutcomeError> {
    let id = project.id.clone().ok_or_else(|| {
        OperationOutcomeError::error(IssueType::exception(), "Project ID not found".to_string())
    })?;
    ProjectId::new(id)
}

pub fn project_select_html(
    config: &ServerConfig,
    tenant: &TenantId,
    projects: &[Project],
    branding: Option<&TenantName>,
) -> Result<Markup, OperationOutcomeError> {
    Ok(page_html(&html! {
        (page_banner(tenant.as_ref(), None, branding))
        div class=(CARD) {
            div class=(CARD_BODY) {
                @if projects.is_empty() {
                    span class="font-semibold leading-tight text-error" { "No projects found. Please contact your administrator." }
                } @else {
                    div class="grid grid-cols-1 gap-3" {
                        @for project in projects.iter() {
                            a href=(admin_app::redirect_url(config, tenant, &get_project_id(project)?).unwrap_or_default())
                            class=(CHOICE) {
                                (project.name.value.as_ref().unwrap_or(&project.id.clone().unwrap_or_else(|| "Unnamed Project".to_string())))
                            }
                        }
                    }
                }
            }
        }
    }))
}
