//! Types the query string of an operation invoked with GET. The spec allows
//! GET only for operations with `affectsState = false` and primitive inputs.
use super::{find_custom_operation, not_defined};
use crate::fhir_client::{
    ServerCTX,
    middleware::{Backend, operation_definitions::Builtins},
};
use haste_fhir_client::request::InvocationRequest;
use haste_fhir_model::r4::generated::{
    resources::{OperationDefinition, Parameters},
    terminology::{BoundCode, IssueType, OperationParameterUse},
};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_fhir_ops::defines;
use serde_json::{Map, Number, Value, json};
use std::sync::Arc;

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

/// `query` as the typed input parameters of `definition`. Each query parameter
/// becomes a parameter of that name, repeats included.
fn typed_parameters(
    definition: &OperationDefinition,
    query: &str,
) -> Result<Parameters, OperationOutcomeError> {
    let inputs = definition.parameter.as_deref().unwrap_or_default();
    let mut typed = Vec::new();

    for (name, raw) in url::form_urlencoded::parse(query.as_bytes()) {
        let input = inputs
            .iter()
            .find(|input| {
                input.use_ == OperationParameterUse::in_()
                    && input.name.value.as_deref() == Some(name.as_ref())
            })
            .ok_or_else(|| invalid(format!("Unknown parameter '{name}'.")))?;
        let kind = input.type_.as_ref().and_then(BoundCode::as_str);
        let value = kind
            .and_then(|kind| primitive(kind, &raw))
            .ok_or_else(|| {
                invalid(format!(
                    "Parameter '{name}' is not a primitive and cannot be sent in a query string."
                ))
            })?
            .map_err(|expected| invalid(format!("Parameter '{name}' must be {expected}.")))?;

        let mut entry = Map::new();
        entry.insert("name".to_string(), Value::String(name.into_owned()));
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
fn parameters_mut(request: &mut InvocationRequest) -> &mut Parameters {
    match request {
        InvocationRequest::Instance(request) => &mut request.parameters,
        InvocationRequest::Type(request) => &mut request.parameters,
        InvocationRequest::System(request) => &mut request.parameters,
    }
}

/// Sets the parameters of `request`, invoked with GET, from its `query`
/// string. A GET may only run an operation that leaves state alone.
pub async fn type_query_invocation<B: Backend>(
    builtins: &Builtins,
    ctx: &Arc<ServerCTX<B>>,
    request: &mut InvocationRequest,
    query: &str,
) -> Result<(), OperationOutcomeError> {
    let custom;
    let definition = match builtins.iter().find(|builtin| defines(builtin, request)) {
        Some(builtin) => *builtin,
        None => {
            custom = find_custom_operation(ctx, request)
                .await?
                .ok_or_else(|| not_defined(request))?;
            &custom
        }
    };

    if definition
        .affectsState
        .as_ref()
        .and_then(|affects| affects.value)
        .unwrap_or(false)
    {
        return Err(OperationOutcomeError::error(
            IssueType::business_rule(),
            format!(
                "Operation '{}' affects state and must be invoked with POST.",
                definition.code.value.as_deref().unwrap_or_default()
            ),
        ));
    }

    *parameters_mut(request) = typed_parameters(definition, query)?;
    Ok(())
}
