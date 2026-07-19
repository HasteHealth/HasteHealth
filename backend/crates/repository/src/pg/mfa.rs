use crate::{
    admin::TenantModelAdmin,
    pg::{PGConnection, StoreError},
    types::mfa::{
        MFAKey, UserMFACredential, UserMFACredentialCreate, UserMFACredentialUpdate,
        UserMFASearchClaims,
    },
};
use haste_fhir_model::r4::generated::terminology::IssueType;
use haste_fhir_operation_error::OperationOutcomeError;
use haste_jwt::TenantId;
use sqlx::{Acquire, Postgres, QueryBuilder};

fn create_user_mfa_credential<'a, 'c, Connection: Acquire<'c, Database = Postgres> + Send + 'a>(
    connection: Connection,
    tenant: &'a TenantId,
    new_mfa_credentials: UserMFACredentialCreate,
) -> impl Future<Output = Result<UserMFACredential, OperationOutcomeError>> + Send + 'a {
    async move {
        let mut conn = connection.acquire().await.map_err(StoreError::SQLXError)?;

        let type_: &str = new_mfa_credentials.credential_type.into();
        let totp_algorithm = new_mfa_credentials
            .totp_algorithm
            .unwrap_or("SHA1".to_string());

        let user_mfa_credential = sqlx::query_as!(
            UserMFACredential,
            r#"INSERT INTO user_mfa_credential (tenant, user_id, credential_type, secret_ciphertext, secret_nonce, key_id, totp_algorithm, totp_digits, totp_period, totp_skew) 
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10) 
            RETURNING 
                id::TEXT,
                tenant as "tenant: TenantId",
                user_id,
                credential_type,
                secret_ciphertext,
                secret_nonce,
                key_id,
                totp_algorithm,
                totp_digits,
                totp_period,
                totp_skew,
                created_at,
                is_active
            "#,
            tenant.as_ref(),
            new_mfa_credentials.user_id.as_ref(),
            type_,
            new_mfa_credentials.secret_ciphertext,
            new_mfa_credentials.secret_nonce,
            new_mfa_credentials.key_id,
            totp_algorithm,
            new_mfa_credentials.totp_digits.unwrap_or(6),
            new_mfa_credentials.totp_period.unwrap_or(30),
            new_mfa_credentials.totp_skew.unwrap_or(1),
        )
        .fetch_one(&mut *conn)
        .await
        .map_err(StoreError::SQLXError)?;

        Ok(user_mfa_credential)
    }
}

fn read_user_mfa<'a, 'c, Connection: Acquire<'c, Database = Postgres> + Send + 'a>(
    connection: Connection,
    tenant: &'a TenantId,
    key: &'a MFAKey,
) -> impl Future<Output = Result<Option<UserMFACredential>, OperationOutcomeError>> + Send + 'a {
    async move {
        let mut conn = connection.acquire().await.map_err(StoreError::SQLXError)?;
        let user_mfa = sqlx::query_as!(
            UserMFACredential,
            r#"SELECT 
                id::TEXT,
                tenant as "tenant: TenantId",
                user_id,
                credential_type,
                secret_ciphertext,
                secret_nonce,
                key_id,
                totp_algorithm,
                totp_digits,
                totp_period,
                totp_skew,
                created_at,
                is_active
            FROM user_mfa_credential where tenant = $1 AND id::text = $2 AND user_id = $3"#,
            tenant.as_ref(),
            key.mfa_id().0,
            key.user_id().as_ref()
        )
        .fetch_optional(&mut *conn)
        .await
        .map_err(StoreError::SQLXError)?;

        Ok(user_mfa)
    }
}

fn delete_user_mfa<'a, 'c, Connection: Acquire<'c, Database = Postgres> + Send + 'a>(
    connection: Connection,
    tenant: &'a TenantId,
    key: &'a MFAKey,
) -> impl Future<Output = Result<(), OperationOutcomeError>> + Send + 'a {
    async move {
        let mut conn = connection.acquire().await.map_err(StoreError::SQLXError)?;
        let _delted_user_mfa = sqlx::query_as!(
            UserMFACredential,
            r#"DELETE FROM user_mfa_credential 
            WHERE tenant = $1 AND id::text = $2 AND user_id = $3
            RETURNING 
                id::TEXT,
                tenant as "tenant: TenantId",
                user_id,
                credential_type,
                secret_ciphertext,
                secret_nonce,
                key_id,
                totp_algorithm,
                totp_digits,
                totp_period,
                totp_skew,
                created_at,
                is_active"#,
            tenant.as_ref(),
            key.mfa_id().0,
            key.user_id().as_ref()
        )
        .fetch_optional(&mut *conn)
        .await
        .map_err(|_e| {
            OperationOutcomeError::error(
                IssueType::NOT_FOUND,
                format!(
                    "User MFA credential '{}' not found or is system created and cannot be deleted.",
                    key.mfa_id().0
                ),
            )
        })?;

        if !_delted_user_mfa.is_some() {
            return Err(OperationOutcomeError::error(
                IssueType::NOT_FOUND,
                format!(
                    "User MFA credential '{}' not found or is system created and cannot be deleted.",
                    key.mfa_id().0
                ),
            ));
        }

        Ok(())
    }
}

