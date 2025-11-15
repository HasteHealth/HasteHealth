use serde::{Deserialize, Serialize};

use haste_jwt::scopes::Scopes;

#[derive(Debug, Clone)]
pub struct ClientId(String);
impl From<ClientId> for String {
    fn from(client_id: ClientId) -> Self {
        client_id.0
    }
}
impl ClientId {
    pub fn new(id: String) -> Self {
        ClientId(id)
    }
}
impl AsRef<str> for ClientId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone)]
pub struct UserId(String);
impl From<UserId> for String {
    fn from(user_id: UserId) -> Self {
        user_id.0
    }
}
impl UserId {
    pub fn new(id: String) -> Self {
        UserId(id)
    }
}
impl AsRef<str> for UserId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[derive(sqlx::FromRow, Debug, Deserialize, Serialize)]
pub struct Scope {
    pub client: String,
    pub user_: String,
    pub scope: Scopes,
}

pub struct UpdateScope {
    pub client: ClientId,
    pub user_: UserId,
    pub scope: Scopes,
}

pub struct ScopeSearchClaims {
    pub user_: Option<UserId>,
    pub client: Option<ClientId>,
}

pub struct CreateScope {
    pub client: ClientId,
    pub user_: UserId,
    pub scope: Scopes,
}

pub struct ScopeKey(pub ClientId, pub UserId);
impl ScopeKey {
    pub fn new(client: ClientId, user_: UserId) -> Self {
        ScopeKey(client, user_)
    }
}
