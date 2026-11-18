use haste_fhir_model::r4::generated::{resources::ClientApplication, terminology::IssueType};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_jwt::TenantId;
use haste_repository::{
    Repository,
    admin::TenantModelAdmin,
    types::{
        authorization_code::{
            AuthorizationCodeKind, AuthorizationCodeSearchClaims, CreateAuthorizationCode,
        },
        user::{CreateUser, UpdateUser},
    },
};
use url::Url;

/// Whether `redirect_url` is one of the client's registered redirect URIs.
///
/// A registered URI is compared as written, except that it may hold one `*`
/// as its whole first host label (`https://*.example.com/callback`), which
/// stands for a single DNS label: the redirect stays on the registered
/// domain, with the registered scheme, port and path.
pub fn is_valid_redirect_url(redirect_url: &str, client: &ClientApplication) -> bool {
    !redirect_url.is_empty()
        && client
            .redirectUri
            .iter()
            .flatten()
            .filter_map(|registered| registered.value.as_deref())
            .any(|registered| matches_registered(registered, redirect_url))
}

/// Whether `redirect_url` is `registered`, with a `*` in it filled in by one
/// host label.
///
/// Only the registered URI is parsed. The redirect URI is redirected to as
/// written, so it is compared as written: however a parser would read it, it
/// is the registered URI with nothing but the label filled in.
fn matches_registered(registered: &str, redirect_url: &str) -> bool {
    let Some((before, after)) = registered.split_once('*') else {
        return registered == redirect_url;
    };
    confines_to_a_domain(registered)
        && before.ends_with("://")
        && redirect_url
            .strip_prefix(before)
            .and_then(|rest| rest.strip_suffix(after))
            .is_some_and(is_host_label)
}

/// Whether a registered wildcard URI names a domain to stay on: its one `*`
/// is the whole first host label, above `localhost` or a dotted domain.
/// Any other wildcard matches nothing.
fn confines_to_a_domain(registered: &str) -> bool {
    let Ok(url) = Url::parse(registered) else {
        return false;
    };
    registered.matches('*').count() == 1
        && url.username().is_empty()
        && url.password().is_none()
        && url.fragment().is_none()
        && url
            .host_str()
            .and_then(|host| host.strip_prefix("*."))
            .is_some_and(|domain| domain == "localhost" || domain.contains('.'))
}

/// One DNS label: nothing in it can end the host or start another part of
/// the URI.
fn is_host_label(label: &str) -> bool {
    !label.is_empty()
        && label.len() <= 63
        && label
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
}

/// Rejects a password that is too easy to guess. The error carries the
/// feedback to show.
pub fn check_password_strength(
    password: &str,
    user_email: &str,
) -> Result<(), OperationOutcomeError> {
    let password_strength = zxcvbn::zxcvbn(password, &[user_email]);

    if u8::from(password_strength.score()) < 3 {
        let feedback = password_strength
            .feedback()
            .map(|f| format!("{}", f))
            .unwrap_or_default();

        return Err(OperationOutcomeError::fatal(
            IssueType::security(),
            feedback,
        ));
    }

    Ok(())
}

pub async fn set_user_password<Repo: Repository>(
    repo: &Repo,
    tenant: &TenantId,
    user_email: &str,
    user_id: &str,
    password: &str,
) -> Result<(), OperationOutcomeError> {
    check_password_strength(password, user_email)?;

    TenantModelAdmin::<CreateUser, _, _, _, String>::update(
        repo,
        tenant,
        UpdateUser {
            id: user_id.to_string(),
            password: Some(password.to_string()),
            email: None,
            role: None,
            method: None,
            provider_id: None,
            email_verified: None,
        },
    )
    .await?;

    Ok(())
}

