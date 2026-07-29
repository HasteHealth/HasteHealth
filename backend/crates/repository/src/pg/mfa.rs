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
use sqlx::{PgExecutor, QueryBuilder};

async fn create_user_mfa_credential<'a, 'e, E>(
    executor: E,
    tenant: &'a TenantId,
    new_mfa_credentials: UserMFACredentialCreate,
) -> Result<UserMFACredential, OperationOutcomeError>
where
    E: PgExecutor<'e>,
{
    let type_: &str = new_mfa_credentials.credential_type.into();
    let totp_algorithm = new_mfa_credentials
        .totp_algorithm
        .unwrap_or_else(|| "SHA1".to_string());

    let user_mfa_credential = sqlx::query_as::<_, UserMFACredential>(
        r"
            INSERT INTO user_mfa_credential (tenant, user_id, credential_type, secret_ciphertext, secret_nonce, key_id, totp_algorithm, totp_digits, totp_period, totp_skew)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING
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
                is_active
        ",
    )
    .bind(tenant.as_ref())
    .bind(new_mfa_credentials.user_id.as_ref())
    .bind(type_)
    .bind(new_mfa_credentials.secret_ciphertext)
    .bind(new_mfa_credentials.secret_nonce)
    .bind(new_mfa_credentials.key_id)
    .bind(totp_algorithm)
    .bind(new_mfa_credentials.totp_digits.unwrap_or(6))
    .bind(new_mfa_credentials.totp_period.unwrap_or(30))
    .bind(new_mfa_credentials.totp_skew.unwrap_or(1))
    .fetch_one(executor)
    .await
    .map_err(StoreError::SQLXError)?;

    Ok(user_mfa_credential)
}

async fn read_user_mfa<'a, 'e, E>(
    executor: E,
    tenant: &'a TenantId,
    key: &'a MFAKey,
) -> Result<Option<UserMFACredential>, OperationOutcomeError>
where
    E: PgExecutor<'e>,
{
    let user_mfa = sqlx::query_as::<_, UserMFACredential>(
        r"
            SELECT
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
                is_active
            FROM user_mfa_credential
            WHERE tenant = $1 AND id::text = $2 AND user_id = $3
        ",
    )
    .bind(tenant.as_ref())
    .bind(&key.mfa_id().0)
    .bind(key.user_id().as_ref())
    .fetch_optional(executor)
    .await
    .map_err(StoreError::SQLXError)?;

    Ok(user_mfa)
}

async fn delete_user_mfa<'a, 'e, E>(
    executor: E,
    tenant: &'a TenantId,
    key: &'a MFAKey,
) -> Result<(), OperationOutcomeError>
where
    E: PgExecutor<'e>,
{
    let rows_affected = sqlx::query(
        r"
            DELETE FROM user_mfa_credential
            WHERE tenant = $1 AND id::text = $2 AND user_id = $3
        ",
    )
    .bind(tenant.as_ref())
    .bind(&key.mfa_id().0)
    .bind(key.user_id().as_ref())
    .execute(executor)
    .await
    .map_err(|_e| {
        OperationOutcomeError::error(
            IssueType::not_found(),
            format!(
                "User MFA credential '{}' not found or is system created and cannot be deleted.",
                key.mfa_id().0
            ),
        )
    })?
    .rows_affected();

    if rows_affected == 0 {
        return Err(OperationOutcomeError::error(
            IssueType::not_found(),
            format!(
                "User MFA credential '{}' not found or is system created and cannot be deleted.",
                key.mfa_id().0
            ),
        ));
    }

    Ok(())
}

async fn search_user_mfa<'a, 'e, E>(
    executor: E,
    tenant: &'a TenantId,
    clauses: &'a UserMFASearchClaims,
) -> Result<Vec<UserMFACredential>, OperationOutcomeError>
where
    E: PgExecutor<'e>,
{
    let mut query_builder: QueryBuilder<sqlx::Postgres> = QueryBuilder::new(
        r"SELECT
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
            is_active FROM user_mfa_credential WHERE ",
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

    let query = query_builder.build_query_as::<UserMFACredential>();

    let user_mfas: Vec<UserMFACredential> =
        query.fetch_all(executor).await.map_err(StoreError::from)?;

    Ok(user_mfas)
}

async fn update_user_mfa<'a, 'e, E>(
    executor: E,
    tenant: &'a TenantId,
    model: UserMFACredentialUpdate,
) -> Result<UserMFACredential, OperationOutcomeError>
where
    E: PgExecutor<'e>,
{
    let mut query_builder = QueryBuilder::new(
        r"
            UPDATE user_mfa_credential SET
        ",
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
        r" RETURNING
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
            is_active",
    );

    let query = query_builder.build_query_as::<UserMFACredential>();

    let user_mfa_credentials = query
        .fetch_one(executor)
        .await
        .map_err(StoreError::SQLXError)?;

    Ok(user_mfa_credentials)
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
                create_user_mfa_credential(pool, tenant, new_user_mfa_credential).await
            }
            PGConnection::Transaction(tx, _) => {
                let mut tx = tx.lock().await;

                create_user_mfa_credential(&mut **tx, tenant, new_user_mfa_credential).await
            }
        }
    }

    async fn read(
        &self,
        tenant: &TenantId,
        id: &MFAKey,
    ) -> Result<Option<UserMFACredential>, haste_fhir_operation_error::OperationOutcomeError> {
        match self {
            PGConnection::Pool(pool, _) => read_user_mfa(pool, tenant, id).await,
            PGConnection::Transaction(tx, _) => {
                let mut tx = tx.lock().await;
                read_user_mfa(&mut **tx, tenant, id).await
            }
        }
    }

    async fn update(
        &self,
        tenant: &TenantId,
        model: UserMFACredentialUpdate,
    ) -> Result<UserMFACredential, haste_fhir_operation_error::OperationOutcomeError> {
        match self {
            PGConnection::Pool(pool, _) => update_user_mfa(pool, tenant, model).await,
            PGConnection::Transaction(tx, _) => {
                let mut tx = tx.lock().await;
                update_user_mfa(&mut **tx, tenant, model).await
            }
        }
    }

    async fn delete(
        &self,
        tenant: &TenantId,
        id: &MFAKey,
    ) -> Result<(), haste_fhir_operation_error::OperationOutcomeError> {
        match self {
            PGConnection::Pool(pool, _) => delete_user_mfa(pool, tenant, id).await,
            PGConnection::Transaction(tx, _) => {
                let mut tx = tx.lock().await;
                delete_user_mfa(&mut **tx, tenant, id).await
            }
        }
    }

    async fn search(
        &self,
        tenant: &TenantId,
        claims: &UserMFASearchClaims,
    ) -> Result<Vec<UserMFACredential>, OperationOutcomeError> {
        match self {
            PGConnection::Pool(pool, _) => search_user_mfa(pool, tenant, claims).await,
            PGConnection::Transaction(tx, _) => {
                let mut tx = tx.lock().await;
                search_user_mfa(&mut **tx, tenant, claims).await
            }
        }
    }
}
