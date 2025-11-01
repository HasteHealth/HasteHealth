use crate::{ProjectId, ResourceId, TenantId, VersionId, VersionIdRef, scopes::Scopes};
use sqlx::{Database, Decode, Encode, Postgres, postgres::PgArgumentBuffer};
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

impl sqlx::Type<Postgres> for TenantId {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        <&str as sqlx::Type<Postgres>>::type_info()
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
        buf.write(self.as_ref().as_bytes())?;
        Ok(sqlx::encode::IsNull::No)
    }
}

impl sqlx::Type<Postgres> for ProjectId {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        <&str as sqlx::Type<Postgres>>::type_info()
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

impl<'r> Encode<'r, Postgres> for VersionId {
    fn encode_by_ref(
        &self,
        buf: &mut PgArgumentBuffer,
    ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
        buf.write(self.0.as_bytes())?;
        Ok(sqlx::encode::IsNull::No)
    }
}

impl<'r, DB: Database> Decode<'r, DB> for VersionId
where
    &'r str: Decode<'r, DB>,
{
    fn decode(
        value: <DB as Database>::ValueRef<'r>,
    ) -> Result<VersionId, Box<dyn Error + 'static + Send + Sync>> {
        let value = <&str as Decode<DB>>::decode(value)?;
        Ok(VersionId::new(value.to_string()))
    }
}

impl sqlx::Type<Postgres> for VersionId {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        <&str as sqlx::Type<Postgres>>::type_info()
    }
}

impl<'r, DB: Database> Decode<'r, DB> for Scopes
where
    &'r str: Decode<'r, DB>,
{
    fn decode(
        value: <DB as Database>::ValueRef<'r>,
    ) -> Result<Scopes, Box<dyn Error + 'static + Send + Sync>> {
        let value = <&str as Decode<DB>>::decode(value)?;
        let scopes = Scopes::try_from(value)?;

        Ok(scopes)
    }
}

impl<'r> Encode<'r, Postgres> for Scopes {
    fn encode_by_ref(
        &self,
        buf: &mut PgArgumentBuffer,
    ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
        let scope_string = String::from(self.clone());
        buf.write(scope_string.as_bytes())?;
        Ok(sqlx::encode::IsNull::No)
    }
}

impl sqlx::Type<Postgres> for Scopes {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        <&str as sqlx::Type<Postgres>>::type_info()
    }
}
