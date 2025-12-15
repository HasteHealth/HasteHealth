use crate::{
    admin::SystemAdmin,
    pg::{PGConnection, StoreError},
    types::user::{User, UserSearchClauses},
};
use haste_fhir_operation_error::OperationOutcomeError;
use sqlx::{Acquire, Postgres, QueryBuilder};

fn search_system_user<'a, 'c, Connection: Acquire<'c, Database = Postgres> + Send + 'a>(
    connection: Connection,
    clauses: &'a UserSearchClauses,
) -> impl Future<Output = Result<Vec<User>, OperationOutcomeError>> + Send + 'a {
    async move {
        let mut conn = connection.acquire().await.map_err(StoreError::SQLXError)?;
        let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new(
            r#"SELECT id, tenant, email, role, method, provider_id FROM users WHERE  "#,
        );

        let mut seperator = query_builder.separated(" AND ");

        if let Some(email) = clauses.email.as_ref() {
            seperator.push(" email = ").push_bind_unseparated(email);
        }

        if let Some(role) = clauses.role.as_ref() {
            seperator.push(" role = ").push_bind_unseparated(role);
        }

        if let Some(method) = clauses.method.as_ref() {
            seperator.push(" method = ").push_bind_unseparated(method);
        }

        let query = query_builder.build_query_as();

        let users: Vec<User> = query
            .fetch_all(&mut *conn)
            .await
            .map_err(StoreError::from)?;

        Ok(users)
    }
}

impl SystemAdmin<User, UserSearchClauses> for PGConnection {
    async fn search(
        &self,
        clauses: &UserSearchClauses,
    ) -> Result<Vec<User>, OperationOutcomeError> {
        match self {
            PGConnection::Pool(pool, _) => {
                let res = search_system_user(pool, clauses).await?;
                Ok(res)
            }
            PGConnection::Transaction(tx, _) => {
                let mut tx = tx.lock().await;
                let res = search_system_user(&mut *tx, clauses).await?;
                Ok(res)
            }
        }
    }
}
