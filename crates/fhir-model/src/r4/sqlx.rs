use sqlx::{
    Decode, Encode, Postgres,
    encode::IsNull,
    error::BoxDynError,
    postgres::{PgArgumentBuffer, PgTypeInfo, PgValueRef},
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

impl<'r, T: 'r> Decode<'r, Postgres> for FHIRJson<T>
where
    T: fhir_serialization_json::FHIRJSONSerializer + fhir_serialization_json::FHIRJSONDeserializer,
{
    fn decode(value: PgValueRef<'r>) -> Result<Self, BoxDynError> {
        let buf = value.as_bytes()?;
        // Need to remove first byte which is a marker for JSONB binary.
        let resource = fhir_serialization_json::from_bytes::<T>(&buf[1..]);
        Ok(FHIRJson::<T>(resource?))
    }
}

// More effecient impl to avoid cloning the value. No need to own as writing bytes and non mutating.
pub struct FHIRJsonRef<'a, T: ?Sized>(pub &'a T);
impl<'a, T> sqlx::Type<Postgres> for FHIRJsonRef<'a, T>
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

impl<'q, T> Encode<'q, Postgres> for FHIRJsonRef<'q, T>
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
        fhir_serialization_json::to_writer(&mut **buf, &*self.0)?;

        Ok(IsNull::No)
    }
}
