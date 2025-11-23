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

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct MessageHeader {
    field_separator: char,
    /// component separator, repetition separator, escape character, and subcomponent separator
    encoding_characters: String,
    sending_application: Option<String>,
    sending_facility: Option<String>,
    receiving_application: Option<String>,
    receiving_facility: Option<String>,
    datetime_of_message: Option<String>,
    security: Option<String>,
    message_type: Option<String>,
    message_control_id: Option<String>,
    processing_id: Option<String>,
    version_id: Option<String>,

    additional_fields: Vec<Field>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Hl7V2Message {
    pub header: MessageHeader,
    pub segments: Vec<Segment>,
}

impl TryFrom<&str> for MessageHeader {
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

        Ok(MessageHeader {
            field_separator: field_separator,
            encoding_characters: fields
                .next()
                .ok_or_else(|| {
                    OperationOutcomeError::error(
                        IssueType::Exception(None),
                        "Missing encoding characters".to_string(),
                    )
                })?
                .to_string(),
            sending_application: fields.next().map(|s| s.to_string()),
            sending_facility: fields.next().map(|s| s.to_string()),
            receiving_application: fields.next().map(|s| s.to_string()),
            receiving_facility: fields.next().map(|s| s.to_string()),
            datetime_of_message: (fields.next().map(|s| s.to_string())),
            security: fields.next().map(|s| s.to_string()),
            message_type: fields.next().map(|s| s.to_string()),
            message_control_id: fields.next().map(|s| s.to_string()),
            processing_id: fields.next().map(|s| s.to_string()),
            version_id: fields.next().map(|s| s.to_string()),
            additional_fields: fields
                .map(|f| Field {
                    value: Some(FieldValue {
                        value: Some(Component {
                            value: Some(f.to_string()),
                            subcomponents: None,
                        }),
                        components: None,
                    }),
                    repetitions: None,
                })
                .collect(),
        })
    }
}

impl TryFrom<&str> for Hl7V2Message {
    type Error = OperationOutcomeError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let mut segments = vec![];
        let mut segment_lines = value.lines();

        let message_header = MessageHeader::try_from(segment_lines.next().ok_or_else(|| {
            OperationOutcomeError::error(
                IssueType::Invalid(None),
                "Missing MSH segment".to_string(),
            )
        })?)?;

        for segment in segment_lines {
            let mut segment = segment.split(message_header.field_separator);
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
                            .encoding_characters
                            .chars()
                            .nth(1)
                            .unwrap_or('~'),
                    )
                    .map(|field| {
                        let components = field
                            .split(
                                message_header
                                    .encoding_characters
                                    .chars()
                                    .nth(0)
                                    .unwrap_or('^'),
                            )
                            .map(|component| {
                                let subcomponent = field
                                    .split(
                                        message_header
                                            .encoding_characters
                                            .chars()
                                            .nth(3)
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
        println!("{:#?}", result);
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