/// Deletes every refresh token issued to `user_id` in `tenant`, in every
/// project.
///
/// A password reset is how a compromised account is recovered, and a refresh
/// token issued before it would otherwise let whoever holds it keep minting
/// access tokens for up to twelve hours after the password changed.
pub async fn revoke_refresh_tokens<Repo: Repository>(
    repo: &Repo,
    tenant: &TenantId,
    user_id: &str,
) -> Result<(), OperationOutcomeError> {
    let refresh_tokens = TenantModelAdmin::<CreateAuthorizationCode, _, _, _, _>::search(
        repo,
        tenant,
        &AuthorizationCodeSearchClaims {
            client_id: None,
            code: None,
            kind: Some(AuthorizationCodeKind::RefreshToken),
            user_id: Some(user_id.to_string()),
            user_agent: None,
            is_expired: None,
        },
    )
    .await?;

    for refresh_token in refresh_tokens {
        TenantModelAdmin::<CreateAuthorizationCode, _, _, _, _>::delete(
            repo,
            tenant,
            &refresh_token.code,
        )
        .await?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use haste_fhir_model::r4::generated::types::FHIRString;

    fn client(redirect_uris: &[&str]) -> ClientApplication {
        ClientApplication {
            redirectUri: Some(
                redirect_uris
                    .iter()
                    .map(|uri| FHIRString {
                        value: Some((*uri).to_string()),
                        ..Default::default()
                    })
                    .collect(),
            ),
            ..Default::default()
        }
    }

    fn allowed(registered: &str, redirect_url: &str) -> bool {
        is_valid_redirect_url(redirect_url, &client(&[registered]))
    }

    #[test]
    fn an_exact_uri_matches_only_itself() {
        let registered = "https://app.example.com/callback";
        assert!(allowed(registered, registered));
        for redirect_url in [
            "",
            "https://app.example.com/callback/",
            "https://app.example.com/callback?next=1",
            "https://app.example.com/other",
            "https://other.example.com/callback",
            "http://app.example.com/callback",
        ] {
            assert!(!allowed(registered, redirect_url), "{redirect_url}");
        }
    }

    #[test]
    fn any_registered_uri_may_match() {
        let client = client(&[
            "https://app.example.com/callback",
            "http://*.localhost:3001",
        ]);
        assert!(is_valid_redirect_url(
            "https://app.example.com/callback",
            &client
        ));
        assert!(is_valid_redirect_url("http://dev.localhost:3001", &client));
        assert!(!is_valid_redirect_url("http://dev.localhost:3002", &client));
    }

    #[test]
    fn a_wildcard_stands_for_one_host_label() {
        for (registered, redirect_url) in [
            (
                "http://*.localhost:3001",
                "http://my-health--system.localhost:3001",
            ),
            (
                "https://*.haste.health",
                "https://acme--system.haste.health",
            ),
            (
                "https://*.example.com/callback",
                "https://app.example.com/callback",
            ),
        ] {
            assert!(allowed(registered, redirect_url), "{redirect_url}");
        }
    }

    #[test]
    fn a_wildcard_does_not_reach_past_its_label() {
        for redirect_url in [
            "http://attacker.example/cb.localhost:3001",
            "http://attacker.example#.localhost:3001",
            "http://attacker.example?.localhost:3001",
            "http://attacker.example\\.localhost:3001",
            "http://attacker.example%2f.localhost:3001",
            "http://user@evil.localhost:3001",
            "http://user:pw@evil.localhost:3001",
            "http://a.b.localhost:3001",
            "http://.localhost:3001",
            "http://localhost:3001",
            "http://*.localhost:3001",
        ] {
            assert!(
                !allowed("http://*.localhost:3001", redirect_url),
                "{redirect_url}"
            );
        }
    }

    #[test]
    fn the_rest_of_a_wildcard_uri_is_compared_as_written() {
        for (registered, redirect_url) in [
            ("https://*.haste.health", "http://acme.haste.health"),
            ("https://*.haste.health", "https://acme.haste.health:8443"),
            ("https://*.haste.health", "https://acme.haste.health/cb"),
            ("https://*.haste.health", "https://acme.haste.health?x=1"),
            ("https://*.haste.health", "https://acme.haste.health#x"),
            (
                "https://*.haste.health",
                "https://acme.haste.health.attacker.example",
            ),
            ("https://*.haste.health", "https://acme.xhaste.health"),
            // Spellings a URL parser would read as a match.
            ("https://*.haste.health", "https://acme.haste.health/"),
            ("https://*.haste.health", "https://acme.haste.health:443"),
            ("https://*.haste.health", "https://acme.Haste.Health"),
            ("https://*.haste.health", "https:acme.haste.health"),
            ("https://*.haste.health", "https:\\\\acme.haste.health"),
            ("https://*.haste.health", "https://acme\u{ff0e}haste.health"),
            ("https://*.haste.health", "https://ac\tme.haste.health"),
            (
                "https://*.example.com/callback",
                "https://app.example.com/x/../callback",
            ),
            ("http://*.localhost:3001", "http://dev.localhost:3002"),
            ("http://*.localhost:3001", "http://dev.localhost"),
        ] {
            assert!(!allowed(registered, redirect_url), "{redirect_url}");
        }
    }

    #[test]
    fn a_wildcard_anywhere_else_matches_nothing() {
        for registered in [
            "https://*",
            "https://*.com",
            "https://a.*.example.com",
            "https://pr-*.example.com",
            "https://*.*.example.com",
            "https://example.com/*",
            "https://*.example.com/*",
            "https://*.example.com:*",
            "https://*@example.com",
            "https://*.example.com#*",
            "*://app.example.com",
            "*",
        ] {
            for redirect_url in [registered.replace('*', "app"), registered.to_string()] {
                assert!(!allowed(registered, &redirect_url), "{registered}");
            }
        }
    }
}
