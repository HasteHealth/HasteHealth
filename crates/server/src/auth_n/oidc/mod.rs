pub mod routes;
pub mod schemas;

pub mod hardcoded_clients;

fn tester() {
    let k = schemas::token_instrospection::OAuth2TokenIntrospectionBody {
        token: "TEST".to_string(),
    };
    schemas::token_body::OAuth2TokenBody::RefreshToken {
        client_id: Some("TEST".to_string()),
        client_secret: Some("TEST".to_string()),
        refresh_token: "TEST".to_string(),
        scope: None,
    };
}
