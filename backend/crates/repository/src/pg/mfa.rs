use crate::{
    admin::TenantAuthAdmin, pg::{PGConnection, StoreError}, types::{
        SupportedFHIRVersions, mfa::{UserMFACredential, UserMFACredentialCreate, UserMFACredentialUpdate, UserMFASearchClaims}, project::{CreateProject, Project, ProjectSearchClaims},
    },
};
use haste_fhir_model::r4::generated::{terminology::IssueType};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_jwt::{ProjectId, TenantId};
use sqlx::{Acquire, Postgres, QueryBuilder};

fn create_user_mfa_credential<'a, 'c, Connection: Acquire<'c, Database = Postgres> + Send + 'a>(
    connection: Connection,
    tenant: &'a TenantId,
    new_mfa_credentials: UserMFACredentialCreate,
) -> impl Future<Output = Result<UserMFACredential, OperationOutcomeError>> + Send + 'a {
    async move {
        let mut conn = connection.acquire().await.map_err(StoreError::SQLXError)?;

        let user_mfa_credential = sqlx::query_as!(
            UserMFACredential,
            r#"INSERT INTO user_mfa_credential (tenant, user_id, credential_type, secret_ciphertext, secret_nonce, key_id, totp_algorithm, totp_digits, totp_period, totp_skew) 
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10) 
            RETURNING id::text, tenant as "tenant: TenantId", user_id, credential_type, secret_ciphertext, secret_nonce, key_id, totp_algorithm, totp_digits, totp_period, totp_skew, created_at, is_active, activated_at"#,
            tenant.as_ref(),
            new_mfa_credentials.user_id.as_ref(),
            new_mfa_credentials.credential_type as i32,
            new_mfa_credentials.secret_ciphertext,
            new_mfa_credentials.secret_nonce,
            new_mfa_credentials.key_id,
            new_mfa_credentials.totp_algorithm as i32,
            new_mfa_credentials.totp_digits,
            new_mfa_credentials.totp_period,
            new_mfa_credentials.totp_skew,
        )
        .fetch_one(&mut *conn)
        .await
        .map_err(StoreError::SQLXError)?;

        Ok(user_mfa_credential)
    }
}

fn read_project<'a, 'c, Connection: Acquire<'c, Database = Postgres> + Send + 'a>(
    connection: Connection,
    tenant: &'a TenantId,
    id: &'a str,
) -> impl Future<Output = Result<Option<Project>, OperationOutcomeError>> + Send + 'a {
    async move {
        let mut conn = connection.acquire().await.map_err(StoreError::SQLXError)?;
        let project = sqlx::query_as!(
            Project,
            r#"SELECT id as "id: ProjectId", tenant as "tenant: TenantId", system_created, fhir_version as "fhir_version: SupportedFHIRVersions" 
            FROM projects where tenant = $1 AND id = $2"#,
            tenant.as_ref(),    
            id
        )
        .fetch_optional(&mut *conn)
        .await
        .map_err(StoreError::SQLXError)?;

        Ok(project)
    }
}

fn delete_project<'a, 'c, Connection: Acquire<'c, Database = Postgres> + Send + 'a>(
    connection: Connection,
    tenant: &'a TenantId,
    id: &'a str,
) -> impl Future<Output = Result<(), OperationOutcomeError>> + Send + 'a {
    async move {
        let mut conn = connection.acquire().await.map_err(StoreError::SQLXError)?;
        let _deleted_project = sqlx::query_as!(
            Project,
            r#"DELETE FROM projects WHERE tenant = $1 AND id = $2 and system_created = false RETURNING id as "id: ProjectId", tenant as "tenant: TenantId", system_created, fhir_version as "fhir_version: SupportedFHIRVersions""#,
            tenant.as_ref(),
            id
        )
        .fetch_optional(&mut *conn)
        .await
        .map_err(|_e| {
            OperationOutcomeError::error(
                IssueType::NotFound(None),
                format!("Project '{}' not found or is system created and cannot be deleted.", id),
            )
        })?;

        if !_deleted_project.is_some() {
            return Err(OperationOutcomeError::error(
                IssueType::NotFound(None),
                format!(
                    "Project '{}' not found or is system created and cannot be deleted.",
                    id
                ),
            ));
        }

        Ok(())
    }
}

