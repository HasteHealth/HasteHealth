use serde::de::DeserializeOwned;
use std::io::BufWriter;
use std::io::Write;
pub use traits::*;

mod deserialize_primitives;
pub mod errors;
mod serialize_primitives;
mod traits;

#[cfg(feature = "derive")]
pub mod derive;

pub fn from_str<'de, T: serde::Deserialize<'de>>(
    s: &'de str,
) -> Result<T, errors::DeserializeError> {
    let value = serde_json::from_str::<T>(s)?;
    Ok(value)
}

pub fn from_bytes<'de, T: serde::Deserialize<'de>>(
    bytes: &'de [u8],
) -> Result<T, errors::DeserializeError> {
    let value = serde_json::from_slice::<T>(bytes)?;
    Ok(value)
}

pub fn from_serde_value<'de, T: DeserializeOwned>(
    value: serde_json::Value,
) -> Result<T, errors::DeserializeError> {
    let v = serde_json::from_value(value)?;

    Ok(v)
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
