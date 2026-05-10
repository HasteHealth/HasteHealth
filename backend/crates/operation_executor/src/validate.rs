use haste_fhir_model::r4::generated::{
    resources::{
        OperationDefinitionParameter, OperationOutcomeIssue, Parameters, ParametersParameter,
    },
    terminology::{IssueSeverity, IssueType, OperationParameterUse},
    types::FHIRString,
};
use haste_fhir_operation_error::OperationOutcomeError;

/// Which direction of `OperationDefinition.parameter` to validate against.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParameterDirection {
    In,
    Out,
}

fn create_issue(
    severity: IssueSeverity,
    type_: IssueType,
    diagnostics: String,
) -> OperationOutcomeIssue {
    OperationOutcomeIssue {
        severity: Box::new(severity),
        code: Box::new(type_),
        diagnostics: Some(Box::new(FHIRString {
            value: Some(diagnostics),
            ..Default::default()
        })),
        ..Default::default()
    }
}

/// Validate a [`Parameters`] resource against an [`OperationDefinition`]'s parameter list.
///
/// Only parameters whose `use` matches `direction` are considered.  
/// Returns `Ok(())` if every constraint is satisfied, otherwise an
/// [`OperationOutcomeError`] that accumulates **all** violations found.
pub fn validate_parameters(
    parameters: &Parameters,
    operation_params: &[OperationDefinitionParameter],
    direction: OperationParameterUse,
) -> Result<(), OperationOutcomeError> {
    let parameter_definitions: Vec<&OperationDefinitionParameter> = operation_params
        .iter()
        .filter(|p| matches!(p.use_, direction))
        .collect();

    let parameters_to_validate: &[ParametersParameter] =
        parameters.parameter.as_deref().unwrap_or_default();

    let mut issues: Vec<OperationOutcomeIssue> = Vec::new();

    // --- Check each definition against what was supplied ---
    for parameter_definition in &parameter_definitions {
        let name = match parameter_definition.name.value.as_deref() {
            Some(n) => n,
            None => continue,
        };

        let found_parameters: Vec<&ParametersParameter> = parameters_to_validate
            .iter()
            .filter(|p| p.name.value.as_deref() == Some(name))
            .collect();

        let count = found_parameters.len() as i64;

        // Minimum cardinality
        let min = parameter_definition.min.value.unwrap_or(0);
        if count < min {
            issues.push(OperationOutcomeIssue {
                severity: Some("error".to_string()),
                code: Some("invalid".to_string()),
                diagnostics: Some(format!(
                    "Parameter '{}' requires at least {} occurrence(s) but only {} were supplied.",
                    name, min, count
                )),
                ..Default::default()
            });
        }

        // Maximum cardinality ("*" means unbounded)
        if let Some(max_str) = parameter_definition.max.value.as_deref() {
            if max_str != "*" {
                if let Ok(max) = max_str.parse::<i64>() {
                    if count > max {
                        issues.push(OperationOutcomeIssue {
                            severity: Some("error".to_string()),
                            code: Some("invalid".to_string()),
                            diagnostics: Some(format!(
                                "Parameter '{}' allows a maximum of {} occurrence(s) but {} were supplied.",
                                name, max, count
                            )),
                            ..Default::default()
                        });
                    }
                }
            }
        }

        // Recursively validate parts when both the definition and the
        // supplied parameter declare nested parts.
        if let Some(part_defs) = &parameter_definition.part {
            for supplied_param in &found_parameters {
                if let Some(supplied_parts) = &supplied_param.part {
                    let parts_as_parameters = Parameters {
                        parameter: Some(supplied_parts.clone()),
                        ..Default::default()
                    };
                    if let Err(part_error) =
                        validate_parameters(&parts_as_parameters, part_defs, direction)
                    {
                        issues.push(OperationOutcomeIssue {
                            severity: Some("error".to_string()),
                            code: Some("invalid".to_string()),
                            diagnostics: Some(format!(
                                "In parameter '{}': {}",
                                name,
                                part_error
                                    .outcome()
                                    .issue
                                    .as_deref()
                                    .unwrap_or_default()
                                    .iter()
                                    .filter_map(|i| i.diagnostics.as_ref()?.value.as_deref())
                                    .collect::<Vec<_>>()
                                    .join("; ")
                            )),
                            ..Default::default()
                        });
                    }
                }
            }
        }
    }

    // --- Warn about parameters that have no matching definition ---
    for supplied_param in parameters_to_validate {
        let name = supplied_param.name.value.as_deref().unwrap_or("<unnamed>");
        let defined = parameter_definitions
            .iter()
            .any(|d| d.name.value.as_deref() == Some(name));
        if !defined {
            issues.push(format!(
                "Parameter '{}' is not defined for the '{}' direction.",
                name,
                direction
                    .into::<Option<String>>()
                    .unwrap_or("<unknown>".to_string())
            ));
        }
    }

    if issues.is_empty() {
        Ok(())
    } else {
        let mut error = OperationOutcomeError::error(IssueType::Invalid(None), issues[0].clone());
        for message in issues.into_iter().skip(1) {
            error.push_issue(IssueType::Invalid(None), message);
        }
        Err(error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use haste_fhir_model::r4::generated::{
        resources::{OperationDefinitionParameter, Parameters, ParametersParameter},
        terminology::OperationParameterUse,
        types::{FHIRCode, FHIRInteger, FHIRString},
    };

    fn make_def(
        name: &str,
        direction: OperationParameterUse,
        min: i64,
        max: &str,
    ) -> OperationDefinitionParameter {
        OperationDefinitionParameter {
            name: Box::new(FHIRCode {
                value: Some(name.to_string()),
                ..Default::default()
            }),
            use_: Box::new(direction),
            min: Box::new(FHIRInteger {
                value: Some(min),
                ..Default::default()
            }),
            max: Box::new(FHIRString {
                value: Some(max.to_string()),
                ..Default::default()
            }),
            ..Default::default()
        }
    }

    fn make_param(name: &str) -> ParametersParameter {
        ParametersParameter {
            name: Box::new(FHIRString {
                value: Some(name.to_string()),
                ..Default::default()
            }),
            ..Default::default()
        }
    }

    #[test]
    fn required_param_missing_fails() {
        let defs = vec![make_def("subject", OperationParameterUse::In(None), 1, "1")];
        let params = Parameters {
            parameter: None,
            ..Default::default()
        };
        assert!(validate_parameters(&params, &defs, OperationParameterUse::In(None)).is_err());
    }

    #[test]
    fn required_param_present_passes() {
        let defs = vec![make_def("subject", OperationParameterUse::In(None), 1, "1")];
        let params = Parameters {
            parameter: Some(vec![make_param("subject")]),
            ..Default::default()
        };
        assert!(validate_parameters(&params, &defs, OperationParameterUse::In(None)).is_ok());
    }

    #[test]
    fn extra_param_is_rejected() {
        let defs = vec![make_def("subject", OperationParameterUse::In(None), 0, "1")];
        let params = Parameters {
            parameter: Some(vec![make_param("unknown")]),
            ..Default::default()
        };
        assert!(validate_parameters(&params, &defs, OperationParameterUse::In(None)).is_err());
    }

    #[test]
    fn max_exceeded_fails() {
        let defs = vec![make_def("subject", OperationParameterUse::In(None), 0, "1")];
        let params = Parameters {
            parameter: Some(vec![make_param("subject"), make_param("subject")]),
            ..Default::default()
        };
        assert!(validate_parameters(&params, &defs, OperationParameterUse::In(None)).is_err());
    }

    #[test]
    fn out_direction_ignored_for_in_validation() {
        // An "out" definition should be invisible when validating "in"
        let defs = vec![make_def("result", OperationParameterUse::Out(None), 1, "1")];
        let params = Parameters {
            parameter: None,
            ..Default::default()
        };
        // No "in" definitions exist, so nothing to violate → should pass.
        assert!(validate_parameters(&params, &defs, OperationParameterUse::In(None)).is_ok());
    }

    #[test]
    fn unbounded_max_passes() {
        let defs = vec![make_def("note", OperationParameterUse::In(None), 0, "*")];
        let params = Parameters {
            parameter: Some(vec![
                make_param("note"),
                make_param("note"),
                make_param("note"),
            ]),
            ..Default::default()
        };
        assert!(validate_parameters(&params, &defs, OperationParameterUse::In(None)).is_ok());
    }
}
