use haste_fhir_model::r4::generated::resources::{
    HL7V2Header, HL7V2SegmentsFields, HL7V2SegmentsFieldsValue, HL7V2SegmentsFieldsValueValue,
};
use haste_fhir_model::r4::generated::terminology::IssueType;
use haste_fhir_operation_error::OperationOutcomeError;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Component {
    pub value: Option<String>,
    pub subcomponents: Option<Vec<String>>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct FieldValue {
    pub value: Option<Component>,
    pub components: Option<Vec<Component>>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Field {
    pub value: Option<FieldValue>,
    pub repetitions: Option<Vec<FieldValue>>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Segment {
    pub id: String,
    pub fields: Vec<Field>,
}

#[derive(Debug, Clone)]
pub struct Hl7V2Message {
    pub header: HL7V2Header,
    pub segments: Vec<Segment>,
}

struct ParsedHL7V2Header(HL7V2Header);

impl TryFrom<&str> for ParsedHL7V2Header {
    type Error = OperationOutcomeError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let h = value.get(..3);
        if h != Some("MSH") {
            return Err(OperationOutcomeError::error(
                IssueType::Exception(None),
                "Not an MSH segment".to_string(),
            ));
        }
        let field_separator = value.chars().nth(3).ok_or_else(|| {
            OperationOutcomeError::error(
                IssueType::Exception(None),
                "Missing field separator".to_string(),
            )
        })?;

        let mut fields = value[4..].split(field_separator);

        Ok(ParsedHL7V2Header(HL7V2Header {
            field_separator: Box::new(field_separator.to_string().into()),
            encodingCharacters: Box::new(
                fields
                    .next()
                    .ok_or_else(|| {
                        OperationOutcomeError::error(
                            IssueType::Exception(None),
                            "Missing encoding characters".to_string(),
                        )
                    })?
                    .to_string()
                    .into(),
            ),
            sendingApplication: fields.next().map(|s| Box::new(s.to_string().into())),
            sendingFacility: fields.next().map(|s| Box::new(s.to_string().into())),
            receivingApplication: fields.next().map(|s| Box::new(s.to_string().into())),
            receivingFacility: fields.next().map(|s| Box::new(s.to_string().into())),
            datetimeOfMessage: fields.next().map(|s| Box::new(s.to_string().into())),
            security: fields.next().map(|s| Box::new(s.to_string().into())),
            messageType: fields.next().map(|s| Box::new(s.to_string().into())),
            messageControlId: fields.next().map(|s| Box::new(s.to_string().into())),
            processingId: fields.next().map(|s| Box::new(s.to_string().into())),
            versionId: fields.next().map(|s| Box::new(s.to_string().into())),
            additionalFields: Some(
                fields
                    .map(|f| HL7V2SegmentsFields {
                        value: Some(HL7V2SegmentsFieldsValue {
                            value: Some(HL7V2SegmentsFieldsValueValue {
                                value: Some(Box::new(f.to_string().into())),
                                subcomponents: None,
                            }),
                            components: None,
                        }),
                        repetitions: None,
                    })
                    .collect(),
            ),
        }))
    }
}

impl TryFrom<&str> for Hl7V2Message {
    type Error = OperationOutcomeError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let mut segments = vec![];
        let mut segment_lines = value.lines();

        let message_header =
            ParsedHL7V2Header::try_from(segment_lines.next().ok_or_else(|| {
                OperationOutcomeError::error(
                    IssueType::Invalid(None),
                    "Missing MSH segment".to_string(),
                )
            })?)?
            .0;

        for segment in segment_lines {
            let mut segment = segment.split(
                message_header
                    .field_separator
                    .value
                    .as_ref()
                    .and_then(|s| s.chars().next())
                    .unwrap_or('|'),
            );
            let segment_id = segment.next().ok_or_else(|| {
                OperationOutcomeError::error(
                    IssueType::Exception(None),
                    "Missing segment ID".to_string(),
                )
            })?;

            let segment_fields = segment.map(|field| {
                let fields = field
                    .split(
                        message_header
                            .encodingCharacters
                            .value
                            .as_ref()
                            .and_then(|s| s.chars().nth(1))
                            .unwrap_or('~'),
                    )
                    .map(|field| {
                        let components = field
                            .split(
                                message_header
                                    .encodingCharacters
                                    .value
                                    .as_ref()
                                    .and_then(|s| s.chars().nth(0))
                                    .unwrap_or('^'),
                            )
                            .map(|component| {
                                let subcomponent = component
                                    .split(
                                        message_header
                                            .encodingCharacters
                                            .value
                                            .as_ref()
                                            .and_then(|s| s.chars().nth(3))
                                            .unwrap_or('&'),
                                    )
                                    .collect::<Vec<_>>();
                                if subcomponent.len() > 1 {
                                    Component {
                                        value: None,
                                        subcomponents: Some(
                                            subcomponent.iter().map(|s| s.to_string()).collect(),
                                        ),
                                    }
                                } else {
                                    Component {
                                        value: Some(component.to_string()),
                                        subcomponents: None,
                                    }
                                }
                            })
                            .collect::<Vec<_>>();
                        if components.len() > 1 {
                            FieldValue {
                                value: None,
                                components: Some(components),
                            }
                        } else {
                            FieldValue {
                                value: Some(components.into_iter().next().unwrap()),
                                components: None,
                            }
                        }
                    })
                    .collect::<Vec<_>>();
                if fields.len() > 1 {
                    Field {
                        value: None,
                        repetitions: Some(fields),
                    }
                } else {
                    Field {
                        value: Some(fields.into_iter().next().unwrap()),
                        repetitions: None,
                    }
                }
            });

            segments.push(Segment {
                id: segment_id.to_string(),
                fields: segment_fields.collect(),
            });
        }

        Ok(Hl7V2Message {
            header: message_header,
            segments,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_hl7v2_message() {
        let input = std::fs::read_to_string("./test_data/message1.bin").unwrap();

        let result = Hl7V2Message::try_from(input.as_str());

        assert!(result.is_ok());

        let message = result.unwrap();
        assert_eq!(message.segments.len(), 7);

        assert_eq!(message.segments[0].id, "SCH");
        assert_eq!(message.segments[0].fields.len(), 25);
        assert_eq!(
            message.segments[0].fields[0]
                .value
                .clone()
                .unwrap()
                .components,
            Some(vec![
                Component {
                    value: Some("10345".to_string()),
                    subcomponents: None,
                },
                Component {
                    value: Some("10345".to_string()),
                    subcomponents: None,
                },
            ])
        );

        assert_eq!(message.segments[1].id, "PID");
        assert_eq!(message.segments[2].id, "PV1");
        assert_eq!(message.segments[3].id, "RGS");
        assert_eq!(message.segments[4].id, "AIG");
        assert_eq!(message.segments[5].id, "AIL");
        assert_eq!(message.segments[6].id, "AIP");
    }
}
