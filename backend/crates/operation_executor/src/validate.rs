use haste_fhir_model::r4::generated::{
    resources::{
        OperationDefinitionParameter, OperationOutcome, OperationOutcomeIssue, Parameters,
        ParametersParameter,
    },
    terminology::{BoundCode, IssueSeverity, IssueType, OperationParameterUse},
};
use haste_fhir_operation_error::OperationOutcomeError;
use haste_reflect::MetaValue as _;

/// Which direction of `OperationDefinition.parameter` to validate against.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParameterDirection {
    In,
    Out,
}

fn create_issue(
    severity: BoundCode<IssueSeverity>,
    type_: BoundCode<IssueType>,
    diagnostics: String,
) -> OperationOutcomeIssue {
    OperationOutcomeIssue {
        severity,
        code: type_,
        diagnostics: Some(Box::new(diagnostics.into())),
        ..Default::default()
    }
}

/// Validates supplied [`Parameters`] against the [`OperationDefinitionParameter`] definitions
/// for the specified operation direction.
///
/// The validation checks parameter cardinality, validates the type of supplied values or
/// resources against the expected parameter type, recursively validates nested parts, and
/// reports parameters that are not defined for the specified direction.
///
/// # Arguments
///
/// * `parameters` - The parameters supplied for the operation.
/// * `operation_params` - The parameter definitions from the operation's
///   [`OperationDefinitionParameter`].
/// * `direction` - The operation parameter direction to validate, such as `in` or `out`.
///
/// # Errors
///
/// Returns an [`OperationOutcomeError`] containing one or more issues if:
///
/// * A parameter has fewer occurrences than its minimum cardinality.
/// * A parameter exceeds its maximum cardinality.
/// * A supplied parameter has a different type than the type defined by the operation.
/// * A nested parameter part does not satisfy its corresponding definition.
/// * A supplied parameter is not defined for the specified operation direction.
///
/// Multiple validation failures are collected and returned together in a single
/// [`OperationOutcomeError`].
pub fn validate_parameters(
    parameters: &Parameters,
    operation_params: &[OperationDefinitionParameter],
    direction: &BoundCode<OperationParameterUse>,
) -> Result<(), OperationOutcomeError> {
    let parameter_definitions: Vec<&OperationDefinitionParameter> = operation_params
        .iter()
        .filter(|p| &p.use_ == direction)
        .collect();

    let parameters_to_validate: &[ParametersParameter] =
        parameters.parameter.as_deref().unwrap_or_default();

    let mut issues: Vec<OperationOutcomeIssue> = Vec::new();

    // --- Check each definition against what was supplied ---
    for parameter_definition in &parameter_definitions {
        let Some(name) = parameter_definition.name.value.as_deref() else {
            continue;
        };

        let found_parameters: Vec<&ParametersParameter> = parameters_to_validate
            .iter()
            .filter(|p| p.name.value.as_deref() == Some(name))
            .collect();

        let count = found_parameters.len() as u64;

        // Minimum cardinality
        let min = parameter_definition.min.value.unwrap_or(0).cast_unsigned();
        if count < min {
            issues.push(create_issue(
                IssueSeverity::error(),
                IssueType::invariant(),
                format!(
                    "Parameter '{name}' requires at least {min} occurrence(s) but only {count} were supplied."
                ),
            ));
        }

        // Maximum cardinality ("*" means unbounded)
        if let Some(max_str) = parameter_definition.max.value.as_deref()
            && max_str != "*"
            && let Ok(max) = max_str.parse::<u64>()
            && count > max
        {
            issues.push(create_issue(IssueSeverity::error(), IssueType::invariant(),
                        format!(
                                "Parameter '{name}' allows a maximum of {max} occurrence(s) but {count} were supplied."
                            )));
        }

        // Validate type if specified. The type of a supplied parameter is determined by:
        // 1. If it has a `resource` field, use the resource type.
        // 2. Otherwise, use the type of the `value` field.
        if let Some(parameter_def_type) = &parameter_definition.type_ {
            let type_name = parameter_def_type.as_str();
            for found_parameter in &found_parameters {
                let type_ = if let Some(resource) = found_parameter.resource.as_ref() {
                    resource.fhir_type()
                } else {
                    found_parameter.value.fhir_type()
                };

                if type_ != type_name.unwrap_or_default() {
                    issues.push(create_issue(
                        IssueSeverity::error(),
                        IssueType::invalid(),
                        format!(
                            "Parameter '{name}' expects type '{}' but found '{type_}'.",
                            type_name.unwrap_or("<unknown>"),
                        ),
                    ));
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
                    validate_parameters(&parts_as_parameters, part_defs, direction)?;
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
            let display_direction = direction.as_str();
            issues.push(create_issue(
                IssueSeverity::error(),
                IssueType::invalid(),
                format!(
                    "Parameter '{}' is not defined for the '{}' direction.",
                    name,
                    display_direction.unwrap_or("<unknown>")
                ),
            ));
        }
    }

    if issues.is_empty() {
        Ok(())
    } else {
        Err(OperationOutcomeError::new(
            None,
            OperationOutcome {
                issue: issues,
                ..Default::default()
            },
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use haste_fhir_model::r4::generated::{
        resources::{
            OperationDefinitionParameter, Parameters, ParametersParameter,
            ParametersParameterValueTypeChoice, Patient, Practitioner, Resource,
        },
        terminology::{AllTypes, OperationParameterUse},
        types::{FHIRCode, FHIRInteger, FHIRString},
    };

    fn make_def(
        name: &str,
        direction: BoundCode<OperationParameterUse>,
        min: i64,
        max: &str,
        type_: Option<BoundCode<AllTypes>>,
    ) -> OperationDefinitionParameter {
        OperationDefinitionParameter {
            name: Box::new(FHIRCode {
                value: Some(name.to_string()),
                ..Default::default()
            }),
            use_: direction,
            min: Box::new(FHIRInteger {
                value: Some(min),
                ..Default::default()
            }),
            max: Box::new(FHIRString {
                value: Some(max.to_string()),
                ..Default::default()
            }),
            type_,
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
        let defs = vec![make_def(
            "subject",
            OperationParameterUse::in_(),
            1,
            "1",
            None,
        )];
        let params = Parameters {
            parameter: None,
            ..Default::default()
        };
        assert!(validate_parameters(&params, &defs, &OperationParameterUse::in_()).is_err());
    }

    #[test]
    fn required_param_present_passes() {
        let defs = vec![make_def(
            "subject",
            OperationParameterUse::in_(),
            1,
            "1",
            None,
        )];
        let params = Parameters {
            parameter: Some(vec![make_param("subject")]),
            ..Default::default()
        };
        assert!(validate_parameters(&params, &defs, &OperationParameterUse::in_()).is_ok());
    }

    #[test]
    fn extra_param_is_rejected() {
        let defs = vec![make_def(
            "subject",
            OperationParameterUse::in_(),
            0,
            "1",
            None,
        )];
        let params = Parameters {
            parameter: Some(vec![make_param("unknown")]),
            ..Default::default()
        };
        assert!(validate_parameters(&params, &defs, &OperationParameterUse::in_()).is_err());
    }

    #[test]
    fn max_exceeded_fails() {
        let defs = vec![make_def(
            "subject",
            OperationParameterUse::in_(),
            0,
            "1",
            None,
        )];
        let params = Parameters {
            parameter: Some(vec![make_param("subject"), make_param("subject")]),
            ..Default::default()
        };
        assert!(validate_parameters(&params, &defs, &OperationParameterUse::in_()).is_err());
    }

    #[test]
    fn out_direction_ignored_for_in_validation() {
        // An "out" definition should be invisible when validating "in"
        let defs = vec![make_def(
            "result",
            OperationParameterUse::out(),
            1,
            "1",
            None,
        )];
        let params = Parameters {
            parameter: None,
            ..Default::default()
        };
        // No "in" definitions exist, so nothing to violate → should pass.
        assert!(validate_parameters(&params, &defs, &OperationParameterUse::in_()).is_ok());
    }

    #[test]
    fn unbounded_max_passes() {
        let defs = vec![make_def("note", OperationParameterUse::in_(), 0, "*", None)];
        let params = Parameters {
            parameter: Some(vec![
                make_param("note"),
                make_param("note"),
                make_param("note"),
            ]),
            ..Default::default()
        };
        assert!(validate_parameters(&params, &defs, &OperationParameterUse::in_()).is_ok());
    }

    #[test]
    fn basic_type_validation() {
        let defs = vec![make_def(
            "note",
            OperationParameterUse::in_(),
            0,
            "*",
            Some(AllTypes::string()),
        )];

        let mut parameter_note = make_param("note");
        parameter_note.value = Some(ParametersParameterValueTypeChoice::String(Box::new(
            FHIRString {
                value: Some("This is a note.".to_string()),
                ..Default::default()
            },
        )));

        let params = Parameters {
            parameter: Some(vec![parameter_note.clone(), parameter_note.clone()]),
            ..Default::default()
        };

        assert!(validate_parameters(&params, &defs, &OperationParameterUse::in_()).is_ok());

        parameter_note.value = Some(ParametersParameterValueTypeChoice::Integer(Box::new(
            FHIRInteger {
                value: Some(42),
                ..Default::default()
            },
        )));

        let params = Parameters {
            parameter: Some(vec![parameter_note]),
            ..Default::default()
        };

        assert!(validate_parameters(&params, &defs, &OperationParameterUse::in_()).is_err());
    }

    #[test]
    fn resource_validation() {
        let defs = vec![make_def(
            "note",
            OperationParameterUse::in_(),
            0,
            "*",
            Some(AllTypes::patient()),
        )];

        let mut parameter_note = make_param("note");
        parameter_note.resource = Some(Box::new(Resource::Patient(Patient {
            ..Default::default()
        })));

        let params = Parameters {
            parameter: Some(vec![parameter_note.clone(), parameter_note.clone()]),
            ..Default::default()
        };

        assert!(validate_parameters(&params, &defs, &OperationParameterUse::in_()).is_ok());

        parameter_note.resource = Some(Box::new(Resource::Practitioner(Practitioner {
            ..Default::default()
        })));

        let params = Parameters {
            parameter: Some(vec![parameter_note]),
            ..Default::default()
        };

        assert!(validate_parameters(&params, &defs, &OperationParameterUse::in_()).is_err());
    }

    #[test]
    fn test_nested() {
        let mut parent = make_def("parent", OperationParameterUse::in_(), 1, "1", None);

        parent.part = Some(vec![make_def(
            "child",
            OperationParameterUse::in_(),
            1,
            "1",
            Some(AllTypes::string()),
        )]);

        let defs = vec![parent];

        let mut child_param = make_param("child");
        child_param.value = Some(ParametersParameterValueTypeChoice::String(Box::new(
            FHIRString {
                value: Some("I am a child parameter.".to_string()),
                ..Default::default()
            },
        )));

        let mut parent_param = make_param("parent");
        parent_param.part = Some(vec![child_param.clone()]);

        let params = Parameters {
            parameter: Some(vec![parent_param.clone()]),
            ..Default::default()
        };

        assert!(validate_parameters(&params, &defs, &OperationParameterUse::in_()).is_ok());

        child_param.value = Some(ParametersParameterValueTypeChoice::Integer(Box::new(
            FHIRInteger {
                value: Some(42),
                ..Default::default()
            },
        )));

        parent_param.part = Some(vec![child_param]);

        let params = Parameters {
            parameter: Some(vec![parent_param.clone()]),
            ..Default::default()
        };

        assert!(validate_parameters(&params, &defs, &OperationParameterUse::in_()).is_err());
    }
}
