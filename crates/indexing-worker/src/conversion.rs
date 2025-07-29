/// Reference of conversions found here https://www.hl7.org/fhir/R4/search.html#table
use oxidized_fhir_model::r4::types::{
    CodeableConcept, Coding, ContactPoint, FHIRBoolean, FHIRCanonical, FHIRCode, FHIRDecimal,
    FHIRId, FHIRInteger, FHIRPositiveInt, FHIRString, FHIRUnsignedInt, FHIRUri, FHIRUrl, FHIRUuid,
    Identifier, SearchParameter,
};
use oxidized_fhir_operation_error::{OperationOutcomeError, derive::OperationOutcomeError};
use oxidized_reflect::MetaValue;

#[derive(Debug)]
struct TokenIndex {
    system: Option<String>,
    code: Option<String>,
}

#[derive(Debug)]
pub enum InsertableIndex {
    String(Vec<String>),
    Number(Vec<f64>),
    URI(Vec<String>),
    Token(Vec<TokenIndex>),
}

#[derive(OperationOutcomeError, Debug)]
enum InsertableIndexError {
    #[fatal(
        code = "exception",
        diagnostic = "Invalid type for insertable index: {arg0}"
    )]
    InvalidType(String),
    #[fatal(
        code = "exception",
        diagnostic = "Failed to downcast value to number: {arg0}"
    )]
    FailedDowncast(String),
}

// "http://hl7.org/fhirpath/System.String" => value
//     .as_any()
//     .downcast_ref::<String>()
//     .map(|v| vec![v.clone()])
//     .ok_or_else(|| InsertableIndexError::FailedDowncast(value.typename().to_string())),

fn convert_fp_string(value: &FHIRString) -> Vec<String> {
    value
        .value
        .as_ref()
        .map(|v| vec![v.to_string()])
        .unwrap_or_else(|| vec![])
}

fn convert_optional_fp_string(value: &Option<Box<FHIRString>>) -> Vec<String> {
    value
        .as_ref()
        .map(|v| convert_fp_string(v))
        .unwrap_or_else(|| vec![])
}

fn convert_optional_fp_string_vec(value: &Option<Vec<Box<FHIRString>>>) -> Vec<String> {
    value
        .as_ref()
        .map(|v| v.iter().flat_map(|s| convert_fp_string(s)).collect())
        .unwrap_or_else(|| vec![])
}

fn index_string(value: &dyn MetaValue) -> Result<Vec<String>, InsertableIndexError> {
    match value.typename() {
        "FHIRString" => {
            let fp_string = value
                .as_any()
                .downcast_ref::<oxidized_fhir_model::r4::types::FHIRString>()
                .ok_or_else(|| {
                    InsertableIndexError::FailedDowncast(value.typename().to_string())
                })?;
            Ok(fp_string
                .value
                .as_ref()
                .map(|v| vec![v.to_string()])
                .unwrap_or_else(|| vec![]))
        }
        // Even though spec states won't encounter this it does. [ImplementationGuide.description]
        "FHIRMarkdown" => {
            let fp_markdown = value
                .as_any()
                .downcast_ref::<oxidized_fhir_model::r4::types::FHIRMarkdown>()
                .ok_or_else(|| {
                    InsertableIndexError::FailedDowncast(value.typename().to_string())
                })?;
            Ok(fp_markdown
                .value
                .as_ref()
                .map(|v| vec![v.to_string()])
                .unwrap_or_else(|| vec![]))
        }
        "HumanName" => {
            let human_name = value
                .as_any()
                .downcast_ref::<oxidized_fhir_model::r4::types::HumanName>()
                .ok_or_else(|| {
                    InsertableIndexError::FailedDowncast(value.typename().to_string())
                })?;

            let mut string_index = vec![];
            string_index.extend(convert_optional_fp_string(&human_name.text));
            string_index.extend(convert_optional_fp_string(&human_name.family));
            string_index.extend(convert_optional_fp_string_vec(&human_name.given));
            string_index.extend(convert_optional_fp_string_vec(&human_name.prefix));
            string_index.extend(convert_optional_fp_string_vec(&human_name.suffix));
            Ok(string_index)
        }
        "Address" => {
            let mut string_index = vec![];
            let address = value
                .as_any()
                .downcast_ref::<oxidized_fhir_model::r4::types::Address>()
                .ok_or_else(|| {
                    InsertableIndexError::FailedDowncast(value.typename().to_string())
                })?;
            string_index.extend(convert_optional_fp_string(&address.text));
            string_index.extend(convert_optional_fp_string_vec(&address.line));
            string_index.extend(convert_optional_fp_string(&address.city));
            string_index.extend(convert_optional_fp_string(&address.district));
            string_index.extend(convert_optional_fp_string(&address.state));
            string_index.extend(convert_optional_fp_string(&address.postalCode));
            string_index.extend(convert_optional_fp_string(&address.country));

            Ok(string_index)
        }

        type_name => Err(InsertableIndexError::FailedDowncast(type_name.to_string())),
    }
}

