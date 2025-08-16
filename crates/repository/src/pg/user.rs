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
use sqlx::{Acquire, Execute, Postgres, QueryBuilder};

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

        let mut query_builder = QueryBuilder::new(
            r#"
                INSERT INTO users(tenant, id, email, role, method, provider_id, password)
            "#,
        );

        query_builder.push(" VALUES (");

        let mut seperator = query_builder.separated(", ");

        seperator
            .push_bind(tenant.as_ref())
            .push_bind(generate_id(None))
            .push_bind(new_user.email)
            .push_bind(new_user.role as UserRole)
            .push_bind(new_user.method as AuthMethod);

        if let Some(provider_id) = new_user.provider_id {
            seperator.push_bind(provider_id);
        } else {
            seperator.push_bind(None::<String>);
        }

        if let Some(password) = new_user.password {
            seperator
                .push("crypt(")
                .push_bind_unseparated(password)
                .push_unseparated(", gen_salt('bf'))");
        } else {
            seperator.push_bind(None::<String>);
        }

        query_builder.push(r#") RETURNING id, provider_id, email, role , method"#);

        let query = query_builder.build_query_as();

        let user = query
            .fetch_one(&mut *conn)
            .await
            .map_err(StoreError::SQLXError)?;

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
                UPDATE users SET 
            "#,
        );

        let mut seperator = query_builder.separated(", ");

        if let Some(provider_id) = model.provider_id {
            seperator
                .push(" provider_id = ")
                .push_bind_unseparated(provider_id);
        }

        seperator
            .push(" tenant = ")
            .push_bind_unseparated(tenant.as_ref())
            .push(" email = ")
            .push_bind_unseparated(model.email)
            .push(" role = ")
            .push_bind_unseparated(model.role)
            .push(" method = ")
            .push_bind_unseparated(model.method);

        if let Some(password) = model.password {
            seperator
                .push(" password = crypt(")
                .push_bind_unseparated(password)
                .push_unseparated(", gen_salt('bf'))");
        }

        query_builder.push(r#" RETURNING id, provider_id, email, role, method"#);

        let query = query_builder.build_query_as();

        let user = query
            .fetch_one(&mut *conn)
            .await
            .map_err(StoreError::SQLXError)?;

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
            r#"SELECT id, email, role as "role: UserRole", method as "method: AuthMethod", provider_id FROM users WHERE  "#,
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
