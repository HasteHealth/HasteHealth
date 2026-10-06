use crate::{
    admin::{Login, TenantModelAdmin},
    pg::{PGConnection, StoreError},
    types::user::{
        AuthMethod, CreateUser, LoginMethod, LoginResult, UpdateUser, User, UserRole,
        UserSearchClauses,
    },
};
use argon2::{
    Argon2, PasswordHasher, PasswordVerifier,
    password_hash::{PasswordHash, SaltString, rand_core::OsRng},
};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_jwt::TenantId;
use sqlx::{PgExecutor, QueryBuilder};
use std::sync::LazyLock;

/// A hash to verify against when there is no user, or the user has no
/// password. Verifying costs the same as a real check, so a login attempt
/// takes as long whether or not the address has an account: without this an
/// unknown address answered in a few milliseconds and a known one only after
/// the Argon2 work, which told a caller which addresses exist.
static NO_PASSWORD_HASH: LazyLock<String> = LazyLock::new(|| {
    hash_password("no such user").expect("hashing a constant password cannot fail")
});

fn hash_password(password: &str) -> Result<String, StoreError> {
    let salt = SaltString::generate(&mut OsRng);
    let hash = Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map_err(StoreError::PasswordHashError)?
        .to_string();

    Ok(hash)
}

async fn login<'a, 'e, E>(
    executor: E,
    tenant: &'a TenantId,
    method: &'a LoginMethod,
) -> Result<LoginResult, OperationOutcomeError>
where
    E: PgExecutor<'e>,
{
    match method {
        LoginMethod::EmailPassword { email, password } => {
            let row = sqlx::query_as::<
                _,
                (
                    String,
                    TenantId,
                    Option<String>,
                    UserRole,
                    AuthMethod,
                    Option<String>,
                    Option<String>,
                    bool,
                ),
            >(
                r"
                    SELECT id, tenant, email, role, method, provider_id, password, email_verified
                    FROM users
                    WHERE tenant = $1 AND method = $2 AND email = $3
                ",
            )
            .bind(tenant.as_ref())
            .bind(AuthMethod::EmailPassword)
            .bind(email)
            .fetch_optional(executor)
            .await
            .map_err(StoreError::from)?;

            // The stored hash when the user exists and has a password, or a
            // stand-in that is verified (and always fails) otherwise, so both
            // outcomes take the same time.
            let (user, password_hash) = match row {
                Some((
                    id,
                    tenant_id,
                    email_val,
                    role,
                    method_val,
                    provider_id,
                    hash,
                    email_verified,
                )) => {
                    let user = User {
                        id,
                        tenant: tenant_id,
                        email: email_val,
                        role,
                        method: method_val,
                        provider_id,
                        email_verified,
                    };
                    match hash {
                        Some(hash) => (Some(user), hash),
                        None => (None, NO_PASSWORD_HASH.clone()),
                    }
                }
                None => (None, NO_PASSWORD_HASH.clone()),
            };

            let password_matches =
                PasswordHash::new(&password_hash)
                    .ok()
                    .is_some_and(|parsed_hash| {
                        Argon2::default()
                            .verify_password(password.as_bytes(), &parsed_hash)
                            .is_ok()
                    });

            // An unverified address is refused only after the hash check, so
            // that refusal takes the same time too.
            let Some(user) = user.filter(|user| password_matches && user.email_verified) else {
                return Ok(LoginResult::Failure);
            };

            Ok(LoginResult::Success { user })
        }
        LoginMethod::OIDC {
            email: _,
            provider_id: _,
        } => Ok(LoginResult::Failure),
    }
}

impl Login for PGConnection {
    async fn login(
        &self,
        tenant: &TenantId,
        method: &LoginMethod,
    ) -> Result<LoginResult, haste_fhir_operation_error::OperationOutcomeError> {
        match &self {
            PGConnection::Pool(pool, _) => {
                let res = login(pool, tenant, method).await?;
                Ok(res)
            }
            PGConnection::Transaction(tx, _, _) => {
                let mut tx = tx.lock().await;

                let res = login(&mut **tx, tenant, method).await?;
                Ok(res)
            }
        }
    }
}