fn index_number(value: &dyn MetaValue) -> Result<Vec<f64>, InsertableIndexError> {
    match value.typename() {
        "FHIRInteger" => {
            let fp_integer = value
                .as_any()
                .downcast_ref::<FHIRInteger>()
                .ok_or_else(|| {
                    InsertableIndexError::FailedDowncast(value.typename().to_string())
                })?;
            Ok(fp_integer
                .value
                .as_ref()
                .map(|v| vec![*v as f64])
                .unwrap_or_else(|| vec![]))
        }
        "FHIRDecimal" => {
            let fp_decimal = value
                .as_any()
                .downcast_ref::<FHIRDecimal>()
                .ok_or_else(|| {
                    InsertableIndexError::FailedDowncast(value.typename().to_string())
                })?;
            Ok(fp_decimal
                .value
                .as_ref()
                .map(|v| vec![*v as f64])
                .unwrap_or_else(|| vec![]))
        }
        "FHIRPositiveInt" => {
            let fp_positive_int = value
                .as_any()
                .downcast_ref::<FHIRPositiveInt>()
                .ok_or_else(|| {
                    InsertableIndexError::FailedDowncast(value.typename().to_string())
                })?;

            Ok(fp_positive_int
                .value
                .as_ref()
                .map(|v| vec![*v as f64])
                .unwrap_or_else(|| vec![]))
        }
        "FHIRUnsignedInt" => {
            let fp_unsigned_int = value
                .as_any()
                .downcast_ref::<FHIRUnsignedInt>()
                .ok_or_else(|| {
                    InsertableIndexError::FailedDowncast(value.typename().to_string())
                })?;

            Ok(fp_unsigned_int
                .value
                .as_ref()
                .map(|v| vec![*v as f64])
                .unwrap_or_else(|| vec![]))
        }
        type_name => Err(InsertableIndexError::FailedDowncast(type_name.to_string())),
    }
}

fn index_uri(value: &dyn MetaValue) -> Result<Vec<String>, InsertableIndexError> {
    match value.typename() {
        "FHIRUrl" => {
            let fp_uri = value.as_any().downcast_ref::<FHIRUrl>().ok_or_else(|| {
                InsertableIndexError::FailedDowncast(value.typename().to_string())
            })?;
            Ok(fp_uri
                .value
                .as_ref()
                .map(|v| vec![v.to_string()])
                .unwrap_or_else(|| vec![]))
        }
        "FHIRUuid" => {
            let fp_uri = value.as_any().downcast_ref::<FHIRUuid>().ok_or_else(|| {
                InsertableIndexError::FailedDowncast(value.typename().to_string())
            })?;
            Ok(fp_uri
                .value
                .as_ref()
                .map(|v| vec![v.to_string()])
                .unwrap_or_else(|| vec![]))
        }
        "FHIRCanonical" => {
            let fp_uri = value
                .as_any()
                .downcast_ref::<FHIRCanonical>()
                .ok_or_else(|| {
                    InsertableIndexError::FailedDowncast(value.typename().to_string())
                })?;
            Ok(fp_uri
                .value
                .as_ref()
                .map(|v| vec![v.to_string()])
                .unwrap_or_else(|| vec![]))
        }
        "FHIRUri" => {
            let fp_uri = value.as_any().downcast_ref::<FHIRUri>().ok_or_else(|| {
                InsertableIndexError::FailedDowncast(value.typename().to_string())
            })?;
            Ok(fp_uri
                .value
                .as_ref()
                .map(|v| vec![v.to_string()])
                .unwrap_or_else(|| vec![]))
        }
        type_name => Err(InsertableIndexError::FailedDowncast(type_name.to_string())),
    }
}

