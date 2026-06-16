use haste_fhir_model::r4::generated::resources::{
    HL7V2, HL7V2Segments, HL7V2SegmentsFields, HL7V2SegmentsFieldsValue,
    HL7V2SegmentsFieldsValueValue,
};
use haste_fhir_model::r4::generated::terminology::IssueType;
use haste_fhir_model::r4::generated::types::{FHIRId, FHIRString};
use haste_fhir_operation_error::OperationOutcomeError;

#[derive(Debug, Clone)]
pub struct ParsedHL7V2Message(pub HL7V2);

fn parse_empty_string(v: String) -> FHIRString {
    let mut fhir_string: FHIRString = v.into();

    fhir_string.id = Some("_non_empty".to_string());
    fhir_string
}

fn parse_empty_id(v: String) -> FHIRId {
    let mut fhir_id: FHIRId = v.into();

    fhir_id.id = Some("_non_empty".to_string());
    fhir_id
}

impl TryFrom<&str> for ParsedHL7V2Message {
    type Error = OperationOutcomeError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let mut segments = vec![];

        let segment_lines = value.split(['\r', '\n']).filter(|s| !s.is_empty());

        let header = value[..3].to_string();

        if header != "MSH" {
            return Err(OperationOutcomeError::error(
                IssueType::Exception(None),
                "Message does not start with MSH segment".to_string(),
            ));
        }

        let field_seperator = value.chars().nth(3).ok_or_else(|| {
            OperationOutcomeError::error(
                IssueType::Exception(None),
                "Missing field separator".to_string(),
            )
        })?;

        let encoding_characters = value[4..].split(field_seperator).next().ok_or_else(|| {
            OperationOutcomeError::error(
                IssueType::Exception(None),
                "Missing encoding characters".to_string(),
            )
        })?;

        for segment in segment_lines {
            let mut segment = segment.split(field_seperator);
            let segment_id = segment.next().ok_or_else(|| {
                OperationOutcomeError::error(
                    IssueType::Exception(None),
                    "Missing segment ID".to_string(),
                )
            })?;

            let segment_fields = segment.map(|field| {
                let fields = field
                    .split(encoding_characters.chars().nth(1).unwrap_or('~'))
                    .map(|field| {
                        let components = field
                            .split(encoding_characters.chars().nth(0).unwrap_or('^'))
                            .map(|component| {
                                let subcomponent = component
                                    .split(encoding_characters.chars().nth(3).unwrap_or('&'))
                                    .collect::<Vec<_>>();
                                if subcomponent.len() > 1 {
                                    HL7V2SegmentsFieldsValueValue {
                                        value: None,
                                        subcomponents: Some(
                                            subcomponent
                                                .iter()
                                                .map(|s| {
                                                    Box::new(parse_empty_string(s.to_string()))
                                                })
                                                .collect(),
                                        ),
                                    }
                                } else {
                                    HL7V2SegmentsFieldsValueValue {
                                        value: Some(Box::new(parse_empty_string(
                                            component.to_string(),
                                        ))),
                                        subcomponents: None,
                                    }
                                }
                            })
                            .collect::<Vec<_>>();
                        if components.len() > 1 {
                            HL7V2SegmentsFieldsValue {
                                value: None,
                                components: Some(components),
                            }
                        } else {
                            HL7V2SegmentsFieldsValue {
                                value: components.into_iter().next(),
                                components: None,
                            }
                        }
                    })
                    .collect::<Vec<_>>();
                if fields.len() > 1 {
                    HL7V2SegmentsFields {
                        value: None,
                        repetitions: Some(fields),
                    }
                } else {
                    HL7V2SegmentsFields {
                        value: Some(fields.into_iter().next().unwrap()),
                        repetitions: None,
                    }
                }
            });

            segments.push(HL7V2Segments {
                id: Box::new(parse_empty_id(segment_id.to_string())),
                fields: Some(segment_fields.collect()),
            });
        }

        Ok(ParsedHL7V2Message(HL7V2 {
            fieldSeparator: Box::new(parse_empty_string(field_seperator.to_string())),
            segments: Some(segments),
            ..Default::default()
        }))
    }
}

#[cfg(test)]
mod tests {
    use crate::serialize::SerializeMessage;

    use super::*;

    #[test]
    fn test_parse_hl7v2_message() {
        let input = std::fs::read_to_string("./test_data/message1.bin").unwrap();

        let result = ParsedHL7V2Message::try_from(input.as_str());

        assert!(result.is_ok());

        let message = result.unwrap().0;
        let segments = message.segments.expect("message should contain segments");
        assert_eq!(segments.len(), 8);

        assert_eq!(segments[1].id.value.as_deref(), Some("SCH"));

        let sch_fields = segments[1]
            .fields
            .clone()
            .expect("SCH should contain fields");
        assert_eq!(sch_fields.len(), 25);
        assert_eq!(
            sch_fields[0]
                .value
                .clone()
                .unwrap()
                .components
                .unwrap()
                .into_iter()
                .map(|c| c.value.unwrap().value.unwrap())
                .collect::<Vec<_>>(),
            vec!["10345".to_string(), "10345".to_string()]
        );

        assert_eq!(segments[2].id.value.as_deref(), Some("PID"));
        assert_eq!(segments[3].id.value.as_deref(), Some("PV1"));
        assert_eq!(segments[4].id.value.as_deref(), Some("RGS"));
        assert_eq!(segments[5].id.value.as_deref(), Some("AIG"));
        assert_eq!(segments[6].id.value.as_deref(), Some("AIL"));
        assert_eq!(segments[7].id.value.as_deref(), Some("AIP"));
    }

    #[test]
    fn round_trip() {
        let input = std::fs::read_to_string("./test_data/message1.bin").unwrap();
        let result = ParsedHL7V2Message::try_from(input.as_str());
        assert!(result.is_ok());

        let message = result.unwrap();
        let serialized: String = (SerializeMessage(&message.0)).into();

        pretty_assertions::assert_eq!(serialized, input);
    }

    #[test]
    fn round_trip_json_serialize() {
        let input = std::fs::read_to_string("./test_data/message1.bin").unwrap();
        let result = ParsedHL7V2Message::try_from(input.as_str());
        assert!(result.is_ok());

        let message = result.unwrap();
        let json = serde_json::to_string_pretty(&message.0).unwrap();
        println!("{}", json);
        let deserialized: HL7V2 = serde_json::from_str(&json).unwrap();
        let serialized: String = (SerializeMessage(&deserialized)).into();

        pretty_assertions::assert_eq!(serialized, input);
    }
}
