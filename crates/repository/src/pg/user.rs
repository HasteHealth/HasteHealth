use crate::{
    admin::{Login, TenantAuthAdmin},
    pg::{PGConnection, StoreError},
    types::{
        TenantId,
        user::{
            AuthMethod, CreateUser, LoginMethod, LoginResult, UpdateUser, User, UserRole,
            UserSearchClauses,
        },
    },
    utilities::generate_id,
};
use oxidized_fhir_operation_error::{OperationOutcomeError, derive::OperationOutcomeError};
use sqlx::{Acquire, Postgres, QueryBuilder};

#[derive(OperationOutcomeError, Debug)]
enum LoginError {
    #[error(code = "login", diagnostic = "Invalid credentials for user.")]
    InvalidCredentials,
}

fn login<'a, 'c, Connection: Acquire<'c, Database = Postgres> + Send + 'a>(
    connection: Connection,
    tenant: &'a TenantId,
    method: &'a LoginMethod,
) -> impl Future<Output = Result<LoginResult, OperationOutcomeError>> + Send + 'a {
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
                    Ok(LoginResult::Success { user })
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

impl Login for PGConnection {
    async fn login(
        &self,
        tenant: &TenantId,
        method: &LoginMethod,
    ) -> Result<LoginResult, oxidized_fhir_operation_error::OperationOutcomeError> {
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
            generate_id(None) as String,
            new_user.provider_id,
            new_user.email,
            new_user.role as UserRole,
            new_user.method as AuthMethod,
        ).fetch_one(&mut *conn).await.map_err(StoreError::SQLXError)?;

        Ok(user)
    }
}

fn read_user<'a, 'c, Connection: Acquire<'c, Database = Postgres> + Send + 'a>(
    connection: Connection,
    tenant: &'a TenantId,
    id: &'a str,
) -> impl Future<Output = Result<User, OperationOutcomeError>> + Send + 'a {
    async move {
        let mut conn = connection.acquire().await.map_err(StoreError::SQLXError)?;
        let user = sqlx::query_as!(
            User,
            r#"
                SELECT id, provider_id, email, role as "role: UserRole", method as "method: AuthMethod"
                FROM users
                WHERE tenant = $1 AND id = $2
            "#,
            tenant.as_ref(),
            id
        ).fetch_one(&mut *conn).await.map_err(StoreError::SQLXError)?;

        Ok(user)
    }
}

fn update_user<'a, 'c, Connection: Acquire<'c, Database = Postgres> + Send + 'a>(
    connection: Connection,
    tenant: &'a TenantId,
    model: UpdateUser,
) -> impl Future<Output = Result<User, OperationOutcomeError>> + Send + 'a {
    async move {
        let mut conn = connection.acquire().await.map_err(StoreError::SQLXError)?;
        let mut query_builder = QueryBuilder::new(
            r#"
                UPDATE users
                SET provider_id = $1, email = $2, role = $3, method = $4
             
            "#,
        );

        query_builder.push(" WHERE tenant = $5 AND id = $6 ");

        let user = sqlx::query_as!(
            User,
            r#"
                UPDATE users
                SET provider_id = $1, email = $2, role = $3, method = $4
                WHERE tenant = $5 AND id = $6
                RETURNING id, provider_id, email, role as "role: UserRole", method as "method: AuthMethod"
            "#,
            model.provider_id,
            model.email,
            model.role as UserRole,
            model.method as AuthMethod,
            tenant.as_ref(),
            model.id
        ).fetch_one(&mut *conn).await.map_err(StoreError::SQLXError)?;

        Ok(user)
    }
}

