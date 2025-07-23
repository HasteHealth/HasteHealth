use sqlx::{
    Decode, Encode, Postgres, encode::IsNull, error::BoxDynError, postgres::PgArgumentBuffer,
};

use crate::r4::types::Resource;

impl<'r, DB: sqlx::Database> Decode<'r, DB> for Resource
where
    &'r str: Decode<'r, DB>,
{
    fn decode(value: <DB as sqlx::Database>::ValueRef<'r>) -> Result<Self, BoxDynError> {
        let value = <&str>::decode(value)?;
        Ok(fhir_serialization_json::from_str::<Self>(value)?)
    }
}

impl<'q> Encode<'q, Postgres> for Resource {
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
        fhir_serialization_json::to_writer(&mut **buf, self)?;

        Ok(IsNull::No)
    }
}
