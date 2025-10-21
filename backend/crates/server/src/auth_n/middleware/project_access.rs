use axum::{Extension, extract::Request, middleware::Next, response::Response};
use oxidized_fhir_model::r4::generated::terminology::IssueType;
use oxidized_fhir_operation_error::OperationOutcomeError;
use std::sync::Arc;

use crate::{
    auth_n::claims::UserTokenClaims,
    extract::path_tenant::{ProjectIdentifier, TenantIdentifier},
};

pub async fn project_access(
    TenantIdentifier { tenant }: TenantIdentifier,
    ProjectIdentifier { project }: ProjectIdentifier,
    // run the `HeaderMap` extractor
    Extension(claims): Extension<Arc<UserTokenClaims>>,
    // you can also add more extractors here but the last
    // extractor must implement `FromRequest` which
    // `Request` does
    request: Request,
    next: Next,
) -> Result<Response, OperationOutcomeError> {
    if claims.tenant != tenant {
        return Err(OperationOutcomeError::error(
            IssueType::Forbidden(None),
            "User does not have access to project".to_string(),
        ));
    }

    let Some(user_project) = &claims.project else {
        return Err(OperationOutcomeError::error(
            IssueType::Forbidden(None),
            "User does not have access to project".to_string(),
        ));
    };

    if user_project != &project {
        return Err(OperationOutcomeError::error(
            IssueType::Forbidden(None),
            "User does not have access to project".to_string(),
        ));
    }

    Ok(next.run(request).await)
}