fn delete_user<'a, 'c, Connection: Acquire<'c, Database = Postgres> + Send + 'a>(
    connection: Connection,
    tenant: &'a TenantId,
    id: &'a str,
) -> impl Future<Output = Result<User, OperationOutcomeError>> + Send + 'a {
    async move {
        let mut conn = connection.acquire().await.map_err(StoreError::SQLXError)?;
        let user = sqlx::query_as!(
            User,
            r#"
                DELETE FROM users
                WHERE tenant = $1 AND id = $2
                RETURNING id, provider_id, email, role as "role: UserRole", method as "method: AuthMethod"
            "#,
            tenant.as_ref(),
            id
        ).fetch_one(&mut *conn).await.map_err(StoreError::SQLXError)?;

        Ok(user)
    }
}

fn search_user<'a, 'c, Connection: Acquire<'c, Database = Postgres> + Send + 'a>(
    connection: Connection,
    tenant: &'a TenantId,
    clauses: &'a UserSearchClauses,
) -> impl Future<Output = Result<Vec<User>, OperationOutcomeError>> + Send + 'a {
    async move {
        let mut conn = connection.acquire().await.map_err(StoreError::SQLXError)?;
        let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new(
            r#"SELECT id, email, role as "role: UserRole", method as "method: AuthMethod", provider_id FROM users WHERE tenant = "#,
        );

        query_builder.push_bind(tenant.as_ref());

        if let Some(email) = clauses.email.as_ref() {
            query_builder.push(" email = ").push_bind(email);
        }

        if let Some(role) = clauses.role.as_ref() {
            if !query_builder.sql().ends_with("WHERE") {
                query_builder.push(" AND");
            }
            query_builder.push(" role = ").push_bind(role);
        }

        let query = query_builder.build_query_as();

        let users: Vec<User> = query
            .fetch_all(&mut *conn)
            .await
            .map_err(StoreError::from)?;

        Ok(users)
    }
}

impl TenantAuthAdmin<CreateUser, User, UserSearchClauses, UpdateUser> for PGConnection {
    async fn create(
        &self,
        tenant: &TenantId,
        new_user: CreateUser,
    ) -> Result<User, OperationOutcomeError> {
        match self {
            PGConnection::PgPool(pool) => {
                let res = create_user(pool, tenant, new_user).await?;
                Ok(res)
            }
            PGConnection::PgTransaction(tx) => {
                let mut tx = tx.lock().await;
                let res = create_user(&mut *tx, tenant, new_user).await?;
                Ok(res)
            }
        }
    }

    async fn read(&self, tenant: &TenantId, id: &str) -> Result<User, OperationOutcomeError> {
        match self {
            PGConnection::PgPool(pool) => {
                let res = read_user(pool, tenant, id).await?;
                Ok(res)
            }
            PGConnection::PgTransaction(tx) => {
                let mut tx = tx.lock().await;
                let res = read_user(&mut *tx, tenant, id).await?;
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
            PGConnection::PgPool(pool) => {
                let res = update_user(pool, &tenant, user).await?;
                Ok(res)
            }
            PGConnection::PgTransaction(tx) => {
                let mut tx = tx.lock().await;
                let res = update_user(&mut *tx, &tenant, user).await?;
                Ok(res)
            }
        }
    }

    async fn delete(&self, tenant: &TenantId, id: &str) -> Result<User, OperationOutcomeError> {
        match self {
            PGConnection::PgPool(pool) => {
                let res = delete_user(pool, tenant, id).await?;
                Ok(res)
            }
            PGConnection::PgTransaction(tx) => {
                let mut tx = tx.lock().await;
                let res = delete_user(&mut *tx, tenant, id).await?;
                Ok(res)
            }
        }
    }

    async fn search(
        &self,
        tenant: &TenantId,
        clauses: &UserSearchClauses,
    ) -> Result<Vec<User>, OperationOutcomeError> {
        match self {
            PGConnection::PgPool(pool) => {
                let res = search_user(pool, tenant, clauses).await?;
                Ok(res)
            }
            PGConnection::PgTransaction(tx) => {
                let mut tx = tx.lock().await;
                let res = search_user(&mut *tx, tenant, clauses).await?;
                Ok(res)
            }
        }
    }
}
