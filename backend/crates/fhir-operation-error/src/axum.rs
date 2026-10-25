use crate::OperationOutcomeError;
use axum::response::IntoResponse;
use haste_fhir_model::r4::generated::terminology::IssueType;
use std::sync::Arc;

impl OperationOutcomeError {
    #[must_use]
    pub fn status(&self) -> axum::http::StatusCode {
        if let Some(status) = self
            .status_override
            .and_then(|s| axum::http::StatusCode::from_u16(s).ok())
        {
            return status;
        }
        match self.outcome.issue.first() {
            Some(issue) => {
                // Children of `invalid` in the IssueType hierarchy.
                if issue.code == IssueType::invalid()
                    || issue.code == IssueType::structure()
                    || issue.code == IssueType::required()
                    || issue.code == IssueType::value()
                    || issue.code == IssueType::invariant()
                {
                    axum::http::StatusCode::BAD_REQUEST
                } else if issue.code == IssueType::not_found() {
                    axum::http::StatusCode::NOT_FOUND
                } else if issue.code == IssueType::forbidden() {
                    axum::http::StatusCode::FORBIDDEN
                } else if issue.code == IssueType::conflict() {
                    axum::http::StatusCode::CONFLICT
                } else if issue.code == IssueType::throttled() {
                    axum::http::StatusCode::TOO_MANY_REQUESTS
                } else if issue.code == IssueType::login()
                    || issue.code == IssueType::unknown()
                    || issue.code == IssueType::expired()
                {
                    // The caller is not (or no longer) authenticated.
                    axum::http::StatusCode::UNAUTHORIZED
                } else if issue.code == IssueType::security()
                    || issue.code == IssueType::suppressed()
                {
                    axum::http::StatusCode::FORBIDDEN
                } else if issue.code == IssueType::not_supported() {
                    axum::http::StatusCode::NOT_IMPLEMENTED
                } else {
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR
                }
            }
            None => axum::http::StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl IntoResponse for OperationOutcomeError {
    fn into_response(self) -> axum::response::Response {
        let status_code = self.status();
        let error = Arc::new(self);
        let outcome = &error.outcome;
        let mut headers = axum::http::HeaderMap::new();
        headers.insert(
            axum::http::header::CONTENT_TYPE,
            "application/fhir+json".parse().unwrap(),
        );
        let response =
            serde_json::to_string(outcome).expect("Failed to serialize OperationOutcome");

        // Attach the original error to the response extensions for logging middleware to access and content-type handling.
        let mut response = (status_code, headers, response).into_response();
        response.extensions_mut().insert(error);

        response
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn status_of(code: haste_fhir_model::r4::generated::terminology::BoundCode<IssueType>) -> u16 {
        OperationOutcomeError::error(code, String::new())
            .status()
            .as_u16()
    }

    #[test]
    fn children_of_invalid_answer_400() {
        for code in [
            IssueType::structure(),
            IssueType::required(),
            IssueType::value(),
            IssueType::invariant(),
        ] {
            assert_eq!(status_of(code), 400);
        }
    }

    #[test]
    fn status_override_wins() {
        let error =
            OperationOutcomeError::error(IssueType::value(), String::new()).with_status(422);
        assert_eq!(error.status().as_u16(), 422);
    }
}
