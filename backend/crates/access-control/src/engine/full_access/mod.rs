use haste_fhir_model::r4::generated::{
    resources::AccessPolicyV2, terminology::AccessPolicyv2Engine,
};

use crate::context::PermissionLevel;

pub fn evaluate(policy: &AccessPolicyV2) -> PermissionLevel {
    // Sanity check to ensure we are only evaluating FullAccess policies here.
    // Note this is done on root lib.rs evaluation of policy.
    if AccessPolicyv2Engine::full_access() == policy.engine {
        PermissionLevel::Allow
    } else {
        PermissionLevel::Deny
    }
}
