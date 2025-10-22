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

        let body = serde_json::from_str::<token_body::OAuth2TokenBody>(
            r#"
            {
               "grant_type": "refresh_token",
               "refresh_token": "hello",
               "scope" : "read write",
               "client_id": "test_client",
               "client_secret": "test_secret",
               "codee": "should not be here"
            }
            "#,
        );

        assert!(!body.is_ok());

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

        let body = serde_json::from_str::<token_body::OAuth2TokenBody>(
            r#"
            {
                "grant_type": "authorization_code",
                "code": "code",
                "redirect_uri": "redirect_uri",
                "code_verifier": "code_verifier",
                "client_id": "client_id",
                "refresh_tokene": "should not be here"
            }
            "#,
        );

        assert!(!body.is_ok());
    }
}
