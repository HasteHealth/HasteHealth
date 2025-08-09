pub mod token_body {
    typify::import_types!(schema = "./src/auth_n/oidc/schemas/oauth2_token_body.schema.json");
}

pub mod token_instrospection {
    typify::import_types!(
        schema = "./src/auth_n/oidc/schemas/oauth2_token_introspection.schema.json"
    );
}
