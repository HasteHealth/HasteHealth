use sqlx::{
    Decode, Encode, Postgres,
    encode::IsNull,
    error::BoxDynError,
    postgres::{PgArgumentBuffer, PgTypeInfo},
};

pub struct FHIRJson<T: ?Sized>(pub T);

impl<T> sqlx::Type<Postgres> for FHIRJson<T>
where
    T: fhir_serialization_json::FHIRJSONSerializer + fhir_serialization_json::FHIRJSONDeserializer,
{
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("jsonb")
    }

    fn compatible(ty: &PgTypeInfo) -> bool {
        *ty == PgTypeInfo::with_name("json") || *ty == PgTypeInfo::with_name("jsonb")
    }
}

impl<'r, DB: sqlx::Database, T> Decode<'r, DB> for FHIRJson<T>
where
    T: fhir_serialization_json::FHIRJSONSerializer + fhir_serialization_json::FHIRJSONDeserializer,
    &'r str: Decode<'r, DB>,
{
    fn decode(value: <DB as sqlx::Database>::ValueRef<'r>) -> Result<Self, BoxDynError> {
        let value = <&str>::decode(value)?;
        let resource = fhir_serialization_json::from_str::<T>(&value[1..]);
        Ok(FHIRJson::<T>(resource?))
    }
}

impl<'q, T> Encode<'q, Postgres> for FHIRJson<T>
where
    T: fhir_serialization_json::FHIRJSONSerializer + fhir_serialization_json::FHIRJSONDeserializer,
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
        fhir_serialization_json::to_writer(&mut **buf, &self.0)?;

        Ok(IsNull::No)
    }
}