fn search_user_mfa<'a, 'c, Connection: Acquire<'c, Database = Postgres> + Send + 'a>(
    connection: Connection,
    tenant: &'a TenantId,
    clauses: &'a UserMFASearchClaims,
) -> impl Future<Output = Result<Vec<UserMFACredential>, OperationOutcomeError>> + Send + 'a {
    async move {
        let mut conn = connection.acquire().await.map_err(StoreError::SQLXError)?;
        let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new(
            r#"SELECT 
                id::TEXT,
                tenant,
                user_id,
                credential_type,
                secret_ciphertext,
                secret_nonce,
                key_id,
                totp_algorithm,
                totp_digits,
                totp_period,
                totp_skew,
                created_at,
                is_active FROM user_mfa_credential WHERE "#,
        );

        let mut and_clauses = query_builder.separated(" AND ");

        and_clauses
            .push(" tenant = ")
            .push_bind_unseparated(tenant.as_ref());

        and_clauses
            .push(" user_id = ")
            .push_bind_unseparated(clauses.user_id.as_ref());

        if let Some(is_active) = clauses.is_active {
            and_clauses
                .push(" is_active = ")
                .push_bind_unseparated(is_active);
        }

        let query = query_builder.build_query_as();

        let user_mfas: Vec<UserMFACredential> = query
            .fetch_all(&mut *conn)
            .await
            .map_err(StoreError::from)?;

        Ok(user_mfas)
    }
}

/// Not allowing updates on internal row just reading to confirm it's existance.
fn update_user_mfa<'a, 'c, Connection: Acquire<'c, Database = Postgres> + Send + 'a>(
    connection: Connection,
    tenant: &'a TenantId,
    model: UserMFACredentialUpdate,
) -> impl Future<Output = Result<UserMFACredential, OperationOutcomeError>> + Send + 'a {
    async move {
        let mut conn = connection.acquire().await.map_err(StoreError::SQLXError)?;
        let mut query_builder = QueryBuilder::new(
            r#"
                UPDATE user_mfa_credential SET 
            "#,
        );

        let mut set_statements = query_builder.separated(", ");

        set_statements
            .push(" is_active = ")
            .push_bind_unseparated(model.is_active);

        query_builder.push(" WHERE ");

        let mut where_statements = query_builder.separated(" AND ");
        where_statements
            .push(" tenant = ")
            .push_bind_unseparated(tenant.as_ref())
            .push(" id::text = ")
            .push_bind_unseparated(model.id)
            .push(" user_id = ")
            .push_bind_unseparated(model.user_id.as_ref());

        query_builder.push(
            r#" RETURNING 
                id::TEXT,
                tenant,
                user_id,
                credential_type,
                secret_ciphertext,
                secret_nonce,
                key_id,
                totp_algorithm,
                totp_digits,
                totp_period,
                totp_skew,
                created_at,
                is_active"#,
        );

        let query = query_builder.build_query_as();

        let user_mfa_credentials = query
            .fetch_one(&mut *conn)
            .await
            .map_err(StoreError::SQLXError)?;

        Ok(user_mfa_credentials)
    }
}

impl
    TenantModelAdmin<
        UserMFACredentialCreate,
        UserMFACredential,
        UserMFASearchClaims,
        UserMFACredentialUpdate,
        MFAKey,
    > for PGConnection
{
    async fn create(
        &self,
        tenant: &TenantId,
        new_user_mfa_credential: UserMFACredentialCreate,
    ) -> Result<UserMFACredential, OperationOutcomeError> {
        match self {
            PGConnection::Pool(pool, _) => {
                let res = create_user_mfa_credential(pool, tenant, new_user_mfa_credential).await?;
                Ok(res)
            }
            PGConnection::Transaction(tx, _) => {
                let mut tx = tx.lock().await;
                let res =
                    create_user_mfa_credential(&mut *tx, tenant, new_user_mfa_credential).await?;
                Ok(res)
            }
        }
    }

    async fn read(
        &self,
        tenant: &TenantId,
        id: &MFAKey,
    ) -> Result<Option<UserMFACredential>, haste_fhir_operation_error::OperationOutcomeError> {
        match self {
            PGConnection::Pool(pool, _) => {
                let res = read_user_mfa(pool, tenant, id).await?;
                Ok(res)
            }
            PGConnection::Transaction(tx, _) => {
                let mut tx = tx.lock().await;
                let res = read_user_mfa(&mut *tx, tenant, id).await?;
                Ok(res)
            }
        }
    }

    async fn update(
        &self,
        tenant: &TenantId,
        model: UserMFACredentialUpdate,
    ) -> Result<UserMFACredential, haste_fhir_operation_error::OperationOutcomeError> {
        match self {
            PGConnection::Pool(pool, _) => {
                let res = update_user_mfa(pool, tenant, model).await?;
                Ok(res)
            }
            PGConnection::Transaction(tx, _) => {
                let mut tx = tx.lock().await;
                let res = update_user_mfa(&mut *tx, tenant, model).await?;
                Ok(res)
            }
        }
    }

    async fn delete(
        &self,
        tenant: &TenantId,
        id: &MFAKey,
    ) -> Result<(), haste_fhir_operation_error::OperationOutcomeError> {
        match self {
            PGConnection::Pool(pool, _) => {
                let res = delete_user_mfa(pool, tenant, id).await?;
                Ok(res)
            }
            PGConnection::Transaction(tx, _) => {
                let mut tx = tx.lock().await;
                let res = delete_user_mfa(&mut *tx, tenant, id).await?;
                Ok(res)
            }
        }
    }

    async fn search(
        &self,
        tenant: &TenantId,
        claims: &UserMFASearchClaims,
    ) -> Result<Vec<UserMFACredential>, OperationOutcomeError> {
        match self {
            PGConnection::Pool(pool, _) => {
                let res = search_user_mfa(pool, tenant, claims).await?;
                Ok(res)
            }
            PGConnection::Transaction(tx, _) => {
                let mut tx = tx.lock().await;
                let res = search_user_mfa(&mut *tx, tenant, claims).await?;
                Ok(res)
            }
        }
    }
}
