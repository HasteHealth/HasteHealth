/// Reference of conversions found here <https://www.hl7.org/fhir/R4/search.html#table>
use haste_fhir_model::r4::{
    datetime::{Date, DateTime, Instant},
    generated::{
        resources::{ResourceType, ResourceTypeError},
        terminology::SearchParamType,
        types::{
            Address, Age, CodeableConcept, Coding, ContactPoint, Duration, FHIRBoolean,
            FHIRCanonical, FHIRDate, FHIRDateTime, FHIRDecimal, FHIRId, FHIRInstant, FHIRInteger,
            FHIRMarkdown, FHIRPositiveInt, FHIRString, FHIRUnsignedInt, FHIRUri, FHIRUrl, FHIRUuid,
            HumanName, Identifier, Money, Period, Quantity, Range, Reference, Timing,
        },
    },
};
use haste_fhir_operation_error::{OperationOutcomeError, derive::OperationOutcomeError};
use haste_reflect::MetaValue;
use serde::{Deserialize, Serialize};

use crate::ResolvedParameter;

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenIndex {
    system: Option<String>,
    code: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum RangeValue {
    Number(f64),
    Infinity,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QuantityRange {
    start_value: RangeValue,
    start_code: Option<String>,
    start_system: Option<String>,
    end_value: RangeValue,
    end_code: Option<String>,
    end_system: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct DateRange {
    /// Milliseconds since epoch.
    pub start: i64,
    pub end: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReferenceIndex {
    id: Option<String>,
    resource_type: Option<String>,
    uri: Option<String>,
}

/// The typed value slot of a project-level (user-submitted) search parameter,
/// matching the fixed `dynamic_parameters.value.*` schema declared in the
/// Elasticsearch mapping. Exactly one field is set, matching the parameter's
/// `SearchParamType`. Unlike system-level parameters, a user-submitted
/// parameter's URL is arbitrary and can't safely become an Elasticsearch
/// field name (mapping explosion, plus Elasticsearch would silently collapse
/// any dot in that URL into a nested object path) - so it's stored as the
/// plain string value of the `url` field instead. See [`DynamicParameterEntry`].
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct DynamicParameterValue {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub string: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub number: Option<Vec<f64>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uri: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<Vec<TokenIndex>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date: Option<Vec<DateRange>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reference: Option<Vec<ReferenceIndex>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quantity: Option<Vec<QuantityRange>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DynamicParameterEntry {
    pub url: String,
    #[serde(rename = "type")]
    pub type_: String,
    pub value: DynamicParameterValue,
}

impl DynamicParameterEntry {
    /// Builds an entry from a search parameter's URL, its FHIR search
    /// parameter type code (e.g. `"string"`, `"token"`), and the leaf
    /// `InsertableIndex` produced for it. Returns `None` for the internal
    /// (`Meta`, `Composite`, `Special`, `DynamicParameters`) variants, which
    /// never occur as a single parameter's evaluated value.
    #[must_use]
    pub fn from_leaf(url: String, type_: &str, leaf: InsertableIndex) -> Option<Self> {
        let value = match leaf {
            InsertableIndex::String(v) => DynamicParameterValue {
                string: Some(v),
                ..Default::default()
            },
            InsertableIndex::Number(v) => DynamicParameterValue {
                number: Some(v),
                ..Default::default()
            },
            InsertableIndex::URI(v) => DynamicParameterValue {
                uri: Some(v),
                ..Default::default()
            },
            InsertableIndex::Token(v) => DynamicParameterValue {
                token: Some(v),
                ..Default::default()
            },
            InsertableIndex::Date(v) => DynamicParameterValue {
                date: Some(v),
                ..Default::default()
            },
            InsertableIndex::Reference(v) => DynamicParameterValue {
                reference: Some(v),
                ..Default::default()
            },
            InsertableIndex::Quantity(v) => DynamicParameterValue {
                quantity: Some(v),
                ..Default::default()
            },
            InsertableIndex::Meta(_)
            | InsertableIndex::Composite(_)
            | InsertableIndex::Special(_)
            | InsertableIndex::DynamicParameters(_) => return None,
        };

        Some(DynamicParameterEntry {
            url,
            type_: type_.to_string(),
            value,
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum InsertableIndex {
    // Used for internal indexing only
    Meta(String),
    // Fhir Indexing types
    String(Vec<String>),
    Number(Vec<f64>),
    URI(Vec<String>),
    Token(Vec<TokenIndex>),
    Date(Vec<DateRange>),
    Reference(Vec<ReferenceIndex>),
    Quantity(Vec<QuantityRange>),
    Composite(Vec<String>),
    Special(Vec<String>),
    DynamicParameters(Vec<DynamicParameterEntry>),
}

#[derive(OperationOutcomeError, Debug)]
pub enum InsertableIndexError {
    #[fatal(
        code = "exception",
        diagnostic = "Invalid type for insertable index: '{arg0}'"
    )]
    InvalidType(String),
    #[fatal(
        code = "exception",
        diagnostic = "Failed to downcast value to number: {arg0}"
    )]
    FailedDowncast(String),
    #[fatal(
        code = "exception",
        diagnostic = "Reference contains invalid resource type."
    )]
    ResourceTypeError(#[from] ResourceTypeError),
}

// "http://hl7.org/fhirpath/System.String" => value
//     .as_any()
//     .downcast_ref::<String>()
//     .map(|v| vec![v.clone()])
//     .ok_or_else(|| InsertableIndexError::FailedDowncast(value.fhir_type().to_string())),

fn convert_fp_string(value: &FHIRString) -> Vec<String> {
    value
        .value
        .as_ref()
        .map_or_else(Vec::new, |v| vec![v.clone()])
}

fn convert_optional_fp_string(value: Option<&FHIRString>) -> Vec<String> {
    value
        .as_ref()
        .map_or_else(Vec::new, |v| convert_fp_string(v))
}

fn convert_optional_fp_string_vec(value: Option<&Vec<FHIRString>>) -> Vec<String> {
    value
        .as_ref()
        .map_or_else(Vec::new, |v| v.iter().flat_map(convert_fp_string).collect())
}

fn index_string(value: &dyn MetaValue) -> Result<Vec<String>, InsertableIndexError> {
    match value.fhir_type() {
        "string" => {
            let fp_string = value.as_any().downcast_ref::<FHIRString>().ok_or_else(|| {
                InsertableIndexError::FailedDowncast(value.fhir_type().to_string())
            })?;
            Ok(fp_string
                .value
                .as_ref()
                .map_or_else(Vec::new, |v| vec![v.clone()]))
        }
        // Even though spec states won't encounter this it does. [ImplementationGuide.description]
        "markdown" => {
            let fp_markdown = value
                .as_any()
                .downcast_ref::<FHIRMarkdown>()
                .ok_or_else(|| {
                    InsertableIndexError::FailedDowncast(value.fhir_type().to_string())
                })?;
            Ok(fp_markdown
                .value
                .as_ref()
                .map_or_else(Vec::new, |v| vec![v.clone()]))
        }
        "HumanName" => {
            let human_name = value.as_any().downcast_ref::<HumanName>().ok_or_else(|| {
                InsertableIndexError::FailedDowncast(value.fhir_type().to_string())
            })?;

            let mut string_index = Vec::new();
            string_index.extend(convert_optional_fp_string(human_name.text.as_deref()));
            string_index.extend(convert_optional_fp_string(human_name.family.as_deref()));
            string_index.extend(convert_optional_fp_string_vec(human_name.given.as_ref()));
            string_index.extend(convert_optional_fp_string_vec(human_name.prefix.as_ref()));
            string_index.extend(convert_optional_fp_string_vec(human_name.suffix.as_ref()));
            Ok(string_index)
        }
        "Address" => {
            let mut string_index = Vec::new();
            let address = value.as_any().downcast_ref::<Address>().ok_or_else(|| {
                InsertableIndexError::FailedDowncast(value.fhir_type().to_string())
            })?;
            string_index.extend(convert_optional_fp_string(address.text.as_deref()));
            string_index.extend(convert_optional_fp_string_vec(address.line.as_ref()));
            string_index.extend(convert_optional_fp_string(address.city.as_deref()));
            string_index.extend(convert_optional_fp_string(address.district.as_deref()));
            string_index.extend(convert_optional_fp_string(address.state.as_deref()));
            string_index.extend(convert_optional_fp_string(address.postalCode.as_deref()));
            string_index.extend(convert_optional_fp_string(address.country.as_deref()));

            Ok(string_index)
        }

        type_name => Err(InsertableIndexError::FailedDowncast(type_name.to_string())),
    }
}

fn index_number(value: &dyn MetaValue) -> Result<Vec<f64>, InsertableIndexError> {
    match value.fhir_type() {
        "integer" => {
            let fp_integer = value
                .as_any()
                .downcast_ref::<FHIRInteger>()
                .ok_or_else(|| {
                    InsertableIndexError::FailedDowncast(value.fhir_type().to_string())
                })?;
            #[allow(clippy::cast_precision_loss)]
            Ok(fp_integer
                .value
                .as_ref()
                .map_or_else(Vec::new, |v| vec![*v as f64]))
        }
        "decimal" => {
            let fp_decimal = value
                .as_any()
                .downcast_ref::<FHIRDecimal>()
                .ok_or_else(|| {
                    InsertableIndexError::FailedDowncast(value.fhir_type().to_string())
                })?;
            #[allow(clippy::cast_precision_loss)]
            Ok(fp_decimal
                .value
                .as_ref()
                .map_or_else(Vec::new, |v| vec![*v]))
        }
        "positiveInt" => {
            let fp_positive_int = value
                .as_any()
                .downcast_ref::<FHIRPositiveInt>()
                .ok_or_else(|| {
                    InsertableIndexError::FailedDowncast(value.fhir_type().to_string())
                })?;

            #[allow(clippy::cast_precision_loss)]
            Ok(fp_positive_int
                .value
                .as_ref()
                .map_or_else(Vec::new, |v| vec![*v as f64]))
        }
        "unsignedInt" => {
            let fp_unsigned_int = value
                .as_any()
                .downcast_ref::<FHIRUnsignedInt>()
                .ok_or_else(|| {
                    InsertableIndexError::FailedDowncast(value.fhir_type().to_string())
                })?;

            #[allow(clippy::cast_precision_loss)]
            Ok(fp_unsigned_int
                .value
                .as_ref()
                .map_or_else(Vec::new, |v| vec![*v as f64]))
        }
        type_name => Err(InsertableIndexError::FailedDowncast(type_name.to_string())),
    }
}

fn index_uri(value: &dyn MetaValue) -> Result<Vec<String>, InsertableIndexError> {
    match value.fhir_type() {
        "url" => {
            let fp_uri = value.as_any().downcast_ref::<FHIRUrl>().ok_or_else(|| {
                InsertableIndexError::FailedDowncast(value.fhir_type().to_string())
            })?;
            Ok(fp_uri
                .value
                .as_ref()
                .map_or_else(Vec::new, |v| vec![v.clone()]))
        }
        "uuid" => {
            let fp_uri = value.as_any().downcast_ref::<FHIRUuid>().ok_or_else(|| {
                InsertableIndexError::FailedDowncast(value.fhir_type().to_string())
            })?;
            Ok(fp_uri
                .value
                .as_ref()
                .map_or_else(Vec::new, |v| vec![v.clone()]))
        }
        "canonical" => {
            let fp_uri = value
                .as_any()
                .downcast_ref::<FHIRCanonical>()
                .ok_or_else(|| {
                    InsertableIndexError::FailedDowncast(value.fhir_type().to_string())
                })?;
            Ok(fp_uri
                .value
                .as_ref()
                .map_or_else(Vec::new, |v| vec![v.clone()]))
        }
        "uri" => {
            let fp_uri = value.as_any().downcast_ref::<FHIRUri>().ok_or_else(|| {
                InsertableIndexError::FailedDowncast(value.fhir_type().to_string())
            })?;
            Ok(fp_uri
                .value
                .as_ref()
                .map_or_else(Vec::new, |v| vec![v.clone()]))
        }
        type_name => Err(InsertableIndexError::FailedDowncast(type_name.to_string())),
    }
}

fn index_token(value: &dyn MetaValue) -> Result<Vec<TokenIndex>, InsertableIndexError> {
    match value.fhir_type() {
        "Coding" => index_token_coding(value),
        "CodeableConcept" => index_token_codeable_concept(value),
        "Identifier" => index_token_identifier(value),
        "ContactPoint" => index_token_contact_point(value),
        "code" => Ok(index_token_code(value)),
        "boolean" => index_token_boolean(value),
        "http://hl7.org/fhirpath/System.String" => index_token_system_string(value),
        "string" => index_token_string(value),
        "id" => index_token_id(value),
        _ => Err(InsertableIndexError::FailedDowncast(
            value.fhir_type().to_string(),
        )),
    }
}

fn index_token_coding(value: &dyn MetaValue) -> Result<Vec<TokenIndex>, InsertableIndexError> {
    let coding = value
        .as_any()
        .downcast_ref::<Coding>()
        .ok_or_else(|| InsertableIndexError::FailedDowncast(value.fhir_type().to_string()))?;

    Ok(vec![TokenIndex {
        system: coding.system.as_ref().and_then(|s| s.value.clone()),
        code: coding.code.as_ref().and_then(|v| v.value.clone()),
    }])
}

fn index_token_codeable_concept(
    value: &dyn MetaValue,
) -> Result<Vec<TokenIndex>, InsertableIndexError> {
    let codeable_concept = value
        .as_any()
        .downcast_ref::<CodeableConcept>()
        .ok_or_else(|| InsertableIndexError::FailedDowncast(value.fhir_type().to_string()))?;

    Ok(codeable_concept
        .coding
        .as_ref()
        .map(|coding| {
            coding
                .iter()
                .map(|c| TokenIndex {
                    system: c.system.as_ref().and_then(|s| s.value.clone()),
                    code: c.code.as_ref().and_then(|v| v.value.clone()),
                })
                .collect()
        })
        .unwrap_or_default())
}

fn index_token_identifier(value: &dyn MetaValue) -> Result<Vec<TokenIndex>, InsertableIndexError> {
    let identifier = value
        .as_any()
        .downcast_ref::<Identifier>()
        .ok_or_else(|| InsertableIndexError::FailedDowncast(value.fhir_type().to_string()))?;

    Ok(vec![TokenIndex {
        system: identifier.system.as_ref().and_then(|s| s.value.clone()),
        code: identifier.value.as_ref().and_then(|v| v.value.clone()),
    }])
}

fn index_token_contact_point(
    value: &dyn MetaValue,
) -> Result<Vec<TokenIndex>, InsertableIndexError> {
    let contact_point = value
        .as_any()
        .downcast_ref::<ContactPoint>()
        .ok_or_else(|| InsertableIndexError::FailedDowncast(value.fhir_type().to_string()))?;

    Ok(vec![TokenIndex {
        system: None,
        code: contact_point.value.as_ref().and_then(|v| v.value.clone()),
    }])
}

fn index_token_code(value: &dyn MetaValue) -> Vec<TokenIndex> {
    let code = value
        .get_field("value")
        .and_then(|v| v.as_any().downcast_ref::<String>());

    vec![TokenIndex {
        system: None,
        code: code.cloned(),
    }]
}

fn index_token_boolean(value: &dyn MetaValue) -> Result<Vec<TokenIndex>, InsertableIndexError> {
    let boolean = value
        .as_any()
        .downcast_ref::<FHIRBoolean>()
        .ok_or_else(|| InsertableIndexError::FailedDowncast(value.fhir_type().to_string()))?;

    Ok(vec![TokenIndex {
        system: Some("http://hl7.org/fhir/special-values".to_string()),
        code: boolean.value.as_ref().map(std::string::ToString::to_string),
    }])
}

fn index_token_system_string(
    value: &dyn MetaValue,
) -> Result<Vec<TokenIndex>, InsertableIndexError> {
    let string = value
        .as_any()
        .downcast_ref::<String>()
        .ok_or_else(|| InsertableIndexError::FailedDowncast(value.fhir_type().to_string()))?;

    Ok(vec![TokenIndex {
        system: None,
        code: Some(string.clone()),
    }])
}

fn index_token_string(value: &dyn MetaValue) -> Result<Vec<TokenIndex>, InsertableIndexError> {
    let string = value
        .as_any()
        .downcast_ref::<FHIRString>()
        .ok_or_else(|| InsertableIndexError::FailedDowncast(value.fhir_type().to_string()))?;

    Ok(vec![TokenIndex {
        system: None,
        code: string.value.clone(),
    }])
}

fn index_token_id(value: &dyn MetaValue) -> Result<Vec<TokenIndex>, InsertableIndexError> {
    let id = value
        .as_any()
        .downcast_ref::<FHIRId>()
        .ok_or_else(|| InsertableIndexError::FailedDowncast(value.fhir_type().to_string()))?;

    Ok(vec![TokenIndex {
        system: None,
        code: id.value.clone(),
    }])
}

fn get_decimal_precision(value: &str) -> u32 {
    let value = value.to_string();
    let decimal_characters = value.split('.').nth(1);
    let mut digits = 0;
    if let Some(decimal_part) = decimal_characters {
        decimal_part.chars().for_each(|_| digits += 1);
    }

    digits
}

#[derive(Debug)]
pub struct DecimalRange {
    pub start: f64,
    pub end: f64,
}

/// Calculates the decimal range used for indexing a number or quantity,
/// based on the precision of the input value.
///
/// # Errors
///
/// Returns [`InsertableIndexError::FailedDowncast`] if `value` cannot be
/// parsed as an `f64`.
pub fn get_decimal_range(value: &str) -> Result<DecimalRange, InsertableIndexError> {
    let decimal_precision = get_decimal_precision(value);
    let parsed_v = value
        .parse::<f64>()
        .map_err(|_e| InsertableIndexError::FailedDowncast(value.to_string()))?;

    Ok(DecimalRange {
        start: parsed_v - 0.5 * 10f64.powi(-(decimal_precision.cast_signed())),
        end: parsed_v + 0.5 * 10f64.powi(-(decimal_precision.cast_signed())),
    })
}

fn fhirdecimal_to_quantity_range(value: Option<&FHIRDecimal>) -> Option<DecimalRange> {
    value.as_ref().and_then(|v| {
        v.value
            .as_ref()
            .and_then(|v| get_decimal_range(&v.to_string()).ok())
    })
}

fn index_quantity(value: &dyn MetaValue) -> Result<Vec<QuantityRange>, InsertableIndexError> {
    match value.fhir_type() {
        "Range" => index_range_quantity(value),
        "Age" => index_age_quantity(value),
        "Money" => index_money_quantity(value),
        "Duration" => index_duration_quantity(value),
        "Quantity" => index_fhir_quantity(value),
        _ => Err(InsertableIndexError::FailedDowncast(
            value.fhir_type().to_string(),
        )),
    }
}

fn index_range_quantity(value: &dyn MetaValue) -> Result<Vec<QuantityRange>, InsertableIndexError> {
    let fp_range = value
        .as_any()
        .downcast_ref::<Range>()
        .ok_or_else(|| InsertableIndexError::FailedDowncast(value.fhir_type().to_string()))?;

    if fp_range.low.is_some() || fp_range.high.is_some() {
        let start_value = fp_range
            .low
            .as_ref()
            .and_then(|v| v.value.as_ref().and_then(|v| v.value));

        let start_system = fp_range
            .low
            .as_ref()
            .and_then(|v| v.system.as_ref().and_then(|s| s.value.clone()));

        let start_code = fp_range
            .low
            .as_ref()
            .and_then(|v| v.code.as_ref().and_then(|c| c.value.clone()));

        let end_value = fp_range
            .high
            .as_ref()
            .and_then(|v| v.value.as_ref().and_then(|v| v.value));

        let end_system = fp_range
            .high
            .as_ref()
            .and_then(|v| v.system.as_ref().and_then(|s| s.value.clone()));

        let end_code = fp_range
            .high
            .as_ref()
            .and_then(|v| v.code.as_ref().and_then(|c| c.value.clone()));

        return Ok(vec![QuantityRange {
            start_system,
            start_code,
            start_value: start_value.map_or(RangeValue::Infinity, RangeValue::Number),
            end_system,
            end_code,
            end_value: end_value.map_or(RangeValue::Infinity, RangeValue::Number),
        }]);
    }

    Ok(Vec::new())
}

fn index_age_quantity(value: &dyn MetaValue) -> Result<Vec<QuantityRange>, InsertableIndexError> {
    let fp_age = value
        .as_any()
        .downcast_ref::<Age>()
        .ok_or_else(|| InsertableIndexError::FailedDowncast(value.fhir_type().to_string()))?;

    Ok(fhirdecimal_to_quantity_range(fp_age.value.as_deref())
        .map(|decimal_range| {
            vec![QuantityRange {
                start_system: fp_age.system.as_ref().and_then(|s| s.value.clone()),
                start_code: fp_age.code.as_ref().and_then(|c| c.value.clone()),
                start_value: RangeValue::Number(decimal_range.start),
                end_system: fp_age.system.as_ref().and_then(|s| s.value.clone()),
                end_code: fp_age.code.as_ref().and_then(|c| c.value.clone()),
                end_value: RangeValue::Number(decimal_range.end),
            }]
        })
        .unwrap_or_default())
}

fn index_money_quantity(value: &dyn MetaValue) -> Result<Vec<QuantityRange>, InsertableIndexError> {
    let fp_money = value
        .as_any()
        .downcast_ref::<Money>()
        .ok_or_else(|| InsertableIndexError::FailedDowncast(value.fhir_type().to_string()))?;

    Ok(fhirdecimal_to_quantity_range(fp_money.value.as_deref())
        .map(|decimal_range| {
            vec![QuantityRange {
                start_system: Some("urn:iso:std:iso:4217".to_string()),
                start_code: fp_money.currency.as_ref().and_then(|c| c.value.clone()),
                start_value: RangeValue::Number(decimal_range.start),
                end_system: Some("urn:iso:std:iso:4217".to_string()),
                end_code: fp_money.currency.as_ref().and_then(|c| c.value.clone()),
                end_value: RangeValue::Number(decimal_range.end),
            }]
        })
        .unwrap_or_default())
}

fn index_duration_quantity(
    value: &dyn MetaValue,
) -> Result<Vec<QuantityRange>, InsertableIndexError> {
    let fp_duration = value
        .as_any()
        .downcast_ref::<Duration>()
        .ok_or_else(|| InsertableIndexError::FailedDowncast(value.fhir_type().to_string()))?;

    Ok(fhirdecimal_to_quantity_range(fp_duration.value.as_deref())
        .map(|decimal_range| {
            vec![QuantityRange {
                start_system: fp_duration.system.as_ref().and_then(|s| s.value.clone()),
                start_code: fp_duration.code.as_ref().and_then(|c| c.value.clone()),
                start_value: RangeValue::Number(decimal_range.start),
                end_system: fp_duration.system.as_ref().and_then(|s| s.value.clone()),
                end_code: fp_duration.code.as_ref().and_then(|c| c.value.clone()),
                end_value: RangeValue::Number(decimal_range.end),
            }]
        })
        .unwrap_or_default())
}

fn index_fhir_quantity(value: &dyn MetaValue) -> Result<Vec<QuantityRange>, InsertableIndexError> {
    let fp_quantity = value
        .as_any()
        .downcast_ref::<Quantity>()
        .ok_or_else(|| InsertableIndexError::FailedDowncast(value.fhir_type().to_string()))?;

    Ok(fhirdecimal_to_quantity_range(fp_quantity.value.as_deref())
        .map(|decimal_range| {
            vec![QuantityRange {
                start_system: fp_quantity.system.as_ref().and_then(|s| s.value.clone()),
                start_code: fp_quantity.code.as_ref().and_then(|c| c.value.clone()),
                start_value: RangeValue::Number(decimal_range.start),
                end_system: fp_quantity.system.as_ref().and_then(|s| s.value.clone()),
                end_code: fp_quantity.code.as_ref().and_then(|c| c.value.clone()),
                end_value: RangeValue::Number(decimal_range.end),
            }]
        })
        .unwrap_or_default())
}

fn year_to_daterange(year: u16) -> Result<DateRange, InsertableIndexError> {
    let start_date = chrono::NaiveDate::from_ymd_opt(i32::from(year), 1, 1)
        .and_then(|d| d.and_hms_opt(0, 0, 0))
        .ok_or_else(|| InsertableIndexError::FailedDowncast("Date".to_string()))?;

    let end_date = chrono::NaiveDate::from_ymd_opt(i32::from(year) + 1, 1, 1)
        .and_then(|d| d.pred_opt())
        .and_then(|d| d.and_hms_milli_opt(23, 59, 59, 999))
        .ok_or_else(|| InsertableIndexError::FailedDowncast("Date".to_string()))?;

    Ok(DateRange {
        start: chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(start_date, chrono::Utc)
            .timestamp_millis(),
        end: chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(end_date, chrono::Utc)
            .timestamp_millis(),
    })
}

fn year_month_to_daterange(year: u16, month: u8) -> Result<DateRange, InsertableIndexError> {
    let start_date = chrono::NaiveDate::from_ymd_opt(i32::from(year), u32::from(month), 1)
        .and_then(|d| d.and_hms_opt(0, 0, 0))
        .ok_or_else(|| InsertableIndexError::FailedDowncast("Date".to_string()))?;

    let end_date = if month < 12 {
        chrono::NaiveDate::from_ymd_opt(i32::from(year), (month + 1).into(), 1)
            .and_then(|d| d.pred_opt())
            .and_then(|d| d.and_hms_milli_opt(23, 59, 59, 999))
    } else {
        chrono::NaiveDate::from_ymd_opt(i32::from(year) + 1, 1, 1)
            .and_then(|d| d.pred_opt())
            .and_then(|d| d.and_hms_milli_opt(23, 59, 59, 999))
    }
    .ok_or_else(|| InsertableIndexError::FailedDowncast("Date".to_string()))?;

    Ok(DateRange {
        start: chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(start_date, chrono::Utc)
            .timestamp_millis(),
        end: chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(end_date, chrono::Utc)
            .timestamp_millis(),
    })
}

fn year_month_day_to_daterange(
    year: u16,
    month: u8,
    day: u8,
) -> Result<DateRange, InsertableIndexError> {
    let start_date =
        chrono::NaiveDate::from_ymd_opt(i32::from(year), u32::from(month), u32::from(day))
            .and_then(|d| d.and_hms_opt(0, 0, 0))
            .ok_or_else(|| InsertableIndexError::FailedDowncast("Date".to_string()))?;

    let end_date =
        chrono::NaiveDate::from_ymd_opt(i32::from(year), u32::from(month), u32::from(day))
            .and_then(|d| d.and_hms_milli_opt(23, 59, 59, 999))
            .ok_or_else(|| InsertableIndexError::FailedDowncast("Date".to_string()))?;

    Ok(DateRange {
        start: chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(start_date, chrono::Utc)
            .timestamp_millis(),
        end: chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(end_date, chrono::Utc)
            .timestamp_millis(),
    })
}

/// Converts a FHIR date/time value into a date range suitable for indexing.
///
///
/// # Errors
///
/// Returns an [`InsertableIndexError`] if the supplied year, year/month, or
/// year/month/day value cannot be converted into a valid date range.
pub fn date_time_range(value: &DateTime) -> Result<DateRange, InsertableIndexError> {
    match value {
        DateTime::Year(year) => Ok(year_to_daterange(*year)?),
        DateTime::YearMonth(year, month) => Ok(year_month_to_daterange(*year, *month)?),
        DateTime::YearMonthDay(year, month, day) => {
            Ok(year_month_day_to_daterange(*year, *month, *day)?)
        }
        DateTime::Iso8601(date_time) => Ok(DateRange {
            start: date_time.timestamp_millis(),
            end: date_time.timestamp_millis(),
        }),
    }
}

fn index_date(value: &dyn MetaValue) -> Result<Vec<DateRange>, InsertableIndexError> {
    match value.fhir_type() {
        "Timing" => {
            let fp_timing = value.as_any().downcast_ref::<Timing>().ok_or_else(|| {
                InsertableIndexError::FailedDowncast(value.fhir_type().to_string())
            })?;

            if let Some(events) = fp_timing.event.as_ref() {
                let date_ranges = events
                    .iter()
                    .map(|event| index_date(event))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(date_ranges.into_iter().flatten().collect())
            } else {
                Ok(Vec::new())
            }
        }
        "date" => {
            let fp_date = value
                .as_any()
                .downcast_ref::<FHIRDate>()
                .ok_or_else(|| InsertableIndexError::FailedDowncast(value.fhir_type().to_string()))?
                .value
                .as_ref();

            match &fp_date {
                Some(Date::Year(year)) => Ok(vec![year_to_daterange(*year)?]),
                Some(Date::YearMonth(year, month)) => {
                    Ok(vec![year_month_to_daterange(*year, *month)?])
                }
                Some(Date::YearMonthDay(year, month, day)) => {
                    Ok(vec![year_month_day_to_daterange(*year, *month, *day)?])
                }
                None => Ok(Vec::new()),
            }
        }
        "dateTime" => {
            let fp_datetime = value
                .as_any()
                .downcast_ref::<FHIRDateTime>()
                .ok_or_else(|| InsertableIndexError::FailedDowncast(value.fhir_type().to_string()))?
                .value
                .as_ref();

            match &fp_datetime {
                Some(date_time) => date_time_range(date_time).map(|date_range| vec![date_range]),
                None => Ok(Vec::new()),
            }
        }
        "instant" => {
            let fp_instant = value
                .as_any()
                .downcast_ref::<FHIRInstant>()
                .ok_or_else(|| {
                    InsertableIndexError::FailedDowncast(value.fhir_type().to_string())
                })?;

            match &fp_instant.value {
                Some(Instant::Iso8601(instant)) => {
                    let timestamp = instant.timestamp_millis();
                    Ok(vec![DateRange {
                        start: timestamp,
                        end: timestamp,
                    }])
                }
                None => Ok(Vec::new()),
            }
        }
        "Period" => {
            let fp_period = value.as_any().downcast_ref::<Period>().ok_or_else(|| {
                InsertableIndexError::FailedDowncast(value.fhir_type().to_string())
            })?;
            let fp_start = if let Some(date) = fp_period.start.as_ref() {
                let date = date.as_ref();
                let date_range = index_date(date)?;
                date_range
                    .first()
                    .ok_or_else(|| {
                        InsertableIndexError::FailedDowncast(value.fhir_type().to_string())
                    })?
                    .start
            } else {
                0
            };

            let fp_end = if let Some(date) = fp_period.end.as_ref() {
                let date = date.as_ref();
                let date_range = index_date(date)?;
                date_range
                    .first()
                    .ok_or_else(|| {
                        InsertableIndexError::FailedDowncast(value.fhir_type().to_string())
                    })?
                    .end
            } else {
                i64::MAX
            };

            Ok(vec![DateRange {
                start: fp_start,
                end: fp_end,
            }])
        }
        _ => Err(InsertableIndexError::FailedDowncast(
            value.fhir_type().to_string(),
        )),
    }
}

fn index_reference(value: &dyn MetaValue) -> Result<Vec<ReferenceIndex>, InsertableIndexError> {
    match value.fhir_type() {
        "Reference" => {
            let fp_reference = value.as_any().downcast_ref::<Reference>().ok_or_else(|| {
                InsertableIndexError::FailedDowncast(value.fhir_type().to_string())
            })?;

            if let Some(reference) = &fp_reference
                .reference
                .as_ref()
                .and_then(|r| r.value.as_ref())
            {
                let parts: Vec<&str> = reference.split('/').collect();
                if parts.len() == 2 {
                    let resource_type = ResourceType::try_from(parts[0])?;
                    let id = parts[1].to_string();
                    return Ok(vec![ReferenceIndex {
                        resource_type: Some(resource_type.as_ref().to_string()),
                        id: Some(id),
                        uri: None,
                    }]);
                }
            }

            Ok(vec![])
        }
        "canonical" => {
            let fp_canonical = value
                .as_any()
                .downcast_ref::<FHIRCanonical>()
                .ok_or_else(|| {
                    InsertableIndexError::FailedDowncast(value.fhir_type().to_string())
                })?;
            if let Some(canonical) = &fp_canonical.value {
                return Ok(vec![ReferenceIndex {
                    id: None,
                    resource_type: None,
                    uri: Some(canonical.clone()),
                }]);
            }
            Ok(vec![])
        }
        "uri" => {
            let fp_uri = value.as_any().downcast_ref::<FHIRUri>().ok_or_else(|| {
                InsertableIndexError::FailedDowncast(value.fhir_type().to_string())
            })?;
            if let Some(uri) = &fp_uri.value {
                return Ok(vec![ReferenceIndex {
                    id: None,
                    resource_type: None,
                    uri: Some(uri.clone()),
                }]);
            }
            Ok(vec![])
        }
        _ => Err(InsertableIndexError::FailedDowncast(
            value.fhir_type().to_string(),
        )),
    }
}

/// Converts resolved search parameter values into an indexable representation.
///
/// Values that cannot be converted to the corresponding index type are
/// skipped.
///
/// # Errors
///
/// Returns [`OperationOutcomeError`] if the search parameter has an
/// unsupported or invalid type.
pub fn to_insertable_index(
    parameter: &ResolvedParameter,
    result: &[&dyn MetaValue],
) -> Result<InsertableIndex, OperationOutcomeError> {
    let search_parameter = &parameter.search_parameter;
    match &search_parameter.type_ {
        param_type if param_type == &SearchParamType::number() => {
            let numbers = result
                .iter()
                .filter_map(|v| index_number(*v).ok())
                .flatten()
                .collect::<Vec<_>>();
            Ok(InsertableIndex::Number(numbers))
        }
        param_type if param_type == &SearchParamType::string() => {
            let strings = result
                .iter()
                .filter_map(|v| index_string(*v).ok())
                .flatten()
                .collect();
            Ok(InsertableIndex::String(strings))
        }
        param_type if param_type == &SearchParamType::uri() => {
            let uris = result
                .iter()
                .filter_map(|v| index_uri(*v).ok())
                .flatten()
                .collect();
            Ok(InsertableIndex::URI(uris))
        }
        param_type if param_type == &SearchParamType::token() => {
            let tokens = result
                .iter()
                .filter_map(|v| index_token(*v).ok())
                .flatten()
                .collect();
            Ok(InsertableIndex::Token(tokens))
        }
        param_type if param_type == &SearchParamType::date() => {
            let dates = result
                .iter()
                .filter_map(|v| index_date(*v).ok())
                .flatten()
                .collect();
            Ok(InsertableIndex::Date(dates))
        }
        param_type if param_type == &SearchParamType::reference() => {
            let references = result
                .iter()
                .filter_map(|v: &&dyn MetaValue| index_reference(*v).ok())
                .flatten()
                .collect();
            Ok(InsertableIndex::Reference(references))
        }
        param_type if param_type == &SearchParamType::quantity() => {
            let quantities = result
                .iter()
                .filter_map(|v| index_quantity(*v).ok())
                .flatten()
                .collect();
            Ok(InsertableIndex::Quantity(quantities))
        }
        // Not Supported yet
        param_type if param_type == &SearchParamType::composite() => {
            Ok(InsertableIndex::Composite(vec![]))
        }
        param_type if param_type == &SearchParamType::special() => {
            Ok(InsertableIndex::Special(vec![]))
        }
        _ => {
            let type_name = search_parameter.type_.as_str();
            Err(InsertableIndexError::InvalidType(
                type_name.map_or("unknown".to_string(), std::string::ToString::to_string),
            )
            .into())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use haste_fhir_model::r4::generated::types::{
        FHIRDate, FHIRDateTime, Period, Reference, Timing,
    };

    #[test]
    fn test_year_month_to_daterange() {
        let year = 2023;
        let month: u8 = 5;
        let date_range = year_month_to_daterange(year, month).unwrap();

        assert_eq!(
            date_range.start,
            chrono::DateTime::parse_from_rfc3339("2023-05-01T00:00:00Z")
                .unwrap()
                .timestamp_millis()
        );
        assert_eq!(
            date_range.end,
            chrono::DateTime::parse_from_rfc3339("2023-05-31T23:59:59.999Z")
                .unwrap()
                .timestamp_millis()
        );
    }

    #[test]
    fn test_year_month_day_to_daterange() {
        let year = 2023;
        let month: u8 = 5;
        let day = 15;
        let date_range = year_month_day_to_daterange(year, month, day).unwrap();

        assert_eq!(
            date_range.start,
            chrono::DateTime::parse_from_rfc3339("2023-05-15T00:00:00Z")
                .unwrap()
                .timestamp_millis()
        );
        assert_eq!(
            date_range.end,
            chrono::DateTime::parse_from_rfc3339("2023-05-15T23:59:59.999Z")
                .unwrap()
                .timestamp_millis()
        );
    }

    #[test]
    fn test_year_to_daterange() {
        let year = 2023;
        let date_range = year_to_daterange(year).unwrap();
        assert_eq!(
            date_range.start,
            chrono::DateTime::parse_from_rfc3339("2023-01-01T00:00:00Z")
                .unwrap()
                .timestamp_millis()
        );
        assert_eq!(
            date_range.end,
            chrono::DateTime::parse_from_rfc3339("2023-12-31T23:59:59.999Z")
                .unwrap()
                .timestamp_millis()
        );
    }

    #[test]
    fn test_index_date() {
        let date_value = FHIRDate {
            id: None,
            extension: None,
            value: Some(Date::Year(2023)),
        };
        let result = index_date(&date_value).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(
            result[0].start,
            chrono::DateTime::parse_from_rfc3339("2023-01-01T00:00:00Z")
                .unwrap()
                .timestamp_millis()
        );
        assert_eq!(
            result[0].end,
            chrono::DateTime::parse_from_rfc3339("2023-12-31T23:59:59.999Z")
                .unwrap()
                .timestamp_millis()
        );
    }

    #[test]
    fn date_range_instant() {
        let fhir_date = FHIRDateTime {
            id: None,
            extension: None,
            value: Some(DateTime::Iso8601(
                chrono::DateTime::parse_from_rfc3339("2023-05-14T11:25:25.234-05:00")
                    .unwrap()
                    .with_timezone(&chrono::Utc),
            )),
        };

        let range = index_date(&fhir_date).unwrap();
        let date_range = range.first().unwrap();

        assert_eq!(
            date_range.start,
            chrono::DateTime::parse_from_rfc3339("2023-05-14T11:25:25.234-05:00")
                .unwrap()
                .with_timezone(&chrono::Utc)
                .timestamp_millis()
        );
        assert_eq!(
            date_range.end,
            chrono::DateTime::parse_from_rfc3339("2023-05-14T11:25:25.234-05:00")
                .unwrap()
                .with_timezone(&chrono::Utc)
                .timestamp_millis()
        );
    }

    #[test]
    fn date_range_period() {
        let start = FHIRDateTime {
            id: None,
            extension: None,
            value: Some(DateTime::Year(2023)),
        };

        let end = FHIRDateTime {
            id: None,
            extension: None,
            value: Some(DateTime::YearMonthDay(2023, 5, 15)),
        };

        let period = Period {
            id: None,
            extension: None,
            start: Some(Box::new(start)),
            end: Some(Box::new(end)),
        };

        let range = index_date(&period).unwrap();
        let date_range = range.first().unwrap();

        assert_eq!(
            date_range.start,
            chrono::DateTime::parse_from_rfc3339("2023-01-01T00:00:00Z")
                .unwrap()
                .timestamp_millis()
        );
        assert_eq!(
            date_range.end,
            chrono::DateTime::parse_from_rfc3339("2023-05-15T23:59:59.999Z")
                .unwrap()
                .timestamp_millis()
        );
    }

    #[test]
    fn date_range_missing() {
        let start = FHIRDateTime {
            id: None,
            extension: None,
            value: Some(DateTime::Year(2023)),
        };

        let end = FHIRDateTime {
            id: None,
            extension: None,
            value: Some(DateTime::YearMonthDay(2023, 5, 15)),
        };

        let period = Period {
            id: None,
            extension: None,
            start: None,
            end: Some(Box::new(end)),
        };

        let range = index_date(&period).unwrap();
        let date_range = range.first().unwrap();

        assert_eq!(date_range.start, 0);
        assert_eq!(
            date_range.end,
            chrono::DateTime::parse_from_rfc3339("2023-05-15T23:59:59.999Z")
                .unwrap()
                .timestamp_millis()
        );

        let period = Period {
            id: None,
            extension: None,
            start: Some(Box::new(start)),
            end: None,
        };

        let range = index_date(&period).unwrap();
        let date_range = range.first().unwrap();

        assert_eq!(
            date_range.start,
            chrono::DateTime::parse_from_rfc3339("2023-01-01T00:00:00Z")
                .unwrap()
                .timestamp_millis()
        );
        assert_eq!(date_range.end, i64::MAX);
    }

    #[test]
    fn test_date_range_end() {
        let year = 2023;
        let month: u8 = 12;
        let day = 31;
        let date_range = year_month_day_to_daterange(year, month, day).unwrap();

        assert_eq!(
            date_range.start,
            chrono::DateTime::parse_from_rfc3339("2023-12-31T00:00:00Z")
                .unwrap()
                .timestamp_millis()
        );

        assert_eq!(
            date_range.end,
            chrono::DateTime::parse_from_rfc3339("2023-12-31T23:59:59.999Z")
                .unwrap()
                .timestamp_millis()
        );

        let date_range = year_month_to_daterange(year, month).unwrap();

        assert_eq!(
            date_range.start,
            chrono::DateTime::parse_from_rfc3339("2023-12-01T00:00:00Z")
                .unwrap()
                .timestamp_millis()
        );

        assert_eq!(
            date_range.end,
            chrono::DateTime::parse_from_rfc3339("2023-12-31T23:59:59.999Z")
                .unwrap()
                .timestamp_millis()
        );
    }

    #[test]
    fn test_timing() {
        let timing = Timing {
            event: Some(vec![
                FHIRDateTime {
                    id: None,
                    extension: None,
                    value: Some(DateTime::YearMonthDay(2023, 12, 31)),
                },
                FHIRDateTime {
                    id: None,
                    extension: None,
                    value: Some(DateTime::YearMonthDay(2024, 1, 1)),
                },
            ]),
            ..Default::default()
        };

        let date_ranges = index_date(&timing).unwrap();
        assert_eq!(date_ranges.len(), 2);

        assert_eq!(
            date_ranges[0],
            DateRange {
                start: chrono::DateTime::parse_from_rfc3339("2023-12-31T00:00:00Z")
                    .unwrap()
                    .timestamp_millis(),
                end: chrono::DateTime::parse_from_rfc3339("2023-12-31T23:59:59.999Z")
                    .unwrap()
                    .timestamp_millis(),
            }
        );

        assert_eq!(
            date_ranges[1],
            DateRange {
                start: chrono::DateTime::parse_from_rfc3339("2024-01-01T00:00:00Z")
                    .unwrap()
                    .timestamp_millis(),
                end: chrono::DateTime::parse_from_rfc3339("2024-01-01T23:59:59.999Z")
                    .unwrap()
                    .timestamp_millis(),
            }
        );
    }

    #[test]
    fn test_indexing_reference() {
        let reference = Reference {
            type_: None,
            identifier_: None,
            display: None,
            id: None,
            extension: None,
            reference: Some(Box::new(FHIRString {
                id: None,
                extension: None,
                value: Some("Patient/123".to_string()),
            })),
        };

        let result = index_reference(&reference).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].resource_type, Some("Patient".to_string()));
        assert_eq!(result[0].id, Some("123".to_string()));
        assert!(result[0].uri.is_none());
    }

    #[test]
    fn test_indexing_invalid_reference() {
        let reference = Reference {
            type_: None,
            identifier_: None,
            display: None,
            id: None,
            extension: None,
            reference: Some(Box::new(FHIRString {
                id: None,
                extension: None,
                value: Some("BadType/123".to_string()),
            })),
        };

        let result = index_reference(&reference);

        assert!(result.is_err());
    }

    #[test]
    fn test_canonical_index() {
        let canonical = FHIRCanonical {
            id: None,
            extension: None,
            value: Some("http://example.com/CanonicalResource".to_string()),
        };
        let result = index_reference(&canonical).unwrap();
        assert_eq!(result.len(), 1);
        assert!(result[0].id.is_none());
        assert!(result[0].resource_type.is_none());
        assert_eq!(
            result[0].uri,
            Some("http://example.com/CanonicalResource".to_string())
        );
    }

    #[test]
    fn test_uri_indexing() {
        let uri = FHIRUri {
            id: None,
            extension: None,
            value: Some("http://example.com/URIResource".to_string()),
        };
        let result = index_reference(&uri).unwrap();
        assert_eq!(result.len(), 1);
        assert!(result[0].id.is_none());
        assert!(result[0].resource_type.is_none());
        assert_eq!(
            result[0].uri,
            Some("http://example.com/URIResource".to_string())
        );
    }
}
