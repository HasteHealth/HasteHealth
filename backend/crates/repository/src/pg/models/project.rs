use crate::{
    admin::TenantModelAdmin,
    pg::{PGConnection, StoreError},
    types::project::{CreateProject, Project, ProjectSearchClaims},
    utilities::{generate_id, validate_id},
};
use haste_fhir_model::r4::generated::terminology::IssueType;
use haste_fhir_operation_error::OperationOutcomeError;
use haste_jwt::{ProjectId, TenantId};
use sqlx::{PgExecutor, QueryBuilder};

async fn create_project<'a, 'e, E>(
    executor: E,
    tenant: &'a TenantId,
    project: CreateProject,
) -> Result<Project, OperationOutcomeError>
where
    E: PgExecutor<'e>,
{
    let id = project.id.unwrap_or(ProjectId::new(generate_id(None)));

    validate_id(id.as_ref())?;

    let project = sqlx::query_as::<_, Project>(
        r"
            INSERT INTO projects (tenant, id, fhir_version, system_created)
            VALUES ($1, $2, $3, $4)
            RETURNING tenant, system_created, id, fhir_version
        ",
    )
    .bind(tenant.as_ref())
    .bind(id.as_ref())
    .bind(project.fhir_version)
    .bind(project.system_created)
    .fetch_one(executor)
    .await
    .map_err(StoreError::SQLXError)?;

    Ok(project)
}

async fn read_project<'a, 'e, E>(
    executor: E,
    tenant: &'a TenantId,
    id: &'a str,
) -> Result<Option<Project>, OperationOutcomeError>
where
    E: PgExecutor<'e>,
{
    let project = sqlx::query_as::<_, Project>(
        r"
            SELECT id, tenant, system_created, fhir_version
            FROM projects
            WHERE tenant = $1 AND id = $2
        ",
    )
    .bind(tenant.as_ref())
    .bind(id)
    .fetch_optional(executor)
    .await
    .map_err(StoreError::SQLXError)?;

    Ok(project)
}

async fn delete_project<'a, 'e, E>(
    executor: E,
    tenant: &'a TenantId,
    id: &'a str,
) -> Result<(), OperationOutcomeError>
where
    E: PgExecutor<'e>,
{
    let rows_affected = sqlx::query(
        r"
            DELETE FROM projects
            WHERE tenant = $1 AND id = $2 AND system_created = false
        ",
    )
    .bind(tenant.as_ref())
    .bind(id)
    .execute(executor)
    .await
    .map_err(|_e| {
        OperationOutcomeError::error(
            IssueType::not_found(),
            format!("Project '{id}' not found or is system created and cannot be deleted."),
        )
    })?
    .rows_affected();

    if rows_affected == 0 {
        return Err(OperationOutcomeError::error(
            IssueType::not_found(),
            format!("Project '{id}' not found or is system created and cannot be deleted."),
        ));
    }

    Ok(())
}

async fn search_project<'a, 'e, E>(
    executor: E,
    tenant: &'a TenantId,
    clauses: &'a ProjectSearchClaims,
) -> Result<Vec<Project>, OperationOutcomeError>
where
    E: PgExecutor<'e>,
{
    let mut query_builder: QueryBuilder<sqlx::Postgres> =
        QueryBuilder::new(r"SELECT tenant, id, fhir_version, system_created FROM projects WHERE ");

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

    let query = query_builder.build_query_as::<Project>();

    let projects: Vec<Project> = query.fetch_all(executor).await.map_err(StoreError::from)?;

    Ok(projects)
}

/// Not allowing updates on internal row just reading to confirm it's existance.
async fn update_project<'a, 'e, E>(
    executor: E,
    tenant: &'a TenantId,
    model: Project,
) -> Result<Project, OperationOutcomeError>
where
    E: PgExecutor<'e>,
{
    read_project(executor, tenant, model.id.as_ref())
        .await?
        .ok_or_else(|| {
            OperationOutcomeError::error(
                IssueType::not_found(),
                format!("Project '{}' not found.", model.id.as_ref()),
            )
        })
}

impl<Key: AsRef<str> + Send + Sync>
    TenantModelAdmin<CreateProject, Project, ProjectSearchClaims, Project, Key> for PGConnection
{
    async fn create(
        &self,
        tenant: &TenantId,
        new_project: CreateProject,
    ) -> Result<Project, OperationOutcomeError> {
        match self {
            PGConnection::Pool(pool, _) => create_project(pool, tenant, new_project).await,
            PGConnection::Transaction(tx, _) => {
                let mut tx = tx.lock().await;
                create_project(&mut **tx, tenant, new_project).await
            }
        }
    }

    async fn read(
        &self,
        tenant: &TenantId,
        id: &Key,
    ) -> Result<Option<Project>, haste_fhir_operation_error::OperationOutcomeError> {
        match self {
            PGConnection::Pool(pool, _) => read_project(pool, tenant, id.as_ref()).await,
            PGConnection::Transaction(tx, _) => {
                let mut tx = tx.lock().await;
                read_project(&mut **tx, tenant, id.as_ref()).await
            }
        }
    }

    async fn update(
        &self,
        tenant: &TenantId,
        model: Project,
    ) -> Result<Project, haste_fhir_operation_error::OperationOutcomeError> {
        match self {
            PGConnection::Pool(pool, _) => update_project(pool, tenant, model).await,
            PGConnection::Transaction(tx, _) => {
                let mut tx = tx.lock().await;
                update_project(&mut **tx, tenant, model).await
            }
        }
    }

    async fn delete(
        &self,
        tenant: &TenantId,
        id: &Key,
    ) -> Result<(), haste_fhir_operation_error::OperationOutcomeError> {
        match self {
            PGConnection::Pool(pool, _) => delete_project(pool, tenant, id.as_ref()).await,
            PGConnection::Transaction(tx, _) => {
                let mut tx = tx.lock().await;
                delete_project(&mut **tx, tenant, id.as_ref()).await
            }
        }
    }

    async fn search(
        &self,
        tenant: &TenantId,
        claims: &ProjectSearchClaims,
    ) -> Result<Vec<Project>, OperationOutcomeError> {
        match self {
            PGConnection::Pool(pool, _) => search_project(pool, tenant, claims).await,
            PGConnection::Transaction(tx, _) => {
                let mut tx = tx.lock().await;
                search_project(&mut **tx, tenant, claims).await
            }
        }
    }
}