fn search_project<'a, 'c, Connection: Acquire<'c, Database = Postgres> + Send + 'a>(
    connection: Connection,
    tenant: &'a TenantId,
    clauses: &'a ProjectSearchClaims,
) -> impl Future<Output = Result<Vec<Project>, OperationOutcomeError>> + Send + 'a {
    async move {
        let mut conn = connection.acquire().await.map_err(StoreError::SQLXError)?;
        let mut query_builder: QueryBuilder<Postgres> =
            QueryBuilder::new(r#"SELECT tenant, id, fhir_version FROM projects WHERE "#);

        let mut and_clauses = query_builder.separated(" AND ");

        and_clauses
            .push(" tenant = ")
            .push_bind_unseparated(tenant.as_ref());

        if let Some(id) = clauses.id.as_ref() {
            and_clauses
                .push(" id = ")
                .push_bind_unseparated(id.as_ref());
        }

        if let Some(fhir_version) = clauses.fhir_version.as_ref() {
            and_clauses
                .push(" fhir_version = ")
                .push_bind_unseparated(fhir_version);
        }

        if let Some(system_created) = clauses.system_created.as_ref() {
            and_clauses
                .push(" system_created = ")
                .push_bind_unseparated(system_created);
        }

        let query = query_builder.build_query_as();

        let projects: Vec<Project> = query
            .fetch_all(&mut *conn)
            .await
            .map_err(StoreError::from)?;

        Ok(projects)
    }
}

/// Not allowing updates on internal row just reading to confirm it's existance.
fn update_project<'a, 'c, Connection: Acquire<'c, Database = Postgres> + Send + 'a>(
    connection: Connection,
    tenant: &'a TenantId,
    model: Project,
) -> impl Future<Output = Result<Project, OperationOutcomeError>> + Send + 'a {
    async move {
        read_project(connection, tenant, model.id.as_ref())
            .await?
            .ok_or_else(|| {
                OperationOutcomeError::error(
                    IssueType::NotFound(None),
                    format!("Project '{}' not found.", model.id.as_ref()),
                )
            })
    }
}

impl<Key: AsRef<str> + Send + Sync>
    TenantAuthAdmin<UserMFACredentialCreate, UserMFACredential, UserMFASearchClaims, UserMFACredentialUpdate, Key> for PGConnection
{
    async fn create(
        &self,
        tenant: &TenantId,
        new_project: UserMFACredentialCreate,
    ) -> Result<UserMFACredential, OperationOutcomeError> {
        match self {
            PGConnection::Pool(pool, _) => {
                let res = create_user_mfa_credential(pool, tenant, new_project).await?;
                Ok(res)
            }
            PGConnection::Transaction(tx, _) => {
                let mut tx = tx.lock().await;
                let res = create_user_mfa_credential(&mut *tx, tenant, new_project).await?;
                Ok(res)
            }
        }
    }

    async fn read(
        &self,
        tenant: &TenantId,
        id: &Key,
    ) -> Result<Option<UserMFACredential>, haste_fhir_operation_error::OperationOutcomeError> {
        match self {
            PGConnection::Pool(pool, _) => {
                let res = read_project(pool, tenant, id.as_ref()).await?;
                Ok(res)
            }
            PGConnection::Transaction(tx, _) => {
                let mut tx = tx.lock().await;
                let res = read_project(&mut *tx, tenant, id.as_ref()).await?;
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
                let res = update_project(pool, tenant, model).await?;
                Ok(res)
            }
            PGConnection::Transaction(tx, _) => {
                let mut tx = tx.lock().await;
                let res = update_project(&mut *tx, tenant, model).await?;
                Ok(res)
            }
        }
    }

    async fn delete(
        &self,
        tenant: &TenantId,
        id: &Key,
    ) -> Result<(), haste_fhir_operation_error::OperationOutcomeError> {
        match self {
            PGConnection::Pool(pool, _) => {
                let res = delete_project(pool, tenant, id.as_ref()).await?;
                Ok(res)
            }
            PGConnection::Transaction(tx, _) => {
                let mut tx = tx.lock().await;
                let res = delete_project(&mut *tx, tenant, id.as_ref()).await?;
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
                let res = search_project(pool, tenant, claims).await?;
                Ok(res)
            }
            PGConnection::Transaction(tx, _) => {
                let mut tx = tx.lock().await;
                let res = search_project(&mut *tx, tenant, claims).await?;
                Ok(res)
            }
        }
    }
}
