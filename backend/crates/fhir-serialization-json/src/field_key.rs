use std::borrow::Cow;
use std::fmt;

use serde::de::{Deserialize, Deserializer, Error, Visitor};

/// An object key read during deserialization. It borrows from the input when
/// the deserializer can lend the text and owns it otherwise.
///
/// The implementations `FHIRSerdeDeserialize` generates for a `complex` and a
/// `resource` type read every key through this type, so a crate that derives
/// one depends on this crate under its own name. Reading with
/// `serde_json::from_str` or `from_slice` lends a key as it is written in the
/// document. A key written with an escape, and one read from a reader or from
/// a `serde_json::Value`, is owned.
pub struct FieldKey<'de>(Cow<'de, str>);

impl FieldKey<'_> {
    /// The key's text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl<'de> Deserialize<'de> for FieldKey<'de> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct KeyVisitor;

        impl<'de> Visitor<'de> for KeyVisitor {
            type Value = FieldKey<'de>;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("a string")
            }

            fn visit_borrowed_str<E: Error>(self, v: &'de str) -> Result<Self::Value, E> {
                Ok(FieldKey(Cow::Borrowed(v)))
            }

            fn visit_str<E: Error>(self, v: &str) -> Result<Self::Value, E> {
                Ok(FieldKey(Cow::Owned(v.to_owned())))
            }

            fn visit_string<E: Error>(self, v: String) -> Result<Self::Value, E> {
                Ok(FieldKey(Cow::Owned(v)))
            }
        }

        deserializer.deserialize_str(KeyVisitor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::de::{IgnoredAny, MapAccess};

    /// Each key of a JSON object in the order it is read, with whether it was
    /// borrowed from the input.
    #[derive(Debug, PartialEq)]
    struct Keys(Vec<(String, bool)>);

    impl<'de> Deserialize<'de> for Keys {
        fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
            struct KeysVisitor;

            impl<'de> Visitor<'de> for KeysVisitor {
                type Value = Keys;

                fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                    f.write_str("a JSON object")
                }

                fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
                    let mut keys = Vec::new();
                    while let Some(key) = map.next_key::<FieldKey<'de>>()? {
                        map.next_value::<IgnoredAny>()?;
                        keys.push((key.as_str().to_owned(), matches!(key.0, Cow::Borrowed(_))));
                    }
                    Ok(Keys(keys))
                }
            }

            deserializer.deserialize_map(KeysVisitor)
        }
    }

    fn keys<const N: usize>(keys: [(&str, bool); N]) -> Keys {
        Keys(
            keys.into_iter()
                .map(|(key, borrowed)| (key.to_owned(), borrowed))
                .collect(),
        )
    }

    const DOCUMENT: &str = r#"{"_birthDate": {}, "id": "a", "valueString": "b"}"#;

    #[test]
    fn a_key_is_borrowed_from_a_str_and_from_a_slice() {
        let borrowed = keys([("_birthDate", true), ("id", true), ("valueString", true)]);

        assert_eq!(serde_json::from_str::<Keys>(DOCUMENT).unwrap(), borrowed);
        assert_eq!(
            serde_json::from_slice::<Keys>(DOCUMENT.as_bytes()).unwrap(),
            borrowed
        );
    }

    #[test]
    fn a_key_written_with_an_escape_is_owned() {
        assert_eq!(
            serde_json::from_str::<Keys>(r#"{"\u0069d": "a", "name": []}"#).unwrap(),
            keys([("id", false), ("name", true)])
        );
    }

    #[test]
    fn a_key_from_a_reader_or_a_value_is_owned() {
        let owned = keys([("_birthDate", false), ("id", false), ("valueString", false)]);

        assert_eq!(
            serde_json::from_reader::<_, Keys>(DOCUMENT.as_bytes()).unwrap(),
            owned
        );

        // The keys of `DOCUMENT` are sorted, as a `Value` keeps them unless
        // `preserve_order` is on.
        let value: serde_json::Value = serde_json::from_str(DOCUMENT).unwrap();
        assert_eq!(serde_json::from_value::<Keys>(value).unwrap(), owned);
    }
}
