use crate::{TenantId, auth::Login, pg::PGConnection};
use oxidized_fhir_operation_error::OperationOutcomeError;
use sqlx::{Acquire, Postgres};

fn login<'a, 'c, Connection: Acquire<'c, Database = Postgres> + Send + 'a>(
    connection: Connection,
    tenant: &'a TenantId,
    method: &crate::auth::LoginMethod,
) -> impl Future<Output = Result<crate::auth::LoginResult, OperationOutcomeError>> + Send + 'a {
    async move { todo!() }
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
            PGConnection::PgConnection(conn) => {
                let mut conn = conn.lock().await;
                let res = login(&mut *conn, tenant, method).await?;
                Ok(res)
            }
        }
    }
}
