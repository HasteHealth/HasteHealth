use serde::{Deserialize, Serialize};

#[derive(sqlx::FromRow, Debug, Deserialize, Serialize)]
pub struct Scope {
    pub client: String,
    pub user_: String,
    pub scope: String,
}

pub struct UpdateScope {
    pub client: String,
    pub user_: String,
    pub scope: String,
}

pub struct ScopeSearchClaims {
    pub user_: Option<String>,
    pub client: Option<String>,
}

pub struct CreateScope {
    pub client: String,
    pub user_: String,
    pub scope: String,
}
