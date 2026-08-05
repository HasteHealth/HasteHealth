use haste_fhir_model::r4::generated::resources::{
    HL7V2, HL7V2Segments, HL7V2SegmentsFields, HL7V2SegmentsFieldsValue,
    HL7V2SegmentsFieldsValueValue,
};

pub struct EncodingInformation {
    pub field_separator: String,
    pub component_separator: String,
    pub repetition_separator: String,
    #[allow(dead_code)]
    pub escape_character: String,
    pub subcomponent_separator: String,
}

#[must_use]
pub fn component_to_string(
    encoding_characters: &EncodingInformation,
    component: &HL7V2SegmentsFieldsValueValue,
) -> Option<String> {
    if let Some(subcomponents) = &component.subcomponents {
        let value = subcomponents
            .iter()
            .map(|s| &s.value)
            .map(|v| {
                if let Some(s) = v {
                    s.clone()
                } else {
                    String::new()
                }
            })
            .collect::<Vec<_>>()
            .join(&encoding_characters.subcomponent_separator);
        Some(value)
    } else {
        component.value.as_ref().and_then(|s| s.value.clone())
    }
}

#[must_use]
pub fn segment_field_repetition_to_string(
    encoding_characters: &EncodingInformation,
    segment: &HL7V2SegmentsFieldsValue,
) -> String {
    let mut result = String::new();

    if let Some(components) = &segment.components {
        result.push_str(
            &components
                .iter()
                .map(|c| component_to_string(encoding_characters, c).unwrap_or_default())
                .collect::<Vec<_>>()
                .join(&encoding_characters.component_separator),
        );
    } else if let Some(value) = &segment.value {
        result.push_str(&component_to_string(encoding_characters, value).unwrap_or_default());
    }

    result
}

#[must_use]
pub fn segment_field_to_string(
    encoding_characters: &EncodingInformation,
    segment: &HL7V2SegmentsFields,
) -> String {
    let mut result = String::new();

    if let Some(repetitions) = &segment.repetitions {
        result.push_str(
            &repetitions
                .iter()
                .map(|r| segment_field_repetition_to_string(encoding_characters, r))
                .collect::<Vec<_>>()
                .join(&encoding_characters.repetition_separator),
        );
    } else if let Some(value) = &segment.value {
        result.push_str(&segment_field_repetition_to_string(
            encoding_characters,
            value,
        ));
    }

    result
}

#[must_use]
pub fn segment_to_string(
    encoding_characters: &EncodingInformation,
    segment: &HL7V2Segments,
) -> String {
    let mut result = segment.id.value.as_deref().map_or("", |s| s).to_string();

    result.push_str(&encoding_characters.field_separator);

    let default_fields = Vec::new();
    result.push_str(
        &segment
            .fields
            .as_ref()
            .unwrap_or(&default_fields)
            .iter()
            .map(|s| segment_field_to_string(encoding_characters, s))
            .collect::<Vec<_>>()
            .join(&encoding_characters.field_separator),
    );

    result
}

fn get_encoding_characters(hl7v2_message: &HL7V2) -> Option<String> {
    let msh = hl7v2_message.segments.as_ref().and_then(|segments| {
        segments
            .iter()
            .find(|s| s.id.value.as_deref() == Some("MSH"))
    })?;

    let encoding_characters_str = msh
        .fields
        .as_ref()
        .and_then(|fields| fields.iter().next())
        .and_then(|field| {
            field.value.as_ref().and_then(|v| {
                v.value
                    .as_ref()
                    .and_then(|s| s.value.as_ref())
                    .and_then(|s| s.value.as_deref())
            })
        })?;

    Some(encoding_characters_str.to_string())
}

pub struct SerializeMessage<'a>(pub &'a HL7V2);

impl<'a> From<SerializeMessage<'a>> for String {
    fn from(value: SerializeMessage<'a>) -> Self {
        let hl7v2_message = value.0;
        let field_seperator = hl7v2_message
            .fieldSeparator
            .value
            .as_deref()
            .map_or("|", |s| s);
        let encoding_characters_str =
            get_encoding_characters(hl7v2_message).unwrap_or("^~\\&".to_string());

        let mut result = String::new();

        let encoding_characters = EncodingInformation {
            field_separator: field_seperator.to_string(),
            component_separator: encoding_characters_str
                .chars()
                .nth(0)
                .unwrap_or('^')
                .to_string(),
            repetition_separator: encoding_characters_str
                .chars()
                .nth(1)
                .unwrap_or('~')
                .to_string(),
            escape_character: encoding_characters_str
                .chars()
                .nth(2)
                .unwrap_or('\\')
                .to_string(),

            subcomponent_separator: encoding_characters_str
                .chars()
                .nth(3)
                .unwrap_or('&')
                .to_string(),
        };

        if let Some(segments) = &hl7v2_message.segments {
            let k = segments
                .iter()
                .map(|s| segment_to_string(&encoding_characters, s))
                .collect::<Vec<_>>()
                .join("\n");

            result.push_str(&k);
        }

        result
    }
}
