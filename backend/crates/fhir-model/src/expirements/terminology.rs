use crate::r4::generated::types::Element;
use haste_reflect::MetaValue;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::{any::Any, fmt, marker::PhantomData};

pub trait ValueSetDef: 'static + Send + Sync {
    const URL: &'static str;
    const CODES: &'static [&'static str]; // sorted; codegen enforces
}

pub struct BoundCode<VS: ValueSetDef> {
    code: Option<u16>, // index into VS::CODES; None = today's `Null` variant
    element: Option<Element>,
    _vs: PhantomData<VS>,
}

impl<VS: ValueSetDef> BoundCode<VS> {
    pub const fn from_index(i: u16) -> Self {
        Self {
            code: Some(i),
            element: None,
            _vs: PhantomData,
        }
    }
    pub const fn null() -> Self {
        Self {
            code: None,
            element: None,
            _vs: PhantomData,
        }
    }

    pub fn new(s: &str) -> Option<Self> {
        VS::CODES
            .binary_search(&s)
            .ok()
            .map(|i| Self::from_index(i as u16))
    }
    pub fn as_str(&self) -> Option<&'static str> {
        self.code.map(|i| VS::CODES[i as usize])
    }
    pub fn element(&self) -> Option<&Element> {
        self.element.as_ref()
    }
    pub fn element_mut(&mut self) -> &mut Element {
        self.element.get_or_insert_with(Default::default)
    }
}

impl<VS: ValueSetDef> Clone for BoundCode<VS> {
    fn clone(&self) -> Self {
        Self {
            code: self.code,
            element: self.element.clone(),
            _vs: PhantomData,
        }
    }
}

impl<VS: ValueSetDef> fmt::Debug for BoundCode<VS> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // e.g. `AdministrativeGender("male")`
        write!(f, "{}({:?})", std::any::type_name::<VS>(), self.as_str())
    }
}

// Code-index equality only, per the earlier discussion: `g == AdministrativeGender::MALE`
// must not go false because the value carries an extension.
impl<VS: ValueSetDef> PartialEq for BoundCode<VS> {
    fn eq(&self, other: &Self) -> bool {
        self.code == other.code
    }
}
impl<VS: ValueSetDef> Eq for BoundCode<VS> {}

impl<VS: ValueSetDef> MetaValue for BoundCode<VS> {
    fn fields(&self) -> Vec<&'static str> {
        vec!["value", "id", "extension"]
    }

    fn get_field<'a>(&'a self, field: &str) -> Option<&'a dyn MetaValue> {
        match field {
            "value" => self
                .code
                .as_ref()
                .map(|i| &VS::CODES[*i as usize] as &dyn MetaValue),
            _ => self.element.as_ref().and_then(|e| e.get_field(field)),
        }
    }

    fn get_field_mut<'a>(&'a mut self, field: &str) -> Option<&'a mut dyn MetaValue> {
        match field {
            "value" => None,
            _ => self.element.as_mut().and_then(|e| e.get_field_mut(field)),
        }
    }

    fn get_index<'a>(&'a self, _index: usize) -> Option<&'a dyn MetaValue> {
        None
    }

    fn get_index_mut<'a>(&'a mut self, _index: usize) -> Option<&'a mut dyn MetaValue> {
        None
    }

    fn flatten(&self) -> Vec<&dyn MetaValue> {
        vec![self]
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn fhir_type(&self) -> &'static str {
        "code"
    }

    fn is_many(&self) -> bool {
        false
    }
}

// Non-generic over VS — monomorphizes per Deserializer, i.e. ~once.
fn parse_code<E: serde::de::Error>(
    codes: &'static [&'static str],
    url: &'static str,
    s: &str,
) -> Result<u16, E> {
    codes
        .binary_search(&s)
        .map(|i| i as u16)
        .map_err(|_| E::custom(format_args!("'{s}' is not a valid code in ValueSet {url}")))
}

impl<'de, VS: ValueSetDef> Deserialize<'de> for BoundCode<VS> {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        // Element/_field handling stays at the parent-struct level,
        // exactly as your current derive does it.
        let s = <&str>::deserialize(d)?; // or Cow, matching current behavior
        parse_code::<D::Error>(VS::CODES, VS::URL, s).map(Self::from_index)
    }
}

impl<VS: ValueSetDef> Serialize for BoundCode<VS> {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        match self.as_str() {
            Some(c) => s.serialize_str(c),
            None => s.serialize_none(), // Null case; match current `_field`-only semantics
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use serde_json;

    #[doc = "http://hl7.org/fhir/ValueSet/administrative-gender"]
    pub struct AdministrativeGender;

    impl ValueSetDef for AdministrativeGender {
        const URL: &'static str = "http://hl7.org/fhir/ValueSet/administrative-gender";
        const CODES: &'static [&'static str] = &["female", "male", "other", "unknown"];
    }

    impl AdministrativeGender {
        #[doc = "Female."]
        pub const FEMALE: BoundCode<Self> = BoundCode::from_index(0);
        #[doc = "Male."]
        pub const MALE: BoundCode<Self> = BoundCode::from_index(1);
        #[doc = "Other."]
        pub const OTHER: BoundCode<Self> = BoundCode::from_index(2);
        #[doc = "Unknown."]
        pub const UNKNOWN: BoundCode<Self> = BoundCode::from_index(3);
        #[doc = "Element present without a value."]
        pub const NULL: BoundCode<Self> = BoundCode::null();
    }

    #[test]
    fn expiremental_valueset_structs() {
        let gender = AdministrativeGender::MALE;
        let serialized = serde_json::to_string(&gender).unwrap();

        assert_eq!(serialized, r#""male""#);
        let deserialized: BoundCode<AdministrativeGender> =
            serde_json::from_str(&serialized).unwrap();

        assert_eq!(gender, deserialized);
    }

    #[test]
    fn bad_value_expiremental_valueset_structs() {
        let serialized = r#""bad-value""#;
        let deserialized: Result<BoundCode<AdministrativeGender>, _> =
            serde_json::from_str(&serialized);

        assert!(deserialized.is_err());
    }
}
