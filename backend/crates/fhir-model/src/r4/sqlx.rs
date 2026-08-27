use crate::r4::generated::resources::ResourceType;
use sqlx::{
    Database, Decode, Encode, Postgres,
    encode::IsNull,
    error::BoxDynError,
    postgres::{PgArgumentBuffer, PgTypeInfo, PgValueRef},
};
use std::io::Write;

#[derive(Debug, Clone)]
pub struct FHIRJson<T: ?Sized>(pub T);

impl<T> sqlx::Type<Postgres> for FHIRJson<T>
where
    T: serde::Serialize + serde::de::DeserializeOwned,
{
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("jsonb")
    }

    fn compatible(ty: &PgTypeInfo) -> bool {
        *ty == PgTypeInfo::with_name("json") || *ty == PgTypeInfo::with_name("jsonb")
    }
}

impl<'r, T: 'r> Decode<'r, Postgres> for FHIRJson<T>
where
    T: serde::Serialize + serde::Deserialize<'r>,
{
    fn decode(value: PgValueRef<'r>) -> Result<Self, BoxDynError> {
        let buf = value.as_bytes()?;
        // Need to remove first byte which is a marker for JSONB binary.
        let resource = serde_json::from_slice::<T>(&buf[1..]);
        Ok(FHIRJson::<T>(resource?))
    }
}

// More effecient impl to avoid cloning the value. No need to own as writing bytes and non mutating.
pub struct FHIRJsonRef<'a, T: ?Sized>(pub &'a T);

impl<T> sqlx::Type<Postgres> for FHIRJsonRef<'_, T>
where
    T: serde::Serialize + serde::de::DeserializeOwned,
{
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("jsonb")
    }

    fn compatible(ty: &PgTypeInfo) -> bool {
        *ty == PgTypeInfo::with_name("json") || *ty == PgTypeInfo::with_name("jsonb")
    }
}

impl<'q, T> Encode<'q, Postgres> for FHIRJsonRef<'q, T>
where
    T: serde::Serialize + serde::de::DeserializeOwned,
{
    fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> Result<IsNull, BoxDynError> {
        // we have a tiny amount of dynamic behavior depending if we are resolved to be JSON
        // instead of JSONB

        // buf.patch(|buf, ty: &PgTypeInfo| {
        //     if *ty == PgTypeInfo::JSON || *ty == PgTypeInfo::JSON_ARRAY {
        //         buf[0] = b' ';
        //     }
        // });

        // JSONB version (as of 2020-03-20)
        buf.push(1);

        // the JSON data written to the buffer is the same regardless of parameter type
        serde_json::to_writer(&mut **buf, self.0)?;

        Ok(IsNull::No)
    }
}

impl<'r, DB: Database> Decode<'r, DB> for ResourceType
where
    &'r str: Decode<'r, DB>,
{
    fn decode(
        value: <DB as Database>::ValueRef<'r>,
    ) -> Result<ResourceType, Box<dyn std::error::Error + 'static + Send + Sync>> {
        let value = <&str as Decode<DB>>::decode(value)?;
        Ok(ResourceType::try_from(value).unwrap())
    }
}

impl Encode<'_, Postgres> for ResourceType {
    fn encode_by_ref(
        &self,
        buf: &mut PgArgumentBuffer,
    ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
        buf.write_all(self.as_ref().as_bytes())?;
        Ok(sqlx::encode::IsNull::No)
    }
}
