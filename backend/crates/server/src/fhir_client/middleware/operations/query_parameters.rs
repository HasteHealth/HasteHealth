//! Types the query string of an operation invoked with GET. The spec allows
//! GET only for operations with `affectsState = false` and primitive inputs.
use haste_fhir_client::request::InvocationRequest;
use haste_fhir_model::r4::generated::{
    resources::{OperationDefinition, Parameters, ParametersParameterValueTypeChoice},
    terminology::{BoundCode, IssueType, OperationParameterUse},
};
use haste_fhir_operation_error::OperationOutcomeError;
use serde_json::{Map, Number, Value, json};

fn invalid(message: String) -> OperationOutcomeError {
    OperationOutcomeError::error(IssueType::invalid(), message)
}

/// The primitive `raw` as the JSON value of a `type` element, or `None` if
/// `type` is not primitive.
fn primitive(kind: &str, raw: &str) -> Option<Result<Value, String>> {
    Some(match kind {
        "boolean" => raw
            .parse()
            .map(Value::Bool)
            .map_err(|_| "a boolean".to_string()),
        "integer" | "positiveInt" | "unsignedInt" => raw
            .parse::<i64>()
            .map(Value::from)
            .map_err(|_| "an integer".to_string()),
        "decimal" => raw
            .parse::<Number>()
            .map(Value::Number)
            .map_err(|_| "a decimal".to_string()),
        "string" | "uri" | "url" | "canonical" | "base64Binary" | "instant" | "date"
        | "dateTime" | "time" | "code" | "oid" | "id" | "markdown" | "uuid" => {
            Ok(Value::String(raw.to_string()))
        }
        _ => return None,
    })
}

/// `query`'s untyped parameters as the typed input parameters of `definition`.
pub fn typed_parameters(
    definition: &OperationDefinition,
    query: &Parameters,
) -> Result<Parameters, OperationOutcomeError> {
    let inputs = definition.parameter.as_deref().unwrap_or_default();
    let mut typed = Vec::new();

    for parameter in query.parameter.as_deref().unwrap_or_default() {
        let name = parameter.name.value.as_deref().unwrap_or_default();
        let Some(ParametersParameterValueTypeChoice::String(raw)) = &parameter.value else {
            continue;
        };
        let raw = raw.value.as_deref().unwrap_or_default();

        let input = inputs
            .iter()
            .find(|input| {
                input.use_ == OperationParameterUse::in_()
                    && input.name.value.as_deref() == Some(name)
            })
            .ok_or_else(|| invalid(format!("Unknown parameter '{name}'.")))?;
        let kind = input.type_.as_ref().and_then(BoundCode::as_str);
        let value = kind
            .and_then(|kind| primitive(kind, raw))
            .ok_or_else(|| {
                invalid(format!(
                    "Parameter '{name}' is not a primitive and cannot be sent in a query string."
                ))
            })?
            .map_err(|expected| invalid(format!("Parameter '{name}' must be {expected}.")))?;

        let mut entry = Map::new();
        entry.insert("name".to_string(), Value::String(name.to_string()));
        let kind = kind.unwrap_or_default();
        entry.insert(
            format!("value{}{}", kind[..1].to_uppercase(), &kind[1..]),
            value,
        );
        typed.push(Value::Object(entry));
    }

    serde_json::from_value(json!({ "resourceType": "Parameters", "parameter": typed }))
        .map_err(|error| invalid(format!("Invalid query parameters: {error}")))
}

/// `request`'s parameters.
pub fn parameters_mut(request: &mut InvocationRequest) -> &mut Parameters {
    match request {
        InvocationRequest::Instance(request) => &mut request.parameters,
        InvocationRequest::Type(request) => &mut request.parameters,
        InvocationRequest::System(request) => &mut request.parameters,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fhir_http::get_operation_body;

    fn definition() -> OperationDefinition {
        serde_json::from_value(json!({
            "resourceType": "OperationDefinition",
            "name": "t", "status": "active", "kind": "operation", "code": "t",
            "system": true, "type": false, "instance": false,
            "parameter": [
                {"name": "flag", "use": "in", "min": 0, "max": "1", "type": "boolean"},
                {"name": "count", "use": "in", "min": 0, "max": "*", "type": "integer"},
                {"name": "valueSet", "use": "in", "min": 0, "max": "1", "type": "ValueSet"}
            ]
        }))
        .unwrap()
    }

    #[test]
    fn types_and_repeats() {
        let typed = typed_parameters(
            &definition(),
            &get_operation_body("flag=true&count=1&count=2"),
        )
        .unwrap();
        let json = serde_json::to_value(&typed).unwrap();
        assert_eq!(
            json["parameter"],
            json!([
                {"name": "flag", "valueBoolean": true},
                {"name": "count", "valueInteger": 1},
                {"name": "count", "valueInteger": 2}
            ])
        );
    }

    #[test]
    fn refuses_unknown_non_primitive_and_malformed() {
        for query in ["nope=1", "valueSet=x", "flag=maybe", "count=1.5"] {
            assert!(typed_parameters(&definition(), &get_operation_body(query)).is_err());
        }
    }
}
