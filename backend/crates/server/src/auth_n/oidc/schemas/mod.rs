pub mod token_body {
    typify::import_types!(schema = "./src/auth_n/oidc/schemas/oauth2_token_body.schema.json");
}

pub mod token_instrospection {
    typify::import_types!(
        schema = "./src/auth_n/oidc/schemas/oauth2_token_introspection.schema.json"
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_body() {
        let body = serde_json::from_str::<token_body::OAuth2TokenBody>(
            r#"
            {
               "grant_type": "refresh_token",
               "refresh_token": "hello",
               "scope" : "read write",
               "client_id": "test_client",
               "client_secret": "test_secret"
            }
            "#,
        );

        assert!(body.is_ok());

        let missing_client_id = serde_json::from_str::<token_body::OAuth2TokenBody>(
            r#"
            {
               "grant_type": "refresh_token",
               "refresh_token": "hello",
               "scope" : "read write",
               "client_secret": "test_secret"
            }
            "#,
        );

        assert!(!missing_client_id.is_ok());

        let body = serde_json::from_str::<token_body::OAuth2TokenBody>(
            r#"
            {
                "grant_type": "authorization_code",
                "code": "code",
                "redirect_uri": "redirect_uri",
                "code_verifier": "code_verifier",
                "client_id": "client_id"
            }
            "#,
        );

        assert!(body.is_ok());

        let missing_grant_type = serde_json::from_str::<token_body::OAuth2TokenBody>(
            r#"
            {
                "code": "code",
                "redirect_uri": "redirect_uri",
                "code_verifier": "code_verifier",
                "client_id": "client_id"
            }
            "#,
        );

        assert!(!missing_grant_type.is_ok());
    }
}
