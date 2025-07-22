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

pub fn to_string<T: FHIRJSONSerializer>(value: &T) -> Option<String> {
    let mut writer = BufWriter::new(Vec::new());
    value.serialize_value(&mut writer).ok()?;
    writer.flush().ok()?;

    Some(String::from_utf8(writer.into_inner().ok()?).ok()?)
}
