use crate::{
    AuthMethod, TenantId, UserRole,
    auth::{Login, LoginMethod, TenantAuthAdmin, User},
    pg::{PGConnection, StoreError},
    utilities::generate_id,
};
use oxidized_fhir_operation_error::{OperationOutcomeError, derive::OperationOutcomeError};
use sqlx::{Acquire, Postgres};

#[derive(OperationOutcomeError, Debug)]
enum LoginError {
    #[error(code = "login", diagnostic = "Invalid credentials for user.")]
    InvalidCredentials,
}

fn login<'a, 'c, Connection: Acquire<'c, Database = Postgres> + Send + 'a>(
    connection: Connection,
    tenant: &'a TenantId,
    method: &'a crate::auth::LoginMethod,
) -> impl Future<Output = Result<crate::auth::LoginResult, OperationOutcomeError>> + Send + 'a {
    async move {
        let mut conn = connection.acquire().await.map_err(StoreError::SQLXError)?;
        match method {
            LoginMethod::EmailPassword { email, password } => {
                let user = sqlx::query_as!(
                    User,
                    r#"
                  SELECT id, email, role as "role: UserRole", method as "method: AuthMethod", provider_id FROM users WHERE tenant = $1 AND method = $2 AND email = $3 AND password = crypt($4, password)
                "#,
                    tenant.as_ref(),
                    AuthMethod::EmailPassword as AuthMethod,
                    email,
                    password
                ).fetch_optional(&mut *conn).await.map_err(StoreError::from)?;

                if let Some(user) = user {
                    Ok(crate::auth::LoginResult::Success { user })
                } else {
                    Err(LoginError::InvalidCredentials.into())
                }
            }
            LoginMethod::OIDC {
                email: _,
                provider_id: _,
            } => {
                todo!();
            }
        }
    }
}

impl<CTX: Send> Login<CTX> for PGConnection {
    async fn login(
        &self,
        _ctx: CTX,
        tenant: &TenantId,
        method: &crate::auth::LoginMethod,
    ) -> Result<crate::auth::LoginResult, oxidized_fhir_operation_error::OperationOutcomeError>
    {
        match &self {
            PGConnection::PgPool(pool) => {
                let res = login(pool, tenant, method).await?;
                Ok(res)
            }
            PGConnection::PgTransaction(tx) => {
                let mut tx = tx.lock().await;

                let res = login(&mut *tx, tenant, method).await?;
                Ok(res)
            }
        }
    }
}

pub struct UserSearchClauses {
    email: Option<String>,
}

pub struct CreateUser {
    email: String,
    role: UserRole,
    provider_id: String,
    method: AuthMethod,
}

fn create_user<'a, 'c, Connection: Acquire<'c, Database = Postgres> + Send + 'a>(
    connection: Connection,
    tenant: &'a TenantId,
    new_user: CreateUser,
) -> impl Future<Output = Result<User, OperationOutcomeError>> + Send + 'a {
    async move {
        let mut conn = connection.acquire().await.map_err(StoreError::SQLXError)?;
        let user = sqlx::query_as!(
            User,
            r#"
               INSERT INTO users(tenant, id, provider_id, email, role, method)
               VALUES($1, $2, $3, $4, $5, $6)
               RETURNING id, provider_id, email, role as "role: UserRole", method as "method: AuthMethod"
            "#,
            tenant.as_ref(),
            generate_id() as String,
            new_user.provider_id,
            new_user.email,
            new_user.role as UserRole,
            new_user.method as AuthMethod,
        ).fetch_one(&mut *conn).await.map_err(StoreError::SQLXError)?;

        Ok(user)
    }
}

impl<CTX: Send> TenantAuthAdmin<CTX, CreateUser, User, UserSearchClauses> for PGConnection {
    async fn create(
        &self,
        _ctx: CTX,
        tenant: TenantId,
        new_user: CreateUser,
    ) -> Result<User, OperationOutcomeError> {
        match self {
            PGConnection::PgPool(pool) => {
                let res = create_user(pool, &tenant, new_user).await?;
                Ok(res)
            }
            PGConnection::PgTransaction(tx) => {
                let mut tx = tx.lock().await;
                let res = create_user(&mut *tx, &tenant, new_user).await?;
                Ok(res)
            }
        }
    }

    async fn read(
        &self,
        ctx: CTX,
        tenant: TenantId,
        id: String,
    ) -> Result<User, OperationOutcomeError> {
        todo!()
    }

    async fn update(
        &self,
        ctx: CTX,
        tenant: TenantId,
        model: User,
    ) -> Result<User, OperationOutcomeError> {
        todo!()
    }

    async fn delete(
        &self,
        ctx: CTX,
        tenant: TenantId,
        id: String,
    ) -> Result<(), OperationOutcomeError> {
        todo!()
    }

    async fn search(
        &self,
        ctx: CTX,
        tenant: TenantId,
        clauses: UserSearchClauses,
    ) -> Result<Vec<User>, OperationOutcomeError> {
        todo!()
    }
}
