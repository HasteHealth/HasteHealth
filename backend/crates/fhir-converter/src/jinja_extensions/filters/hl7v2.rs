use minijinja::Value;

use crate::jinja_extensions::conversions::hl7v2::JHL7V2;

pub fn hl7v2_segments(value: Value) -> Value {
    if let Some(object) = value.as_object() {
        let _k = object.downcast_ref::<JHL7V2>();
    }

    let none = Value::from("hl7v2_segments".to_string());

    none
}
