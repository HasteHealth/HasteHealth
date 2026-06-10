use std::sync::{Arc, LazyLock};

use haste_fhir_model::r4::generated::resources::{
    HL7V2, HL7V2Segments, HL7V2SegmentsFields, HL7V2SegmentsFieldsValue,
};
use haste_hl7v2::serialize::{
    EncodingInformation, SerializeMessage, segment_field_repititon_to_string,
    segment_field_to_string, segment_to_string,
};
use minijinja::{
    Value,
    value::{Enumerator, Object, ObjectRepr},
};

#[derive(Debug)]
#[allow(dead_code)]
pub struct JHL7V2(HL7V2);
impl JHL7V2 {
    pub fn new(hl7v2: HL7V2) -> Self {
        Self(hl7v2)
    }
}

impl Object for JHL7V2 {
    fn repr(self: &Arc<Self>) -> ObjectRepr {
        ObjectRepr::Plain
    }

    fn get_value(self: &Arc<Self>, key: &Value) -> Option<Value> {
        if let Some(key) = key.as_str() {
            let segments = self.0.segments.as_ref()?;

            let found_segments = segments
                .iter()
                .filter(|segment| segment.id.value.as_ref().map(|s| s.as_str()) == Some(key))
                .collect::<Vec<_>>();

            if found_segments.len() == 0 {
                return None;
            }

            return Some(Value::from(
                found_segments
                    .into_iter()
                    .map(|segment| {
                        Value::from_dyn_object(unsafe {
                            let segment = std::mem::transmute::<
                                &HL7V2Segments,
                                &'static HL7V2Segments,
                            >(segment);
                            Arc::new(JHL7V2Segment(segment))
                        })
                    })
                    .collect::<Vec<_>>(),
            ));
        }

        None
    }

    fn enumerate(self: &Arc<Self>) -> Enumerator {
        Enumerator::NonEnumerable
    }

    fn render(self: &Arc<Self>, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    where
        Self: Sized + 'static,
    {
        let hl7v2_string: String = SerializeMessage(&self.0).into();
        write!(f, "{}", hl7v2_string)
    }
}

static DEFAULT_ENCODING: LazyLock<EncodingInformation> = LazyLock::new(|| EncodingInformation {
    field_separator: "|".to_string(),
    component_separator: "^".to_string(),
    repetition_separator: "~".to_string(),
    escape_character: "\\".to_string(),
    subcomponent_separator: "&".to_string(),
});

#[derive(Debug)]
pub struct JHL7V2Segment<'a>(&'a HL7V2Segments);
impl<'a> Object for JHL7V2Segment<'a> {
    fn repr(self: &Arc<Self>) -> ObjectRepr {
        ObjectRepr::Plain
    }

    fn get_value(self: &Arc<Self>, key: &Value) -> Option<Value> {
        if let Some(index) = key.as_usize() {
            self.0.fields.as_ref()?.get(index).map(|field| {
                Value::from_dyn_object(unsafe {
                    let field = std::mem::transmute::<
                        &HL7V2SegmentsFields,
                        &'static HL7V2SegmentsFields,
                    >(field);

                    Arc::new(JHL7V2SegmentsFields(field))
                })
            })
        } else {
            None
        }
    }

    fn render(self: &Arc<Self>, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    where
        Self: Sized + 'static,
    {
        write!(f, "{}", segment_to_string(&*DEFAULT_ENCODING, self.0))
    }
}

#[derive(Debug)]
pub struct JHL7V2SegmentsFields<'a>(&'a HL7V2SegmentsFields);
impl<'a> Object for JHL7V2SegmentsFields<'a> {
    fn repr(self: &Arc<Self>) -> ObjectRepr {
        ObjectRepr::Plain
    }

    fn get_value(self: &Arc<Self>, key: &Value) -> Option<Value> {
        let _ = key;
        None
    }

    fn get_value_by_str(self: &Arc<Self>, key: &str) -> Option<Value> {
        self.get_value(&Value::from(key))
    }

    fn render(self: &Arc<Self>, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    where
        Self: Sized + 'static,
    {
        write!(f, "{}", segment_field_to_string(&*DEFAULT_ENCODING, self.0))
    }
}

#[derive(Debug)]
pub struct JHL7V2SegmentsFieldsValue<'a>(&'a HL7V2SegmentsFieldsValue);
impl<'a> Object for JHL7V2SegmentsFieldsValue<'a> {
    fn repr(self: &Arc<Self>) -> ObjectRepr {
        ObjectRepr::Plain
    }

    fn get_value(self: &Arc<Self>, key: &Value) -> Option<Value> {
        let _ = key;
        None
    }

    fn get_value_by_str(self: &Arc<Self>, key: &str) -> Option<Value> {
        self.get_value(&Value::from(key))
    }

    fn render(self: &Arc<Self>, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    where
        Self: Sized + 'static,
    {
        write!(
            f,
            "{}",
            segment_field_repititon_to_string(&*DEFAULT_ENCODING, self.0)
        )
    }
}
