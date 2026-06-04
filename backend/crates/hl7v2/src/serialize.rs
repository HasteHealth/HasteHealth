use crate::parser::ParsedHL7V2Header;

impl From<ParsedHL7V2Header> for String {
    fn from(value: ParsedHL7V2Header) -> Self {
        let hl7v2_message = value.0;
        let field_seperator = hl7v2_message
            .field_separator
            .value
            .unwrap_or('|'.to_string());
        let mut result = [
            "MSH",
            &field_seperator,
            &hl7v2_message
                .encodingCharacters
                .value
                .as_ref()
                .map(|s| s.as_str())
                .unwrap_or(""),
            hl7v2_message
                .sendingApplication
                .as_ref()
                .and_then(|s| s.value.as_ref())
                .map(|s| s.as_str())
                .unwrap_or(""),
            hl7v2_message
                .sendingFacility
                .as_ref()
                .and_then(|s| s.value.as_ref())
                .map(|s| s.as_str())
                .unwrap_or(""),
            hl7v2_message
                .receivingApplication
                .as_ref()
                .and_then(|s| s.value.as_ref())
                .map(|s| s.as_str())
                .unwrap_or(""),
            hl7v2_message
                .receivingFacility
                .as_ref()
                .and_then(|s| s.value.as_ref())
                .map(|s| s.as_str())
                .unwrap_or(""),
        ]
        .join(&field_seperator);

        if let Some(timestamp) = value.timestamp.value {
            result.push_str(&timestamp);
        }

        if let Some(security) = value.security.value {
            result.push_str(&security);
        }

        result
    }
}