async fn create_user<'a, 'e, E>(
    executor: E,
    tenant: &'a TenantId,
    new_user: CreateUser,
) -> Result<User, OperationOutcomeError>
where
    E: PgExecutor<'e>,
{
    let mut query_builder = QueryBuilder::new(
        r"
            INSERT INTO users(tenant, id, email, role, method, email_verified, provider_id, password)
        ",
    );

    query_builder.push(" VALUES (");

    let mut seperator = query_builder.separated(", ");

    seperator
        .push_bind(tenant.as_ref())
        .push_bind(new_user.id)
        .push_bind(new_user.email)
        .push_bind(new_user.role)
        .push_bind(new_user.method)
        .push_bind(new_user.email_verified);

    if let Some(provider_id) = new_user.provider_id {
        seperator.push_bind(provider_id);
    } else {
        seperator.push_bind(None::<String>);
    }

    if let Some(password) = new_user.password {
        let hashed_password = hash_password(&password)?;
        seperator.push_bind(hashed_password);
    } else {
        seperator.push_bind(None::<String>);
    }

    query_builder.push(r") RETURNING id, tenant, provider_id, email, role, method, email_verified");

    let query = query_builder.build_query_as::<User>();

    let user = query
        .fetch_one(executor)
        .await
        .map_err(StoreError::SQLXError)?;

    Ok(user)
}

async fn read_user<'a, 'e, E>(
    executor: E,
    tenant: &'a TenantId,
    id: &'a str,
) -> Result<Option<User>, OperationOutcomeError>
where
    E: PgExecutor<'e>,
{
    let user = sqlx::query_as::<_, User>(
        r"
            SELECT id, tenant, provider_id, email, role, method, email_verified
            FROM users
            WHERE tenant = $1 AND id = $2
        ",
    )
    .bind(tenant.as_ref())
    .bind(id)
    .fetch_optional(executor)
    .await
    .map_err(StoreError::SQLXError)?;

    Ok(user)
}

async fn update_user<'a, 'e, E>(
    executor: E,
    tenant: &'a TenantId,
    model: UpdateUser,
) -> Result<User, OperationOutcomeError>
where
    E: PgExecutor<'e>,
{
    let mut query_builder = QueryBuilder::new(
        r"
            UPDATE users SET
        ",
    );

    let mut update_clauses = query_builder.separated(", ");

    if let Some(provider_id) = model.provider_id {
        update_clauses
            .push(" provider_id = ")
            .push_bind_unseparated(provider_id);
    }

    if let Some(email) = model.email.as_ref() {
        update_clauses
            .push(" email = ")
            .push_bind_unseparated(email);
    }

    if let Some(role) = model.role.as_ref() {
        update_clauses.push(" role = ").push_bind_unseparated(role);
    }

    if let Some(method) = model.method.as_ref() {
        update_clauses
            .push(" method = ")
            .push_bind_unseparated(method);
    }

    if let Some(password) = model.password {
        let hashed_password = hash_password(&password)?;
        update_clauses
            .push(" password = ")
            .push_bind_unseparated(hashed_password);
    }

    if let Some(email_verified) = model.email_verified {
        update_clauses
            .push(" email_verified = ")
            .push_bind_unseparated(email_verified);
    }

    update_clauses
        .push(" tenant = ")
        .push_bind_unseparated(tenant.as_ref());

    query_builder.push(" WHERE id = ");
    query_builder.push_bind(model.id);

    query_builder.push(r" RETURNING id, tenant, provider_id, email, role, method, email_verified");

    let query = query_builder.build_query_as::<User>();

    let user = query
        .fetch_one(executor)
        .await
        .map_err(StoreError::SQLXError)?;

    Ok(user)
}

