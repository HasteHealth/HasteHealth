use crate::{
    admin::ProjectModelAdmin,
    pg::{PGConnection, StoreError},
    types::membership::{CreateMembership, Membership, MembershipSearchClaims},
};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_jwt::{ProjectId, TenantId};
use sqlx::{PgExecutor, QueryBuilder};

async fn create_membership<'a, 'e, E>(
    executor: E,
    tenant: &'a TenantId,
    project: &'a ProjectId,
    membership: CreateMembership,
) -> Result<Membership, OperationOutcomeError>
where
    E: PgExecutor<'e>,
{
    let mut query_builder = QueryBuilder::new(
        r"
            INSERT INTO memberships(tenant, project, user_id, role, resource_id) VALUES (
        ",
    );

    let mut seperator = query_builder.separated(", ");

    seperator
        .push_bind(tenant.as_ref())
        .push_bind(project.as_ref())
        .push_bind(&membership.user_id)
        .push_bind(membership.role)
        .push_bind(&membership.resource_id);

    query_builder.push(r") RETURNING tenant, project, user_id, role, resource_id");

    let query = query_builder.build_query_as::<Membership>();

    let membership = query
        .fetch_one(executor)
        .await
        .map_err(StoreError::SQLXError)?;

    Ok(membership)
}

async fn read_membership<'a, 'e, E>(
    executor: E,
    tenant: &'a TenantId,
    project: &'a ProjectId,
    user_id: &'a str,
) -> Result<Option<Membership>, OperationOutcomeError>
where
    E: PgExecutor<'e>,
{
    let membership = sqlx::query_as::<_, Membership>(
        r"
            SELECT tenant, project, user_id, role, resource_id
            FROM memberships
            WHERE tenant = $1 AND project = $2 AND user_id = $3
        ",
    )
    .bind(tenant.as_ref())
    .bind(project.as_ref())
    .bind(user_id)
    .fetch_optional(executor)
    .await
    .map_err(StoreError::SQLXError)?;

    Ok(membership)
}

async fn update_membership<'a, 'e, E>(
    executor: E,
    tenant: &'a TenantId,
    project: &'a ProjectId,
    model: Membership,
) -> Result<Membership, OperationOutcomeError>
where
    E: PgExecutor<'e>,
{
    let mut query_builder = QueryBuilder::new(
        r"
            INSERT INTO memberships(tenant, project, user_id, role, resource_id) VALUES (
        ",
    );

    let mut seperator = query_builder.separated(", ");

    seperator
        .push_bind(tenant.as_ref())
        .push_bind(project.as_ref())
        .push_bind(&model.user_id)
        .push_bind(model.role.clone())
        .push_bind(&model.resource_id);

    query_builder.push(r") ON CONFLICT (tenant, project, user_id) DO UPDATE SET ");

    let mut set_statements = query_builder.separated(", ");

    set_statements
        .push(" role = ")
        .push_bind_unseparated(model.role);

    set_statements
        .push(" resource_id = ")
        .push_bind_unseparated(&model.resource_id);

    query_builder.push(r" RETURNING tenant, project, user_id, role, resource_id");

    let query = query_builder.build_query_as::<Membership>();

    let membership = query
        .fetch_one(executor)
        .await
        .map_err(StoreError::SQLXError)?;

    Ok(membership)
}

async fn delete_membership<'a, 'e, E>(
    executor: E,
    tenant: &'a TenantId,
    project: &'a ProjectId,
    user_id: &'a str,
) -> Result<(), OperationOutcomeError>
where
    E: PgExecutor<'e>,
{
    sqlx::query(
        r"
            DELETE FROM memberships
            WHERE tenant = $1 AND project = $2 AND user_id = $3
        ",
    )
    .bind(tenant.as_ref())
    .bind(project.as_ref())
    .bind(user_id)
    .execute(executor)
    .await
    .map_err(StoreError::SQLXError)?;

    Ok(())
}

async fn search_memberships<'a, 'e, E>(
    executor: E,
    tenant: &'a TenantId,
    project: &'a ProjectId,
    clauses: &'a MembershipSearchClaims,
) -> Result<Vec<Membership>, OperationOutcomeError>
where
    E: PgExecutor<'e>,
{
    let mut query_builder: QueryBuilder<sqlx::Postgres> = QueryBuilder::new(
        r"SELECT user_id, tenant, project, role, resource_id FROM memberships WHERE ",
    );

    let mut seperator = query_builder.separated(" AND ");
    seperator
        .push(" tenant = ")
        .push_bind_unseparated(tenant.as_ref())
        .push(" project = ")
        .push_bind_unseparated(project.as_ref());

    if let Some(user_id) = clauses.user_id.as_ref() {
        seperator
            .push(" user_id = ")
            .push_bind_unseparated(user_id.as_ref());
    }

    if let Some(role) = clauses.role.as_ref() {
        seperator.push(" role = ").push_bind_unseparated(role);
    }

    let query = query_builder.build_query_as::<Membership>();

    let memberships: Vec<Membership> = query.fetch_all(executor).await.map_err(StoreError::from)?;

    Ok(memberships)
}

impl<Key: AsRef<str> + Send + Sync>
    ProjectModelAdmin<CreateMembership, Membership, MembershipSearchClaims, Membership, Key>
    for PGConnection
{
    async fn create(
        &self,
        tenant: &TenantId,
        project: &ProjectId,
        new_membership: CreateMembership,
    ) -> Result<Membership, OperationOutcomeError> {
        match self {
            PGConnection::Pool(pool, _) => {
                create_membership(pool, tenant, project, new_membership).await
            }
            PGConnection::Transaction(tx, _, _) => {
                let mut tx = tx.lock().await;
                create_membership(&mut **tx, tenant, project, new_membership).await
            }
        }
    }

    async fn read(
        &self,
        tenant: &TenantId,
        project: &ProjectId,
        id: &Key,
    ) -> Result<Option<Membership>, OperationOutcomeError> {
        match self {
            PGConnection::Pool(pool, _) => {
                read_membership(pool, tenant, project, id.as_ref()).await
            }
            PGConnection::Transaction(tx, _, _) => {
                let mut tx = tx.lock().await;
                read_membership(&mut **tx, tenant, project, id.as_ref()).await
            }
        }
    }

    async fn update(
        &self,
        tenant: &TenantId,
        project: &ProjectId,
        model: Membership,
    ) -> Result<Membership, OperationOutcomeError> {
        match self {
            PGConnection::Pool(pool, _) => update_membership(pool, tenant, project, model).await,
            PGConnection::Transaction(tx, _, _) => {
                let mut tx = tx.lock().await;
                update_membership(&mut **tx, tenant, project, model).await
            }
        }
    }

    async fn delete(
        &self,
        tenant: &TenantId,
        project: &ProjectId,
        id: &Key,
    ) -> Result<(), OperationOutcomeError> {
        match self {
            PGConnection::Pool(pool, _) => {
                delete_membership(pool, tenant, project, id.as_ref()).await
            }
            PGConnection::Transaction(tx, _, _) => {
                let mut tx = tx.lock().await;
                delete_membership(&mut **tx, tenant, project, id.as_ref()).await
            }
        }
    }

    async fn search(
        &self,
        tenant: &TenantId,
        project: &ProjectId,
        clauses: &MembershipSearchClaims,
    ) -> Result<Vec<Membership>, OperationOutcomeError> {
        match self {
            PGConnection::Pool(pool, _) => search_memberships(pool, tenant, project, clauses).await,
            PGConnection::Transaction(tx, _, _) => {
                let mut tx = tx.lock().await;
                search_memberships(&mut **tx, tenant, project, clauses).await
            }
        }
    }
}
