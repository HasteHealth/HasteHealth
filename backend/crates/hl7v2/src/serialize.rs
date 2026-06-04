use crate::parser::ParsedHL7V2Message;
use haste_fhir_model::r4::generated::resources::{
    HL7V2, HL7V2Segments, HL7V2SegmentsFields, HL7V2SegmentsFieldsValue,
    HL7V2SegmentsFieldsValueValue,
};

struct EncodingInformation {
    field_separator: String,
    component_separator: String,
    repetition_separator: String,
    #[allow(dead_code)]
    escape_character: String,
    subcomponent_separator: String,
}

// component separator, repetition separator, escape character, and subcomponent separator.

fn component_to_string(
    encoding_characters: &EncodingInformation,
    component: HL7V2SegmentsFieldsValueValue,
) -> Option<String> {
    if let Some(subcomponents) = component.subcomponents {
        let value = subcomponents
            .into_iter()
            .map(|s| s.value)
            .map(|v| if let Some(s) = v { s } else { "".to_string() })
            .collect::<Vec<_>>()
            .join(&encoding_characters.subcomponent_separator);
        Some(value)
    } else {
        component.value.and_then(|s| s.value)
    }
}

fn segment_field_repititon_to_string(
    encoding_characters: &EncodingInformation,
    segment: HL7V2SegmentsFieldsValue,
) -> String {
    let mut result = "".to_string();

    if let Some(components) = segment.components {
        result.push_str(
            &components
                .into_iter()
                .map(|c| component_to_string(encoding_characters, c).unwrap_or_default())
                .collect::<Vec<_>>()
                .join(&encoding_characters.component_separator),
        )
    } else if let Some(value) = segment.value {
        result.push_str(&component_to_string(encoding_characters, value).unwrap_or_default());
    }

    result
}

fn segment_field_to_string(
    encoding_characters: &EncodingInformation,
    segment: HL7V2SegmentsFields,
) -> String {
    let mut result = "".to_string();

    if let Some(repititions) = segment.repetitions {
        result.push_str(
            &repititions
                .into_iter()
                .map(|r| segment_field_repititon_to_string(encoding_characters, r))
                .collect::<Vec<_>>()
                .join(&encoding_characters.repetition_separator),
        );
    } else if let Some(value) = segment.value {
        result.push_str(&segment_field_repititon_to_string(
            encoding_characters,
            value,
        ));
    }

    result
}

fn segment_to_string(encoding_characters: &EncodingInformation, segment: HL7V2Segments) -> String {
    let mut result = segment
        .id
        .value
        .as_ref()
        .map(|s| s.as_str())
        .unwrap_or("")
        .to_string();

    result.push_str(&encoding_characters.field_separator);

    result.push_str(
        &segment
            .fields
            .unwrap_or_default()
            .into_iter()
            .map(|s| segment_field_to_string(encoding_characters, s))
            .collect::<Vec<_>>()
            .join(&encoding_characters.field_separator),
    );

    result
}

fn get_encoding_characters(hl7v2_message: &HL7V2) -> Option<String> {
    let Some(msh) = hl7v2_message.segments.as_ref().and_then(|segments| {
        segments
            .iter()
            .find(|s| s.id.value.as_ref().map(|s| s.as_str()) == Some("MSH"))
    }) else {
        return None;
    };

    let Some(encoding_characters_str) = msh
        .fields
        .as_ref()
        .and_then(|fields| fields.into_iter().next())
        .and_then(|field| {
            field.value.as_ref().and_then(|v| {
                v.value
                    .as_ref()
                    .and_then(|s| s.value.as_ref())
                    .and_then(|s| s.value.as_ref().map(|s| s.as_str()))
            })
        })
    else {
        return None;
    };

    Some(encoding_characters_str.to_string())
}

impl From<ParsedHL7V2Message> for String {
    fn from(value: ParsedHL7V2Message) -> Self {
        let hl7v2_message = value.0;
        let field_seperator = hl7v2_message
            .fieldSeparator
            .value
            .as_ref()
            .map(|s| s.as_str())
            .unwrap_or("|");
        let encoding_characters_str =
            get_encoding_characters(&hl7v2_message).unwrap_or("^~\\&".to_string());

        let mut result = "".to_string();

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

        if let Some(segments) = hl7v2_message.segments {
            let k = segments
                .into_iter()
                .map(|s| segment_to_string(&encoding_characters, s))
                .collect::<Vec<_>>()
                .join("\n");

            result.push_str(&k);
        }

        result
    }
}