async fn delete_user<'a, 'e, E>(
    executor: E,
    tenant: &'a TenantId,
    id: &'a str,
) -> Result<(), OperationOutcomeError>
where
    E: PgExecutor<'e>,
{
    sqlx::query(
        r"
            DELETE FROM users
            WHERE tenant = $1 AND id = $2
        ",
    )
    .bind(tenant.as_ref())
    .bind(id)
    .execute(executor)
    .await
    .map_err(StoreError::SQLXError)?;

    Ok(())
}

async fn search_user<'a, 'e, E>(
    executor: E,
    tenant: &'a TenantId,
    clauses: &'a UserSearchClauses,
) -> Result<Vec<User>, OperationOutcomeError>
where
    E: PgExecutor<'e>,
{
    let mut query_builder: QueryBuilder<sqlx::Postgres> = QueryBuilder::new(
        r"SELECT id, tenant, email, role, method, provider_id, email_verified FROM users WHERE ",
    );

    let mut seperator = query_builder.separated(" AND ");
    seperator
        .push(" tenant = ")
        .push_bind_unseparated(tenant.as_ref());

    if let Some(email) = clauses.email.as_ref() {
        seperator.push(" email = ").push_bind_unseparated(email);
    }

    if let Some(role) = clauses.role.as_ref() {
        seperator.push(" role = ").push_bind_unseparated(role);
    }

    if let Some(method) = clauses.method.as_ref() {
        seperator.push(" method = ").push_bind_unseparated(method);
    }

    let query = query_builder.build_query_as::<User>();

    let users: Vec<User> = query.fetch_all(executor).await.map_err(StoreError::from)?;

    Ok(users)
}

impl<Key: AsRef<str> + Send + Sync>
    TenantModelAdmin<CreateUser, User, UserSearchClauses, UpdateUser, Key> for PGConnection
{
    async fn create(
        &self,
        tenant: &TenantId,
        new_user: CreateUser,
    ) -> Result<User, OperationOutcomeError> {
        match self {
            PGConnection::Pool(pool, _) => {
                let res = create_user(pool, tenant, new_user).await?;
                Ok(res)
            }
            PGConnection::Transaction(tx, _, _) => {
                let mut tx = tx.lock().await;
                let res = create_user(&mut **tx, tenant, new_user).await?;
                Ok(res)
            }
        }
    }

    async fn read(
        &self,
        tenant: &TenantId,
        id: &Key,
    ) -> Result<Option<User>, OperationOutcomeError> {
        match self {
            PGConnection::Pool(pool, _) => {
                let res = read_user(pool, tenant, id.as_ref()).await?;
                Ok(res)
            }
            PGConnection::Transaction(tx, _, _) => {
                let mut tx = tx.lock().await;
                let res = read_user(&mut **tx, tenant, id.as_ref()).await?;
                Ok(res)
            }
        }
    }

    async fn update(
        &self,
        tenant: &TenantId,
        user: UpdateUser,
    ) -> Result<User, OperationOutcomeError> {
        match self {
            PGConnection::Pool(pool, _) => update_user(pool, tenant, user).await,
            PGConnection::Transaction(tx, _, _) => {
                let mut tx = tx.lock().await;
                update_user(&mut **tx, tenant, user).await
            }
        }
    }

    async fn delete(&self, tenant: &TenantId, id: &Key) -> Result<(), OperationOutcomeError> {
        match self {
            PGConnection::Pool(pool, _) => delete_user(pool, tenant, id.as_ref()).await,
            PGConnection::Transaction(tx, _, _) => {
                let mut tx = tx.lock().await;
                delete_user(&mut **tx, tenant, id.as_ref()).await
            }
        }
    }

    async fn search(
        &self,
        tenant: &TenantId,
        clauses: &UserSearchClauses,
    ) -> Result<Vec<User>, OperationOutcomeError> {
        match self {
            PGConnection::Pool(pool, _) => search_user(pool, tenant, clauses).await,
            PGConnection::Transaction(tx, _, _) => {
                let mut tx = tx.lock().await;
                search_user(&mut **tx, tenant, clauses).await
            }
        }
    }
}
