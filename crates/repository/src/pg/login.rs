use crate::{
    AuthMethod, TenantId, UserRole,
    auth::{Login, LoginMethod},
    pg::{PGConnection, StoreError},
};
use oxidized_fhir_operation_error::OperationOutcomeError;
use sqlx::{Acquire, Postgres};

// switch (type) {
//       case "email-password": {
//         const where: s.users.Whereable = {
//           tenant,
//           method: "email-password",
//           email: parameters.email,
//           password: db.sql`${db.self} = crypt(${db.param((parameters as LoginParameters["email-password"]).password)}, ${db.self})`,
//         };

//         const usersFound: User[] = await db
//           .select("users", where, { columns: USER_QUERY_COLS })
//           .run(this._pgClient);

//         // Sanity check should never happen given unique check on email.
//         if (usersFound.length > 1)
//           throw new Error(
//             "Multiple users found with the same email and password",
//           );

//         const user = usersFound[0];

//         if (user?.email_verified === false) {
//           return { type: "failed", errors: ["email-not-verified"] };
//         }
//         if (!user) {
//           return { type: "failed", errors: ["invalid-credentials"] };
//         }
//         return { type: "successful", user: user };
//       }
//       default: {
//         throw new Error("Invalid login method.");
//       }
//     }

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
                    crate::auth::User,
                    r#"
                  SELECT fhir_user_id, email, role as "role: UserRole" FROM users WHERE tenant = $1 AND method = $2 AND email = $3 AND password = crypt($4, password)
                "#,
                    tenant.as_ref(),
                    AuthMethod::EmailPassword as AuthMethod,
                    email,
                    password
                ).fetch_one(&mut *conn).await.map_err(StoreError::from)?;

                Ok(crate::auth::LoginResult::Success { user })
            }
            LoginMethod::OIDC { email, provider_id } => {
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
            PGConnection::PgConnection(conn) => {
                let mut conn = conn.lock().await;
                let res = login(&mut *conn, tenant, method).await?;
                Ok(res)
            }
        }
    }
}