fn index_token(value: &dyn MetaValue) -> Result<Vec<TokenIndex>, InsertableIndexError> {
    match value.typename() {
        "Coding" => {
            let fp_coding = value.as_any().downcast_ref::<Coding>().ok_or_else(|| {
                InsertableIndexError::FailedDowncast(value.typename().to_string())
            })?;

            Ok(vec![TokenIndex {
                system: fp_coding.system.as_ref().and_then(|s| s.value.clone()),
                code: fp_coding.code.as_ref().and_then(|v| v.value.clone()),
            }])
        }
        "CodeableConcept" => {
            let fp_codeable_concept = value
                .as_any()
                .downcast_ref::<CodeableConcept>()
                .ok_or_else(|| {
                    InsertableIndexError::FailedDowncast(value.typename().to_string())
                })?;

            Ok(fp_codeable_concept
                .coding
                .as_ref()
                .and_then(|coding| {
                    Some(
                        coding
                            .iter()
                            .map(|c| TokenIndex {
                                system: c.system.as_ref().and_then(|s| s.value.clone()),
                                code: c.code.as_ref().and_then(|v| v.value.clone()),
                            })
                            .collect::<Vec<_>>(),
                    )
                })
                .unwrap_or_else(|| vec![]))
        }
        "Identifier" => {
            let fp_identifier = value.as_any().downcast_ref::<Identifier>().ok_or_else(|| {
                InsertableIndexError::FailedDowncast(value.typename().to_string())
            })?;

            Ok(vec![TokenIndex {
                system: fp_identifier.system.as_ref().and_then(|s| s.value.clone()),
                code: fp_identifier.value.as_ref().and_then(|v| v.value.clone()),
            }])
        }
        "ContactPoint" => {
            let fp_contact_point =
                value
                    .as_any()
                    .downcast_ref::<ContactPoint>()
                    .ok_or_else(|| {
                        InsertableIndexError::FailedDowncast(value.typename().to_string())
                    })?;

            Ok(vec![TokenIndex {
                system: None,
                code: fp_contact_point
                    .value
                    .as_ref()
                    .and_then(|v| v.value.clone()),
            }])
        }
        "FHIRCode" => {
            let fp_code = value.as_any().downcast_ref::<FHIRCode>().ok_or_else(|| {
                InsertableIndexError::FailedDowncast(value.typename().to_string())
            })?;

            Ok(vec![TokenIndex {
                system: None,
                code: fp_code.value.as_ref().map(|v| v.to_string()),
            }])
        }
        "FHIRBoolean" => {
            let fp_boolean = value
                .as_any()
                .downcast_ref::<FHIRBoolean>()
                .ok_or_else(|| {
                    InsertableIndexError::FailedDowncast(value.typename().to_string())
                })?;

            Ok(vec![TokenIndex {
                system: Some("http://hl7.org/fhir/special-values".to_string()),
                code: fp_boolean.value.as_ref().map(|v| v.to_string()),
            }])
        }
        "http://hl7.org/fhirpath/System.String" => {
            let string = value.as_any().downcast_ref::<String>().ok_or_else(|| {
                InsertableIndexError::FailedDowncast(value.typename().to_string())
            })?;

            Ok(vec![TokenIndex {
                system: None,
                code: Some(string.clone()),
            }])
        }
        "FHIRString" => {
            let fp_string = value.as_any().downcast_ref::<FHIRString>().ok_or_else(|| {
                InsertableIndexError::FailedDowncast(value.typename().to_string())
            })?;

            Ok(vec![TokenIndex {
                system: None,
                code: fp_string.value.as_ref().map(|v| v.to_string()),
            }])
        }
        "FHIRId" => {
            let fp_id = value.as_any().downcast_ref::<FHIRId>().ok_or_else(|| {
                InsertableIndexError::FailedDowncast(value.typename().to_string())
            })?;

            Ok(vec![TokenIndex {
                system: None,
                code: fp_id.value.as_ref().map(|v| v.to_string()),
            }])
        }
        _ => Err(InsertableIndexError::FailedDowncast(
            value.typename().to_string(),
        )),
    }
}

pub fn to_insertable_index(
    parameter: &SearchParameter,
    result: Vec<&dyn MetaValue>,
) -> Result<InsertableIndex, OperationOutcomeError> {
    match parameter.type_.value.as_ref().map(|v| v.as_str()) {
        Some("number") => {
            let numbers = result
                .iter()
                .filter_map(|v| index_number(*v).ok())
                .flatten()
                .collect::<Vec<_>>();
            Ok(InsertableIndex::Number(numbers))
        }
        Some("string") => {
            let strings = result
                .iter()
                .filter_map(|v| index_string(*v).ok())
                .flatten()
                .collect();
            Ok(InsertableIndex::String(strings))
        }
        Some("uri") => {
            let uris = result
                .iter()
                .filter_map(|v| index_uri(*v).ok())
                .flatten()
                .collect();
            Ok(InsertableIndex::URI(uris))
        }
        Some("token") => {
            let tokens = result
                .iter()
                .filter_map(|v| index_token(*v).ok())
                .flatten()
                .collect();
            Ok(InsertableIndex::Token(tokens))
        }
        Some("composite") | Some("reference") | Some("date") | Some("quantity") => {
            panic!()
        }
        Some(_) | None => Err(InsertableIndexError::InvalidType(
            parameter
                .type_
                .value
                .as_ref()
                .map(|v| v.as_str())
                .unwrap_or_else(|| "<unknown>")
                .to_string(),
        )
        .into()),
    }
}
