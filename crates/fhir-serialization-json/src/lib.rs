mod deserialize_primitives;
pub mod errors;
mod serialize_primitives;
mod traits;
pub use traits::*;

#[cfg(feature = "derive")]
pub mod derive;

pub fn from_str<T: FHIRJSONDeserializer>(s: &str) -> Result<T, errors::DeserializeError> {
    T::from_json_str(s)
}

pub fn to_string<T: FHIRJSONSerializer>(value: &T) -> Option<String> {
    value.serialize_value()
}
