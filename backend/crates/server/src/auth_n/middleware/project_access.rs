use crate::{
    auth_n::middleware::jwt::User,
    extract::path_tenant::{ProjectIdentifier, TenantIdentifier},
};
use axum::{Extension, extract::Request, middleware::Next, response::Response};
use axum_extra::extract::Cached;
use haste_fhir_model::r4::generated::terminology::IssueType;
use haste_fhir_operation_error::OperationOutcomeError;
use std::sync::Arc;

pub async fn project_access(
    Cached(TenantIdentifier { tenant }): Cached<TenantIdentifier>,
    Cached(ProjectIdentifier { project }): Cached<ProjectIdentifier>,
    // run the `HeaderMap` extractor
    Extension(user): Extension<Arc<User>>,
    // you can also add more extractors here but the last
    // extractor must implement `FromRequest` which
    // `Request` does
    request: Request,
    next: Next,
) -> Result<Response, OperationOutcomeError> {
    if user.claims.tenant != tenant {
        return Err(OperationOutcomeError::error(
            IssueType::Forbidden(None),
            format!("User does not have access to tenant '{}'.", tenant),
        ));
    }

    let Some(user_project) = &user.claims.project else {
        return Err(OperationOutcomeError::error(
            IssueType::Forbidden(None),
            format!("User does not have access to project '{}'.", project),
        ));
    };

    if user_project != &project {
        return Err(OperationOutcomeError::error(
            IssueType::Forbidden(None),
            format!("User does not have access to project '{}'.", project),
        ));
    }

    Ok(next.run(request).await)
}
