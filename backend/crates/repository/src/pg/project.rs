use crate::{
    admin::TenantAuthAdmin,
    pg::{PGConnection, StoreError},
    types::{
        SupportedFHIRVersions, 
        project::{CreateProject, Project, ProjectSearchClaims},
    },
    utilities::{generate_id, validate_id},
};
use oxidized_jwt::{ProjectId, TenantId};
use oxidized_fhir_model::r4::generated::terminology::IssueType;
use oxidized_fhir_operation_error::OperationOutcomeError;
use sqlx::{Acquire, Postgres, QueryBuilder};

fn create_project<'a, 'c, Connection: Acquire<'c, Database = Postgres> + Send + 'a>(
    connection: Connection,
    tenant: &'a TenantId,
    project: CreateProject,
) -> impl Future<Output = Result<Project, OperationOutcomeError>> + Send + 'a {
    async move {
        let mut conn = connection.acquire().await.map_err(StoreError::SQLXError)?;
        let id = project.id.unwrap_or(ProjectId::new(generate_id(None)));

        validate_id(id.as_ref())?;

        let project = sqlx::query_as!(
            Project,
            r#"INSERT INTO projects (tenant, id, fhir_version, system_created) VALUES ($1, $2, $3, $4) RETURNING tenant as "tenant: TenantId", id as "id: ProjectId", fhir_version as "fhir_version: SupportedFHIRVersions""#,
            tenant.as_ref(),
            id.as_ref(),
            project.fhir_version as SupportedFHIRVersions,
            project.system_created
        )
        .fetch_one(&mut *conn)
        .await
        .map_err(StoreError::SQLXError)?;

        Ok(project)
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
            r#"SELECT id as "id: ProjectId", tenant as "tenant: TenantId", fhir_version as "fhir_version: SupportedFHIRVersions" FROM projects where tenant = $1 AND id = $2"#,
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
) -> impl Future<Output = Result<Project, OperationOutcomeError>> + Send + 'a {
    async move {
        let mut conn = connection.acquire().await.map_err(StoreError::SQLXError)?;
        let deleted_project = sqlx::query_as!(
            Project,
            r#"DELETE FROM projects WHERE tenant = $1 AND id = $2 and system_created = false RETURNING id as "id: ProjectId", tenant as "tenant: TenantId", fhir_version as "fhir_version: SupportedFHIRVersions""#,
            tenant.as_ref(),
            id
        )
        .fetch_one(&mut *conn)
        .await
        .map_err(|_e| {
            OperationOutcomeError::error(
                IssueType::NotFound(None),
                format!("Project '{}' not found or is system created and cannot be deleted.", id),
            )
        })?;

        Ok(deleted_project)
    }
}

fn search_project<'a, 'c, Connection: Acquire<'c, Database = Postgres> + Send + 'a>(
    connection: Connection,
    tenant: &'a TenantId,
    clauses: &'a ProjectSearchClaims,
) -> impl Future<Output = Result<Vec<Project>, OperationOutcomeError>> + Send + 'a {
    async move {
        let mut conn = connection.acquire().await.map_err(StoreError::SQLXError)?;
        let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new(
            r#"SELECT tenant, id, fhir_version FROM projects WHERE "#,
        );

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

        let query = query_builder.build_query_as();

        let projects: Vec<Project> = query
            .fetch_all(&mut *conn)
            .await
            .map_err(StoreError::from)?;

        Ok(projects)
    }
}

impl<Key: AsRef<str> + Send + Sync> TenantAuthAdmin<CreateProject, Project, ProjectSearchClaims, Project, Key> for PGConnection {
    async fn create(
        &self,
        tenant: &TenantId,
        new_project: CreateProject,
    ) -> Result<Project, OperationOutcomeError> {
        match self {
            PGConnection::Pool(pool) => {
                let res = create_project(pool, tenant, new_project).await?;
                Ok(res)
            }
            PGConnection::Transaction(tx) => {
                let mut tx = tx.lock().await;
                let res = create_project(&mut *tx, tenant, new_project).await?;
                Ok(res)
            }
        }
    }

    async fn read(
        &self,
        tenant: &TenantId,
        id: &Key,
    ) -> Result<Option<Project>, oxidized_fhir_operation_error::OperationOutcomeError> {
        match self {
            PGConnection::Pool(pool) => {
                let res = read_project(pool, tenant, id.as_ref()).await?;
                Ok(res)
            }
            PGConnection::Transaction(tx) => {
                let mut tx = tx.lock().await;
                let res = read_project(&mut *tx, tenant, id.as_ref()).await?;
                Ok(res)
            }
        }
    }

    async fn update(
        &self,
        _tenant: &TenantId,
        _model: Project,
    ) -> Result<Project, oxidized_fhir_operation_error::OperationOutcomeError> {
        Err(OperationOutcomeError::error(
            IssueType::NotSupported(None),
            "Projects cannot be updated.".to_string(),
        ))
    }

    async fn delete(
        &self,
        tenant: &TenantId,
        id: &Key,
    ) -> Result<Project, oxidized_fhir_operation_error::OperationOutcomeError> {
        match self {
            PGConnection::Pool(pool) => {
                let res = delete_project(pool, tenant, id.as_ref()).await?;
                Ok(res)
            }
            PGConnection::Transaction(tx) => {
                let mut tx = tx.lock().await;
                let res = delete_project(&mut *tx, tenant, id.as_ref()).await?;
                Ok(res)
            }
        }
    }

    async fn search(
        &self,
        tenant: &TenantId,
        claims: &ProjectSearchClaims,
    ) -> Result<Vec<Project>, OperationOutcomeError> {
        match self {
            PGConnection::Pool(pool) => {
                let res = search_project(pool, tenant, claims).await?;
                Ok(res)
            }
            PGConnection::Transaction(tx) => {
                let mut tx = tx.lock().await;
                let res = search_project(&mut *tx, tenant, claims).await?;
                Ok(res)
            }
        }
    }
}
