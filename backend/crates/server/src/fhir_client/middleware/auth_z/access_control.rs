use crate::fhir_client::{
    ServerCTX,
    middleware::{
        ServerMiddlewareContext, ServerMiddlewareNext, ServerMiddlewareOutput,
        ServerMiddlewareState,
    },
    utilities::{is_search_match, map_search_entries},
};
use haste_access_control::context::{PolicyContext, PolicyEnvironment, UserInfo};
use haste_fhir_client::{
    FHIRClient,
    middleware::MiddlewareChain,
    request::{FHIRReadRequest, FHIRRequest, FHIRResponse},
};
use haste_fhir_model::r4::generated::resources::{AccessPolicyV2, BundleEntry, Resource};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_fhir_search::SearchEngine;
use haste_fhir_terminology::FHIRTerminology;
use haste_jwt::{ProjectId, TenantId, UserRole};
use haste_repository::Repository;
use std::sync::Arc;

/// Reusable pieces for evaluating extra policy checks without re-deriving
/// them per resource.
struct IncludeAuthContext<
    Client: FHIRClient<Arc<ServerCTX<Client>>, OperationOutcomeError> + 'static,
> {
    client: Arc<Client>,
    system_ctx: Arc<ServerCTX<Client>>,
    tenant: TenantId,
    project: ProjectId,
    user_info: Arc<UserInfo>,
}

/// Evaluates `policies` against `request`; `Err` means denied.
///
/// Shared by the primary per-request check and the per-resource include
/// check, so the two can't drift apart.
///
/// Evaluates a clone rather than recovering a "rewritten" request via
/// `Arc::try_unwrap` — that always failed once a rule referenced `%request`
/// at all, and no rule can rewrite the request anyway
/// (`RequestReflection::get_field_mut` is always `None`).
async fn evaluate_request_policy<
    Client: FHIRClient<Arc<ServerCTX<Client>>, OperationOutcomeError> + 'static,
>(
    auth: &IncludeAuthContext<Client>,
    request: &FHIRRequest,
    policies: &Vec<Arc<AccessPolicyV2>>,
) -> Result<(), OperationOutcomeError> {
    haste_access_control::evaluate_policies(
        PolicyContext::new(
            auth.client.clone(),
            auth.system_ctx.clone(),
            PolicyEnvironment::new(
                auth.tenant.clone(),
                auth.project.clone(),
                request.clone(),
                auth.user_info.clone(),
            ),
        ),
        policies,
    )
    .await?;

    Ok(())
}

/// Whether `policies` permit a direct `Read` of `resource`.
async fn resource_is_readable<
    Client: FHIRClient<Arc<ServerCTX<Client>>, OperationOutcomeError> + 'static,
>(
    resource: &Resource,
    auth: &IncludeAuthContext<Client>,
    policies: &Vec<Arc<AccessPolicyV2>>,
) -> bool {
    let Some(id) = resource.id().clone() else {
        return false;
    };

    let read_request = FHIRRequest::Read(FHIRReadRequest {
        resource_type: resource.resource_type(),
        id,
    });

    evaluate_request_policy(auth, &read_request, policies)
        .await
        .is_ok()
}

/// Drops include/revinclude entries the caller's policies don't grant `Read`
/// on — an include must not bypass the check a direct read would face.
async fn filter_unauthorized_includes<
    Client: FHIRClient<Arc<ServerCTX<Client>>, OperationOutcomeError> + 'static,
>(
    entries: Vec<BundleEntry>,
    auth: &IncludeAuthContext<Client>,
    policies: &Vec<Arc<AccessPolicyV2>>,
) -> Vec<BundleEntry> {
    let mut kept = Vec::with_capacity(entries.len());

    for entry in entries {
        let authorized = is_search_match(&entry)
            || match entry.resource.as_deref() {
                Some(resource) => resource_is_readable(resource, auth, policies).await,
                None => false,
            };

        if authorized {
            kept.push(entry);
        }
    }

    kept
}

pub struct AccessControlMiddleware {}
impl AccessControlMiddleware {
    pub fn new() -> Self {
        Self {}
    }
}
impl<
    Repo: Repository + Send + Sync + 'static,
    Search: SearchEngine + Send + Sync + 'static,
    Terminology: FHIRTerminology + Send + Sync + 'static,
    Client: FHIRClient<Arc<ServerCTX<Client>>, OperationOutcomeError> + 'static,
