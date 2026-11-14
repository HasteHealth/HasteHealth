use crate::fhir_client::{
    ClientState, ServerCTX,
    middleware::{Backend, Middleware, Next, Output},
};
use haste_fhir_client::request::{FHIRRequest, UpdateRequest};
use haste_fhir_model::r4::generated::{
    resources::{AccessPolicyV3, AccessPolicyV3Rule, Resource},
    terminology::{Accesspolicyv3rulekind, Accesspolicyv3ruletype, IssueType},
};
use haste_fhir_operation_error::OperationOutcomeError;
use std::sync::Arc;

mod references;

fn invalid(path: &str, message: &str) -> OperationOutcomeError {
    OperationOutcomeError::fatal(IssueType::invalid(), format!("{path}: {message}."))
}

/// Refuses a rule, or a nested one, that mixes elements of the other kind.
fn validate_rules(rules: &[AccessPolicyV3Rule], parent: &str) -> Result<(), OperationOutcomeError> {
    for (index, rule) in rules.iter().enumerate() {
        let path = format!("{parent}.rule[{index}]");
        let has_data = rule.data.as_ref().is_some_and(|data| !data.is_empty());

        if rule.kind == Accesspolicyv3rulekind::constraint() {
            if !has_data {
                return Err(invalid(&path, "a constraint rule requires data"));
            }
            if rule.condition.is_some() {
                return Err(invalid(&path, "a constraint rule cannot carry a condition"));
            }
        } else if has_data {
            return Err(invalid(&path, "a decision rule cannot carry data"));
        }

        let has_permit_only = rule.field.as_ref().is_some_and(|f| !f.is_empty())
            || rule.obligation.as_ref().is_some_and(|o| !o.is_empty());
        if has_permit_only && rule.type_ != Accesspolicyv3ruletype::permit() {
            return Err(invalid(
                &path,
                "field and obligation are only allowed on permit rules",
            ));
        }

        if let Some(nested) = &rule.rule {
            validate_rules(nested, &path)?;
        }
    }
    Ok(())
}

/// Checks the policy's shape and expressions, then stores what its rules reference in
/// place of whatever manifest the client sent.
fn prepare(policy: &mut AccessPolicyV3) -> Result<(), OperationOutcomeError> {
    validate_rules(policy.rule.as_deref().unwrap_or_default(), "AccessPolicyV3")?;
    let manifest = references::manifest(policy)?;
    policy.manifest = (!manifest.is_empty()).then_some(manifest);
    Ok(())
}

/// Validates an `AccessPolicyV3` on save and records the attributes each rule needs.
pub struct AccessPolicyV3Validation;

impl<B: Backend> Middleware<B> for AccessPolicyV3Validation {
    fn call<'a>(
        &'a self,
        state: &'a ClientState<B>,
        ctx: Arc<ServerCTX<B>>,
        mut request: FHIRRequest,
        next: Next<'a, B>,
    ) -> Output<'a> {
        let resource = match &mut request {
            FHIRRequest::Create(create) => Some(&mut create.resource),
            FHIRRequest::Update(UpdateRequest::Instance(update)) => Some(&mut update.resource),
            FHIRRequest::Update(UpdateRequest::Conditional(update)) => Some(&mut update.resource),
            _ => None,
        };
        if let Some(Resource::AccessPolicyV3(policy)) = resource
            && let Err(error) = prepare(policy)
        {
            return Box::pin(std::future::ready(Err(error)));
        }
        next.run(state, ctx, request)
    }
}
