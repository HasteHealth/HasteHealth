pub mod client_app;
pub mod scopes;

#[cfg(test)]
mod tests {
    use oxidized_fhir_model::r4::generated::resources::ResourceType;

    use super::scopes::*;
    use super::*;
    #[test]
    fn test_multiple_correct() {
        assert_eq!(
            scopes::Scopes::try_from("openid profile email offline_access launch/patient user/*.*")
                .unwrap(),
            Scopes(vec![
                Scope::OIDC(OIDCScope::OpenId),
                Scope::OIDC(OIDCScope::Profile),
                Scope::OIDC(OIDCScope::Email),
                Scope::OIDC(OIDCScope::OfflineAccess),
                Scope::SMART(SmartScope::LaunchType(LaunchTypeScope {
                    launch_type: LaunchType::Patient,
                })),
                Scope::SMART(SmartScope::Resource(SMARTResourceScope {
                    user: SmartResourceScopeUser::User,
                    level: SmartResourceScopeLevel::AllResources,
                    permissions: SmartResourceScopePermissions {
                        create: true,
                        read: true,
                        update: true,
                        delete: true,
                        search: true,
                    },
                })),
            ]),
        );

        assert_eq!(
            scopes::Scopes::try_from("launch/encounter   system/Patient.cud").unwrap(),
            Scopes(vec![
                Scope::SMART(SmartScope::LaunchType(LaunchTypeScope {
                    launch_type: LaunchType::Encounter,
                })),
                Scope::SMART(SmartScope::Resource(SMARTResourceScope {
                    user: SmartResourceScopeUser::System,
                    level: SmartResourceScopeLevel::ResourceType(ResourceType::Patient),
                    permissions: SmartResourceScopePermissions {
                        create: true,
                        read: false,
                        update: true,
                        delete: true,
                        search: false,
                    },
                })),
            ]),
        );
    }

    #[test]
    fn invalid_order() {
        assert_eq!(
            scopes::Scopes::try_from("launch/encounter   system/Patient.duc").is_err(),
            true
        );
    }

    #[test]
    fn invalid_system() {
        assert_eq!(
            scopes::Scopes::try_from("launch/encounter   sytem/Patient.cud").is_err(),
            true
        );
    }

    #[test]
    fn unknown_scope() {
        assert_eq!(
            scopes::Scopes::try_from("badscope  sytem/Patient.cud").is_err(),
            true
        );
    }
}
