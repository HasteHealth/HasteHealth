#![allow(non_snake_case)]
#[doc = "This operation is used to search for and return notifications that have been previously triggered by"]
#[doc = "a topic-based Subscription in R4."]
pub mod BackportSubscriptionEvents {
    use haste_fhir_model::r4::generated::resources::{
        Bundle, Parameters, ParametersParameter, Resource,
    };
    use haste_fhir_model::r4::generated::types::{FHIRCode, FHIRString};
    use haste_fhir_operation_error::OperationOutcomeError;
    use haste_fhir_ops::derive::{FromParameters, ToParameters};
    pub const CODE: &str = "events";
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Input {
        #[doc = "The starting event number, inclusive of this event (lower bound)."]
        pub eventsSinceNumber: Option<FHIRString>,
        #[doc = "The ending event number, inclusive of this event (upper bound)."]
        pub eventsUntilNumber: Option<FHIRString>,
        #[doc = "Requested content style of returned data. Codes from backport-content-value-set (e.g., empty,"]
        #[doc = "id-only, full-resource). This is a hint to the server what a client would prefer, and MAY be"]
        #[doc = "ignored."]
        pub content: Option<FHIRCode>,
    }
    impl From<Input> for Resource {
        fn from(value: Input) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
    #[derive(Debug, FromParameters)]
    pub struct Output {
        #[doc = "The operation returns a valid notification bundle, with the first entry being the subscription"]
        #[doc = "status information resource. The bundle type is \"history\"."]
        #[parameter_rename = "return"]
        pub return_: Bundle,
    }
    impl From<Output> for Resource {
        fn from(value: Output) -> Self {
            Resource::Bundle(value.return_)
        }
    }
}
#[doc = "This operation is used to get a token for a websocket client to use in order to bind to one or more"]
#[doc = "subscriptions."]
pub mod BackportSubscriptionGetWsBindingToken {
    use haste_fhir_model::r4::generated::resources::{Parameters, ParametersParameter, Resource};
    use haste_fhir_model::r4::generated::types::{FHIRDateTime, FHIRId, FHIRString, FHIRUrl};
    use haste_fhir_operation_error::OperationOutcomeError;
    use haste_fhir_ops::derive::{FromParameters, ToParameters};
    pub const CODE: &str = "get-ws-binding-token";
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Input {
        #[doc = "At the Instance level, this parameter is ignored. At the Resource level, one or more parameters"]
        #[doc = "containing a FHIR id for a Subscription to get a token for. In the absense of any specified ids, the"]
        #[doc = "server may either return a token for all Subscriptions available to the caller with a channel-type"]
        #[doc = "of websocket or fail the request."]
        pub id: Option<Vec<FHIRId>>,
    }
    impl From<Input> for Resource {
        fn from(value: Input) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Output {
        #[doc = "An access token that a client may use to show authorization during a websocket connection."]
        pub token: FHIRString,
        #[doc = "The date and time this token is valid until."]
        pub expiration: FHIRDateTime,
        #[doc = "The subscriptions this token is valid for."]
        pub subscription: Option<Vec<FHIRString>>,
        #[doc = "The URL the client should use to connect to Websockets."]
        #[parameter_rename = "websocket-url"]
        pub websocket_url: FHIRUrl,
    }
    impl From<Output> for Resource {
        fn from(value: Output) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
}
#[doc = "This operation is used to resend notifications that have been previously triggered by a topic-based"]
#[doc = "Subscription in R4."]
pub mod BackportSubscriptionResend {
    use haste_fhir_model::r4::generated::resources::{
        OperationOutcome, Parameters, ParametersParameter, Resource,
    };
    use haste_fhir_model::r4::generated::types::{FHIRInstant, FHIRString};
    use haste_fhir_operation_error::OperationOutcomeError;
    use haste_fhir_ops::derive::{FromParameters, ToParameters};
    pub const CODE: &str = "resend";
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Input {
        #[doc = "The starting event number, inclusive of this event (lower bound)."]
        pub eventsSinceNumber: Option<FHIRString>,
        #[doc = "The ending event number, inclusive of this event (upper bound)."]
        pub eventsUntilNumber: Option<FHIRString>,
        #[doc = "The starting event date/time, inclusive of this instant (lower bound)."]
        pub eventsSinceInstant: Option<FHIRInstant>,
        #[doc = "The ending event date/time, inclusive of this instant (upper bound)."]
        pub eventsUntilInstant: Option<FHIRInstant>,
    }
    impl From<Input> for Resource {
        fn from(value: Input) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
    #[derive(Debug, FromParameters)]
    pub struct Output {
        #[doc = "Outcome of this request."]
        #[parameter_rename = "return"]
        pub return_: OperationOutcome,
    }
    impl From<Output> for Resource {
        fn from(value: Output) -> Self {
            Resource::OperationOutcome(value.return_)
        }
    }
}
#[doc = "This operation is used to return the current status information about one or more topic-based"]
#[doc = "Subscriptions in R4."]
pub mod BackportSubscriptionStatus {
    use haste_fhir_model::r4::generated::resources::{
        Bundle, Parameters, ParametersParameter, Resource,
    };
    use haste_fhir_model::r4::generated::types::{FHIRCode, FHIRId, FHIRString};
    use haste_fhir_operation_error::OperationOutcomeError;
    use haste_fhir_ops::derive::{FromParameters, ToParameters};
    pub const CODE: &str = "status";
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Input {
        #[doc = "At the Instance level, this parameter is ignored. At the Resource level, one or more parameters"]
        #[doc = "containing a FHIR id for a Subscription to get status information for. In the absence of any"]
        #[doc = "specified ids, the server returns the status for all Subscriptions available to the caller. Multiple"]
        #[doc = "values are joined via OR (e.g., \"id1\" OR \"id2\")."]
        pub id: Option<Vec<FHIRId>>,
        #[doc = "At the Instance level, this parameter is ignored. At the Resource level, a Subscription status to"]
        #[doc = "filter by (e.g., \"active\"). In the absence of any specified status values, the server does not"]
        #[doc = "filter contents based on the status. Multiple values are joined via OR (e.g., \"error\" OR \"off\")."]
        pub status: Option<Vec<FHIRCode>>,
    }
    impl From<Input> for Resource {
        fn from(value: Input) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
    #[derive(Debug, FromParameters)]
    pub struct Output {
        #[doc = "The operation returns a bundle containing one or more subscription status resources, one per"]
        #[doc = "Subscription being queried. The Bundle type is \"searchset\"."]
        #[parameter_rename = "return"]
        pub return_: Bundle,
    }
    impl From<Output> for Resource {
        fn from(value: Output) -> Self {
            Resource::Bundle(value.return_)
        }
    }
}
#[doc = "Get Project resource for the current project."]
pub mod ProjectInformation {
    use haste_fhir_model::r4::generated::resources::{
        Parameters, ParametersParameter, Project, Resource,
    };
    use haste_fhir_model::r4::generated::types::FHIRString;
    use haste_fhir_operation_error::OperationOutcomeError;
    use haste_fhir_ops::derive::{FromParameters, ToParameters};
    pub const CODE: &str = "current-project";
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Input {}
    impl From<Input> for Resource {
        fn from(value: Input) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Output {
        #[doc = "Users current project."]
        pub project: Project,
    }
    impl From<Output> for Resource {
        fn from(value: Output) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
}
#[doc = "Get tenant information for the current tenant."]
pub mod TenantInformation {
    use haste_fhir_model::r4::generated::resources::{Parameters, ParametersParameter, Resource};
    use haste_fhir_model::r4::generated::types::{FHIRCode, FHIRString};
    use haste_fhir_operation_error::OperationOutcomeError;
    use haste_fhir_ops::derive::{FromParameters, ToParameters};
    pub const CODE: &str = "current-tenant";
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Input {}
    impl From<Input> for Resource {
        fn from(value: Input) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Output {
        #[doc = "tenant id"]
        pub id: FHIRString,
        #[doc = "tenant subscription level"]
        pub subscription: FHIRCode,
    }
    impl From<Output> for Resource {
        fn from(value: Output) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
}
#[doc = "Get tenant endpoint information for the current tenant."]
pub mod TenantEndpointInformation {
    use haste_fhir_model::r4::generated::resources::{Parameters, ParametersParameter, Resource};
    use haste_fhir_model::r4::generated::types::{FHIRString, FHIRUri};
    use haste_fhir_operation_error::OperationOutcomeError;
    use haste_fhir_ops::derive::{FromParameters, ToParameters};
    pub const CODE: &str = "endpoints";
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Input {}
    impl From<Input> for Resource {
        fn from(value: Input) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Output {
        #[doc = "FHIR R4 Endpoint URL."]
        #[parameter_rename = "fhir-r4-base-url"]
        pub fhir_r4_base_url: FHIRUri,
        #[doc = "FHIR R4 Capabilities URL."]
        #[parameter_rename = "fhir-r4-capabilities-url"]
        pub fhir_r4_capabilities_url: FHIRUri,
        #[doc = "OIDC Discovery URL."]
        #[parameter_rename = "oidc-discovery-url"]
        pub oidc_discovery_url: FHIRUri,
        #[doc = "OIDC Token Endpoint."]
        #[parameter_rename = "oidc-token-endpoint"]
        pub oidc_token_endpoint: FHIRUri,
        #[doc = "OIDC Authorize Endpoint."]
        #[parameter_rename = "oidc-authorize-endpoint"]
        pub oidc_authorize_endpoint: FHIRUri,
        #[doc = "OIDC JWKS Endpoint."]
        #[parameter_rename = "oidc-jwks-endpoint"]
        pub oidc_jwks_endpoint: FHIRUri,
        #[doc = "Model context protocol endpoint."]
        #[parameter_rename = "mcp-endpoint"]
        pub mcp_endpoint: FHIRUri,
    }
    impl From<Output> for Resource {
        fn from(value: Output) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
}
#[doc = "Evaluate an Access Policy."]
pub mod HasteHealthEvaluatePolicy {
    use haste_fhir_model::r4::generated::resources::{
        Bundle, OperationOutcome, Parameters, ParametersParameter, Resource,
    };
    use haste_fhir_model::r4::generated::types::{FHIRString, Reference};
    use haste_fhir_operation_error::OperationOutcomeError;
    use haste_fhir_ops::derive::{FromParameters, ToParameters};
    pub const CODE: &str = "evaluate-policy";
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Input {
        #[doc = "The user to evaluate the policy against. Defaults to logged in user if not present."]
        pub user: Option<Reference>,
        #[doc = "The requests to evaluate against the policy."]
        pub request: Bundle,
    }
    impl From<Input> for Resource {
        fn from(value: Input) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
    #[derive(Debug, FromParameters)]
    pub struct Output {
        #[doc = "The result of the policy evaluation."]
        #[parameter_rename = "return"]
        pub return_: OperationOutcome,
    }
    impl From<Output> for Resource {
        fn from(value: Output) -> Self {
            Resource::OperationOutcome(value.return_)
        }
    }
}
#[doc = "Parse `HL7v2` messages."]
pub mod Hl7v2Parse {
    use haste_fhir_model::r4::generated::resources::{
        HL7V2, Parameters, ParametersParameter, Resource,
    };
    use haste_fhir_model::r4::generated::types::FHIRString;
    use haste_fhir_operation_error::OperationOutcomeError;
    use haste_fhir_ops::derive::{FromParameters, ToParameters};
    pub const CODE: &str = "hl7v2-parse";
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Input {
        #[doc = "`HL7v2` message to be parsed."]
        pub hl7v2: FHIRString,
    }
    impl From<Input> for Resource {
        fn from(value: Input) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Output {
        #[doc = "Parsed `HL7v2` message."]
        pub hl7v2: HL7V2,
    }
    impl From<Output> for Resource {
        fn from(value: Output) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
}
#[doc = "Get the registration information for an identity provider."]
pub mod HasteHealthIdpRegistrationInfo {
    use haste_fhir_model::r4::generated::resources::{Parameters, ParametersParameter, Resource};
    use haste_fhir_model::r4::generated::types::FHIRString;
    use haste_fhir_operation_error::OperationOutcomeError;
    use haste_fhir_ops::derive::{FromParameters, ToParameters};
    pub const CODE: &str = "registration-info";
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Input {}
    impl From<Input> for Resource {
        fn from(value: Input) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct OutputInformation {
        #[doc = "The name of the property."]
        pub name: FHIRString,
        #[doc = "the value of the property."]
        pub value: FHIRString,
    }
    impl From<OutputInformation> for Resource {
        fn from(value: OutputInformation) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Output {
        #[doc = "`IdentityProviders` registration information."]
        #[parameter_nested]
        pub information: Option<Vec<OutputInformation>>,
    }
    impl From<Output> for Resource {
        fn from(value: Output) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
}
#[doc = "Show resources that failed search indexing for the current tenant and project."]
pub mod HasteHealthIndexingErrors {
    use haste_fhir_model::r4::generated::resources::{Parameters, ParametersParameter, Resource};
    use haste_fhir_model::r4::generated::types::{
        FHIRBoolean, FHIRCode, FHIRDateTime, FHIRId, FHIRInteger, FHIRString,
    };
    use haste_fhir_operation_error::OperationOutcomeError;
    use haste_fhir_ops::derive::{FromParameters, ToParameters};
    pub const CODE: &str = "indexing-errors";
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Input {
        #[doc = "If true, mark the indexing errors as resolved."]
        pub resolve: Option<FHIRBoolean>,
    }
    impl From<Input> for Resource {
        fn from(value: Input) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct OutputErrors {
        #[doc = "Version id of the resource that failed to index."]
        pub id: FHIRId,
        #[doc = "Version id of the resource that failed to index."]
        pub version_id: FHIRId,
        #[doc = "FHIR resource type of the resource that failed to index."]
        pub resource_type: FHIRCode,
        #[doc = "The operation (create, update or delete) that failed to index."]
        pub fhir_method: FHIRCode,
        #[doc = "Sequence position of the resource in the indexing stream."]
        pub sequence: FHIRInteger,
        #[doc = "Number of times indexing has been attempted for this resource."]
        pub attempt_count: FHIRInteger,
        #[doc = "The error that occurred while indexing."]
        pub error_message: FHIRString,
        #[doc = "When this resource first failed indexing."]
        pub first_failed_at: FHIRDateTime,
        #[doc = "When this resource most recently failed indexing."]
        pub last_failed_at: FHIRDateTime,
        #[doc = "When this failure was marked resolved, if it has been."]
        pub resolved_at: Option<FHIRDateTime>,
    }
    impl From<OutputErrors> for Resource {
        fn from(value: OutputErrors) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Output {
        #[doc = "The resources that failed search indexing."]
        #[parameter_nested]
        pub errors: Option<Vec<OutputErrors>>,
    }
    impl From<Output> for Resource {
        fn from(value: Output) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
}
#[doc = "Delete refresh token from the user for the client."]
pub mod HasteHealthDeleteRefreshToken {
    use haste_fhir_model::r4::generated::resources::{
        OperationOutcome, Parameters, ParametersParameter, Resource,
    };
    use haste_fhir_model::r4::generated::types::{FHIRId, FHIRString};
    use haste_fhir_operation_error::OperationOutcomeError;
    use haste_fhir_ops::derive::{FromParameters, ToParameters};
    pub const CODE: &str = "delete-refresh-token";
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Input {
        #[doc = "Client id for refresh token to delete."]
        pub client_id: FHIRId,
        #[doc = "User agent of the refresh token to delete."]
        pub user_agent: Option<FHIRString>,
    }
    impl From<Input> for Resource {
        fn from(value: Input) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
    #[derive(Debug, FromParameters)]
    pub struct Output {
        #[doc = "Result of the delete operation."]
        #[parameter_rename = "return"]
        pub return_: OperationOutcome,
    }
    impl From<Output> for Resource {
        fn from(value: Output) -> Self {
            Resource::OperationOutcome(value.return_)
        }
    }
}
#[doc = "Show list of users refresh tokens."]
pub mod HasteHealthListRefreshTokens {
    use haste_fhir_model::r4::generated::resources::{Parameters, ParametersParameter, Resource};
    use haste_fhir_model::r4::generated::types::{FHIRDateTime, FHIRId, FHIRString};
    use haste_fhir_operation_error::OperationOutcomeError;
    use haste_fhir_ops::derive::{FromParameters, ToParameters};
    pub const CODE: &str = "refresh-tokens";
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Input {}
    impl From<Input> for Resource {
        fn from(value: Input) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct OutputRefreshTokens {
        #[doc = "Client for refresh token."]
        pub client_id: FHIRId,
        #[doc = "User agent of the refresh token."]
        pub user_agent: FHIRString,
        #[doc = "When the refresh token was created."]
        pub created_at: FHIRDateTime,
    }
    impl From<OutputRefreshTokens> for Resource {
        fn from(value: OutputRefreshTokens) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Output {
        #[doc = "The result of the operation."]
        #[parameter_rename = "refresh-tokens"]
        #[parameter_nested]
        pub refresh_tokens: Option<Vec<OutputRefreshTokens>>,
    }
    impl From<Output> for Resource {
        fn from(value: Output) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
}
#[doc = "Delete scope from user accepted scopes for the client."]
pub mod HasteHealthDeleteScope {
    use haste_fhir_model::r4::generated::resources::{
        OperationOutcome, Parameters, ParametersParameter, Resource,
    };
    use haste_fhir_model::r4::generated::types::{FHIRId, FHIRString};
    use haste_fhir_operation_error::OperationOutcomeError;
    use haste_fhir_ops::derive::{FromParameters, ToParameters};
    pub const CODE: &str = "delete-scope";
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Input {
        #[doc = "Client for which scopes are being shown."]
        pub client_id: FHIRId,
    }
    impl From<Input> for Resource {
        fn from(value: Input) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
    #[derive(Debug, FromParameters)]
    pub struct Output {
        #[doc = "Result of the delete operation."]
        #[parameter_rename = "return"]
        pub return_: OperationOutcome,
    }
    impl From<Output> for Resource {
        fn from(value: Output) -> Self {
            Resource::OperationOutcome(value.return_)
        }
    }
}
#[doc = "Show list of user accepted scopes for apps."]
pub mod HasteHealthListScopes {
    use haste_fhir_model::r4::generated::resources::{Parameters, ParametersParameter, Resource};
    use haste_fhir_model::r4::generated::types::{FHIRDateTime, FHIRId, FHIRString};
    use haste_fhir_operation_error::OperationOutcomeError;
    use haste_fhir_ops::derive::{FromParameters, ToParameters};
    pub const CODE: &str = "scopes";
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Input {}
    impl From<Input> for Resource {
        fn from(value: Input) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct OutputScopes {
        #[doc = "Client for which scopes are being shown."]
        pub client_id: FHIRId,
        #[doc = "Scopes user accepted."]
        pub scopes: FHIRString,
        #[doc = "When the scopes were accepted."]
        pub created_at: FHIRDateTime,
    }
    impl From<OutputScopes> for Resource {
        fn from(value: OutputScopes) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Output {
        #[doc = "The result of the operation."]
        #[parameter_nested]
        pub scopes: Option<Vec<OutputScopes>>,
    }
    impl From<Output> for Resource {
        fn from(value: Output) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
}
#[doc = "Set display customization for the current tenant, such as its display name and logo."]
pub mod HasteHealthTenantCustomization {
    use haste_fhir_model::r4::generated::resources::{Parameters, ParametersParameter, Resource};
    use haste_fhir_model::r4::generated::types::{Attachment, FHIRString};
    use haste_fhir_operation_error::OperationOutcomeError;
    use haste_fhir_ops::derive::{FromParameters, ToParameters};
    pub const CODE: &str = "tenant-customization";
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Input {
        #[doc = "Display name for the tenant. Omit to clear the tenant's display name."]
        pub name: Option<FHIRString>,
        #[doc = "Logo for the tenant. Omit to clear the tenant's logo."]
        pub logo: Option<Attachment>,
    }
    impl From<Input> for Resource {
        fn from(value: Input) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Output {}
    impl From<Output> for Resource {
        fn from(value: Output) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
}
#[doc = "Get the current tenant's display name and logo, if set."]
pub mod HasteHealthTenantBranding {
    use haste_fhir_model::r4::generated::resources::{Parameters, ParametersParameter, Resource};
    use haste_fhir_model::r4::generated::types::{Attachment, FHIRString};
    use haste_fhir_operation_error::OperationOutcomeError;
    use haste_fhir_ops::derive::{FromParameters, ToParameters};
    pub const CODE: &str = "tenant-branding";
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Input {}
    impl From<Input> for Resource {
        fn from(value: Input) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Output {
        #[doc = "Display name for the tenant, if one has been set."]
        pub name: Option<FHIRString>,
        #[doc = "Logo for the tenant, if one has been set."]
        pub logo: Option<Attachment>,
    }
    impl From<Output> for Resource {
        fn from(value: Output) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
}
#[doc = "The apply operation applies a definition in a specific context"]
pub mod ActivityDefinitionApply {
    use haste_fhir_model::r4::generated::resources::{
        ActivityDefinition, Parameters, ParametersParameter, Resource,
    };
    use haste_fhir_model::r4::generated::types::{CodeableConcept, FHIRString};
    use haste_fhir_operation_error::OperationOutcomeError;
    use haste_fhir_ops::derive::{FromParameters, ToParameters};
    pub const CODE: &str = "apply";
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Input {
        #[doc = "The activity definition to apply. If the operation is invoked on an instance, this parameter is not"]
        #[doc = "allowed. If the operation is invoked at the type level, this parameter is required"]
        pub activityDefinition: Option<ActivityDefinition>,
        #[doc = "The subject(s) that is/are the target of the activity definition to be applied. The subject may be a"]
        #[doc = "Patient, Practitioner, Organization, Location, Device, or Group. Subjects provided in this parameter"]
        #[doc = "will be resolved as the subject of the `PlanDefinition` based on the type of the subject. If"]
        #[doc = "multiple subjects of the same type are provided, the behavior is implementation-defined"]
        pub subject: Vec<FHIRString>,
        #[doc = "The encounter in context, if any"]
        pub encounter: Option<FHIRString>,
        #[doc = "The practitioner in context"]
        pub practitioner: Option<FHIRString>,
        #[doc = "The organization in context"]
        pub organization: Option<FHIRString>,
        #[doc = "The type of user initiating the request, e.g. patient, healthcare provider, or specific type of"]
        #[doc = "healthcare provider (physician, nurse, etc.)"]
        pub userType: Option<CodeableConcept>,
        #[doc = "Preferred language of the person using the system"]
        pub userLanguage: Option<CodeableConcept>,
        #[doc = "The task the system user is performing, e.g. laboratory results review, medication list review, etc."]
        #[doc = "This information can be used to tailor decision support outputs, such as recommended information"]
        #[doc = "resources"]
        pub userTaskContext: Option<CodeableConcept>,
        #[doc = "The current setting of the request (inpatient, outpatient, etc.)"]
        pub setting: Option<CodeableConcept>,
        #[doc = "Additional detail about the setting of the request, if any"]
        pub settingContext: Option<CodeableConcept>,
    }
    impl From<Input> for Resource {
        fn from(value: Input) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
    #[derive(Debug, FromParameters)]
    pub struct Output {
        #[doc = "The resource that is the result of applying the definition"]
        #[parameter_rename = "return"]
        pub return_: Resource,
    }
    impl From<Output> for Resource {
        fn from(value: Output) -> Self {
            value.return_
        }
    }
}
#[doc = "The data-requirements operation aggregates and returns the parameters and data requirements for the"]
#[doc = "activity definition and all its dependencies as a single module definition library"]
pub mod ActivityDefinitionDataRequirements {
    use haste_fhir_model::r4::generated::resources::{
        Library, Parameters, ParametersParameter, Resource,
    };
    use haste_fhir_operation_error::OperationOutcomeError;
    use haste_fhir_ops::derive::{FromParameters, ToParameters};
    pub const CODE: &str = "data-requirements";
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Input {}
    impl From<Input> for Resource {
        fn from(value: Input) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
    #[derive(Debug, FromParameters)]
    pub struct Output {
        #[doc = "The result of the requirements gathering represented as a module-definition Library that describes"]
        #[doc = "the aggregate parameters, data requirements, and dependencies of the activity definition"]
        #[parameter_rename = "return"]
        pub return_: Library,
    }
    impl From<Output> for Resource {
        fn from(value: Output) -> Self {
            Resource::Library(value.return_)
        }
    }
}
#[doc = "This operation asks the server to check that it implements all the resources, interactions, search"]
#[doc = "parameters, and operations that the client provides in its capability statement. The client provides"]
#[doc = "both capability statements by reference, and must ensure that all the referenced resources are"]
#[doc = "available to the conformance server"]
pub mod CapabilityStatementConforms {
    use haste_fhir_model::r4::generated::resources::{
        CapabilityStatement, OperationOutcome, Parameters, ParametersParameter, Resource,
    };
    use haste_fhir_model::r4::generated::types::{FHIRCanonical, FHIRCode, FHIRString};
    use haste_fhir_operation_error::OperationOutcomeError;
    use haste_fhir_ops::derive::{FromParameters, ToParameters};
    pub const CODE: &str = "conforms";
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Input {
        #[doc = "A canonical reference to the left-hand system's capability statement"]
        pub left: Option<FHIRCanonical>,
        #[doc = "A canonical reference to the right-hand system's capability statement"]
        pub right: Option<FHIRCanonical>,
        #[doc = "What kind of comparison to perform - server to server, or client to server (use the codes"]
        #[doc = "'server/server' or 'client/server')"]
        pub mode: Option<FHIRCode>,
    }
    impl From<Input> for Resource {
        fn from(value: Input) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Output {
        #[doc = "Outcome of the `CapabilityStatement` test"]
        pub issues: OperationOutcome,
        #[doc = "The intersection of the functionality described by the `CapabilityStatement` resources"]
        #[parameter_rename = "union"]
        pub union_: Option<CapabilityStatement>,
        #[doc = "The union of the functionality described by the `CapabilityStatement` resources"]
        pub intersection: Option<CapabilityStatement>,
    }
    impl From<Output> for Resource {
        fn from(value: Output) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
}
#[doc = "This operation asks the server to check that it implements all the resources, interactions, search"]
#[doc = "parameters, and operations that the client provides in its capability statement. The client provides"]
#[doc = "its capability statement inline, or by referring the server to the canonical URL of its capability"]
#[doc = "statement"]
pub mod CapabilityStatementImplements {
    use haste_fhir_model::r4::generated::resources::{
        CapabilityStatement, OperationOutcome, Parameters, ParametersParameter, Resource,
    };
    use haste_fhir_model::r4::generated::types::{FHIRCanonical, FHIRString};
    use haste_fhir_operation_error::OperationOutcomeError;
    use haste_fhir_ops::derive::{FromParameters, ToParameters};
    pub const CODE: &str = "implements";
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Input {
        #[doc = "A canonical reference to the server capability statement - use this if the implements is not invoked"]
        #[doc = "on an instance (or on the /metadata end-point)"]
        pub server: Option<FHIRCanonical>,
        #[doc = "A canonical reference to the client capability statement - use this if the implements is not invoked"]
        #[doc = "on an instance (or on the /metadata end-point)"]
        pub client: Option<FHIRCanonical>,
        #[doc = "The client capability statement, provided inline"]
        pub resource: Option<CapabilityStatement>,
    }
    impl From<Input> for Resource {
        fn from(value: Input) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
    #[derive(Debug, FromParameters)]
    pub struct Output {
        #[doc = "Outcome of the `CapabilityStatement` test"]
        #[parameter_rename = "return"]
        pub return_: OperationOutcome,
    }
    impl From<Output> for Resource {
        fn from(value: Output) -> Self {
            Resource::OperationOutcome(value.return_)
        }
    }
}
#[doc = "This operation asks the server to return a subset of the `CapabilityStatement` resource - just the"]
#[doc = "REST parts that relate to a set of nominated resources - the resources that the client is interested"]
#[doc = "in"]
pub mod CapabilityStatementSubset {
    use haste_fhir_model::r4::generated::resources::{
        CapabilityStatement, Parameters, ParametersParameter, Resource,
    };
    use haste_fhir_model::r4::generated::types::{FHIRCode, FHIRString, FHIRUri};
    use haste_fhir_operation_error::OperationOutcomeError;
    use haste_fhir_ops::derive::{FromParameters, ToParameters};
    pub const CODE: &str = "subset";
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Input {
        #[doc = "The canonical URL - use this if the subset is not invoked on an instance (or on the /metadata"]
        #[doc = "end-point)"]
        pub server: Option<FHIRUri>,
        #[doc = "A resource that the client would like to include in the return"]
        pub resource: Vec<FHIRCode>,
    }
    impl From<Input> for Resource {
        fn from(value: Input) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
    #[derive(Debug, FromParameters)]
    pub struct Output {
        #[doc = "The subsetted `CapabilityStatement` resource that is returned. This should be tagged with the"]
        #[doc = "SUBSETTED code"]
        #[parameter_rename = "return"]
        pub return_: CapabilityStatement,
    }
    impl From<Output> for Resource {
        fn from(value: Output) -> Self {
            Resource::CapabilityStatement(value.return_)
        }
    }
}
#[doc = "Using the [`FHIR Version Mime Type Parameter`](http.html#version-parameter), a server can support"]
#[doc = "[`multiple versions on the same end-point`](versioning.html#mt-version). The only way for client to"]
#[doc = "find out what versions a server supports in this fashion is the $versions operation. The client"]
#[doc = "invokes the operation with no parameters. and the server returns the list of supported versions,"]
#[doc = "along with the default version it will use if no fhirVersion parameter is present"]
pub mod CapabilityStatementVersions {
    use haste_fhir_model::r4::generated::resources::{Parameters, ParametersParameter, Resource};
    use haste_fhir_model::r4::generated::types::{FHIRCode, FHIRString};
    use haste_fhir_operation_error::OperationOutcomeError;
    use haste_fhir_ops::derive::{FromParameters, ToParameters};
    pub const CODE: &str = "versions";
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Input {}
    impl From<Input> for Resource {
        fn from(value: Input) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Output {
        #[doc = "A version supported by the server. Use the major.minor version like 3.0"]
        pub version: Vec<FHIRCode>,
        #[doc = "The default version for the server. Use the major.minor version like 3.0"]
        pub default: FHIRCode,
    }
    impl From<Output> for Resource {
        fn from(value: Output) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
}
#[doc = "The apply operation applies a definition in a specific context"]
pub mod ChargeItemDefinitionApply {
    use haste_fhir_model::r4::generated::resources::{Parameters, ParametersParameter, Resource};
    use haste_fhir_model::r4::generated::types::{FHIRString, Reference};
    use haste_fhir_operation_error::OperationOutcomeError;
    use haste_fhir_ops::derive::{FromParameters, ToParameters};
    pub const CODE: &str = "apply";
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Input {
        #[doc = "The `ChargeItem` on which the definition is to ba applies"]
        pub chargeItem: Reference,
        #[doc = "The account in context, if any"]
        pub account: Option<Reference>,
    }
    impl From<Input> for Resource {
        fn from(value: Input) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
    #[derive(Debug, FromParameters)]
    pub struct Output {
        #[doc = "The resource that is the result of applying the definition"]
        #[parameter_rename = "return"]
        pub return_: Resource,
    }
    impl From<Output> for Resource {
        fn from(value: Output) -> Self {
            value.return_
        }
    }
}
#[doc = "This operation is used to submit a Claim, `Pre-Authorization` or `Pre-Determination` (all instances"]
#[doc = "of Claim resources) for adjudication either as a single Claim resource instance or as a Bundle"]
#[doc = "containing the Claim and other referenced resources, or Bundle containing a batch of Claim"]
#[doc = "resources, either as single Claims resources or Bundle resources, for processing. The only input"]
#[doc = "parameter is the single Claim or Bundle resource and the only output is a single `ClaimResponse`,"]
#[doc = "Bundle of `ClaimResponses` or an `OperationOutcome` resource."]
pub mod ClaimSubmit {
    use haste_fhir_model::r4::generated::resources::{Parameters, ParametersParameter, Resource};
    use haste_fhir_model::r4::generated::types::FHIRString;
    use haste_fhir_operation_error::OperationOutcomeError;
    use haste_fhir_ops::derive::{FromParameters, ToParameters};
    pub const CODE: &str = "submit";
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Input {
        #[doc = "A Claim resource or Bundle of claims, either as individual Claim resources or as Bundles each"]
        #[doc = "containing a single Claim plus referenced resources."]
        pub resource: Resource,
    }
    impl From<Input> for Resource {
        fn from(value: Input) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Output {
        #[doc = "A `ClaimResponse` resource or Bundle of claim responses, either as individual `ClaimResponse`"]
        #[doc = "resources or as Bundles each containing a single `ClaimResponse` plus referenced resources."]
        #[parameter_rename = "return"]
        pub return_: Resource,
    }
    impl From<Output> for Resource {
        fn from(value: Output) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
}
#[doc = "Given a set of properties (and text), return one or more possible matching codes"]
#[doc = ""]
#[doc = "This operation takes a set of properties, and examines the code system looking for codes in the code"]
#[doc = "system that match a set of known properties."]
#[doc = ""]
#[doc = "When looking for matches, there are 3 possible types of match:"]
#[doc = "* a complete match - a code that represents all the provided properties correctly"]
#[doc = "* a partial match - a code that represents some of the provided properties correctly, and not others"]
#[doc = "* a possible match - a code that may represent the provided properties closely, but may capture less"]
#[doc = "or more precise information for some of the properties"]
#[doc = ""]
#[doc = "The $find-matches operation can be called in one of 2 modes:"]
#[doc = "* By a human, looking for the best match for a set of properties. In this mode, the server returns a"]
#[doc = "list of complete, possible or partial matches (possibly with comments), so that the user can choose"]
#[doc = "(or not) the most appropriate code"]
#[doc = "* By a machine (typically in a system interface performing a transformation). In this mode, the"]
#[doc = "server returns only a list of complete and partial matches, but no possible matches. The machine can"]
#[doc = "choose a code from the list (or not) based on what properties are not coded"]
#[doc = ""]
#[doc = "These modes are differentiated by the `exact` parameter, so the client can indicate whether it only"]
#[doc = "wants exact matches (including partial matches) or whether potential matches based on text matching"]
#[doc = "are desired"]
#[doc = "The find-matches operation is still preliminary. The interface can be expected to change as more"]
#[doc = "experience is gained from implementations."]
pub mod CodeSystemFindMatches {
    use haste_fhir_model::r4::generated::resources::{
        Parameters, ParametersParameter, ParametersParameterValueTypeChoice, Resource,
    };
    use haste_fhir_model::r4::generated::types::{
        Coding, FHIRBoolean, FHIRCode, FHIRString, FHIRUri,
    };
    use haste_fhir_operation_error::OperationOutcomeError;
    use haste_fhir_ops::derive::{FromParameters, ToParameters};
    pub const CODE: &str = "find-matches";
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct InputPropertySubproperty {
        #[doc = "Identifies the sub-property provided"]
        pub code: FHIRCode,
        #[doc = "The value of the sub-property provided"]
        pub value: ParametersParameterValueTypeChoice,
    }
    impl From<InputPropertySubproperty> for Resource {
        fn from(value: InputPropertySubproperty) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct InputProperty {
        #[doc = "Identifies the property provided"]
        pub code: FHIRCode,
        #[doc = "The value of the property provided"]
        pub value: Option<ParametersParameterValueTypeChoice>,
        #[doc = "Nested Properties (mainly used for SNOMED CT composition, for relationship Groups)"]
        #[parameter_nested]
        pub subproperty: Option<Vec<InputPropertySubproperty>>,
    }
    impl From<InputProperty> for Resource {
        fn from(value: InputProperty) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Input {
        #[doc = "The system in which composition is to be performed. This must be provided unless the operation is"]
        #[doc = "invoked on a code system instance"]
        pub system: Option<FHIRUri>,
        #[doc = "The version of the system for the inferencing to be performed"]
        pub version: Option<FHIRString>,
        #[doc = "One or more properties that contain information to be composed into the code"]
        #[parameter_nested]
        pub property: Option<Vec<InputProperty>>,
        #[doc = "Whether the operation is being used by a human (`false`), or a machine (`true`). If the operation is"]
        #[doc = "being used by a human, the terminology server can return a list of possible matches, with"]
        #[doc = "commentary. For a machine, the server returns complete or partial matches, not possible matches. The"]
        #[doc = "default value is `false`"]
        pub exact: FHIRBoolean,
        #[doc = "Post-coordinated expressions are allowed to be returned in the matching codes (mainly for SNOMED"]
        #[doc = "CT). Default = false"]
        pub compositional: Option<FHIRBoolean>,
    }
    impl From<Input> for Resource {
        fn from(value: Input) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct OutputMatchUnmatchedProperty {
        #[doc = "Identifies the sub-property provided"]
        pub code: FHIRCode,
        #[doc = "The value of the sub-property provided"]
        pub value: ParametersParameterValueTypeChoice,
    }
    impl From<OutputMatchUnmatchedProperty> for Resource {
        fn from(value: OutputMatchUnmatchedProperty) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct OutputMatchUnmatched {
        #[doc = "Identifies the property provided"]
        pub code: FHIRCode,
        #[doc = "The value of the property provided"]
        pub value: ParametersParameterValueTypeChoice,
        #[doc = "Nested Properties (mainly used for SNOMED CT composition, for relationship Groups)"]
        #[parameter_nested]
        pub property: Option<Vec<OutputMatchUnmatchedProperty>>,
    }
    impl From<OutputMatchUnmatched> for Resource {
        fn from(value: OutputMatchUnmatched) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct OutputMatch {
        #[doc = "A code that matches the properties provided"]
        pub code: Coding,
        #[doc = "One or more properties that contain properties that could not be matched into the code"]
        #[parameter_nested]
        pub unmatched: Option<Vec<OutputMatchUnmatched>>,
        #[doc = "Information about the quality of the match, if operation is for a human"]
        pub comment: Option<FHIRString>,
    }
    impl From<OutputMatch> for Resource {
        fn from(value: OutputMatch) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Output {
        #[doc = "Concepts returned by the server as a result of the inferencing operation"]
        #[parameter_rename = "match"]
        #[parameter_nested]
        pub match_: Option<Vec<OutputMatch>>,
    }
    impl From<Output> for Resource {
        fn from(value: Output) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
}
#[doc = "Given a code/system, or a Coding, get additional details about the concept, including definition,"]
#[doc = "status, designations, and properties. One of the products of this operation is a full decomposition"]
#[doc = "of a code from a structured terminology."]
#[doc = ""]
#[doc = "When invoking this operation, a client SHALL provide both a system and a code, either using the"]
#[doc = "system+code parameters, or in the coding parameter. Other parameters are optional"]
pub mod CodeSystemLookup {
    use haste_fhir_model::r4::generated::resources::{
        Parameters, ParametersParameter, ParametersParameterValueTypeChoice, Resource,
    };
    use haste_fhir_model::r4::generated::types::{
        Coding, FHIRCode, FHIRDateTime, FHIRString, FHIRUri,
    };
    use haste_fhir_operation_error::OperationOutcomeError;
    use haste_fhir_ops::derive::{FromParameters, ToParameters};
    pub const CODE: &str = "lookup";
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Input {
        #[doc = "The code that is to be located. If a code is provided, a system must be provided"]
        pub code: Option<FHIRCode>,
        #[doc = "The system for the code that is to be located"]
        pub system: Option<FHIRUri>,
        #[doc = "The version of the system, if one was provided in the source data"]
        pub version: Option<FHIRString>,
        #[doc = "A coding to look up"]
        pub coding: Option<Coding>,
        #[doc = "The date for which the information should be returned. Normally, this is the current conditions"]
        #[doc = "(which is the default value) but under some circumstances, systems need to acccess this information"]
        #[doc = "as it would have been in the past. A typical example of this would be where code selection is"]
        #[doc = "constrained to the set of codes that were available when the patient was treated, not when the"]
        #[doc = "record is being edited. Note that which date is appropriate is a matter for implementation policy."]
        pub date: Option<FHIRDateTime>,
        #[doc = "The requested language for display (see $expand.displayLanguage)"]
        pub displayLanguage: Option<FHIRCode>,
        #[doc = "A property that the client wishes to be returned in the output. If no properties are specified, the"]
        #[doc = "server chooses what to return. The following properties are defined for all code systems: url, name,"]
        #[doc = "version (code system info) and code information: display, definition, designation, parent and child,"]
        #[doc = "and for designations, lang.X where X is a designation language code. Some of the properties are"]
        #[doc = "returned explicit in named parameters (when the names match), and the rest (except for lang.X) in"]
        #[doc = "the property parameter group"]
        pub property: Option<Vec<FHIRCode>>,
    }
    impl From<Input> for Resource {
        fn from(value: Input) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct OutputDesignation {
        #[doc = "The language this designation is defined for"]
        pub language: Option<FHIRCode>,
        #[doc = "A code that details how this designation would be used"]
        #[parameter_rename = "use"]
        pub use_: Option<Coding>,
        #[doc = "The text value for this designation"]
        pub value: FHIRString,
    }
    impl From<OutputDesignation> for Resource {
        fn from(value: OutputDesignation) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct OutputPropertySubproperty {
        #[doc = "Identifies the sub-property returned"]
        pub code: FHIRCode,
        #[doc = "The value of the sub-property returned"]
        pub value: ParametersParameterValueTypeChoice,
        #[doc = "Human Readable representation of the property value (e.g. display for a code)"]
        pub description: Option<FHIRString>,
    }
    impl From<OutputPropertySubproperty> for Resource {
        fn from(value: OutputPropertySubproperty) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct OutputProperty {
        #[doc = "Identifies the property returned"]
        pub code: FHIRCode,
        #[doc = "The value of the property returned"]
        pub value: Option<ParametersParameterValueTypeChoice>,
        #[doc = "Human Readable representation of the property value (e.g. display for a code)"]
        pub description: Option<FHIRString>,
        #[doc = "Nested Properties (mainly used for SNOMED CT decomposition, for relationship Groups)"]
        #[parameter_nested]
        pub subproperty: Option<Vec<OutputPropertySubproperty>>,
    }
    impl From<OutputProperty> for Resource {
        fn from(value: OutputProperty) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Output {
        #[doc = "A display name for the code system"]
        pub name: FHIRString,
        #[doc = "The version that these details are based on"]
        pub version: Option<FHIRString>,
        #[doc = "The preferred display for this concept"]
        pub display: FHIRString,
        #[doc = "Additional representations for this concept"]
        #[parameter_nested]
        pub designation: Option<Vec<OutputDesignation>>,
        #[doc = "One or more properties that contain additional information about the code, including status. For"]
        #[doc = "complex terminologies (e.g. SNOMED CT, LOINC, medications), these properties serve to decompose the"]
        #[doc = "code"]
        #[parameter_nested]
        pub property: Option<Vec<OutputProperty>>,
    }
    impl From<Output> for Resource {
        fn from(value: Output) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
}
#[doc = "Test the subsumption relationship between code/Coding A and code/Coding B given the semantics of"]
#[doc = "subsumption in the underlying code system (see"]
#[doc = "[`hierarchyMeaning`](codesystem-definitions.html#CodeSystem.hierarchyMeaning))."]
#[doc = ""]
#[doc = "When invoking this operation, a client SHALL provide both a and codes, either as code or Coding"]
#[doc = "parameters. The system parameter is required unless the operation is invoked on an instance of a"]
#[doc = "code system resource. Other parameters are optional"]
pub mod CodeSystemSubsumes {
    use haste_fhir_model::r4::generated::resources::{Parameters, ParametersParameter, Resource};
    use haste_fhir_model::r4::generated::types::{Coding, FHIRCode, FHIRString, FHIRUri};
    use haste_fhir_operation_error::OperationOutcomeError;
    use haste_fhir_ops::derive::{FromParameters, ToParameters};
    pub const CODE: &str = "subsumes";
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Input {
        #[doc = "The \"A\" code that is to be tested. If a code is provided, a system must be provided"]
        pub codeA: Option<FHIRCode>,
        #[doc = "The \"B\" code that is to be tested. If a code is provided, a system must be provided"]
        pub codeB: Option<FHIRCode>,
        #[doc = "The code system in which subsumption testing is to be performed. This must be provided unless the"]
        #[doc = "operation is invoked on a code system instance"]
        pub system: Option<FHIRUri>,
        #[doc = "The version of the code system, if one was provided in the source data"]
        pub version: Option<FHIRString>,
        #[doc = "The \"A\" Coding that is to be tested. The code system does not have to match the specified"]
        #[doc = "subsumption code system, but the relationships between the code systems must be well established"]
        pub codingA: Option<Coding>,
        #[doc = "The \"B\" Coding that is to be tested. The code system does not have to match the specified"]
        #[doc = "subsumption code system, but the relationships between the code systems must be well established"]
        pub codingB: Option<Coding>,
    }
    impl From<Input> for Resource {
        fn from(value: Input) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Output {
        #[doc = "The subsumption relationship between code/Coding \"A\" and code/Coding \"B\". There are 4 possible codes"]
        #[doc = "to be returned (equivalent, subsumes, subsumed-by, and not-subsumed) as defined in the"]
        #[doc = "concept-subsumption-outcome value set. If the server is unable to determine the relationship between"]
        #[doc = "the codes/Codings, then it returns an error response with an `OperationOutcome`."]
        pub outcome: FHIRCode,
    }
    impl From<Output> for Resource {
        fn from(value: Output) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
}
#[doc = "Validate that a coded value is in the code system. If the operation is not called at the instance"]
#[doc = "level, one of the parameters \"url\" or \"codeSystem\" must be provided. The operation returns a result"]
#[doc = "(true / false), an error message, and the recommended display for the code."]
#[doc = ""]
#[doc = "When invoking this operation, a client SHALL provide one (and only one) of the parameters"]
#[doc = "(code+system, coding, or codeableConcept). Other parameters (including version and display) are"]
#[doc = "optional"]
pub mod CodeSystemValidateCode {
    use haste_fhir_model::r4::generated::resources::{
        CodeSystem, Parameters, ParametersParameter, Resource,
    };
    use haste_fhir_model::r4::generated::types::{
        CodeableConcept, Coding, FHIRBoolean, FHIRCode, FHIRDateTime, FHIRString, FHIRUri,
    };
    use haste_fhir_operation_error::OperationOutcomeError;
    use haste_fhir_ops::derive::{FromParameters, ToParameters};
    pub const CODE: &str = "validate-code";
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Input {
        #[doc = "`CodeSystem` URL. The server must know the code system (e.g. it is defined explicitly in the"]
        #[doc = "server'scode systems, or it is known implicitly by the server"]
        pub url: Option<FHIRUri>,
        #[doc = "The codeSystem is provided directly as part of the request. Servers may choose not to accept code"]
        #[doc = "systems in this fashion. This parameter is used when the client wants the server to check against a"]
        #[doc = "code system that is not stored on the server"]
        pub codeSystem: Option<CodeSystem>,
        #[doc = "The code that is to be validated"]
        pub code: Option<FHIRCode>,
        #[doc = "The version of the code system, if one was provided in the source data"]
        pub version: Option<FHIRString>,
        #[doc = "The display associated with the code, if provided. If a display is provided a code must be provided."]
        #[doc = "If no display is provided, the server cannot validate the display value, but may choose to return a"]
        #[doc = "recommended display name in an extension in the outcome. Whether displays are case sensitive is code"]
        #[doc = "system dependent"]
        pub display: Option<FHIRString>,
        #[doc = "A coding to validate. The system must match the specified code system"]
        pub coding: Option<Coding>,
        #[doc = "A full codeableConcept to validate. The server returns true if one of the coding values is in the"]
        #[doc = "code system, and may also validate that the codings are not in conflict with each other if more than"]
        #[doc = "one is present"]
        pub codeableConcept: Option<CodeableConcept>,
        #[doc = "The date for which the validation should be checked. Normally, this is the current conditions (which"]
        #[doc = "is the default values) but under some circumstances, systems need to validate that a correct code"]
        #[doc = "was used at some point in the past. A typical example of this would be where code selection is"]
        #[doc = "constrained to the set of codes that were available when the patient was treated, not when the"]
        #[doc = "record is being edited. Note that which date is appropriate is a matter for implementation policy."]
        pub date: Option<FHIRDateTime>,
        #[doc = "If this parameter has a value of true, the client is stating that the validation is being performed"]
        #[doc = "in a context where a concept designated as `abstract` is appropriate/allowed to be used, and the"]
        #[doc = "server should regard abstract codes as valid. If this parameter is false, abstract codes are not"]
        #[doc = "considered to be valid."]
        #[doc = ""]
        #[doc = "Note that. `abstract` is a property defined by many HL7 code systems that indicates that the concept"]
        #[doc = "is a logical grouping concept that is not intended to be used asa `concrete` concept to in an actual"]
        #[doc = "patient/care/process record. This language is borrowed from Object Orienated theory where `asbtract`"]
        #[doc = "objects are never instantiated. However in the general record and terminology eco-system, there are"]
        #[doc = "many contexts where it is appropraite to use these codes e.g. as decision making criterion, or when"]
        #[doc = "editing value sets themselves. This parameter allows a client to indicate to the server that it is"]
        #[doc = "working in such a context."]
        #[parameter_rename = "abstract"]
        pub abstract_: Option<FHIRBoolean>,
        #[doc = "Specifies the language to be used for description when validating the display property"]
        pub displayLanguage: Option<FHIRCode>,
    }
    impl From<Input> for Resource {
        fn from(value: Input) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Output {
        #[doc = "True if the concept details supplied are valid"]
        pub result: FHIRBoolean,
        #[doc = "Error details, if result = false. If this is provided when result = true, the message carries hints"]
        #[doc = "and warnings"]
        pub message: Option<FHIRString>,
        #[doc = "A valid display for the concept if the system wishes to display this to a user"]
        pub display: Option<FHIRString>,
    }
    impl From<Output> for Resource {
        fn from(value: Output) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
}
#[doc = "A client can ask a server to generate a fully bundled document from a composition resource. The"]
#[doc = "server takes the composition resource, locates all the referenced resources and other additional"]
#[doc = "resources as configured or requested and either returns a full document bundle, or returns an error."]
#[doc = "Note that since this is a search operation, the document bundle is wrapped inside the search bundle."]
#[doc = "If some of the resources are located on other servers, it is at the discretion of the server whether"]
#[doc = "to retrieve them or return an error. If the correct version of the document that would be generated"]
#[doc = "already exists, then the server can return the existing one."]
pub mod CompositionDocument {
    use haste_fhir_model::r4::generated::resources::{Parameters, ParametersParameter, Resource};
    use haste_fhir_model::r4::generated::types::{FHIRBoolean, FHIRString, FHIRUri};
    use haste_fhir_operation_error::OperationOutcomeError;
    use haste_fhir_ops::derive::{FromParameters, ToParameters};
    pub const CODE: &str = "document";
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Input {
        #[doc = "Identifies the composition to use. This can either be a simple id, which identifies a composition,"]
        #[doc = "or it can be a full URL, which identifies a composition on another server."]
        #[doc = ""]
        #[doc = "Notes:"]
        #[doc = ""]
        #[doc = "* `GET [base]/Composition/[id]/$document` is identical in meaning to `GET"]
        #[doc = "[base]/Composition/$document?id=[id]`"]
        #[doc = "* the id parameter SHALL NOT be used if the operation is requested on a particular composition (e.g."]
        #[doc = "`GET [base]/Composition/[id]/$document?id=[id]` is not allowed)"]
        #[doc = "* Servers are not required to support generating documents on Compositions located on another server"]
        pub id: Option<FHIRUri>,
        #[doc = "Whether to store the document at the bundle end-point (/Bundle) or not once it is generated. Value ="]
        #[doc = "true or false (default is for the server to decide). If the document is stored, it's location can be"]
        #[doc = "inferred from the `Bundle.id`, but it SHOULD be provided explicitly in the HTTP Location header in"]
        #[doc = "the response"]
        pub persist: Option<FHIRBoolean>,
        #[doc = "Canonical reference to a `GraphDefinition`. If a URL is provided, it is the canonical reference to a"]
        #[doc = "[`GraphDefinition`](graphdefinition.html) that it controls what resources are to be added to the"]
        #[doc = "bundle when building the document. The `GraphDefinition` can also specify profiles that apply to the"]
        #[doc = "various resources"]
        pub graph: Option<FHIRUri>,
    }
    impl From<Input> for Resource {
        fn from(value: Input) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Output {}
    impl From<Output> for Resource {
        fn from(value: Output) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
}
#[doc = "This operation provides support for ongoing maintenance of a client-side [transitive closure"]
#[doc = "table](https://en.wikipedia.org/wiki/Transitive_closure#In_graph_theory) based on server-side"]
#[doc = "terminological logic. For details of how this is used, see [`Maintaining a Closure"]
#[doc = "Table`](terminology-service.html#closure)"]
pub mod ConceptMapClosure {
    use haste_fhir_model::r4::generated::resources::{
        ConceptMap, Parameters, ParametersParameter, Resource,
    };
    use haste_fhir_model::r4::generated::types::{Coding, FHIRString};
    use haste_fhir_operation_error::OperationOutcomeError;
    use haste_fhir_ops::derive::{FromParameters, ToParameters};
    pub const CODE: &str = "closure";
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Input {
        #[doc = "The name that defines the particular context for the subsumption based closure table"]
        pub name: FHIRString,
        #[doc = "Concepts to add to the closure table"]
        pub concept: Option<Vec<Coding>>,
        #[doc = "A request to resynchronise - request to send all new entries since the nominated version was sent by"]
        #[doc = "the server"]
        pub version: Option<FHIRString>,
    }
    impl From<Input> for Resource {
        fn from(value: Input) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
    #[derive(Debug, FromParameters)]
    pub struct Output {
        #[doc = "A list of new entries (code / system --> code/system) that the client should add to its closure"]
        #[doc = "table. The only kind of entry mapping equivalences that can be returned are equal, specializes,"]
        #[doc = "subsumes and unmatched"]
        #[parameter_rename = "return"]
        pub return_: ConceptMap,
    }
    impl From<Output> for Resource {
        fn from(value: Output) -> Self {
            Resource::ConceptMap(value.return_)
        }
    }
}
#[doc = "Translate a code from one value set to another, based on the existing value set and concept maps"]
#[doc = "resources, and/or other additional knowledge available to the server."]
#[doc = ""]
#[doc = "One (and only one) of the in parameters (code, coding, codeableConcept) must be provided, to"]
#[doc = "identify the code that is to be translated."]
#[doc = ""]
#[doc = "The operation returns a set of parameters including a `result` for whether there is an acceptable"]
#[doc = "match, and a list of possible matches. Note that the list of matches may include notes of codes for"]
#[doc = "which mapping is specifically excluded, so implementers have to check the match.equivalence for each"]
#[doc = "match"]
pub mod ConceptMapTranslate {
    use haste_fhir_model::r4::generated::resources::{
        ConceptMap, Parameters, ParametersParameter, Resource,
    };
    use haste_fhir_model::r4::generated::types::{
        CodeableConcept, Coding, FHIRBoolean, FHIRCode, FHIRString, FHIRUri,
    };
    use haste_fhir_operation_error::OperationOutcomeError;
    use haste_fhir_ops::derive::{FromParameters, ToParameters};
    pub const CODE: &str = "translate";
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct InputDependency {
        #[doc = "The element for this dependency"]
        pub element: Option<FHIRUri>,
        #[doc = "The value for this dependency"]
        pub concept: Option<CodeableConcept>,
    }
    impl From<InputDependency> for Resource {
        fn from(value: InputDependency) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Input {
        #[doc = "A canonical URL for a concept map. The server must know the concept map (e.g. it is defined"]
        #[doc = "explicitly in the server's concept maps, or it is defined implicitly by some code system known to"]
        #[doc = "the server."]
        pub url: Option<FHIRUri>,
        #[doc = "The concept map is provided directly as part of the request. Servers may choose not to accept"]
        #[doc = "concept maps in this fashion."]
        pub conceptMap: Option<ConceptMap>,
        #[doc = "The identifier that is used to identify a specific version of the concept map to be used for the"]
        #[doc = "translation. This is an arbitrary value managed by the concept map author and is not expected to be"]
        #[doc = "globally unique. For example, it might be a timestamp (e.g. yyyymmdd) if a managed version is not"]
        #[doc = "available."]
        pub conceptMapVersion: Option<FHIRString>,
        #[doc = "The code that is to be translated. If a code is provided, a system must be provided"]
        pub code: Option<FHIRCode>,
        #[doc = "The system for the code that is to be translated"]
        pub system: Option<FHIRUri>,
        #[doc = "The version of the system, if one was provided in the source data"]
        pub version: Option<FHIRString>,
        #[doc = "Identifies the value set used when the concept (system/code pair) was chosen. May be a logical id,"]
        #[doc = "or an absolute or relative location. The source value set is an optional parameter because in some"]
        #[doc = "cases, the client cannot know what the source value set is. However, without a source value set, the"]
        #[doc = "server may be unable to safely identify an applicable concept map, and would return an error. For"]
        #[doc = "this reason, a source value set SHOULD always be provided. Note that servers may be able to identify"]
        #[doc = "an appropriate concept map without a source value set if there is a full mapping for the entire code"]
        #[doc = "system in the concept map, or by manual intervention"]
        pub source: Option<FHIRUri>,
        #[doc = "A coding to translate"]
        pub coding: Option<Coding>,
        #[doc = "A full codeableConcept to validate. The server can translate any of the coding values (e.g. existing"]
        #[doc = "translations) as it chooses"]
        pub codeableConcept: Option<CodeableConcept>,
        #[doc = "Identifies the value set in which a translation is sought. May be a logical id, or an absolute or"]
        #[doc = "relative location. If there's no target specified, the server should return all known translations,"]
        #[doc = "along with their source"]
        pub target: Option<FHIRUri>,
        #[doc = "identifies a target code system in which a mapping is sought. This parameter is an alternative to"]
        #[doc = "the target parameter - only one is required. Searching for any translation to a target code system"]
        #[doc = "irrespective of the context (e.g. target valueset) may lead to unsafe results, and it is at the"]
        #[doc = "discretion of the server to decide when to support this operation"]
        pub targetsystem: Option<FHIRUri>,
        #[doc = "Another element that may help produce the correct mapping"]
        #[parameter_nested]
        pub dependency: Option<Vec<InputDependency>>,
        #[doc = "if this is true, then the operation should return all the codes that might be mapped to this code."]
        #[doc = "This parameter reverses the meaning of the source and target parameters"]
        pub reverse: Option<FHIRBoolean>,
    }
    impl From<Input> for Resource {
        fn from(value: Input) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct OutputMatchProduct {
        #[doc = "The element for this product"]
        pub element: Option<FHIRUri>,
        #[doc = "The value for this product"]
        pub concept: Option<Coding>,
    }
    impl From<OutputMatchProduct> for Resource {
        fn from(value: OutputMatchProduct) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct OutputMatch {
        #[doc = "A code indicating the equivalence of the translation, using values from"]
        #[doc = "[`ConceptMapEquivalence`](valueset-concept-map-equivalence.html)"]
        pub equivalence: Option<FHIRCode>,
        #[doc = "The translation outcome. Note that this would never have userSelected = true, since the process of"]
        #[doc = "translations implies that the user is not selecting the code (and only the client could know"]
        #[doc = "differently)"]
        pub concept: Option<Coding>,
        #[doc = "Another element that is the product of this mapping"]
        #[parameter_nested]
        pub product: Option<Vec<OutputMatchProduct>>,
        #[doc = "The canonical reference to the concept map from which this mapping comes from"]
        pub source: Option<FHIRUri>,
    }
    impl From<OutputMatch> for Resource {
        fn from(value: OutputMatch) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Output {
        #[doc = "True if the concept could be translated successfully. The value can only be true if at least one"]
        #[doc = "returned match has an equivalence which is not unmatched or disjoint"]
        pub result: FHIRBoolean,
        #[doc = "Error details, for display to a human. If this is provided when result = true, the message carries"]
        #[doc = "hints and warnings (e.g. a note that the matches could be improved by providing additional detail)"]
        pub message: Option<FHIRString>,
        #[doc = "A concept in the target value set with an equivalence. Note that there may be multiple matches of"]
        #[doc = "equal or differing equivalence, and the matches may include equivalence values that mean that there"]
        #[doc = "is no match"]
        #[parameter_rename = "match"]
        #[parameter_nested]
        pub match_: Option<Vec<OutputMatch>>,
    }
    impl From<Output> for Resource {
        fn from(value: Output) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
}
#[doc = "This operation is used to submit an `EligibilityRequest` for assessment either as a single"]
#[doc = "`EligibilityRequest` resource instance or as a Bundle containing the `EligibilityRequest` and other"]
#[doc = "referenced resources, or Bundle containing a batch of `EligibilityRequest` resources, either as"]
#[doc = "single `EligibilityRequests` resources or Bundle resources, for processing. The only input parameter"]
#[doc = "is the single `EligibilityRequest` or Bundle resource and the only output is a single"]
#[doc = "`EligibilityResponse`, Bundle of `EligibilityResponses` or an `OperationOutcome` resource."]
pub mod CoverageEligibilityRequestSubmit {
    use haste_fhir_model::r4::generated::resources::{Parameters, ParametersParameter, Resource};
    use haste_fhir_model::r4::generated::types::FHIRString;
    use haste_fhir_operation_error::OperationOutcomeError;
    use haste_fhir_ops::derive::{FromParameters, ToParameters};
    pub const CODE: &str = "submit";
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Input {
        #[doc = "An `EligibilityRequest` resource or Bundle of `EligibilityRequests`, either as individual"]
        #[doc = "`EligibilityRequest` resources or as Bundles each containing a single `EligibilityRequest` plus"]
        #[doc = "referenced resources."]
        pub resource: Resource,
    }
    impl From<Input> for Resource {
        fn from(value: Input) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Output {
        #[doc = "An `EligibilityResponse` resource or Bundle of `EligibilityResponse` responses, either as individual"]
        #[doc = "`EligibilityResponse` resources or as Bundles each containing a single `EligibilityResponse` plus"]
        #[doc = "referenced resources."]
        #[parameter_rename = "return"]
        pub return_: Resource,
    }
    impl From<Output> for Resource {
        fn from(value: Output) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
}
#[doc = "This operation is used to return all the information related to an encounter described in the"]
#[doc = "resource on which this operation is invoked. The response is a bundle of type \"searchset\". At a"]
#[doc = "minimum, the encounter resource itself is returned, along with any other resources that the server"]
#[doc = "has available for the given encounter for the user. The server also returns whatever resources are"]
#[doc = "needed to support the records - e.g. linked practitioners, locations, organizations etc. The"]
#[doc = "principle intended use for this operation is to provide a patient with access to their record, or to"]
#[doc = "allow a client to retrieve everything for an encounter for efficient display). The server SHOULD"]
#[doc = "return all resources it has that: * are included in the encounter compartment for the identified"]
#[doc = "encounter (have a reference to the encounter) * are referenced by the standard extenstion for"]
#[doc = "associating an encounter (where no reference element exists)"]
#[doc = "`http://hl7.org/fhir/StructureDefinition/encounter-associatedEncounter` * the server believes are"]
#[doc = "relevant to the context of the encounter for any other reason (internally defined/decided) * any"]
#[doc = "resource referenced by the above, including binaries and attachments (to make a more complete"]
#[doc = "package) In the US Realm, at a mimimum, the resources returned SHALL include all the data covered by"]
#[doc = "the meaningful use common data elements (see [DAF](http://hl7.org/fhir/us/daf) for further"]
#[doc = "guidance). Other applicable implementation guides may make additional rules about the information"]
#[doc = "that is returned. Note that for many resources, the exact nature of the link to encounter can be"]
#[doc = "ambiguous (e.g. for a `DiagnosticReport`, is it the encounter when it was initiated, or when it was"]
#[doc = "reported?)"]
pub mod EncounterEverything {
    use haste_fhir_model::r4::generated::resources::{
        Bundle, Parameters, ParametersParameter, Resource,
    };
    use haste_fhir_model::r4::generated::types::{FHIRCode, FHIRInstant, FHIRInteger, FHIRString};
    use haste_fhir_operation_error::OperationOutcomeError;
    use haste_fhir_ops::derive::{FromParameters, ToParameters};
    pub const CODE: &str = "everything";
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Input {
        #[doc = "Resources updated after this period will be included in the response. The intent of this parameter"]
        #[doc = "is to allow a client to request only records that have changed since the last request, based on"]
        #[doc = "either the return header time, or or (for asynchronous use), the transaction time"]
        #[parameter_rename = "_since"]
        pub since: Option<FHIRInstant>,
        #[doc = "One or more parameters, each containing one or more comma-delimited FHIR resource types to include"]
        #[doc = "in the return resources. In the absense of any specified types, the server returns all resource"]
        #[doc = "types"]
        #[parameter_rename = "_type"]
        pub type_: Option<Vec<FHIRCode>>,
        #[doc = "See discussion below on the utility of paging through the results of the $everything operation"]
        #[parameter_rename = "_count"]
        pub count: Option<FHIRInteger>,
    }
    impl From<Input> for Resource {
        fn from(value: Input) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
    #[derive(Debug, FromParameters)]
    pub struct Output {
        #[doc = "The bundle type is \"searchset\""]
        #[parameter_rename = "return"]
        pub return_: Bundle,
    }
    impl From<Output> for Resource {
        fn from(value: Output) -> Self {
            Resource::Bundle(value.return_)
        }
    }
}
#[doc = "This operation is used to return all the information related to one or more patients that are part"]
#[doc = "of the group on which this operation is invoked. The response is a bundle of type \"searchset\". At a"]
#[doc = "minimum, the patient resource(s) itself is returned, along with any other resources that the server"]
#[doc = "has that are related to the patient(s), and that are available for the given user. The server also"]
#[doc = "returns whatever resources are needed to support the records - e.g. linked practitioners,"]
#[doc = "medications, locations, organizations etc. The intended use for this operation is for a provider or"]
#[doc = "other user to perform a bulk data download. The server SHOULD return at least all resources that it"]
#[doc = "has that are in the patient compartment for the identified patient(s), and any resource referenced"]
#[doc = "from those, including binaries and attachments. In the US Realm, at a mimimum, the resources"]
#[doc = "returned SHALL include all the data covered by the meaningful use common data elements as defined in"]
#[doc = "[US-Core](http://hl7.org/fhir/us/coref). Other applicable implementation guides may make additional"]
#[doc = "rules about how much information that is returned."]
pub mod GroupEverything {
    use haste_fhir_model::r4::generated::resources::{
        Bundle, Parameters, ParametersParameter, Resource,
    };
    use haste_fhir_model::r4::generated::types::{
        FHIRCode, FHIRDate, FHIRInstant, FHIRInteger, FHIRString,
    };
    use haste_fhir_operation_error::OperationOutcomeError;
    use haste_fhir_ops::derive::{FromParameters, ToParameters};
    pub const CODE: &str = "everything";
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Input {
        #[doc = "The date range relates to care dates, not record currency dates - e.g. all records relating to care"]
        #[doc = "provided in a certain date range. If no start date is provided, all records prior to the end date"]
        #[doc = "are in scope."]
        pub start: Option<FHIRDate>,
        #[doc = "The date range relates to care dates, not record currency dates - e.g. all records relating to care"]
        #[doc = "provided in a certain date range. If no end date is provided, all records subsequent to the start"]
        #[doc = "date are in scope."]
        pub end: Option<FHIRDate>,
        #[doc = "Resources updated after this period will be included in the response. The intent of this parameter"]
        #[doc = "is to allow a client to request only records that have changed since the last request, based on"]
        #[doc = "either the return header time, or or (for asynchronous use), the transaction time"]
        #[parameter_rename = "_since"]
        pub since: Option<FHIRInstant>,
        #[doc = "One or more parameters, each containing one or more comma-delimited FHIR resource types to include"]
        #[doc = "in the return resources. In the absense of any specified types, the server returns all resource"]
        #[doc = "types"]
        #[parameter_rename = "_type"]
        pub type_: Option<Vec<FHIRCode>>,
        #[doc = "See discussion below on the utility of paging through the results of the $everything operation"]
        #[parameter_rename = "_count"]
        pub count: Option<FHIRInteger>,
    }
    impl From<Input> for Resource {
        fn from(value: Input) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
    #[derive(Debug, FromParameters)]
    pub struct Output {
        #[doc = "The bundle type is \"searchset\""]
        #[parameter_rename = "return"]
        pub return_: Bundle,
    }
    impl From<Output> for Resource {
        fn from(value: Output) -> Self {
            Resource::Bundle(value.return_)
        }
    }
}
#[doc = "The data-requirements operation aggregates and returns the parameters and data requirements for a"]
#[doc = "resource and all its dependencies as a single module definition"]
pub mod LibraryDataRequirements {
    use haste_fhir_model::r4::generated::resources::{
        Library, Parameters, ParametersParameter, Resource,
    };
    use haste_fhir_model::r4::generated::types::FHIRString;
    use haste_fhir_operation_error::OperationOutcomeError;
    use haste_fhir_ops::derive::{FromParameters, ToParameters};
    pub const CODE: &str = "data-requirements";
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Input {
        #[doc = "The target of the data requirements operation"]
        pub target: Option<FHIRString>,
    }
    impl From<Input> for Resource {
        fn from(value: Input) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
    #[derive(Debug, FromParameters)]
    pub struct Output {
        #[doc = "The result of the requirements gathering"]
        #[parameter_rename = "return"]
        pub return_: Library,
    }
    impl From<Output> for Resource {
        fn from(value: Output) -> Self {
            Resource::Library(value.return_)
        }
    }
}
#[doc = "This operation allows a client to find an identified list for a particular function by its function."]
#[doc = "The operation takes two parameters, the identity of a patient, and the name of a functional list."]
#[doc = "The list of defined functional lists can be found at [`Current Resource"]
#[doc = "Lists`](lifecycle.html#lists). Applications are not required to support all the lists, and may"]
#[doc = "define additional lists of their own. If the system is able to locate a list that serves the"]
#[doc = "identified purpose, it returns it as the body of the response with a 200 OK status. If the resource"]
#[doc = "cannot be located, the server returns a 404 not found (optionally with an `OperationOutcome`"]
#[doc = "resource)"]
pub mod ListFind {
    use haste_fhir_model::r4::generated::resources::{Parameters, ParametersParameter, Resource};
    use haste_fhir_model::r4::generated::types::{FHIRCode, FHIRId, FHIRString};
    use haste_fhir_operation_error::OperationOutcomeError;
    use haste_fhir_ops::derive::{FromParameters, ToParameters};
    pub const CODE: &str = "find";
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Input {
        #[doc = "The id of a patient resource located on the server on which this operation is executed"]
        pub patient: FHIRId,
        #[doc = "The code for the functional list that is being found"]
        pub name: FHIRCode,
    }
    impl From<Input> for Resource {
        fn from(value: Input) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Output {}
    impl From<Output> for Resource {
        fn from(value: Output) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
}
#[doc = "The care-gaps operation is used to determine gaps-in-care based on the results of quality measures"]
pub mod MeasureCareGaps {
    use haste_fhir_model::r4::generated::resources::{
        Bundle, Parameters, ParametersParameter, Resource,
    };
    use haste_fhir_model::r4::generated::types::{FHIRDate, FHIRString};
    use haste_fhir_operation_error::OperationOutcomeError;
    use haste_fhir_ops::derive::{FromParameters, ToParameters};
    pub const CODE: &str = "care-gaps";
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Input {
        #[doc = "The start of the measurement period. In keeping with the semantics of the date parameter used in the"]
        #[doc = "FHIR search operation, the period will start at the beginning of the period implied by the supplied"]
        #[doc = "timestamp. E.g. a value of 2014 would set the period s"]
        pub periodStart: FHIRDate,
        #[doc = "The end of the measurement period. The period will end at the end of the period implied by the"]
        #[doc = "supplied timestamp. E.g. a value of 2014 would set the period end to be 2014-12-31T23:59:59"]
        #[doc = "inclusive"]
        pub periodEnd: FHIRDate,
        #[doc = "The topic to be used to determine which measures are considered for the care gaps report. Any"]
        #[doc = "measure with the given topic will be included in the report"]
        pub topic: FHIRString,
        #[doc = "Subject for which the care gaps report will be produced"]
        pub subject: FHIRString,
    }
    impl From<Input> for Resource {
        fn from(value: Input) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
    #[derive(Debug, FromParameters)]
    pub struct Output {
        #[doc = "The result of the care gaps report will be returned as a document bundle with a `MeasureReport`"]
        #[doc = "entry for each included measure"]
        #[parameter_rename = "return"]
        pub return_: Bundle,
    }
    impl From<Output> for Resource {
        fn from(value: Output) -> Self {
            Resource::Bundle(value.return_)
        }
    }
}
#[doc = "The collect-data operation is used to collect the data-of-interest for the given measure."]
pub mod MeasureCollectData {
    use haste_fhir_model::r4::generated::resources::{
        MeasureReport, Parameters, ParametersParameter, Resource,
    };
    use haste_fhir_model::r4::generated::types::{FHIRDate, FHIRDateTime, FHIRString};
    use haste_fhir_operation_error::OperationOutcomeError;
    use haste_fhir_ops::derive::{FromParameters, ToParameters};
    pub const CODE: &str = "collect-data";
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Input {
        #[doc = "The start of the measurement period. In keeping with the semantics of the date parameter used in the"]
        #[doc = "FHIR search operation, the period will start at the beginning of the period implied by the supplied"]
        #[doc = "timestamp. E.g. a value of 2014 would set the period s"]
        pub periodStart: FHIRDate,
        #[doc = "The end of the measurement period. The period will end at the end of the period implied by the"]
        #[doc = "supplied timestamp. E.g. a value of 2014 would set the period end to be 2014-12-31T23:59:59"]
        #[doc = "inclusive"]
        pub periodEnd: FHIRDate,
        #[doc = "The measure to evaluate. This parameter is only required when the operation is invoked on the"]
        #[doc = "resource type, it is not used when invoking the operation on a Measure instance"]
        pub measure: Option<FHIRString>,
        #[doc = "Subject for which the measure will be collected. If not specified, measure data will be collected"]
        #[doc = "for all subjects that meet the requirements of the measure. If specified, the measure will only be"]
        #[doc = "calculated for the referenced subject(s)"]
        pub subject: Option<FHIRString>,
        #[doc = "Practitioner for which the measure will be collected. If specified, measure data will be collected"]
        #[doc = "only for subjects that have a primary relationship to the identified practitioner"]
        pub practitioner: Option<FHIRString>,
        #[doc = "The date the results of this measure were last received. This parameter used to indicate when the"]
        #[doc = "last time data for this measure was collected. This information is used to support incremental data"]
        #[doc = "collection scenarios"]
        pub lastReceivedOn: Option<FHIRDateTime>,
    }
    impl From<Input> for Resource {
        fn from(value: Input) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Output {
        #[doc = "A `MeasureReport` of type data-collection detailing the results of the operation"]
        pub measureReport: MeasureReport,
        #[doc = "The result resources that make up the data-of-interest for the measure"]
        pub resource: Option<Vec<Resource>>,
    }
    impl From<Output> for Resource {
        fn from(value: Output) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
}
#[doc = "The data-requirements operation aggregates and returns the parameters and data requirements for the"]
#[doc = "measure and all its dependencies as a single module definition"]
pub mod MeasureDataRequirements {
    use haste_fhir_model::r4::generated::resources::{
        Library, Parameters, ParametersParameter, Resource,
    };
    use haste_fhir_model::r4::generated::types::{FHIRDate, FHIRString};
    use haste_fhir_operation_error::OperationOutcomeError;
    use haste_fhir_ops::derive::{FromParameters, ToParameters};
    pub const CODE: &str = "data-requirements";
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Input {
        #[doc = "The start of the measurement period. In keeping with the semantics of the date parameter used in the"]
        #[doc = "FHIR search operation, the period will start at the beginning of the period implied by the supplied"]
        #[doc = "timestamp. E.g. a value of 2014 would set the period start to be 2014-01-01T00:00:00 inclusive"]
        pub periodStart: FHIRDate,
        #[doc = "The end of the measurement period. The period will end at the end of the period implied by the"]
        #[doc = "supplied timestamp. E.g. a value of 2014 would set the period end to be 2014-12-31T23:59:59"]
        #[doc = "inclusive"]
        pub periodEnd: FHIRDate,
    }
    impl From<Input> for Resource {
        fn from(value: Input) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
    #[derive(Debug, FromParameters)]
    pub struct Output {
        #[doc = "The result of the requirements gathering is a module-definition Library that describes the aggregate"]
        #[doc = "parameters, data requirements, and dependencies of the measure"]
        #[parameter_rename = "return"]
        pub return_: Library,
    }
    impl From<Output> for Resource {
        fn from(value: Output) -> Self {
            Resource::Library(value.return_)
        }
    }
}
#[doc = "The evaluate-measure operation is used to calculate an eMeasure and obtain the results"]
pub mod MeasureEvaluateMeasure {
    use haste_fhir_model::r4::generated::resources::{
        MeasureReport, Parameters, ParametersParameter, Resource,
    };
    use haste_fhir_model::r4::generated::types::{FHIRCode, FHIRDate, FHIRDateTime, FHIRString};
    use haste_fhir_operation_error::OperationOutcomeError;
    use haste_fhir_ops::derive::{FromParameters, ToParameters};
    pub const CODE: &str = "evaluate-measure";
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Input {
        #[doc = "The start of the measurement period. In keeping with the semantics of the date parameter used in the"]
        #[doc = "FHIR search operation, the period will start at the beginning of the period implied by the supplied"]
        #[doc = "timestamp. E.g. a value of 2014 would set the period start to be 2014-01-01T00:00:00 inclusive"]
        pub periodStart: FHIRDate,
        #[doc = "The end of the measurement period. The period will end at the end of the period implied by the"]
        #[doc = "supplied timestamp. E.g. a value of 2014 would set the period end to be 2014-12-31T23:59:59"]
        #[doc = "inclusive"]
        pub periodEnd: FHIRDate,
        #[doc = "The measure to evaluate. This parameter is only required when the operation is invoked on the"]
        #[doc = "resource type, it is not used when invoking the operation on a Measure instance"]
        pub measure: Option<FHIRString>,
        #[doc = "The type of measure report: subject, subject-list, or population. If not specified, a default value"]
        #[doc = "of subject will be used if the subject parameter is supplied, otherwise, population will be used"]
        pub reportType: Option<FHIRCode>,
        #[doc = "Subject for which the measure will be calculated. If not specified, the measure will be calculated"]
        #[doc = "for all subjects that meet the requirements of the measure. If specified, the measure will only be"]
        #[doc = "calculated for the referenced subject(s)"]
        pub subject: Option<FHIRString>,
        #[doc = "Practitioner for which the measure will be calculated. If specified, the measure will be calculated"]
        #[doc = "only for subjects that have a primary relationship to the identified practitioner"]
        pub practitioner: Option<FHIRString>,
        #[doc = "The date the results of this measure were last received. This parameter is only valid for"]
        #[doc = "patient-level reports and is used to indicate when the last time a result for this patient was"]
        #[doc = "received. This information can be used to limit the set of resources returned for a patient-level"]
        #[doc = "report"]
        pub lastReceivedOn: Option<FHIRDateTime>,
    }
    impl From<Input> for Resource {
        fn from(value: Input) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
    #[derive(Debug, FromParameters)]
    pub struct Output {
        #[doc = "The results of the measure calculation. See the `MeasureReport` resource for a complete description"]
        #[doc = "of the output of this operation. Note that implementations may choose to return a `MeasureReport`"]
        #[doc = "with a status of pending to indicate that the report is still being generated. In this case, the"]
        #[doc = "client can use a polling method to continually request the `MeasureReport` until the status is"]
        #[doc = "updated to complete"]
        #[parameter_rename = "return"]
        pub return_: MeasureReport,
    }
    impl From<Output> for Resource {
        fn from(value: Output) -> Self {
            Resource::MeasureReport(value.return_)
        }
    }
}
#[doc = "The submit-data operation is used to submit data-of-interest for a measure. There is no expectation"]
#[doc = "that the submitted data represents all the data-of-interest, only that all the data submitted is"]
#[doc = "relevant to the calculation of the measure for a particular subject or population"]
pub mod MeasureSubmitData {
    use haste_fhir_model::r4::generated::resources::{
        MeasureReport, Parameters, ParametersParameter, Resource,
    };
    use haste_fhir_model::r4::generated::types::FHIRString;
    use haste_fhir_operation_error::OperationOutcomeError;
    use haste_fhir_ops::derive::{FromParameters, ToParameters};
    pub const CODE: &str = "submit-data";
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Input {
        #[doc = "The measure report being submitted"]
        pub measureReport: MeasureReport,
        #[doc = "The individual resources that make up the data-of-interest being submitted"]
        pub resource: Option<Vec<Resource>>,
    }
    impl From<Input> for Resource {
        fn from(value: Input) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Output {}
    impl From<Output> for Resource {
        fn from(value: Output) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
}
#[doc = "This operation is used to return all the information related to one or more products described in"]
#[doc = "the resource or context on which this operation is invoked. The response is a bundle of type"]
#[doc = "\"searchset\". At a minimum, the product resource(s) itself is returned, along with any other"]
#[doc = "resources that the server has that are related to the products(s), and that are available for the"]
#[doc = "given user. This is typically the marketing authorisations, ingredients, packages, therapeutic"]
#[doc = "indications and so on. The server also returns whatever resources are needed to support the records"]
#[doc = "- e.g. linked organizations, document references etc."]
pub mod MedicinalProductEverything {
    use haste_fhir_model::r4::generated::resources::{
        Bundle, Parameters, ParametersParameter, Resource,
    };
    use haste_fhir_model::r4::generated::types::{FHIRInstant, FHIRInteger, FHIRString};
    use haste_fhir_operation_error::OperationOutcomeError;
    use haste_fhir_ops::derive::{FromParameters, ToParameters};
    pub const CODE: &str = "everything";
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Input {
        #[doc = "Resources updated after this period will be included in the response. The intent of this parameter"]
        #[doc = "is to allow a client to request only records that have changed since the last request, based on"]
        #[doc = "either the return header time, or or (for asynchronous use), the transaction time"]
        #[parameter_rename = "_since"]
        pub since: Option<FHIRInstant>,
        #[doc = "See discussion below on the utility of paging through the results of the $everything operation"]
        #[parameter_rename = "_count"]
        pub count: Option<FHIRInteger>,
    }
    impl From<Input> for Resource {
        fn from(value: Input) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
    #[derive(Debug, FromParameters)]
    pub struct Output {
        #[doc = "The bundle type is \"searchset\""]
        #[parameter_rename = "return"]
        pub return_: Bundle,
    }
    impl From<Output> for Resource {
        fn from(value: Output) -> Self {
            Resource::Bundle(value.return_)
        }
    }
}
#[doc = "This operation accepts a message, processes it according to the definition of the event in the"]
#[doc = "message header, and returns one or more response messages."]
#[doc = ""]
#[doc = "In addition to processing the message event, a server may choose to retain all or some the resources"]
#[doc = "and make them available on a `RESTful` interface, but is not required to do so."]
pub mod MessageHeaderProcessMessage {
    use haste_fhir_model::r4::generated::resources::{
        Bundle, Parameters, ParametersParameter, Resource,
    };
    use haste_fhir_model::r4::generated::types::{FHIRBoolean, FHIRString, FHIRUrl};
    use haste_fhir_operation_error::OperationOutcomeError;
    use haste_fhir_ops::derive::{FromParameters, ToParameters};
    pub const CODE: &str = "process-message";
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Input {
        #[doc = "The message to process (or, if using asynchronous messaging, it may be a response message to accept)"]
        pub content: Bundle,
        #[doc = "If `true` the message is processed using the asynchronous messaging pattern"]
        #[parameter_rename = "async"]
        pub async_: Option<FHIRBoolean>,
        #[doc = "A URL to submit response messages to, if asynchronous messaging is being used, and if the"]
        #[doc = "`MessageHeader.source.endpoint` is not the appropriate place to submit responses"]
        #[parameter_rename = "response-url"]
        pub response_url: Option<FHIRUrl>,
    }
    impl From<Input> for Resource {
        fn from(value: Input) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
    #[derive(Debug, FromParameters)]
    pub struct Output {
        #[doc = "A response message, if synchronous messaging is being used (mandatory in this case). For"]
        #[doc = "asynchronous messaging, there is no return value"]
        #[parameter_rename = "return"]
        pub return_: Option<Bundle>,
    }
    impl From<Output> for Resource {
        fn from(value: Output) -> Self {
            Resource::Bundle(value.return_.unwrap_or_default())
        }
    }
}
#[doc = "This operation returns the preferred identifiers for identifiers, and terminologies. The operation"]
#[doc = "takes 2 parameters:"]
#[doc = ""]
#[doc = "* a system identifier - either a URI, an OID, or a v2 table 0396 (other) code"]
#[doc = "* a code for what kind of identifier is desired (URI, OID, v2 table 0396 identifier)"]
#[doc = ""]
#[doc = "and returns either the requested identifier, or an HTTP errors response with an `OperationOutcome`"]
#[doc = "because either the provided identifier was not recognized, or the requested identiifer type is not"]
#[doc = "known."]
#[doc = ""]
#[doc = "The principle use of this operation is when converting between v2, CDA and FHIR `Identifier/CX/II`"]
#[doc = "and `CodeableConcepts/C`(N/W)E/CD but the operation may also find use when converting metadata such"]
#[doc = "as profiles."]
pub mod NamingSystemPreferredId {
    use haste_fhir_model::r4::generated::resources::{Parameters, ParametersParameter, Resource};
    use haste_fhir_model::r4::generated::types::{FHIRCode, FHIRString};
    use haste_fhir_operation_error::OperationOutcomeError;
    use haste_fhir_ops::derive::{FromParameters, ToParameters};
    pub const CODE: &str = "preferred-id";
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Input {
        #[doc = "The server parses the provided id to see what type it is (mary a URI, an OID as a URI, a plain OID,"]
        #[doc = "or a v2 table 0396 code). If the server can't tell what type of identifier it is, it can try it as"]
        #[doc = "multiple types. It is an error if more than one system matches the provided identifier"]
        pub id: FHIRString,
        #[doc = "type_"]
        #[parameter_rename = "type"]
        pub type_: FHIRCode,
    }
    impl From<Input> for Resource {
        fn from(value: Input) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Output {
        #[doc = "`OIDs` are return as plain `OIDs` (not the URI form)."]
        pub result: FHIRString,
    }
    impl From<Output> for Resource {
        fn from(value: Output) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
}
#[doc = "The *lastn query* meets the common need for searching for the most recent or last n=number of"]
#[doc = "observations for a subject. For example, retrieving the last 5 temperatures for a patient to view"]
#[doc = "trends or fetching the most recent laboratory results or vitals signs. To ask a server to return the"]
#[doc = "last n=number of observations, the *lastn* query uses the [`normal search"]
#[doc = "parameters`](observation.html#search) defined for the Observation resource. However, rather than"]
#[doc = "their normal use, they are interpreted as inputs - i.e.. instead of requiring that the resources"]
#[doc = "literally contain the search parameters, they are passed to a server algorithm of some kind that"]
#[doc = "uses them to determine the most appropriate matches."]
#[doc = ""]
#[doc = "The request for a lastn query SHALL include:"]
#[doc = ""]
#[doc = "* A `$lastn` operation parameter"]
#[doc = "* A subject using either the `patient` or `subject` search parameter"]
#[doc = "* A `category` parameter and/or a search parameter that contains a code element in its `FHIRpath`"]
#[doc = "expression. ( e.g., `code` or `code-value-concept`)"]
#[doc = ""]
#[doc = "The request for a lastn query MAY include:"]
#[doc = ""]
#[doc = "* Other Observation search parameters and modifiers"]
#[doc = ""]
#[doc = "The response from a lastn query is a set of observations:"]
#[doc = ""]
#[doc = "* Filtered by additional parameters"]
#[doc = "* If not explicitly filtered by status then will include statuses of `entered-in-error`"]
#[doc = "* 'GROUP BY' `Observation.code`"]
#[doc = "* Codes SHALL be considered equivalent if the `coding.value` *and* `coding.system` are the same."]
#[doc = "* Text only codes SHALL be treated and grouped based on the text."]
#[doc = "* For codes with translations (multiple codings), the code translations are assumed to be equal and"]
#[doc = "the grouping by code SHALL follow the transitive property of equality."]
#[doc = ""]
#[doc = "for example:"]
#[doc = ""]
#[doc = "|`Observation.code for observation a`|`Observation.code for observation b`|`Observation.code for observation c`|`number of groups [codes/text in each group]`|"]
#[doc = "|---|---|---|---|"]
#[doc = "|`a`|`b`|`c`|`3 [a],[b],[c]`|"]
#[doc = "|`a`|`b`|`a,c`|`2 [a.c],[b]`|"]
#[doc = "|`a`|`b`|`a,b`|`1 [a,b]`|"]
#[doc = "|`textM`|`Text`|`t e x t`|`3 [text],[Text],[t e x t]`|"]
#[doc = ""]
#[doc = "* Sorted from most recent to the oldest"]
#[doc = "* Limited to the number of requested responses per group specified by the optional *max* query"]
#[doc = "parameter"]
#[doc = "* In case of a tie - when the effective times for >1 Observations are equal - both will be returned."]
#[doc = "Therefore, more Observations may be returned than is specified in *max*. For example, 4 Observations"]
#[doc = "instead of 3 if the 3rd and 4th most recent observation had the same effective time."]
#[doc = "* If no maximum number is given then only the most recent Observation in each group is returned."]
#[doc = ""]
#[doc = "The set of returned observations should represent distinct real world observations and not the same"]
#[doc = "observation with changes in status or versions. If there are no matches, the *lastn* query SHALL"]
#[doc = "return an empty search set with no error, but may include an operation outcome with further advice."]
pub mod ObservationLastn {
    use haste_fhir_model::r4::generated::resources::{
        Bundle, Parameters, ParametersParameter, Resource,
    };
    use haste_fhir_model::r4::generated::types::{FHIRPositiveInt, FHIRString};
    use haste_fhir_operation_error::OperationOutcomeError;
    use haste_fhir_ops::derive::{FromParameters, ToParameters};
    pub const CODE: &str = "lastn";
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Input {
        #[doc = "`max` is an optional input parameter to the *lastn* query operation. It is used to specify the"]
        #[doc = "maximum number of Observations to return from each group. For example for the query \"Fetch the last"]
        #[doc = "3 results for all vitals for a patient\" `max` = 3."]
        pub max: Option<FHIRPositiveInt>,
    }
    impl From<Input> for Resource {
        fn from(value: Input) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
    #[derive(Debug, FromParameters)]
    pub struct Output {
        #[doc = "The set of most recent N Observations that match the *lastn* query search criteria."]
        #[parameter_rename = "return"]
        pub return_: Bundle,
    }
    impl From<Output> for Resource {
        fn from(value: Output) -> Self {
            Resource::Bundle(value.return_)
        }
    }
}
#[doc = "The Statistics operation performs a set of statistical calculations on a set of clinical"]
#[doc = "measurements such as a blood pressure as stored on the server. This operation evaluates"]
#[doc = "[`Observation`](observation.html) resources having valueQuantity elements that have UCUM unit codes."]
#[doc = "Observations with a status of `entered-in-error` will be excluded from the calculations. The set of"]
#[doc = "Observations is defined by 4 parameters: * the subject of the observations for which the statistics"]
#[doc = "are being generated (`subject`) * which observations to generate statistics for (`code` and"]
#[doc = "`system`, or `coding`) * the time period over which to generate statistics 'duration` or `period`) *"]
#[doc = "the set of statistical analyses to return (`statistic`) Possible statistical analyses (see"]
#[doc = "[StatisticsCode](valueset-observation-statistics.html)): - **average** (\"Average\"): The"]
#[doc = "[mean](https://en.wikipedia.org/wiki/Arithmetic_mean) of N measurements over the stated period."]
#[doc = "- **maximum** (\"Maximum\"): The [maximum](https://en.wikipedia.org/wiki/Maximal_element) value of N"]
#[doc = "measurements over the stated period."]
#[doc = "- **minimum** (\"Minimum\"): The [minimum](https://en.wikipedia.org/wiki/Minimal_element) value of N"]
#[doc = "measurements over the stated period."]
#[doc = "- **count** (\"Count\"): The [number] of valid measurements over the stated period that contributed to"]
#[doc = "the other statistical outputs."]
#[doc = "- **total-count** (\"Total Count\"): The total [number] of valid measurements over the stated period,"]
#[doc = "including observations that were ignored because they did not contain valid result values."]
#[doc = "- **median** (\"Median\"): The [median](https://en.wikipedia.org/wiki/Median) of N measurements over"]
#[doc = "the stated period."]
#[doc = "- **std-dev** (\"Standard Deviation\"): The [standard"]
#[doc = "deviation](https://en.wikipedia.org/wiki/Standard_deviation) of N measurements over the stated"]
#[doc = "period."]
#[doc = "- **sum** (\"Sum\"): The [sum](https://en.wikipedia.org/wiki/Summation) of N measurements over the"]
#[doc = "stated period."]
#[doc = "- **variance** (\"Variance\"): The [variance](https://en.wikipedia.org/wiki/Variance) of N"]
#[doc = "measurements over the stated period."]
#[doc = "- **20-percent** (\"20th Percentile\"): The 20th"]
#[doc = "[Percentile](https://en.wikipedia.org/wiki/Percentile) of N measurements over the stated period."]
#[doc = "- **80-percent** (\"80th Percentile\"): The 80th"]
#[doc = "[Percentile](https://en.wikipedia.org/wiki/Percentile) of N measurements over the stated period."]
#[doc = "- **4-lower** (\"Lower Quartile\"): The lower [Quartile](https://en.wikipedia.org/wiki/Quartile)"]
#[doc = "Boundary of N measurements over the stated period."]
#[doc = "- **4-upper** (\"Upper Quartile\"): The upper [Quartile](https://en.wikipedia.org/wiki/Quartile)"]
#[doc = "Boundary of N measurements over the stated period."]
#[doc = "- **4-dev** (\"Quartile Deviation\"): The difference between the upper and lower"]
#[doc = "[Quartiles](https://en.wikipedia.org/wiki/Quartile) is called the Interquartile range. (IQR = Q3-Q1)"]
#[doc = "Quartile deviation or Semi-interquartile range is one-half the difference between the first and the"]
#[doc = "third quartiles."]
#[doc = "- **5-1** (\"1st Quintile\"): The lowest of four values that divide the N measurements into a"]
#[doc = "frequency distribution of five classes with each containing one fifth of the total population."]
#[doc = "- **5-2** (\"2nd Quintile\"): The second of four values that divide the N measurements into a"]
#[doc = "frequency distribution of five classes with each containing one fifth of the total population."]
#[doc = "- **5-3** (\"3rd Quintile\"): The third of four values that divide the N measurements into a frequency"]
#[doc = "distribution of five classes with each containing one fifth of the total population."]
#[doc = "- **5-4** (\"4th Quintile\"): The fourth of four values that divide the N measurements into a"]
#[doc = "frequency distribution of five classes with each containing one fifth of the total population."]
#[doc = "- **skew** (\"Skew\"): Skewness is a measure of the asymmetry of the probability distribution of a"]
#[doc = "real-valued random variable about its mean. The skewness value can be positive or negative, or even"]
#[doc = "undefined. Source: [Wikipedia](https://en.wikipedia.org/wiki/Skewness)."]
#[doc = "- **kurtosis** (\"Kurtosis\"): Kurtosis is a measure of the \"tailedness\" of the probability"]
#[doc = "distribution of a real-valued random variable. Source:"]
#[doc = "[Wikipedia](https://en.wikipedia.org/wiki/Kurtosis)."]
#[doc = "- **regression** (\"Regression\"): Linear regression is an approach for modeling two-dimensional"]
#[doc = "sample points with one independent variable and one dependent variable (conventionally, the x and y"]
#[doc = "coordinates in a Cartesian coordinate system) and finds a linear function (a non-vertical straight"]
#[doc = "line) that, as accurately as possible, predicts the dependent variable values as a function of the"]
#[doc = "independent variables. Source: [Wikipedia](https://en.wikipedia.org/wiki/Simple_linear_regression)"]
#[doc = "This Statistic code will return both a gradient and an intercept value."]
#[doc = "If successful, the operation returns an Observation resource for each code with the results of the"]
#[doc = "statistical calculations as component value pairs where the component code = the statistical code."]
#[doc = "The Observation also contains the input parameters `patient`,`code` and `duration` parameters. If"]
#[doc = "unsuccessful, an [`OperationOutcome`](operationoutcome.html) with an error message will be returned."]
#[doc = "The client can request that all the observations on which the statistics are based be returned as"]
#[doc = "well, using the include parameter. If an include parameter is specified, a limit may also be"]
#[doc = "specified; the sources observations are subsetted at the server's discretion if count > limit. This"]
#[doc = "functionality is included with the intent of supporting graphical presentation"]
pub mod ObservationStats {
    use haste_fhir_model::r4::generated::resources::{
        Observation, Parameters, ParametersParameter, Resource,
    };
    use haste_fhir_model::r4::generated::types::{
        Coding, FHIRBoolean, FHIRCode, FHIRDecimal, FHIRPositiveInt, FHIRString, FHIRUri, Period,
    };
    use haste_fhir_operation_error::OperationOutcomeError;
    use haste_fhir_ops::derive::{FromParameters, ToParameters};
    pub const CODE: &str = "stats";
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Input {
        #[doc = "The subject of the relevant Observations, which has the value of the"]
        #[doc = "`Observation.subject.reference.` E.g. 'Patient/123'. Reference can be to an absolute URL, but"]
        #[doc = "servers only perform stats on their own observations"]
        pub subject: FHIRUri,
        #[doc = "The test code(s) upon which the statistics are being performed. Provide along with a system, or as a"]
        #[doc = "coding. For example, the LOINC code = 2339-0 (Glucose [Mass/\u{200b}volume] in Blood) will evaluate all"]
        #[doc = "relevant Observations with this code in `Observation.code` and `Observation.component.code`. For"]
        #[doc = "LOINC codes that are panels, e.g., 85354-9(Blood pressure panel with all children optional), the"]
        #[doc = "stats operation returns statistics for each of the individual panel measurements. That means it will"]
        #[doc = "include and evaluate all values grouped by code for all the individual observations that are: 1)"]
        #[doc = "referenced in `.related` for `.related.type` = `has-member` and 2) component observations in"]
        #[doc = "`Observation.component`."]
        pub code: Option<Vec<FHIRString>>,
        #[doc = "The system for the code(s). Or provide a coding instead"]
        pub system: Option<FHIRUri>,
        #[doc = "The test code upon which the statistics are being performed, as a Coding"]
        pub coding: Option<Vec<Coding>>,
        #[doc = "The time period of interest given as hours. For example, the duration = \"1\" represents the last hour"]
        #[doc = "- the time period from on hour ago to now"]
        pub duration: Option<FHIRDecimal>,
        #[doc = "The time period over which the calculations to be performed, if a duration is not provided"]
        pub period: Option<Period>,
        #[doc = "average|max|min|count The statistical operations to be performed on the relevant operations."]
        #[doc = "Multiple statistics operations can be specified. These codes are defined"]
        #[doc = "[`here`](valueset-observation-statistics.html)"]
        pub statistic: Vec<FHIRCode>,
        #[doc = "Whether to return the observations on which the statistics are based"]
        pub include: Option<FHIRBoolean>,
        #[doc = "If an include parameter is specified, a limit may also be specified to limit the number of source"]
        #[doc = "Observations returned. If the include paramter is absent or equal to \"false\" the limit parameter"]
        #[doc = "SHALL be ignored by the server"]
        pub limit: Option<FHIRPositiveInt>,
    }
    impl From<Input> for Resource {
        fn from(value: Input) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Output {
        #[doc = "A set of observations, one observation for each code, each containing one component for each"]
        #[doc = "statistic. The `Observation.component.code` contains the statistic, and is relative to the"]
        #[doc = "`Observation.code` and cannot be interpreted independently. The Observation will also contain a"]
        #[doc = "subject, effectivePeriod, and code reflecting the input parameters. The status is fixed to `final`."]
        pub statistics: Vec<Observation>,
        #[doc = "Source observations on which the statistics are based"]
        pub source: Option<Vec<Observation>>,
    }
    impl From<Output> for Resource {
        fn from(value: Output) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
}
#[doc = "This operation is used to return all the information related to one or more patients described in"]
#[doc = "the resource or context on which this operation is invoked. The response is a bundle of type"]
#[doc = "\"searchset\". At a minimum, the patient resource(s) itself is returned, along with any other"]
#[doc = "resources that the server has that are related to the patient(s), and that are available for the"]
#[doc = "given user. The server also returns whatever resources are needed to support the records - e.g."]
#[doc = "linked practitioners, medications, locations, organizations etc."]
#[doc = ""]
#[doc = "The intended use for this operation is to provide a patient with access to their entire record (e.g."]
#[doc = "\"Blue Button\"), or for provider or other user to perform a bulk data download. The server SHOULD"]
#[doc = "return at least all resources that it has that are in the patient compartment for the identified"]
#[doc = "patient(s), and any resource referenced from those, including binaries and attachments. In the US"]
#[doc = "Realm, at a minimum, the resources returned SHALL include all the data covered by the meaningful use"]
#[doc = "common data elements as defined in the US Core Implementation Guide. Other applicable implementation"]
#[doc = "guides may make additional rules about how much information that is returned."]
pub mod PatientEverything {
    use haste_fhir_model::r4::generated::resources::{
        Bundle, Parameters, ParametersParameter, Resource,
    };
    use haste_fhir_model::r4::generated::types::{
        FHIRCode, FHIRDate, FHIRInstant, FHIRInteger, FHIRString,
    };
    use haste_fhir_operation_error::OperationOutcomeError;
    use haste_fhir_ops::derive::{FromParameters, ToParameters};
    pub const CODE: &str = "everything";
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Input {
        #[doc = "The date range relates to care dates, not record currency dates - e.g. all records relating to care"]
        #[doc = "provided in a certain date range. If no start date is provided, all records prior to the end date"]
        #[doc = "are in scope."]
        pub start: Option<FHIRDate>,
        #[doc = "The date range relates to care dates, not record currency dates - e.g. all records relating to care"]
        #[doc = "provided in a certain date range. If no end date is provided, all records subsequent to the start"]
        #[doc = "date are in scope."]
        pub end: Option<FHIRDate>,
        #[doc = "Resources updated after this period will be included in the response. The intent of this parameter"]
        #[doc = "is to allow a client to request only records that have changed since the last request, based on"]
        #[doc = "either the return header time, or or (for asynchronous use), the transaction time"]
        #[parameter_rename = "_since"]
        pub since: Option<FHIRInstant>,
        #[doc = "One or more parameters, each containing one or more comma-delimited FHIR resource types to include"]
        #[doc = "in the return resources. In the absence of any specified types, the server returns all resource"]
        #[doc = "types"]
        #[parameter_rename = "_type"]
        pub type_: Option<Vec<FHIRCode>>,
        #[doc = "See discussion below on the utility of paging through the results of the $everything operation"]
        #[parameter_rename = "_count"]
        pub count: Option<FHIRInteger>,
    }
    impl From<Input> for Resource {
        fn from(value: Input) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
    #[derive(Debug, FromParameters)]
    pub struct Output {
        #[doc = "The bundle type is \"searchset\""]
        #[parameter_rename = "return"]
        pub return_: Bundle,
    }
    impl From<Output> for Resource {
        fn from(value: Output) -> Self {
            Resource::Bundle(value.return_)
        }
    }
}
#[doc = "A Master Patient Index ([MPI](http://en.wikipedia.org/wiki/Enterprise_master_patient_index) ) is a"]
#[doc = "service used to manage patient identification in a context where multiple patient databases exist."]
#[doc = "Healthcare applications and middleware use the MPI to match patients between the databases, and to"]
#[doc = "store new patient details as they are encountered. `MPIs` are highly specialized applications, often"]
#[doc = "tailored extensively to the institution's particular mix of patients. `MPIs` can also be run on a"]
#[doc = "regional and national basis."]
#[doc = ""]
#[doc = "To ask an MPI to match a patient, clients use the \"$match\" operation, which accepts a patient"]
#[doc = "resource which may be only partially complete. The data provided is interpreted as an MPI input and"]
#[doc = "processed by an algorithm of some kind that uses the data to determine the most appropriate matches"]
#[doc = "in the patient set. Note that different MPI matching algorithms have different required inputs. The"]
#[doc = "generic $match operation does not specify any particular algorithm, nor a minimum set of information"]
#[doc = "that must be provided when asking for an MPI match operation to be performed, but many"]
#[doc = "implementations will have a set of minimum information, which may be declared in their definition of"]
#[doc = "the $match operation by specifying a profile on the resource parameter, indicating which properties"]
#[doc = "are required in the search. The patient resource submitted to the operation does not have to be"]
#[doc = "complete, nor does it need to pass validation (i.e. mandatory fields don't need to be populated),"]
#[doc = "but it does have to be a valid instance, as it is used as the reference data to match against."]
pub mod PatientMatch {
    use haste_fhir_model::r4::generated::resources::{
        Bundle, Parameters, ParametersParameter, Resource,
    };
    use haste_fhir_model::r4::generated::types::{FHIRBoolean, FHIRInteger, FHIRString};
    use haste_fhir_operation_error::OperationOutcomeError;
    use haste_fhir_ops::derive::{FromParameters, ToParameters};
    pub const CODE: &str = "match";
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Input {
        #[doc = "Use this to provide an entire set of patient details for the MPI to match against (e.g. POST a"]
        #[doc = "patient record to Patient/$match)."]
        pub resource: Resource,
        #[doc = "If there are multiple potential matches, then the match should not return the results with this flag"]
        #[doc = "set to true. When false, the server may return multiple results with each result graded accordingly."]
        pub onlyCertainMatches: Option<FHIRBoolean>,
        #[doc = "The maximum number of records to return. If no value is provided, the server decides how many"]
        #[doc = "matches to return. Note that clients should be careful when using this, as it may prevent probable -"]
        #[doc = "and valid - matches from being returned"]
        pub count: Option<FHIRInteger>,
    }
    impl From<Input> for Resource {
        fn from(value: Input) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
    #[derive(Debug, FromParameters)]
    pub struct Output {
        #[doc = "A bundle contain a set of Patient records that represent possible matches, optionally it may also"]
        #[doc = "contain an `OperationOutcome` with further information about the search results (such as warnings or"]
        #[doc = "information messages, such as a count of records that were close but eliminated) If the operation"]
        #[doc = "was unsuccessful, then an `OperationOutcome` may be returned along with a `BadRequest` status Code"]
        #[doc = "(e.g. security issue, or insufficient properties in patient fragment - check against profile)"]
        #[parameter_rename = "return"]
        pub return_: Bundle,
    }
    impl From<Output> for Resource {
        fn from(value: Output) -> Self {
            Resource::Bundle(value.return_)
        }
    }
}
#[doc = "The apply operation applies a `PlanDefinition` to a given context"]
pub mod PlanDefinitionApply {
    use haste_fhir_model::r4::generated::resources::{
        CarePlan, Parameters, ParametersParameter, PlanDefinition, Resource,
    };
    use haste_fhir_model::r4::generated::types::{CodeableConcept, FHIRString};
    use haste_fhir_operation_error::OperationOutcomeError;
    use haste_fhir_ops::derive::{FromParameters, ToParameters};
    pub const CODE: &str = "apply";
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Input {
        #[doc = "The plan definition to be applied. If the operation is invoked at the instance level, this parameter"]
        #[doc = "is not allowed; if the operation is invoked at the type level, this parameter is required"]
        pub planDefinition: Option<PlanDefinition>,
        #[doc = "The subject(s) that is/are the target of the plan to be applied. The subject may be a Patient,"]
        #[doc = "Practitioner, Organization, Location, Device, or Group. Subjects provided in this parameter will be"]
        #[doc = "resolved as the subject of the `PlanDefinition` based on the type of the subject. If multiple"]
        #[doc = "subjects of the same type are provided, the behavior is implementation-defined"]
        pub subject: Vec<FHIRString>,
        #[doc = "The encounter in context, if any"]
        pub encounter: Option<FHIRString>,
        #[doc = "The practitioner applying the plan definition"]
        pub practitioner: Option<FHIRString>,
        #[doc = "The organization applying the plan definition"]
        pub organization: Option<FHIRString>,
        #[doc = "The type of user initiating the request, e.g. patient, healthcare provider, or specific type of"]
        #[doc = "healthcare provider (physician, nurse, etc.)"]
        pub userType: Option<CodeableConcept>,
        #[doc = "Preferred language of the person using the system"]
        pub userLanguage: Option<CodeableConcept>,
        #[doc = "The task the system user is performing, e.g. laboratory results review, medication list review, etc."]
        #[doc = "This information can be used to tailor decision support outputs, such as recommended information"]
        #[doc = "resources"]
        pub userTaskContext: Option<CodeableConcept>,
        #[doc = "The current setting of the request (inpatient, outpatient, etc.)"]
        pub setting: Option<CodeableConcept>,
        #[doc = "Additional detail about the setting of the request, if any"]
        pub settingContext: Option<CodeableConcept>,
    }
    impl From<Input> for Resource {
        fn from(value: Input) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
    #[derive(Debug, FromParameters)]
    pub struct Output {
        #[doc = "The `CarePlan` that is the result of applying the plan definition"]
        #[parameter_rename = "return"]
        pub return_: CarePlan,
    }
    impl From<Output> for Resource {
        fn from(value: Output) -> Self {
            Resource::CarePlan(value.return_)
        }
    }
}
#[doc = "The data-requirements operation aggregates and returns the parameters and data requirements for the"]
#[doc = "plan definition and all its dependencies as a single module definition library"]
pub mod PlanDefinitionDataRequirements {
    use haste_fhir_model::r4::generated::resources::{
        Library, Parameters, ParametersParameter, Resource,
    };
    use haste_fhir_operation_error::OperationOutcomeError;
    use haste_fhir_ops::derive::{FromParameters, ToParameters};
    pub const CODE: &str = "data-requirements";
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Input {}
    impl From<Input> for Resource {
        fn from(value: Input) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
    #[derive(Debug, FromParameters)]
    pub struct Output {
        #[doc = "The result of the requirements gathering is a module-definition Library that describes the aggregate"]
        #[doc = "parameters, data requirements, and dependencies of the plan definition"]
        #[parameter_rename = "return"]
        pub return_: Library,
    }
    impl From<Output> for Resource {
        fn from(value: Output) -> Self {
            Resource::Library(value.return_)
        }
    }
}
#[doc = "This operation takes a resource in one form, and returns to in another form. Both input and output"]
#[doc = "are a single resource. The primary use of this operation is to convert between formats (e.g. (XML ->"]
#[doc = "JSON or vice versa)"]
pub mod ResourceConvert {
    use haste_fhir_model::r4::generated::resources::{Parameters, ParametersParameter, Resource};
    use haste_fhir_model::r4::generated::types::FHIRString;
    use haste_fhir_operation_error::OperationOutcomeError;
    use haste_fhir_ops::derive::{FromParameters, ToParameters};
    pub const CODE: &str = "convert";
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Input {
        #[doc = "The resource that is to be converted"]
        pub input: Resource,
    }
    impl From<Input> for Resource {
        fn from(value: Input) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Output {
        #[doc = "The resource after conversion"]
        pub output: Resource,
    }
    impl From<Output> for Resource {
        fn from(value: Output) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
}
#[doc = "Return an entire graph of resources based on a [`GraphDefinition`](graphdefinition.html). The"]
#[doc = "operation is invoked on a specific instance of a resource, and the graph definition tells the server"]
#[doc = "what other resources to return in the same packaage"]
pub mod ResourceGraph {
    use haste_fhir_model::r4::generated::resources::{
        Bundle, Parameters, ParametersParameter, Resource,
    };
    use haste_fhir_model::r4::generated::types::{FHIRString, FHIRUri};
    use haste_fhir_operation_error::OperationOutcomeError;
    use haste_fhir_ops::derive::{FromParameters, ToParameters};
    pub const CODE: &str = "graph";
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Input {
        #[doc = "Servers MAY choose to allow any graph definition to be specified, but MAY require that the client"]
        #[doc = "choose a graph definition from a specific list of known supported definitions. The server is not"]
        #[doc = "required to support a formal definition of the graph on the end point"]
        pub graph: FHIRUri,
    }
    impl From<Input> for Resource {
        fn from(value: Input) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Output {
        #[doc = "The set of resources that were in the graph based on the provided definition"]
        pub result: Bundle,
    }
    impl From<Output> for Resource {
        fn from(value: Output) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
}
#[doc = "Execute a graphql statement on a since resource or against the entire system. See the [`Using"]
#[doc = "GraphQL with FHIR`](graphql.html) page for further details."]
#[doc = ""]
#[doc = "For the purposes of graphQL compatibility, this operation can also be invoked using a POST with the"]
#[doc = "graphQL as the body, or a JSON body (see [graphQL spec](http://graphql.org/) for details)"]
pub mod ResourceGraphql {
    use haste_fhir_model::r4::generated::resources::{
        Binary, Parameters, ParametersParameter, Resource,
    };
    use haste_fhir_model::r4::generated::types::FHIRString;
    use haste_fhir_operation_error::OperationOutcomeError;
    use haste_fhir_ops::derive::{FromParameters, ToParameters};
    pub const CODE: &str = "graphql";
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Input {
        #[doc = "query"]
        pub query: FHIRString,
    }
    impl From<Input> for Resource {
        fn from(value: Input) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Output {
        #[doc = "The content is always returned as application/json; this SHOULD be specified in the Accept header"]
        pub result: Binary,
    }
    impl From<Output> for Resource {
        fn from(value: Output) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
}
#[doc = "This operation retrieves a summary of the profiles, tags, and security labels for the given scope;"]
#[doc = "e.g. for each scope:"]
#[doc = ""]
#[doc = "* system-wide: a list of all profiles, tags and security labels in use by the system"]
#[doc = "* resource-type level: A list of all profiles, tags, and security labels for the resource type"]
#[doc = "* individual resource level: A list of all profiles, tags, and security labels for the current"]
#[doc = "version of the resource. Also, as a special case, this operation (and other meta operations) can be"]
#[doc = "performed on a historical version of a resource)"]
pub mod ResourceMeta {
    use haste_fhir_model::r4::generated::resources::{Parameters, ParametersParameter, Resource};
    use haste_fhir_model::r4::generated::types::{FHIRString, Meta};
    use haste_fhir_operation_error::OperationOutcomeError;
    use haste_fhir_ops::derive::{FromParameters, ToParameters};
    pub const CODE: &str = "meta";
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Input {}
    impl From<Input> for Resource {
        fn from(value: Input) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Output {
        #[doc = "The meta returned by the operation"]
        #[parameter_rename = "return"]
        pub return_: Meta,
    }
    impl From<Output> for Resource {
        fn from(value: Output) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
}
#[doc = "This operation takes a meta, and adds the profiles, tags, and security labels found in it to the"]
#[doc = "nominated resource"]
pub mod ResourceMetaAdd {
    use haste_fhir_model::r4::generated::resources::{Parameters, ParametersParameter, Resource};
    use haste_fhir_model::r4::generated::types::{FHIRString, Meta};
    use haste_fhir_operation_error::OperationOutcomeError;
    use haste_fhir_ops::derive::{FromParameters, ToParameters};
    pub const CODE: &str = "meta-add";
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Input {
        #[doc = "Profiles, tags, and security labels to add to the existing resource. Note that profiles, tags, and"]
        #[doc = "security labels are sets, and duplicates are not created. The identity of a tag or security label is"]
        #[doc = "the system+code. When matching existing tags during adding, version and display are ignored. For"]
        #[doc = "profiles, matching is based on the full URL"]
        pub meta: Meta,
    }
    impl From<Input> for Resource {
        fn from(value: Input) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Output {
        #[doc = "Resulting meta for the resource"]
        #[parameter_rename = "return"]
        pub return_: Meta,
    }
    impl From<Output> for Resource {
        fn from(value: Output) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
}
#[doc = "This operation takes a meta, and deletes the profiles, tags, and security labels found in it from"]
#[doc = "the nominated resource"]
pub mod ResourceMetaDelete {
    use haste_fhir_model::r4::generated::resources::{Parameters, ParametersParameter, Resource};
    use haste_fhir_model::r4::generated::types::{FHIRString, Meta};
    use haste_fhir_operation_error::OperationOutcomeError;
    use haste_fhir_ops::derive::{FromParameters, ToParameters};
    pub const CODE: &str = "meta-delete";
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Input {
        #[doc = "Profiles, tags, and security labels to delete from the existing resource. It is not an error if"]
        #[doc = "these tags, profiles, and labels do not exist. The identity of a tag or security label is the"]
        #[doc = "system+code. When matching existing tags during deletion, version and display are ignored. For"]
        #[doc = "profiles, matching is based on the full URL"]
        pub meta: Meta,
    }
    impl From<Input> for Resource {
        fn from(value: Input) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Output {
        #[doc = "Resulting meta for the resource"]
        #[parameter_rename = "return"]
        pub return_: Meta,
    }
    impl From<Output> for Resource {
        fn from(value: Output) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
}
#[doc = "The validate operation checks whether the attached content would be acceptable either generally, as"]
#[doc = "a create, an update or as a delete to an existing resource. The action the server takes depends on"]
#[doc = "the mode parameter:"]
#[doc = ""]
#[doc = "* [mode not provided]: The server checks the content of the resource against any schema, constraint"]
#[doc = "rules, and other general terminology rules"]
#[doc = "* create: The server checks the content, and then checks that the content would be acceptable as a"]
#[doc = "create (e.g. that the content would not violate any uniqueness constraints)"]
#[doc = "* update: The server checks the content, and then checks that it would accept it as an update"]
#[doc = "against the nominated specific resource (e.g. that there are no changes to immutable fields the"]
#[doc = "server does not allow to change, and checking version integrity if appropriate)"]
#[doc = "* delete: The server ignores the content, and checks that the nominated resource is allowed to be"]
#[doc = "deleted (e.g. checking referential integrity rules)"]
#[doc = ""]
#[doc = "Modes update and delete can only be used when the operation is invoked at the resource instance"]
#[doc = "level. The return from this operation is an [`OperationOutcome`](operationoutcome.html)"]
#[doc = ""]
#[doc = "Note that this operation is not the only way to validate resources - see [`Validating"]
#[doc = "Resources`](validation.html) for further information."]
pub mod ResourceValidate {
    use haste_fhir_model::r4::generated::resources::{
        OperationOutcome, Parameters, ParametersParameter, Resource,
    };
    use haste_fhir_model::r4::generated::types::{FHIRCode, FHIRString, FHIRUri};
    use haste_fhir_operation_error::OperationOutcomeError;
    use haste_fhir_ops::derive::{FromParameters, ToParameters};
    pub const CODE: &str = "validate";
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Input {
        #[doc = "Must be present unless the mode is \"delete\""]
        pub resource: Option<Resource>,
        #[doc = "Default is 'no action'; (e.g. general validation)"]
        pub mode: Option<FHIRCode>,
        #[doc = "If this is nominated, then the resource is validated against this specific profile. If a profile is"]
        #[doc = "nominated, and the server cannot validate against the nominated profile, it SHALL return an error"]
        pub profile: Option<FHIRUri>,
    }
    impl From<Input> for Resource {
        fn from(value: Input) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
    #[derive(Debug, FromParameters)]
    pub struct Output {
        #[doc = "If the operation outcome does not list any errors, and a mode was specified, then this is an"]
        #[doc = "indication that the operation would be expected to succeed (excepting for transactional integrity"]
        #[doc = "issues, see below)"]
        #[parameter_rename = "return"]
        pub return_: OperationOutcome,
    }
    impl From<Output> for Resource {
        fn from(value: Output) -> Self {
            Resource::OperationOutcome(value.return_)
        }
    }
}
#[doc = "Generates a [`Questionnaire`](questionnaire.html) instance based on a specified"]
#[doc = "[`StructureDefinition`](structuredefinition.html), creating questions for each core element or"]
#[doc = "extension element found in the [`StructureDefinition`](structuredefinition.html)."]
#[doc = ""]
#[doc = "If the operation is not called at the instance level, one of the *identifier*, *profile* or *url*"]
#[doc = "`in` parameters must be provided. If more than one is specified, servers may raise an error or may"]
#[doc = "resolve with the parameter of their choice. If called at the instance level, these parameters will"]
#[doc = "be ignored. The response will contain a [`Questionnaire`](questionnaire.html) instance based on the"]
#[doc = "specified [`StructureDefinition`](structuredefinition.html) and/or an"]
#[doc = "[`OperationOutcome`](operationoutcome.html) resource with errors or warnings. Nested groups are used"]
#[doc = "to handle complex structures and data types. If the `supportedOnly` parameter is set to true, only"]
#[doc = "those elements marked as \"must support\" will be included."]
#[doc = ""]
#[doc = "This operation is intended to enable auto-generation of simple interfaces for arbitrary profiles."]
#[doc = "The `questionnaire` approach to data entry has limitations that will make it less optimal than"]
#[doc = "custom-defined interfaces. However, this function may be useful for simple applications or for"]
#[doc = "systems that wish to support \"non-core\" resources with minimal development effort."]
pub mod StructureDefinitionQuestionnaire {
    use haste_fhir_model::r4::generated::resources::{
        Parameters, ParametersParameter, Questionnaire, Resource,
    };
    use haste_fhir_model::r4::generated::types::{FHIRBoolean, FHIRCanonical, FHIRString};
    use haste_fhir_operation_error::OperationOutcomeError;
    use haste_fhir_ops::derive::{FromParameters, ToParameters};
    pub const CODE: &str = "questionnaire";
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Input {
        #[doc = "A logical identifier (i.e. '`StructureDefinition.identifier`''). The server must know the"]
        #[doc = "`StructureDefinition` or be able to retrieve it from other known repositories."]
        #[parameter_rename = "identifier"]
        pub identifier_: Option<FHIRCanonical>,
        #[doc = "The [`StructureDefinition`](structuredefinition.html) is provided directly as part of the request."]
        #[doc = "Servers may choose not to accept profiles in this fashion"]
        pub profile: Option<FHIRString>,
        #[doc = "The `StructureDefinition's` official URL (i.e. '`StructureDefinition.url`'). The server must know"]
        #[doc = "the `StructureDefinition` or be able to retrieve it from other known repositories."]
        pub url: Option<FHIRCanonical>,
        #[doc = "If true, the questionnaire will only include those elements marked as \"mustSupport=`true`\" in the"]
        #[doc = "`StructureDefinition`."]
        pub supportedOnly: Option<FHIRBoolean>,
    }
    impl From<Input> for Resource {
        fn from(value: Input) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
    #[derive(Debug, FromParameters)]
    pub struct Output {
        #[doc = "The questionnaire form generated based on the `StructureDefinition`."]
        #[parameter_rename = "return"]
        pub return_: Questionnaire,
    }
    impl From<Output> for Resource {
        fn from(value: Output) -> Self {
            Resource::Questionnaire(value.return_)
        }
    }
}
#[doc = "Generates a [`StructureDefinition`](structuredefinition.html) instance with a snapshot, based on a"]
#[doc = "differential in a specified [`StructureDefinition`](structuredefinition.html)."]
#[doc = ""]
#[doc = "If the operation is not called at the instance level, either *definition* or *url* `in` parameters"]
#[doc = "must be provided. If more than one is specified, servers may raise an error or may resolve with the"]
#[doc = "parameter of their choice. If called at the instance level, these parameters will be ignored."]
pub mod StructureDefinitionSnapshot {
    use haste_fhir_model::r4::generated::resources::{
        Parameters, ParametersParameter, Resource, StructureDefinition,
    };
    use haste_fhir_model::r4::generated::types::FHIRString;
    use haste_fhir_operation_error::OperationOutcomeError;
    use haste_fhir_ops::derive::{FromParameters, ToParameters};
    pub const CODE: &str = "snapshot";
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Input {
        #[doc = "The [`StructureDefinition`](structuredefinition.html) is provided directly as part of the request."]
        #[doc = "Servers may choose not to accept profiles in this fashion"]
        pub definition: Option<StructureDefinition>,
        #[doc = "The `StructureDefinition's` canonical URL (i.e. '`StructureDefinition.url`'). The server must know"]
        #[doc = "the structure definition, or be able to retrieve it from other known repositories."]
        pub url: Option<FHIRString>,
    }
    impl From<Input> for Resource {
        fn from(value: Input) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
    #[derive(Debug, FromParameters)]
    pub struct Output {
        #[doc = "The structure definition with a snapshot"]
        #[parameter_rename = "return"]
        pub return_: StructureDefinition,
    }
    impl From<Output> for Resource {
        fn from(value: Output) -> Self {
            Resource::StructureDefinition(value.return_)
        }
    }
}
#[doc = "The transform operation takes input content, applies a structure map transform, and then returns the"]
#[doc = "output."]
pub mod StructureMapTransform {
    use haste_fhir_model::r4::generated::resources::{Parameters, ParametersParameter, Resource};
    use haste_fhir_model::r4::generated::types::{FHIRString, FHIRUri};
    use haste_fhir_operation_error::OperationOutcomeError;
    use haste_fhir_ops::derive::{FromParameters, ToParameters};
    pub const CODE: &str = "transform";
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Input {
        #[doc = "The structure map to apply. This is only needed if the operation is invoked at the resource level."]
        #[doc = "If the $transform operation is invoked on a particular structure map, this will be ignored by the"]
        #[doc = "server"]
        pub source: Option<FHIRUri>,
        #[doc = "The logical content to transform"]
        pub content: Resource,
    }
    impl From<Input> for Resource {
        fn from(value: Input) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Output {
        #[doc = "The result of the transform"]
        #[parameter_rename = "return"]
        pub return_: Resource,
    }
    impl From<Output> for Resource {
        fn from(value: Output) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
}
#[doc = "The definition of a value set is used to create a simple collection of codes suitable for use for"]
#[doc = "data entry or validation."]
#[doc = ""]
#[doc = "If the operation is not called at the instance level, one of the in parameters url, context or"]
#[doc = "valueSet must be provided. An expanded value set will be returned, or an `OperationOutcome` with an"]
#[doc = "error message."]
pub mod ValueSetExpand {
    use haste_fhir_model::r4::generated::resources::{
        Parameters, ParametersParameter, Resource, ValueSet,
    };
    use haste_fhir_model::r4::generated::types::{
        FHIRBoolean, FHIRCanonical, FHIRCode, FHIRDateTime, FHIRInteger, FHIRString, FHIRUri,
    };
    use haste_fhir_operation_error::OperationOutcomeError;
    use haste_fhir_ops::derive::{FromParameters, ToParameters};
    pub const CODE: &str = "expand";
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Input {
        #[doc = "A canonical reference to a value set. The server must know the value set (e.g. it is defined"]
        #[doc = "explicitly in the server's value sets, or it is defined implicitly by some code system known to the"]
        #[doc = "server"]
        pub url: Option<FHIRUri>,
        #[doc = "The value set is provided directly as part of the request. Servers may choose not to accept value"]
        #[doc = "sets in this fashion"]
        pub valueSet: Option<ValueSet>,
        #[doc = "The identifier that is used to identify a specific version of the value set to be used when"]
        #[doc = "generating the expansion. This is an arbitrary value managed by the value set author and is not"]
        #[doc = "expected to be globally unique. For example, it might be a timestamp (e.g. yyyymmdd) if a managed"]
        #[doc = "version is not available."]
        pub valueSetVersion: Option<FHIRString>,
        #[doc = "The context of the value set, so that the server can resolve this to a value set to expand. The"]
        #[doc = "recommended format for this URI is [Structure Definition URL]#[name or path into structure"]
        #[doc = "definition] e.g."]
        #[doc = "`http://hl7.org/fhir/StructureDefinition/observation-hspc-height-hspcheight#Observation.interpretation`."]
        #[doc = "Other forms may be used but are not defined. This form is only usable if the terminology server also"]
        #[doc = "has access to the conformance registry that the server is using, but can be used to delegate the"]
        #[doc = "mapping from an application context to a binding at run-time"]
        pub context: Option<FHIRUri>,
        #[doc = "If a context is provided, a context direction may also be provided. Valid values are:"]
        #[doc = ""]
        #[doc = "* `incoming`: the codes a client can use for PUT/POST operations, and"]
        #[doc = "* `outgoing`, the codes a client might receive from the server."]
        #[doc = ""]
        #[doc = "The purpose is to inform the server whether to use the value set associated with the context for"]
        #[doc = "reading or writing purposes (note: for most elements, this is the same value set, but there are a"]
        #[doc = "few elements where the reading and writing value sets are different)"]
        pub contextDirection: Option<FHIRCode>,
        #[doc = "A text filter that is applied to restrict the codes that are returned (this is useful in a UI"]
        #[doc = "context). The interpretation of this is delegated to the server in order to allow to determine the"]
        #[doc = "most optimal search approach for the context. The server can document the way this parameter works"]
        #[doc = "in `TerminologyCapabilities.expansion.textFilter.` Typical usage of this parameter includes"]
        #[doc = "functionality like:"]
        #[doc = ""]
        #[doc = "* using left matching e.g. \"acut ast\""]
        #[doc = "* allowing for wild cards such as %, &, ?"]
        #[doc = "* searching on definition as well as display(s)"]
        #[doc = "* allowing for search conditions (and / or / exclusions)"]
        #[doc = ""]
        #[doc = "Text Search engines such as Lucene or Solr, long with their considerable functionality, might also"]
        #[doc = "be used. The optional text search might also be code system specific, and servers might have"]
        #[doc = "different implementations for different code systems"]
        pub filter: Option<FHIRString>,
        #[doc = "The date for which the expansion should be generated. if a date is provided, it means that the"]
        #[doc = "server should use the value set / code system definitions as they were on the given date, or return"]
        #[doc = "an error if this is not possible. Normally, the date is the current conditions (which is the default"]
        #[doc = "value) but under some circumstances, systems need to generate an expansion as it would have been in"]
        #[doc = "the past. A typical example of this would be where code selection is constrained to the set of codes"]
        #[doc = "that were available when the patient was treated, not when the record is being edited. Note that"]
        #[doc = "which date is appropriate is a matter for implementation policy."]
        pub date: Option<FHIRDateTime>,
        #[doc = "Paging support - where to start if a subset is desired (default = 0). Offset is number of records"]
        #[doc = "(not number of pages)"]
        pub offset: Option<FHIRInteger>,
        #[doc = "Paging support - how many codes should be provided in a partial page view. Paging only applies to"]
        #[doc = "flat expansions - servers ignore paging if the expansion is not flat. If count = 0, the client is"]
        #[doc = "asking how large the expansion is. Servers SHOULD honor this request for hierarchical expansions as"]
        #[doc = "well, and simply return the overall count"]
        pub count: Option<FHIRInteger>,
        #[doc = "Controls whether concept designations are to be included or excluded in value set expansions"]
        pub includeDesignations: Option<FHIRBoolean>,
        #[doc = "A [`token`](search.html#token) that specifies a system+code that is either a use or a language."]
        #[doc = "Designations that match by language or use are included in the expansion. If no designation is"]
        #[doc = "specified, it is at the server discretion which designations to return"]
        pub designation: Option<Vec<FHIRString>>,
        #[doc = "Controls whether the value set definition is included or excluded in value set expansions"]
        pub includeDefinition: Option<FHIRBoolean>,
        #[doc = "Controls whether inactive concepts are included or excluded in value set expansions. Note that if"]
        #[doc = "the value set explicitly specifies that inactive codes are included, this parameter can still remove"]
        #[doc = "them from a specific expansion, but this parameter cannot include them if the value set excludes"]
        #[doc = "them"]
        pub activeOnly: Option<FHIRBoolean>,
        #[doc = "Controls whether or not the value set expansion nests codes or not (i.e."]
        #[doc = "`ValueSet.expansion.contains.contains`)"]
        pub excludeNested: Option<FHIRBoolean>,
        #[doc = "Controls whether or not the value set expansion is assembled for a user interface use or not. Value"]
        #[doc = "sets intended for User Interface might include [``abstract` codes`](codesystem.html#status) or have"]
        #[doc = "nested contains with items with no code or abstract = true, with the sole purpose of helping a user"]
        #[doc = "navigate through the list efficiently, where as a value set not generated for UI use might be flat,"]
        #[doc = "and only contain the selectable codes in the value set. The exact implications of 'for UI' depend on"]
        #[doc = "the code system, and what properties it exposes for a terminology server to use. In the FHIR"]
        #[doc = "Specification itself, the value set expansions are generated with exclude`NotForUI` = false, and the"]
        #[doc = "expansions used when generated schema / code etc, or performing validation, are all"]
        #[doc = "exclude`NotForUI` = true."]
        pub excludeNotForUI: Option<FHIRBoolean>,
        #[doc = "Controls whether or not the value set expansion includes post coordinated codes"]
        pub excludePostCoordinated: Option<FHIRBoolean>,
        #[doc = "Specifies the language to be used for description in the expansions i.e. the language to be used for"]
        #[doc = "`ValueSet.expansion.contains.display`"]
        pub displayLanguage: Option<FHIRCode>,
        #[doc = "Code system, or a particular version of a code system to be excluded from the value set expansion."]
        #[doc = "The format is the same as a canonical URL: `[system]|[version] - e.g. http://loinc.org|2.56`"]
        #[parameter_rename = "exclude-system"]
        pub exclude_system: Option<Vec<FHIRCanonical>>,
        #[doc = "Specifies a version to use for a system, if the value set does not specify which one to use. The"]
        #[doc = "format is the same as a canonical URL: `[system]|[version] - e.g. http://loinc.org|2.56`"]
        #[parameter_rename = "system-version"]
        pub system_version: Option<Vec<FHIRCanonical>>,
        #[doc = "Edge Case: Specifies a version to use for a system. If a value set specifies a different version, an"]
        #[doc = "error is returned instead of the expansion. The format is the same as a canonical URL:"]
        #[doc = "`[system]|[version] - e.g. http://loinc.org|2.56`"]
        #[parameter_rename = "check-system-version"]
        pub check_system_version: Option<Vec<FHIRCanonical>>,
        #[doc = "Edge Case: Specifies a version to use for a system. This parameter overrides any specified version"]
        #[doc = "in the value set (and any it depends on). The format is the same as a canonical URL:"]
        #[doc = "`[system]|[version] - e.g. http://loinc.org|2.56`. Note that this has obvious safety issues, in that"]
        #[doc = "it may result in a value set expansion giving a different list of codes that is both wrong and"]
        #[doc = "unsafe, and implementers should only use this capability reluctantly. It primarily exists to deal"]
        #[doc = "with situations where specifications have fallen into decay as time passes. If the value is"]
        #[doc = "override, the version used SHALL explicitly be represented in the expansion parameters"]
        #[parameter_rename = "force-system-version"]
        pub force_system_version: Option<Vec<FHIRCanonical>>,
    }
    impl From<Input> for Resource {
        fn from(value: Input) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
    #[derive(Debug, FromParameters)]
    pub struct Output {
        #[doc = "The result of the expansion. Servers generating expansions SHOULD ensure that all the parameters"]
        #[doc = "that affect the contents of the expansion are recorded in the `ValueSet.expansion.parameter` list"]
        #[parameter_rename = "return"]
        pub return_: ValueSet,
    }
    impl From<Output> for Resource {
        fn from(value: Output) -> Self {
            Resource::ValueSet(value.return_)
        }
    }
}
#[doc = "Validate that a coded value is in the set of codes allowed by a value set."]
#[doc = ""]
#[doc = "If the operation is not called at the instance level, one of the in parameters url, context or"]
#[doc = "valueSet must be provided. One (and only one) of the in parameters code, coding, or codeableConcept"]
#[doc = "must be provided. The operation returns a result (true / false), an error message, and the"]
#[doc = "recommended display for the code"]
pub mod ValueSetValidateCode {
    use haste_fhir_model::r4::generated::resources::{
        Parameters, ParametersParameter, Resource, ValueSet,
    };
    use haste_fhir_model::r4::generated::types::{
        CodeableConcept, Coding, FHIRBoolean, FHIRCode, FHIRDateTime, FHIRString, FHIRUri,
    };
    use haste_fhir_operation_error::OperationOutcomeError;
    use haste_fhir_ops::derive::{FromParameters, ToParameters};
    pub const CODE: &str = "validate-code";
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Input {
        #[doc = "Value set Canonical URL. The server must know the value set (e.g. it is defined explicitly in the"]
        #[doc = "server's value sets, or it is defined implicitly by some code system known to the server"]
        pub url: Option<FHIRUri>,
        #[doc = "The context of the value set, so that the server can resolve this to a value set to validate"]
        #[doc = "against. The recommended format for this URI is [Structure Definition URL]#[name or path into"]
        #[doc = "structure definition] e.g."]
        #[doc = "`http://hl7.org/fhir/StructureDefinition/observation-hspc-height-hspcheight#Observation.interpretation`."]
        #[doc = "Other forms may be used but are not defined. This form is only usable if the terminology server also"]
        #[doc = "has access to the conformance registry that the server is using, but can be used to delegate the"]
        #[doc = "mapping from an application context to a binding at run-time"]
        pub context: Option<FHIRUri>,
        #[doc = "The value set is provided directly as part of the request. Servers may choose not to accept value"]
        #[doc = "sets in this fashion. This parameter is used when the client wants the server to expand a value set"]
        #[doc = "that is not stored on the server"]
        pub valueSet: Option<ValueSet>,
        #[doc = "The identifier that is used to identify a specific version of the value set to be used when"]
        #[doc = "validating the code. This is an arbitrary value managed by the value set author and is not expected"]
        #[doc = "to be globally unique. For example, it might be a timestamp (e.g. yyyymmdd) if a managed version is"]
        #[doc = "not available."]
        pub valueSetVersion: Option<FHIRString>,
        #[doc = "The code that is to be validated. If a code is provided, a system or a context must be provided (if"]
        #[doc = "a context is provided, then the server SHALL ensure that the code is not ambiguous without a system)"]
        pub code: Option<FHIRCode>,
        #[doc = "The system for the code that is to be validated"]
        pub system: Option<FHIRUri>,
        #[doc = "The version of the system, if one was provided in the source data"]
        pub systemVersion: Option<FHIRString>,
        #[doc = "The display associated with the code, if provided. If a display is provided a code must be provided."]
        #[doc = "If no display is provided, the server cannot validate the display value, but may choose to return a"]
        #[doc = "recommended display name using the display parameter in the outcome. Whether displays are case"]
        #[doc = "sensitive is code system dependent"]
        pub display: Option<FHIRString>,
        #[doc = "A coding to validate"]
        pub coding: Option<Coding>,
        #[doc = "A full codeableConcept to validate. The server returns true if one of the coding values is in the"]
        #[doc = "value set, and may also validate that the codings are not in conflict with each other if more than"]
        #[doc = "one is present"]
        pub codeableConcept: Option<CodeableConcept>,
        #[doc = "The date for which the validation should be checked. Normally, this is the current conditions (which"]
        #[doc = "is the default values) but under some circumstances, systems need to validate that a correct code"]
        #[doc = "was used at some point in the past. A typical example of this would be where code selection is"]
        #[doc = "constrained to the set of codes that were available when the patient was treated, not when the"]
        #[doc = "record is being edited. Note that which date is appropriate is a matter for implementation policy."]
        pub date: Option<FHIRDateTime>,
        #[doc = "If this parameter has a value of true, the client is stating that the validation is being performed"]
        #[doc = "in a context where a concept designated as `abstract` is appropriate/allowed to be used, and the"]
        #[doc = "server should regard abstract codes as valid. If this parameter is false, abstract codes are not"]
        #[doc = "considered to be valid."]
        #[doc = ""]
        #[doc = "Note that. `abstract` is a property defined by many HL7 code systems that indicates that the concept"]
        #[doc = "is a logical grouping concept that is not intended to be used asa `concrete` concept to in an actual"]
        #[doc = "patient/care/process record. This language is borrowed from Object Orienated theory where `asbtract`"]
        #[doc = "objects are never instantiated. However in the general record and terminology eco-system, there are"]
        #[doc = "many contexts where it is appropraite to use these codes e.g. as decision making criterion, or when"]
        #[doc = "editing value sets themselves. This parameter allows a client to indicate to the server that it is"]
        #[doc = "working in such a context."]
        #[parameter_rename = "abstract"]
        pub abstract_: Option<FHIRBoolean>,
        #[doc = "Specifies the language to be used for description when validating the display property"]
        pub displayLanguage: Option<FHIRCode>,
    }
    impl From<Input> for Resource {
        fn from(value: Input) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Output {
        #[doc = "True if the concept details supplied are valid"]
        pub result: FHIRBoolean,
        #[doc = "Error details, if result = false. If this is provided when result = true, the message carries hints"]
        #[doc = "and warnings"]
        pub message: Option<FHIRString>,
        #[doc = "A valid display for the concept if the system wishes to display this to a user"]
        pub display: Option<FHIRString>,
    }
    impl From<Output> for Resource {
        fn from(value: Output) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
}
#[doc = "Execute a view definition against supplied or server data."]
pub mod ViewDefinitionRun {
    use haste_fhir_model::r4::generated::resources::{
        Binary, Parameters, ParametersParameter, Resource, ViewDefinition,
    };
    use haste_fhir_model::r4::generated::types::{
        FHIRBoolean, FHIRCode, FHIRInstant, FHIRInteger, FHIRString, Reference,
    };
    use haste_fhir_operation_error::OperationOutcomeError;
    use haste_fhir_ops::derive::{FromParameters, ToParameters};
    pub const CODE: &str = "viewdefinition-run";
    #[derive(Debug, FromParameters, ToParameters)]
    pub struct Input {
        #[doc = "Output format for the result (for example json, ndjson, csv, parquet). Optional; if omitted, the"]
        #[doc = "server returns ndjson by default."]
        #[parameter_rename = "_format"]
        pub format: Option<FHIRCode>,
        #[doc = "Include CSV headers (default true). Applies only when csv output is requested."]
        pub header: Option<FHIRBoolean>,
        #[doc = "Reference to a `ViewDefinition` stored on the server."]
        pub viewReference: Option<Reference>,
        #[doc = "Inline `ViewDefinition` resource to execute."]
        pub viewResource: Option<ViewDefinition>,
        #[doc = "Restrict execution to the specified patient."]
        pub patient: Option<Reference>,
        #[doc = "Restrict execution to members of the given group(s)."]
        pub group: Option<Vec<Reference>>,
        #[doc = "External data source to use (for example a URI or bucket name)."]
        pub source: Option<FHIRString>,
        #[doc = "FHIR resources to transform instead of using server data."]
        pub resource: Option<Vec<Resource>>,
        #[doc = "Maximum number of rows to return."]
        #[parameter_rename = "_limit"]
        pub limit: Option<FHIRInteger>,
        #[doc = "Include only resources modified after this instant."]
        #[parameter_rename = "_since"]
        pub since: Option<FHIRInstant>,
    }
    impl From<Input> for Resource {
        fn from(value: Input) -> Self {
            let parameters: Vec<ParametersParameter> = value.into();
            Resource::Parameters(Parameters {
                parameter: Some(parameters),
                ..Default::default()
            })
        }
    }
    #[derive(Debug, FromParameters)]
    pub struct Output {
        #[doc = "Transformed data encoded in the requested output format."]
        #[parameter_rename = "return"]
        pub return_: Binary,
    }
    impl From<Output> for Resource {
        fn from(value: Output) -> Self {
            Resource::Binary(value.return_)
        }
    }
}
