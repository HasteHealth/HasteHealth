use crate::{
    admin::TenantAuthAdmin,
    pg::{PGConnection, StoreError},
    types::{
        ProjectId, SupportedFHIRVersions, TenantId,
        project::{CreateProject, Project, ProjectSearchClaims},
    },
    utilities::generate_id,
};
use oxidized_fhir_model::r4::generated::terminology::IssueType;
use oxidized_fhir_operation_error::OperationOutcomeError;
use sqlx::{Acquire, Postgres, QueryBuilder};

fn create_project<'a, 'c, Connection: Acquire<'c, Database = Postgres> + Send + 'a>(
    connection: Connection,
    project: CreateProject,
) -> impl Future<Output = Result<Project, OperationOutcomeError>> + Send + 'a {
    async move {
        let mut conn = connection.acquire().await.map_err(StoreError::SQLXError)?;
        let id = project.id.unwrap_or(ProjectId::new(generate_id(None)));

        let project = sqlx::query_as!(
            Project,
            r#"INSERT INTO projects (tenant, id, fhir_version) VALUES ($1, $2, $3) RETURNING tenant, id, fhir_version as "fhir_version: SupportedFHIRVersions""#,
            project.tenant.as_ref(),
            id.as_ref(),
            project.fhir_version as SupportedFHIRVersions,
        )
        .fetch_one(&mut *conn)
        .await
        .map_err(StoreError::SQLXError)?;

        Ok(project)
    }
}

fn read_project<'a, 'c, Connection: Acquire<'c, Database = Postgres> + Send + 'a>(
    connection: Connection,
    id: &'a str,
) -> impl Future<Output = Result<Project, OperationOutcomeError>> + Send + 'a {
    async move {
        let mut conn = connection.acquire().await.map_err(StoreError::SQLXError)?;
        let project = sqlx::query_as!(
            Project,
            r#"SELECT id, tenant, fhir_version as "fhir_version: SupportedFHIRVersions" FROM projects where id = $1"#,
            id
        )
        .fetch_one(&mut *conn)
        .await
        .map_err(StoreError::SQLXError)?;

        Ok(project)
    }
}

fn delete_project<'a, 'c, Connection: Acquire<'c, Database = Postgres> + Send + 'a>(
    connection: Connection,
    id: &'a str,
) -> impl Future<Output = Result<Project, OperationOutcomeError>> + Send + 'a {
    async move {
        let mut conn = connection.acquire().await.map_err(StoreError::SQLXError)?;
        let deleted_project = sqlx::query_as!(
            Project,
            r#"DELETE FROM projects WHERE id = $1 and system_created = false RETURNING id, tenant, fhir_version as "fhir_version: SupportedFHIRVersions""#,
            id
        )
        .fetch_one(&mut *conn)
        .await
        .map_err(StoreError::SQLXError)?;

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
            r#"SELECT tenant, id, fhir_version as "fhir_version: SupportedFHIRVersions" FROM projects WHERE "#,
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

impl TenantAuthAdmin<CreateProject, Project, ProjectSearchClaims> for PGConnection {
    async fn create(
        &self,
        _tenant: &TenantId,
        new_project: CreateProject,
    ) -> Result<Project, OperationOutcomeError> {
        match self {
            PGConnection::PgPool(pool) => {
                let res = create_project(pool, new_project).await?;
                Ok(res)
            }
            PGConnection::PgTransaction(tx) => {
                let mut tx = tx.lock().await;
                let res = create_project(&mut *tx, new_project).await?;
                Ok(res)
            }
        }
    }

    async fn read(
        &self,
        _tenant: &TenantId,
        id: &str,
    ) -> Result<Project, oxidized_fhir_operation_error::OperationOutcomeError> {
        match self {
            PGConnection::PgPool(pool) => {
                let res = read_project(pool, id).await?;
                Ok(res)
            }
            PGConnection::PgTransaction(tx) => {
                let mut tx = tx.lock().await;
                let res = read_project(&mut *tx, id).await?;
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
        _tenant: &TenantId,
        id: &str,
    ) -> Result<Project, oxidized_fhir_operation_error::OperationOutcomeError> {
        match self {
            PGConnection::PgPool(pool) => {
                let res = delete_project(pool, id).await?;
                Ok(res)
            }
            PGConnection::PgTransaction(tx) => {
                let mut tx = tx.lock().await;
                let res = delete_project(&mut *tx, id).await?;
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
            PGConnection::PgPool(pool) => {
                let res = search_project(pool, tenant, claims).await?;
                Ok(res)
            }
            PGConnection::PgTransaction(tx) => {
                let mut tx = tx.lock().await;
                let res = search_project(&mut *tx, tenant, claims).await?;
                Ok(res)
            }
        }
    }
}
