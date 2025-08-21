use crate::types::{ProjectId, ResourceId, TenantId, VersionIdRef};
use sqlx::{Database, Decode, Encode, Postgres};
use sqlx_postgres::PgArgumentBuffer;
use std::{error::Error, io::Write};

impl<'r, DB: Database> Decode<'r, DB> for TenantId
where
    &'r str: Decode<'r, DB>,
{
    fn decode(
        value: <DB as Database>::ValueRef<'r>,
    ) -> Result<TenantId, Box<dyn Error + 'static + Send + Sync>> {
        let value = <&str as Decode<DB>>::decode(value)?;
        Ok(TenantId::new(value.to_string()))
    }
}

impl<'r> Encode<'r, Postgres> for TenantId {
    fn encode_by_ref(
        &self,
        buf: &mut PgArgumentBuffer,
    ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
        buf.write(self.as_ref().as_bytes())?;
        Ok(sqlx::encode::IsNull::No)
    }
}

impl<'r, DB: Database> Decode<'r, DB> for ProjectId
where
    &'r str: Decode<'r, DB>,
{
    fn decode(
        value: <DB as Database>::ValueRef<'r>,
    ) -> Result<ProjectId, Box<dyn Error + 'static + Send + Sync>> {
        let value = <&str as Decode<DB>>::decode(value)?;
        Ok(ProjectId::new(value.to_string()))
    }
}
impl<'r> Encode<'r, Postgres> for ProjectId {
    fn encode_by_ref(
        &self,
        buf: &mut PgArgumentBuffer,
    ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
        buf.write(self.0.as_bytes())?;
        Ok(sqlx::encode::IsNull::No)
    }
}

impl<'r, DB: Database> Decode<'r, DB> for ResourceId
where
    &'r str: Decode<'r, DB>,
{
    fn decode(
        value: <DB as Database>::ValueRef<'r>,
    ) -> Result<ResourceId, Box<dyn Error + 'static + Send + Sync>> {
        let value = <&str as Decode<DB>>::decode(value)?;
        Ok(ResourceId::new(value.to_string()))
    }
}

impl<'r> Encode<'r, Postgres> for ResourceId {
    fn encode_by_ref(
        &self,
        buf: &mut PgArgumentBuffer,
    ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
        buf.write(self.0.as_bytes())?;
        Ok(sqlx::encode::IsNull::No)
    }
}

impl<'r> Encode<'r, Postgres> for VersionIdRef<'r> {
    fn encode_by_ref(
        &self,
        buf: &mut PgArgumentBuffer,
    ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
        buf.write(self.0.as_bytes())?;
        Ok(sqlx::encode::IsNull::No)
    }
}
