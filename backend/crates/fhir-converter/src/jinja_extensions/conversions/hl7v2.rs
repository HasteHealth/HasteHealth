use std::sync::{Arc, LazyLock};

use haste_fhir_model::r4::generated::resources::{
    HL7V2, HL7V2Segments, HL7V2SegmentsFields, HL7V2SegmentsFieldsValue,
    HL7V2SegmentsFieldsValueValue,
};
use haste_hl7v2::serialize::{
    EncodingInformation, SerializeMessage, component_to_string, segment_field_repetition_to_string,
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
                .filter(|segment| segment.id.value.as_deref() == Some(key))
                .collect::<Vec<_>>();

            if found_segments.is_empty() {
                return None;
            }

            let field_separator = self.0.fieldSeparator.value.as_deref().unwrap_or("|");

            return Some(Value::from(
                found_segments
                    .into_iter()
                    .map(|segment| {
                        Value::from_dyn_object(unsafe {
                            let segment = std::mem::transmute::<
                                &HL7V2Segments,
                                &'static HL7V2Segments,
                            >(segment);
                            let field_separator =
                                std::mem::transmute::<&str, &'static str>(field_separator);
                            Arc::new(JHL7V2Segment::new(segment, field_separator))
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
        write!(f, "{hl7v2_string}")
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
pub struct JHL7V2Segment<'a> {
    segment: &'a HL7V2Segments,
    /// The message's field separator, which *is* `MSH-1`. Held so `MSH[1]`
    /// can answer with it; see [`JHL7V2Segment::field_index`].
    field_separator: &'a str,
}

impl<'a> JHL7V2Segment<'a> {
    fn new(segment: &'a HL7V2Segments, field_separator: &'a str) -> Self {
        Self {
            segment,
            field_separator,
        }
    }

    fn is_msh(&self) -> bool {
        self.segment.id.value.as_deref() == Some("MSH")
    }

    /// Where HL7 field number `number` sits in the parsed `fields` array.
    ///
    /// Templates address fields by their HL7 number -- `PID[5]` is the patient
    /// name, as the standard numbers it. The parser drops the segment id, so an
    /// ordinary segment's field `N` is stored at `fields[N - 1]`.
    ///
    /// `MSH` is shifted once more. Its field separator *is* `MSH-1`, and it is
    /// consumed as the delimiter rather than stored, so the first stored value
    /// is `MSH-2` (the encoding characters) and field `N` is at `fields[N - 2]`.
    /// `MSH-1` has no array slot and is answered from the separator instead.
    fn field_index(&self, number: usize) -> Option<usize> {
        if self.is_msh() {
            // MSH-1 is the separator; the caller handles it.
            number.checked_sub(2)
        } else {
            number.checked_sub(1)
        }
    }
}

impl Object for JHL7V2Segment<'_> {
    fn repr(self: &Arc<Self>) -> ObjectRepr {
        ObjectRepr::Plain
    }

    fn get_value(self: &Arc<Self>, key: &Value) -> Option<Value> {
        let number = key.as_usize()?;

        // There is no field 0 in HL7 v2; numbering starts at 1. Answering it
        // with the first field would let an off-by-one look like it works.
        if number == 0 {
            return None;
        }

        // `MSH-1` is the field separator itself, which has no array slot.
        if self.is_msh() && number == 1 {
            return Some(Value::from_safe_string(self.field_separator.to_string()));
        }

        let index = self.field_index(number)?;

        self.segment.fields.as_ref()?.get(index).map(|field| {
            Value::from_dyn_object(unsafe {
                let field = std::mem::transmute::<
                    &HL7V2SegmentsFields,
                    &'static HL7V2SegmentsFields,
                >(field);

                Arc::new(JHL7V2SegmentsFields(field))
            })
        })
    }

    fn render(self: &Arc<Self>, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    where
        Self: Sized + 'static,
    {
        write!(f, "{}", segment_to_string(&DEFAULT_ENCODING, self.segment))
    }
}

#[derive(Debug)]
pub struct JHL7V2SegmentsFields<'a>(&'a HL7V2SegmentsFields);
impl Object for JHL7V2SegmentsFields<'_> {
    fn repr(self: &Arc<Self>) -> ObjectRepr {
        ObjectRepr::Plain
    }

    fn get_value(self: &Arc<Self>, key: &Value) -> Option<Value> {
        let number = key.as_usize()?;
        if number == 0 {
            return None;
        }
        // Repetitions are numbered from 1, like everything else in HL7 v2.
        let index = number - 1;

        // if value is present pass key down chain otherwise treat as repetition which requires an indice.
        if let Some(field_value) = self.0.value.as_ref() {
            Arc::new(JHL7V2SegmentsFieldsValue(field_value)).get_value(key)
        } else {
            self.0.repetitions.as_ref()?.get(index).map(|field| {
                Value::from_dyn_object(unsafe {
                    let field_value = std::mem::transmute::<
                        &HL7V2SegmentsFieldsValue,
                        &'static HL7V2SegmentsFieldsValue,
                    >(field);
                    Arc::new(JHL7V2SegmentsFieldsValue(field_value))
                })
            })
        }
    }

    fn get_value_by_str(self: &Arc<Self>, key: &str) -> Option<Value> {
        self.get_value(&Value::from(key))
    }

    fn render(self: &Arc<Self>, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    where
        Self: Sized + 'static,
    {
        write!(f, "{}", segment_field_to_string(&DEFAULT_ENCODING, self.0))
    }
}

#[derive(Debug)]
pub struct JHL7V2SegmentsFieldsValue<'a>(&'a HL7V2SegmentsFieldsValue);
impl Object for JHL7V2SegmentsFieldsValue<'_> {
    fn repr(self: &Arc<Self>) -> ObjectRepr {
        ObjectRepr::Plain
    }

    fn get_value(self: &Arc<Self>, key: &Value) -> Option<Value> {
        let number = key.as_usize()?;
        if number == 0 {
            return None;
        }
        // Components are numbered from 1: `PID-5.1` is the family name.
        let index = number - 1;

        // A field with no component separator is its own first component, so
        // `.1` answers with the whole value.
        if index == 0
            && let Some(component) = self.0.value.as_ref()
        {
            Some(Value::from_dyn_object(unsafe {
                let components = std::mem::transmute::<
                    &HL7V2SegmentsFieldsValueValue,
                    &'static HL7V2SegmentsFieldsValueValue,
                >(component);
                Arc::new(JHL7V2SegmentsFieldComponent(components))
            }))
        } else {
            self.0.components.as_ref()?.get(index).map(|component| {
                Value::from_dyn_object(unsafe {
                    let components = std::mem::transmute::<
                        &HL7V2SegmentsFieldsValueValue,
                        &'static HL7V2SegmentsFieldsValueValue,
                    >(component);
                    Arc::new(JHL7V2SegmentsFieldComponent(components))
                })
            })
        }
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
            segment_field_repetition_to_string(&DEFAULT_ENCODING, self.0)
        )
    }
}

#[derive(Debug)]
pub struct JHL7V2SegmentsFieldComponent<'a>(&'a HL7V2SegmentsFieldsValueValue);
impl Object for JHL7V2SegmentsFieldComponent<'_> {
    fn repr(self: &Arc<Self>) -> ObjectRepr {
        ObjectRepr::Plain
    }
    fn get_value(self: &Arc<Self>, key: &Value) -> Option<Value> {
        let number = key.as_usize()?;
        if number == 0 {
            return None;
        }
        // Subcomponents are numbered from 1, as components are.
        let index = number - 1;

        if index == 0
            && let Some(value) = self.0.value.as_ref()
        {
            Some(Value::from_safe_string(
                value.value.as_deref().map_or("", |s| s).to_string(),
            ))
        } else {
            self.0
                .subcomponents
                .as_ref()?
                .get(index)
                .map(|subcomponent| {
                    Value::from_safe_string(
                        subcomponent.value.as_deref().map_or("", |s| s).to_string(),
                    )
                })
        }
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
            component_to_string(&DEFAULT_ENCODING, self.0).unwrap_or_default()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use haste_hl7v2::parser::ParsedHL7V2Message;
    use minijinja::Environment;

    const MESSAGE: &str = include_str!("../../../test_data/message1.bin");

    /// Renders one expression against the test message.
    fn render(expr: &str) -> String {
        let parsed = ParsedHL7V2Message::try_from(MESSAGE)
            .expect("message parses")
            .0;
        let value = Value::from_dyn_object(Arc::new(JHL7V2::new(parsed)));

        let env = Environment::new();
        env.render_str(
            &format!("{{{{ {expr} }}}}"),
            minijinja::context! { hl7v2 => value },
        )
        .expect("template renders")
    }

    /// Fields are addressed by their HL7 number. `PID-5` is the patient name,
    /// stored at `fields[4]`, so reading `PID[0][5]` relies on the accessor
    /// applying the offset.
    #[test]
    fn a_field_is_addressed_by_its_hl7_number() {
        assert_eq!(render("hl7v2.PID[0][5][1]"), "BEEBLEBROX");
        assert_eq!(render("hl7v2.PID[0][5][2]"), "ZAPHOD");
    }

    /// The bug this guards. The old accessor indexed the array directly, so a
    /// template asking for the MRN as `PID[0][6]` landed on array slot 6,
    /// which is PID-7 -- the birth date. Under HL7 numbering the MRN is
    /// `PID[0][3]` and the birth date is `PID[0][7]`, and each returns its own
    /// value.
    #[test]
    fn pid_fields_land_on_the_field_the_number_names() {
        assert_eq!(render("hl7v2.PID[0][3][1]"), "42", "PID-3 is the MRN");
        assert_eq!(
            render("hl7v2.PID[0][7][1]"),
            "19781012",
            "PID-7 is the birth date"
        );
        assert_eq!(render("hl7v2.PID[0][8][1]"), "M", "PID-8 is the sex");
        // PID-4 is empty in this message, which is itself the point: the old
        // indexing could never return an empty field where a value existed.
        assert_eq!(render("hl7v2.PID[0][4][1]"), "");
    }

    /// MSH is shifted one further than every other segment: its field
    /// separator is MSH-1 and is consumed as the delimiter, so the first
    /// stored field is MSH-2.
    #[test]
    fn msh_fields_account_for_the_separator_being_field_one() {
        // MSH-9 is the message type; the old indexing returned 'P' (MSH-11).
        assert_eq!(render("hl7v2.MSH[0][9][1]"), "SIU");
        assert_eq!(render("hl7v2.MSH[0][9][2]"), "S12");
        assert_eq!(render("hl7v2.MSH[0][10][1]"), "24916560");
        assert_eq!(render("hl7v2.MSH[0][12][1]"), "2.3");
        assert_eq!(render("hl7v2.MSH[0][3][1]"), "SENDING_APPLICATION");
        assert_eq!(render("hl7v2.MSH[0][5][1]"), "RECEIVING_APPLICATION");
    }

    /// MSH-1 has no array slot; it is the separator character itself.
    #[test]
    fn msh_1_is_the_field_separator() {
        assert_eq!(render("hl7v2.MSH[0][1]"), "|");
    }

    /// MSH-2 is the encoding characters, the first value the parser stores.
    ///
    /// Its value contains the component separator, so the parser splits it
    /// into components like any other field; `.1` is the first of them. The
    /// whole field renders when no component is named.
    #[test]
    fn msh_2_is_the_encoding_characters() {
        assert_eq!(render("hl7v2.MSH[0][2]"), "^~\\&");
        assert_eq!(render("hl7v2.MSH[0][2][1]"), "^");
    }

    /// There is no field, component or subcomponent zero. Answering index 0
    /// with the first element is what let an off-by-one look correct.
    #[test]
    fn field_zero_does_not_resolve() {
        for expr in ["hl7v2.PID[0][0]", "hl7v2.MSH[0][0]", "hl7v2.PID[0][5][0]"] {
            assert_eq!(render(expr), "", "{expr} should not resolve");
        }
    }

    /// Segment selection is an ordinary list, so it stays zero-based: `[0]` is
    /// the first segment of that name. Only HL7's own numbering is 1-based.
    #[test]
    fn segment_selection_stays_zero_based() {
        assert_eq!(render("hl7v2.PID[0][5][1]"), "BEEBLEBROX");
        assert_eq!(render("hl7v2.PID[1]"), "");
    }

    /// A field with no component separator is its own first component.
    #[test]
    fn a_single_valued_field_answers_component_one() {
        assert_eq!(render("hl7v2.PID[0][3][1]"), "42");
    }
}
