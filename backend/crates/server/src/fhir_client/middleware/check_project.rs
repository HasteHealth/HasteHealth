use crate::fhir_client::{
    ClientState, ServerCTX,
    middleware::{Backend, Middleware, Next, Output},
    utilities::request_to_resource_type,
};
use haste_fhir_client::{
    request::{FHIRRequest, FHIRSearchTypeRequest, SearchRequest},
    url::ParsedParameters,
};
use haste_fhir_model::r4::generated::{resources::ResourceType, terminology::IssueType};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_jwt::ProjectId;
use std::sync::Arc;
use tracing::instrument;

/// Refuses every request made outside `project_id`.
pub struct CheckProject {
    project_id: ProjectId,
}

impl CheckProject {
    pub fn new(project_id: ProjectId) -> Self {
        Self { project_id }
    }
}

impl<B: Backend> Middleware<B> for CheckProject {
    #[instrument(
        name = "Check Project Middleware",
        skip(self, state, ctx, request, next)
    )]
    fn call<'a>(
        &'a self,
        state: &'a ClientState<B>,
        ctx: Arc<ServerCTX<B>>,
        request: FHIRRequest,
        next: Next<'a, B>,
    ) -> Output<'a> {
        if ctx.project == self.project_id {
            next.run(state, ctx, request)
        } else {
            Box::pin(std::future::ready(Err(OperationOutcomeError::fatal(
                IssueType::security(),
                format!(
                    "Must be in project {} to access this resource, not {}",
                    self.project_id, ctx.project,
                ),
            ))))
        }
    }
}

/// Reads of `resource_types` run in `project_id` whichever project the caller
/// is in; writes, and reads of any other type, stay where they are.
///
/// A search moved from another project returns its matches alone: its
/// `_include` and `_revinclude` would be resolved in `project_id` as well,
/// where they reach types other than `resource_types`.
pub struct SetProjectReadOnly {
    project_id: ProjectId,
    resource_types: Vec<ResourceType>,
}

impl SetProjectReadOnly {
    pub fn new(project_id: ProjectId, resource_types: Vec<ResourceType>) -> Self {
        Self {
            project_id,
            resource_types,
        }
    }

    /// Whether `request` is a read this runs in `project_id`.
    fn moves(&self, request: &FHIRRequest) -> bool {
        matches!(
            request,
            FHIRRequest::Read(_) | FHIRRequest::VersionRead(_) | FHIRRequest::Search(_)
        ) && request_to_resource_type(request)
            .is_some_and(|resource_type| self.resource_types.contains(resource_type))
    }
}

/// `request` without its `_include` and `_revinclude` parameters, when it is
/// a search.
fn without_includes(request: FHIRRequest) -> FHIRRequest {
    match request {
        FHIRRequest::Search(SearchRequest::Type(search)) => {
            FHIRRequest::Search(SearchRequest::Type(FHIRSearchTypeRequest {
                resource_type: search.resource_type,
                parameters: ParsedParameters::new(
                    search
                        .parameters
                        .owned_parameters()
                        .into_iter()
                        .filter(|parameter| !matches!(parameter.name(), "_include" | "_revinclude"))
                        .collect(),
                ),
            }))
        }
        request => request,
    }
}

impl<B: Backend> Middleware<B> for SetProjectReadOnly {
    fn call<'a>(
        &'a self,
        state: &'a ClientState<B>,
        ctx: Arc<ServerCTX<B>>,
        request: FHIRRequest,
        next: Next<'a, B>,
    ) -> Output<'a> {
        if !self.moves(&request) {
            return next.run(state, ctx, request);
        }

        let request = if ctx.project == self.project_id {
            request
        } else {
            without_includes(request)
        };
        let ctx = Arc::new(
            ServerCTX::new(
                ctx.tenant.clone(),
                self.project_id.clone(),
                ctx.fhir_version.clone(),
                ctx.user.clone(),
                ctx.client.clone(),
                ctx.rate_limit.clone(),
            )
            .with_tracing_id(ctx.tracing_id.clone()),
        );
        next.run(state, ctx, request)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use haste_fhir_client::request::{FHIRCreateRequest, FHIRReadRequest, FHIRVersionReadRequest};
    use haste_fhir_model::r4::generated::resources::{Resource, User};
    use haste_jwt::VersionId;

    fn tenant_auth() -> SetProjectReadOnly {
        SetProjectReadOnly::new(
            ProjectId::System,
            vec![ResourceType::User, ResourceType::Project],
        )
    }

    fn read(resource_type: ResourceType) -> FHIRRequest {
        FHIRRequest::Read(FHIRReadRequest {
            resource_type,
            id: "id".to_string(),
        })
    }

    fn search(resource_type: ResourceType, query: &str) -> FHIRRequest {
        FHIRRequest::Search(SearchRequest::Type(FHIRSearchTypeRequest {
            resource_type,
            parameters: ParsedParameters::try_from(query).expect("query parses"),
        }))
    }

    #[test]
    fn reads_of_the_named_types_move() {
        let middleware = tenant_auth();

        assert!(middleware.moves(&read(ResourceType::User)));
        assert!(middleware.moves(&read(ResourceType::Project)));
        assert!(
            middleware.moves(&FHIRRequest::VersionRead(FHIRVersionReadRequest {
                resource_type: ResourceType::User,
                id: "id".to_string(),
                version_id: VersionId::new("1".to_string()),
            }))
        );
        assert!(middleware.moves(&search(ResourceType::User, "email=a@example.org")));
    }

    #[test]
    fn reads_of_other_types_and_writes_stay() {
        let middleware = tenant_auth();

        assert!(!middleware.moves(&read(ResourceType::IdentityProvider)));
        assert!(!middleware.moves(&search(ResourceType::IdentityProvider, "name=idp")));
        assert!(!middleware.moves(&FHIRRequest::Create(FHIRCreateRequest {
            resource_type: ResourceType::User,
            resource: Resource::User(User::default()),
        })));
    }

    #[test]
    fn a_moved_search_keeps_every_parameter_but_its_includes() {
        let FHIRRequest::Search(SearchRequest::Type(moved)) = without_includes(search(
            ResourceType::User,
            "email=a@example.org&_include=User:identityProvider\
             &_revinclude=Membership:user&_include:iterate=*&_count=5",
        )) else {
            panic!("a search stays a search");
        };

        let names: Vec<&str> = moved
            .parameters
            .parameters()
            .iter()
            .map(|parameter| parameter.name())
            .collect();
        assert_eq!(moved.resource_type, ResourceType::User);
        assert_eq!(names, vec!["email", "_count"]);
    }
}
