use std::io::BufWriter;
use std::io::Write;
pub use traits::*;

mod deserialize_primitives;
pub mod errors;
mod serialize_primitives;
mod traits;

#[cfg(feature = "derive")]
pub mod derive;

pub fn from_str<T: FHIRJSONDeserializer>(s: &str) -> Result<T, errors::DeserializeError> {
    T::from_json_str(s)
}

pub fn from_bytes<T: FHIRJSONDeserializer>(bytes: &[u8]) -> Result<T, errors::DeserializeError> {
    let value = serde_json::from_slice(bytes)?;
    T::from_serde_value(&value, Context::AsValue)
}

pub fn to_string<T: FHIRJSONSerializer>(value: &T) -> Result<String, SerializeError> {
    let mut writer = BufWriter::new(Vec::new());
    value.serialize_value(&mut writer)?;
    writer.flush()?;
    let content = writer.into_inner()?;

    Ok(String::from_utf8(content)?)
}

pub fn to_writer<T: FHIRJSONSerializer>(
    writer: &mut dyn Write,
    value: &T,
) -> Result<bool, SerializeError> {
    value.serialize_value(writer)
}
