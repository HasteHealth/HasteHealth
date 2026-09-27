//! Scope-aware filtering of the advertised MCP tool list.
//!
//! `tools/call` is already enforced by [`SMARTScopeAccessMiddleware`] on the way
//! through the FHIR client, so this module is not the security boundary for
//! reads and writes. It exists so the tool list an MCP client sees matches what
//! the caller can actually do -- advertising `fhir_r4_delete` to a token holding
//! only `user/Patient.rs` invites the model to plan work that is guaranteed to
//! fail.
//!
//! [`SMARTScopeAccessMiddleware`]: crate::fhir_client::middleware::auth_z::scope_check::SMARTScopeAccessMiddleware

use crate::mcp::operations::operation_tools::OperationTool;
use haste_fhir_model::r4::generated::resources::ResourceType;
use haste_jwt::scopes::{
    Scope, Scopes, SmartResourceScopeLevel, SmartResourceScopePermission, SmartResourceScopeUser,
    SmartScope,
};

/// Does any granted scope allow `permission` on `resource_type`?
///
/// `resource_type` of `None` means "any resource type", used for tools that are
/// not bound to one type (search across types, bundles, operations at the system
/// level); those pass if the permission is held anywhere.
pub fn allows(
    scopes: &Scopes,
    permission: &SmartResourceScopePermission,
    resource_type: Option<&ResourceType>,
) -> bool {
    scopes.0.iter().any(|scope| {
        let Scope::SMART(SmartScope::Resource(resource_scope)) = scope else {
            return false;
        };

        // Patient-level scopes are rejected outright by the scope-check
        // middleware, so advertising tools for them would be misleading.
        if resource_scope.user == SmartResourceScopeUser::Patient {
            return false;
        }

        if !resource_scope.permissions.has_permission(permission) {
            return false;
        }

        match (&resource_scope.level, resource_type) {
            (SmartResourceScopeLevel::AllResources, _) => true,
            (SmartResourceScopeLevel::ResourceType(_), None) => true,
            (SmartResourceScopeLevel::ResourceType(scoped), Some(requested)) => scoped == requested,
        }
    })
}

/// Whether the caller may invoke `operation`.
///
/// An operation that changes state needs a write permission; a read-only one
/// needs read or search. When the operation names resource types, the caller
/// must hold the permission on at least one of them.
pub fn allows_operation(scopes: &Scopes, operation: &OperationTool) -> bool {
    let required: &[SmartResourceScopePermission] = if operation.affects_state() {
        &[
            SmartResourceScopePermission::Create,
            SmartResourceScopePermission::Update,
            SmartResourceScopePermission::Delete,
        ]
    } else {
        &[
            SmartResourceScopePermission::Read,
            SmartResourceScopePermission::Search,
        ]
    };

    let resource_types = operation.resource_types();

    if resource_types.is_empty() {
        return required
            .iter()
            .any(|permission| allows(scopes, permission, None));
    }

    resource_types.iter().any(|resource_type| {
        required
            .iter()
            .any(|permission| allows(scopes, permission, Some(resource_type)))
    })
}

/// Does the caller hold `permission` on at least one resource type?
pub fn allows_any(scopes: &Scopes, permission: &SmartResourceScopePermission) -> bool {
    allows(scopes, permission, None)
}