>
    MiddlewareChain<
        ServerMiddlewareState<Repo, Search, Terminology>,
        Arc<ServerCTX<Client>>,
        FHIRRequest,
        FHIRResponse,
        OperationOutcomeError,
    > for AccessControlMiddleware
{
    fn call(
        &self,
        state: ServerMiddlewareState<Repo, Search, Terminology>,
        context: ServerMiddlewareContext<Client>,
        next: Option<
            Arc<ServerMiddlewareNext<Client, ServerMiddlewareState<Repo, Search, Terminology>>>,
        >,
    ) -> ServerMiddlewareOutput<Client> {
        Box::pin(async move {
            match context.ctx.user.claims.user_role {
                // Admin and Owner roles are allowed to proceed without restrictions
                UserRole::Admin | UserRole::Owner => {
                    if let Some(next) = next {
                        return next(state, context).await;
                    }
                    Ok(context)
                }
                UserRole::Member => {
                    let policies = state
                        .repo
                        .read_by_version_ids(
                            &context.ctx.tenant,
                            &context.ctx.project,
                            &context
                                .ctx
                                .user
                                .claims
                                .access_policy_version_ids
                                .iter()
                                .collect::<Vec<_>>(),
                            haste_repository::fhir::CachePolicy::Cache,
                        )
                        .await?
                        .into_iter()
                        .filter_map(|v| match v {
                            Resource::AccessPolicyV2(policy) => Some(Arc::new(policy)),
                            _ => None,
                        })
                        .collect();

                    // Use System context for policy evaluation.
                    let system_ctx = Arc::new(ServerCTX::system(
                        context.ctx.tenant.clone(),
                        context.ctx.project.clone(),
                        context.ctx.client.clone(),
                        context.ctx.rate_limit.clone(),
                    ));
                    let user_info = Arc::new(UserInfo {
                        id: context.ctx.user.claims.user_id.as_ref().to_string(),
                    });
                    let auth = IncludeAuthContext {
                        client: context.ctx.client.clone(),
                        system_ctx,
                        tenant: context.ctx.tenant.clone(),
                        project: context.ctx.project.clone(),
                        user_info,
                    };

                    evaluate_request_policy(&auth, &context.request, &policies).await?;

                    let mut result = if let Some(next) = next {
                        next(state, context).await?
                    } else {
                        context
                    };

                    if let Some(response) = result.response.take() {
                        result.response = Some(
                            map_search_entries(response, |entries| {
                                filter_unauthorized_includes(entries, &auth, &policies)
                            })
                            .await,
                        );
                    }

                    Ok(result)
                }
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth_n::middleware::jwt::User;
    use haste_fhir_client::{
        request::{FHIRRequest, FHIRResponse},
        url::ParsedParameters,
    };
    use haste_fhir_model::r4::generated::{
        resources::{
            AccessPolicyV2Attribute, AccessPolicyV2AttributeOperation, AccessPolicyV2Rule,
            AccessPolicyV2RuleCondition, Bundle, BundleEntrySearch, CapabilityStatement,
            Observation, Parameters, ResourceType,
        },
        terminology::{
            AccessPolicyAttributeOperationTypes, AccessPolicyv2Engine, ObservationStatus,
            SearchEntryMode,
        },
        types::{CodeableConcept, Expression, FHIRCode, FHIRId, FHIRString, Reference},
    };
    use haste_jwt::{
        AuthorId, AuthorKind, ProjectId, TenantId, UserRole,
        claims::{SubscriptionTier, UserTokenClaims},
        scopes::Scopes,
    };
    use haste_rate_limit::{RateLimit, RateLimitError};
    use haste_repository::types::SupportedFHIRVersions;
    use json_patch::Patch;
    use std::{collections::HashMap, pin::Pin};

    struct NoopRateLimit;
    impl RateLimit for NoopRateLimit {
        fn check<'a>(
            &'a self,
            _rate_key: &'a str,
            _max: i32,
            _points: i32,
            _window_in_seconds: i32,
        ) -> Pin<Box<dyn Future<Output = Result<i32, RateLimitError>> + Send + 'a>> {
            Box::pin(async { Ok(0) })
        }
    }

    /// Stands in for the server so PIP's attribute lookup resolves without
    /// Postgres/search-engine wiring. Every other method is unused here.
    #[derive(Clone, Default)]
    struct FakeClient {
        resources: HashMap<(ResourceType, String), Resource>,
    }

    impl FHIRClient<Arc<ServerCTX<FakeClient>>, OperationOutcomeError> for FakeClient {
        async fn read(
            &self,
            _ctx: Arc<ServerCTX<FakeClient>>,
            resource_type: ResourceType,
            id: String,
        ) -> Result<Option<Resource>, OperationOutcomeError> {
            Ok(self.resources.get(&(resource_type, id)).cloned())
        }

        async fn request(
            &self,
            _ctx: Arc<ServerCTX<FakeClient>>,
            _request: FHIRRequest,
        ) -> Result<FHIRResponse, OperationOutcomeError> {
            unimplemented!("not needed for policy evaluation")
        }
        async fn capabilities(
            &self,
            _ctx: Arc<ServerCTX<FakeClient>>,
        ) -> Result<CapabilityStatement, OperationOutcomeError> {
            unimplemented!("not needed for policy evaluation")
        }
        async fn search_system(
            &self,
            _ctx: Arc<ServerCTX<FakeClient>>,
            _parameters: ParsedParameters,
        ) -> Result<Bundle, OperationOutcomeError> {
            unimplemented!("not needed for policy evaluation")
        }
        async fn search_type(
            &self,
            _ctx: Arc<ServerCTX<FakeClient>>,
            _resource_type: ResourceType,
            _parameters: ParsedParameters,
        ) -> Result<Bundle, OperationOutcomeError> {
            unimplemented!("not needed for policy evaluation")
        }
        async fn create(
            &self,
            _ctx: Arc<ServerCTX<FakeClient>>,
            _resource_type: ResourceType,
            _resource: Resource,
        ) -> Result<Resource, OperationOutcomeError> {
            unimplemented!("not needed for policy evaluation")
        }
        async fn update(
            &self,
            _ctx: Arc<ServerCTX<FakeClient>>,
            _resource_type: ResourceType,
            _id: String,
            _resource: Resource,
        ) -> Result<Resource, OperationOutcomeError> {
            unimplemented!("not needed for policy evaluation")
        }
        async fn conditional_update(
            &self,
            _ctx: Arc<ServerCTX<FakeClient>>,
            _resource_type: ResourceType,
            _parameters: ParsedParameters,
            _resource: Resource,
        ) -> Result<Resource, OperationOutcomeError> {
            unimplemented!("not needed for policy evaluation")
        }
        async fn patch(
            &self,
            _ctx: Arc<ServerCTX<FakeClient>>,
            _resource_type: ResourceType,
            _id: String,
            _patches: Patch,
        ) -> Result<Resource, OperationOutcomeError> {
            unimplemented!("not needed for policy evaluation")
        }
        async fn vread(
            &self,
            _ctx: Arc<ServerCTX<FakeClient>>,
            _resource_type: ResourceType,
            _id: String,
            _version_id: String,
        ) -> Result<Option<Resource>, OperationOutcomeError> {
            unimplemented!("not needed for policy evaluation")
        }
        async fn delete_instance(
            &self,
            _ctx: Arc<ServerCTX<FakeClient>>,
            _resource_type: ResourceType,
            _id: String,
        ) -> Result<(), OperationOutcomeError> {
            unimplemented!("not needed for policy evaluation")
        }
        async fn delete_type(
            &self,
            _ctx: Arc<ServerCTX<FakeClient>>,
            _resource_type: ResourceType,
            _parameters: ParsedParameters,
        ) -> Result<(), OperationOutcomeError> {
            unimplemented!("not needed for policy evaluation")
        }
        async fn delete_system(
            &self,
            _ctx: Arc<ServerCTX<FakeClient>>,
            _parameters: ParsedParameters,
        ) -> Result<(), OperationOutcomeError> {
            unimplemented!("not needed for policy evaluation")
        }
        async fn history_system(
            &self,
            _ctx: Arc<ServerCTX<FakeClient>>,
            _parameters: ParsedParameters,
        ) -> Result<Bundle, OperationOutcomeError> {
            unimplemented!("not needed for policy evaluation")
        }
        async fn history_type(
            &self,
            _ctx: Arc<ServerCTX<FakeClient>>,
            _resource_type: ResourceType,
            _parameters: ParsedParameters,
        ) -> Result<Bundle, OperationOutcomeError> {
            unimplemented!("not needed for policy evaluation")
        }
        async fn history_instance(
            &self,
            _ctx: Arc<ServerCTX<FakeClient>>,
            _resource_type: ResourceType,
            _id: String,
            _parameters: ParsedParameters,
        ) -> Result<Bundle, OperationOutcomeError> {
            unimplemented!("not needed for policy evaluation")
        }
        async fn invoke_instance(
            &self,
            _ctx: Arc<ServerCTX<FakeClient>>,
            _resource_type: ResourceType,
            _id: String,
            _operation: String,
            _parameters: Parameters,
        ) -> Result<Resource, OperationOutcomeError> {
            unimplemented!("not needed for policy evaluation")
        }
        async fn invoke_type(
            &self,
            _ctx: Arc<ServerCTX<FakeClient>>,
            _resource_type: ResourceType,
            _operation: String,
            _parameters: Parameters,
        ) -> Result<Resource, OperationOutcomeError> {
            unimplemented!("not needed for policy evaluation")
        }
        async fn invoke_system(
            &self,
            _ctx: Arc<ServerCTX<FakeClient>>,
            _operation: String,
            _parameters: Parameters,
        ) -> Result<Resource, OperationOutcomeError> {
            unimplemented!("not needed for policy evaluation")
        }
        async fn transaction(
            &self,
            _ctx: Arc<ServerCTX<FakeClient>>,
            _bundle: Bundle,
        ) -> Result<Bundle, OperationOutcomeError> {
            unimplemented!("not needed for policy evaluation")
        }
        async fn batch(
            &self,
            _ctx: Arc<ServerCTX<FakeClient>>,
            _bundle: Bundle,
        ) -> Result<Bundle, OperationOutcomeError> {
            unimplemented!("not needed for policy evaluation")
        }
    }

    fn text_fhirpath(expression: &str) -> Box<Expression> {
        Box::new(Expression {
            language: Box::new(FHIRCode::from("text/fhirpath".to_string())),
            expression: Some(Box::new(FHIRString::from(expression.to_string()))),
            ..Default::default()
        })
    }

    /// Permits `Read` only when the PIP-fetched resource's `subject.reference`
    /// names the caller's own id — the untested compartment-policy pattern
    /// this file adds coverage for.
    fn compartment_policy() -> AccessPolicyV2 {
        AccessPolicyV2 {
            name: Box::new(FHIRString::from("compartment-test-policy".to_string())),
            engine: AccessPolicyv2Engine::rule_engine(),
            attribute: Some(vec![AccessPolicyV2Attribute {
                attributeId: Box::new(FHIRId::from("target".to_string())),
                operation: Some(AccessPolicyV2AttributeOperation {
                    type_: AccessPolicyAttributeOperationTypes::read(),
                    path: Some(text_fhirpath("%request.resource_type + '/' + %request.id")),
                    params: None,
                }),
            }]),
            rule: Some(vec![AccessPolicyV2Rule {
                name: Box::new(FHIRString::from("subject-is-caller".to_string())),
                condition: Some(AccessPolicyV2RuleCondition {
                    expression: text_fhirpath("%target.subject.reference = 'Patient/' + %user.id"),
                }),
                ..Default::default()
            }]),
            ..Default::default()
        }
    }

    fn observation(id: &str, subject_patient_id: &str) -> Resource {
        Resource::Observation(Observation {
            id: Some(id.to_string()),
            status: ObservationStatus::final_(),
            code: Box::new(CodeableConcept::default()),
            subject: Some(Box::new(Reference {
                reference: Some(Box::new(FHIRString::from(format!(
                    "Patient/{subject_patient_id}"
                )))),
                ..Default::default()
            })),
            ..Default::default()
        })
    }

    fn member_auth(
        client_resources: HashMap<(ResourceType, String), Resource>,
        user_id: &str,
    ) -> IncludeAuthContext<FakeClient> {
        let tenant = TenantId::new("test-tenant".to_string());
        let project = ProjectId::new("test-project".to_string());
        let fake_client = Arc::new(FakeClient {
            resources: client_resources,
        });

        let user = Arc::new(User {
            token: None,
            claims: UserTokenClaims {
                sub: AuthorId::new(user_id.to_string()),
                exp: 0,
                aud: AuthorKind::Membership.as_ref().to_string(),
                user_role: UserRole::Member,
                project: Some(project.clone()),
                tenant: tenant.clone(),
                subscription_tier: SubscriptionTier::Free,
                scope: Scopes(vec![]),
                fhir_user: None,
                user_id: AuthorId::new(user_id.to_string()),
                resource_type: AuthorKind::Membership,
                access_policy_version_ids: vec![],
                membership: None,
                fhir_version: SupportedFHIRVersions::R4,
            },
        });

        let system_ctx = Arc::new(ServerCTX::new(
            tenant.clone(),
            project.clone(),
            SupportedFHIRVersions::R4,
            user,
            fake_client.clone(),
            Arc::new(NoopRateLimit),
        ));

        IncludeAuthContext {
            client: fake_client,
            system_ctx,
            tenant,
            project,
            user_info: Arc::new(UserInfo {
                id: user_id.to_string(),
            }),
        }
    }

    /// Regression test: the old `Arc::try_unwrap`-based
    /// `evaluate_request_policy` denied every request once a rule referenced
    /// `%request` at all, regardless of the condition's actual verdict.
    #[tokio::test]
    async fn a_condition_referencing_request_does_not_break_policy_evaluation() {
        let resources = HashMap::new();
        let auth = member_auth(resources, "auth-patient");
        let policy = AccessPolicyV2 {
            name: Box::new(FHIRString::from("minimal".to_string())),
            engine: AccessPolicyv2Engine::rule_engine(),
            rule: Some(vec![AccessPolicyV2Rule {
                name: Box::new(FHIRString::from("r".to_string())),
                condition: Some(AccessPolicyV2RuleCondition {
                    expression: text_fhirpath("%request.type = 'read'"),
                }),
                ..Default::default()
            }]),
            ..Default::default()
        };
        let policies = vec![Arc::new(policy)];
        let read_request = FHIRRequest::Read(FHIRReadRequest {
            resource_type: ResourceType::Observation,
            id: "whatever".to_string(),
        });
        let result = evaluate_request_policy(&auth, &read_request, &policies).await;
        assert!(result.is_ok(), "policy evaluation error: {result:?}");
    }

    #[tokio::test]
    async fn a_compartment_policy_permits_reading_the_callers_own_resource() {
        let mut resources = HashMap::new();
        resources.insert(
            (ResourceType::Observation, "auth-obs".to_string()),
            observation("auth-obs", "auth-patient"),
        );
        let auth = member_auth(resources, "auth-patient");
        let policies = vec![Arc::new(compartment_policy())];

        let target = observation("auth-obs", "auth-patient");
        assert!(resource_is_readable(&target, &auth, &policies).await);
    }

    #[tokio::test]
    async fn a_compartment_policy_denies_reading_someone_elses_resource() {
        let mut resources = HashMap::new();
        resources.insert(
            (ResourceType::Observation, "other-obs".to_string()),
            observation("other-obs", "other-patient"),
        );
        let auth = member_auth(resources, "auth-patient");
        let policies = vec![Arc::new(compartment_policy())];

        let target = observation("other-obs", "other-patient");
        assert!(!resource_is_readable(&target, &auth, &policies).await);
    }

    #[tokio::test]
    async fn filter_unauthorized_includes_drops_only_the_denied_resource() {
        let mut resources = HashMap::new();
        resources.insert(
            (ResourceType::Observation, "auth-obs".to_string()),
            observation("auth-obs", "auth-patient"),
        );
        resources.insert(
            (ResourceType::Observation, "other-obs".to_string()),
            observation("other-obs", "other-patient"),
        );
        let auth = member_auth(resources, "auth-patient");
        let policies = vec![Arc::new(compartment_policy())];

        let match_entry = BundleEntry {
            resource: Some(Box::new(observation("match-obs", "irrelevant"))),
            search: Some(BundleEntrySearch {
                mode: Some(SearchEntryMode::match_()),
                ..Default::default()
            }),
            ..Default::default()
        };
        let authorized_include = BundleEntry {
            resource: Some(Box::new(observation("auth-obs", "auth-patient"))),
            search: Some(BundleEntrySearch {
                mode: Some(SearchEntryMode::include()),
                ..Default::default()
            }),
            ..Default::default()
        };
        let denied_include = BundleEntry {
            resource: Some(Box::new(observation("other-obs", "other-patient"))),
            search: Some(BundleEntrySearch {
                mode: Some(SearchEntryMode::include()),
                ..Default::default()
            }),
            ..Default::default()
        };

        let kept = filter_unauthorized_includes(
            vec![match_entry, authorized_include, denied_include],
            &auth,
            &policies,
        )
        .await;

        assert_eq!(
            kept.len(),
            2,
            "the match and the authorized include both survive"
        );
        let ids: Vec<_> = kept
            .iter()
            .filter_map(|e| e.resource.as_deref())
            .filter_map(|r| r.id().clone())
            .collect();
        assert!(ids.contains(&"match-obs".to_string()));
        assert!(ids.contains(&"auth-obs".to_string()));
        assert!(!ids.contains(&"other-obs".to_string()));
    }
}
