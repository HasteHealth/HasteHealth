//! Tri-state evaluation of one `AccessPolicyV3`.
//!
//! A rule is permit, deny or not applicable; a rule whose expression fails is
//! indeterminate, which every combining algorithm counts as a deny. The policy's rules
//! combine under its algorithm and nested rules under their parent's, so a rule means
//! the same thing at any depth.

mod combining;

use crate::request_reflection::RequestReflection;
use combining::{Algorithm, algorithm, finish, step};
use haste_fhir_model::r4::{
    conversion::downcast_bool,
    generated::{
        resources::{AccessPolicyV3, AccessPolicyV3Rule, AccessPolicyV3RuleObligation},
        terminology::{Accesspolicyv3rulekind, Accesspolicyv3ruletype},
    },
};
use haste_fhirpath::{Config, ExternalConstantResolver, FHIRPathError, FPEngine};
use haste_reflect::{MetaValue, derive::Reflect};
use std::{collections::HashMap, fmt, future::Future, ops::ControlFlow, pin::Pin, sync::Arc};

pub use crate::request_reflection::Channel;

/// What a rule, or a whole policy, decides.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Decision {
    Permit,
    Deny,
    /// No rule matched the request.
    NotApplicable,
    /// An expression failed. Counts as a deny.
    Indeterminate,
}

impl fmt::Display for Decision {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Decision::Permit => "permit",
            Decision::Deny => "deny",
            Decision::NotApplicable => "not-applicable",
            Decision::Indeterminate => "indeterminate",
        })
    }
}

/// The caller, as `%principal`.
#[derive(Debug, Reflect)]
#[fhir_type = "BackboneElement"]
pub struct Principal {
    /// The token's subject.
    pub user: String,
}

/// What a policy's expressions can see: `%request` and `%principal`.
pub struct Environment {
    pub request: RequestReflection,
    pub principal: Principal,
}

/// A decision and, when it is a permit, the obligations of every rule that permitted.
pub type Evaluation<'p> = (Decision, Vec<&'p AccessPolicyV3RuleObligation>);

/// Evaluates one policy against the request in `environment`.
pub async fn evaluate<'p>(environment: &Environment, policy: &'p AccessPolicyV3) -> Evaluation<'p> {
    let variables: HashMap<String, &dyn MetaValue> = HashMap::from([
        (
            "request".to_string(),
            &environment.request as &dyn MetaValue,
        ),
        (
            "principal".to_string(),
            &environment.principal as &dyn MetaValue,
        ),
    ]);
    let cx = Cx {
        engine: FPEngine::new(),
        config: Arc::new(
            Config::builder()
                .with_variable_resolver(ExternalConstantResolver::Variable(Arc::new(variables))),
        ),
        policy: policy.id.as_deref().unwrap_or_default(),
    };

    let mut obligations = Vec::new();
    let decision = combine(
        &cx,
        algorithm(Some(&policy.combining)),
        policy.rule.as_deref().unwrap_or_default(),
        None,
        &mut obligations,
    )
    .await;
    (decision, obligations)
}

/// What every rule of one evaluation shares.
struct Cx<'e> {
    engine: FPEngine,
    config: Arc<Config<'e>>,
    /// Names the policy in the log entry of an indeterminate rule.
    policy: &'e str,
}

/// A rule's position, `rule[1].rule[0]`; formatted only when it is logged.
struct RulePath<'a> {
    parent: Option<&'a RulePath<'a>>,
    index: usize,
}

impl fmt::Display for RulePath<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(parent) = self.parent {
            write!(f, "{parent}.")?;
        }
        write!(f, "rule[{}]", self.index)
    }
}

/// Why a rule is indeterminate.
#[derive(Debug, thiserror::Error)]
enum Fault {
    #[error("{0}")]
    Expression(#[from] FHIRPathError),
    #[error("the result is not a single boolean")]
    NotBoolean,
}

/// Evaluates a target or condition the `FHIRPath` way: empty is false, one boolean is
/// itself, anything else is a fault.
async fn holds(cx: &Cx<'_>, expression: &str) -> Result<bool, Fault> {
    let result = cx
        .engine
        .evaluate_with_config(expression, vec![], cx.config.clone())
        .await?;
    match result.values() {
        [] => Ok(false),
        [value] => downcast_bool(*value).map_err(|_| Fault::NotBoolean),
        _ => Err(Fault::NotBoolean),
    }
}

/// One rule: its target and condition gate it, then its nested rules or its type decide.
async fn evaluate_rule<'p>(
    cx: &Cx<'_>,
    rule: &'p AccessPolicyV3Rule,
    path: &RulePath<'_>,
    obligations: &mut Vec<&'p AccessPolicyV3RuleObligation>,
) -> Decision {
    // A constraint narrows the data a request reaches; it decides nothing.
    if rule.kind == Accesspolicyv3rulekind::constraint() {
        return Decision::NotApplicable;
    }

    let target = rule.target.as_ref().map(|target| &target.expression);
    let condition = rule
        .condition
        .as_ref()
        .map(|condition| &condition.expression);
    for gate in [target, condition].into_iter().flatten() {
        let expression = gate.value.as_deref().unwrap_or_default();
        match holds(cx, expression).await {
            Ok(true) => {}
            Ok(false) => return Decision::NotApplicable,
            Err(fault) => {
                tracing::warn!(
                    policy = cx.policy,
                    rule = %path,
                    expression,
                    error = %fault,
                    "AccessPolicyV3 rule is indeterminate"
                );
                return Decision::Indeterminate;
            }
        }
    }

    let decision = match rule.rule.as_deref() {
        // A parent decides by its nested rules under its own combining; its type is unused.
        Some(nested) if !nested.is_empty() => {
            combine(
                cx,
                algorithm(rule.combining.as_ref()),
                nested,
                Some(path),
                obligations,
            )
            .await
        }
        _ if rule.type_ == Accesspolicyv3ruletype::permit() => Decision::Permit,
        _ => Decision::Deny,
    };
    if decision == Decision::Permit {
        obligations.extend(rule.obligation.iter().flatten());
    }
    decision
}

/// Folds the rules' decisions under `algorithm`, stopping once the result is settled.
/// Obligations gathered on the way stay only when the result is a permit.
fn combine<'a, 'p: 'a>(
    cx: &'a Cx<'a>,
    algorithm: Algorithm,
    rules: &'p [AccessPolicyV3Rule],
    parent: Option<&'a RulePath<'a>>,
    obligations: &'a mut Vec<&'p AccessPolicyV3RuleObligation>,
) -> Pin<Box<dyn Future<Output = Decision> + Send + 'a>> {
    Box::pin(async move {
        let mark = obligations.len();
        let mut result = Decision::NotApplicable;
        for (index, rule) in rules.iter().enumerate() {
            let path = RulePath { parent, index };
            match step(
                algorithm,
                result,
                evaluate_rule(cx, rule, &path, obligations).await,
            ) {
                ControlFlow::Continue(next) => result = next,
                ControlFlow::Break(settled) => {
                    result = settled;
                    break;
                }
            }
        }

        let decision = finish(algorithm, result);
        if decision != Decision::Permit {
            obligations.truncate(mark);
        }
        decision
    })
}
