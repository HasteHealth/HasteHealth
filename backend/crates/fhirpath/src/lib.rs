#![allow(unused)]

mod error;
mod parser;
use crate::{
    error::{FunctionError, OperationError},
    parser::{Expression, FunctionInvocation, Identifier, Invocation, Literal, Operation, Term},
};
use dashmap::DashMap;
pub use error::FHIRPathError;
use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    marker::PhantomData,
    rc::Rc,
    sync::{Arc, LazyLock, Mutex},
};
// use owning_ref::BoxRef;
use once_cell::sync::Lazy;
use oxidized_fhir_model::r4::generated::{
    resources::ResourceType,
    types::{
        FHIRBase64Binary, FHIRBoolean, FHIRCanonical, FHIRCode, FHIRDecimal, FHIRInteger, FHIROid,
        FHIRPositiveInt, FHIRString, FHIRUnsignedInt, FHIRUri, FHIRUrl, FHIRUuid, FHIRXhtml,
        Reference,
    },
};
use oxidized_reflect::MetaValue;
use oxidized_reflect_derive::Reflect;

/// Number types to use in FHIR evaluation
static NUMBER_TYPES: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    let mut m = HashSet::new();
    m.insert("FHIRInteger");
    m.insert("FHIRDecimal");
    m.insert("FHIRPositiveInt");
    m.insert("FHIRUnsignedInt");
    m.insert("http://hl7.org/fhirpath/System.Decimal");
    m.insert("http://hl7.org/fhirpath/System.Integer");
    m
});

static BOOLEAN_TYPES: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    let mut m = HashSet::new();
    m.insert("FHIRBoolean");
    m.insert("http://hl7.org/fhirpath/System.Boolean");
    m
});

static DATE_TIME_TYPES: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    let mut m = HashSet::new();
    m.insert("FHIRDate");
    m.insert("FHIRDateTime");
    m.insert("FHIRInstant");
    m.insert("FHIRTime");
    m.insert("http://hl7.org/fhirpath/System.DateTime");
    m.insert("http://hl7.org/fhirpath/System.Instant");
    m.insert("http://hl7.org/fhirpath/System.Date");
    m.insert("http://hl7.org/fhirpath/System.Time");
    m
});

static STRING_TYPES: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    let mut m = HashSet::new();
    m.insert("FHIRBase64Binary");
    m.insert("FHIRCanonical");

    m.insert("FHIRCode");
    m.insert("FHIRString");
    m.insert("FHIROid");
    m.insert("FHIRUri");
    m.insert("FHIRUrl");
    m.insert("FHIRUuid");
    m.insert("FHIRXhtml");

    m.insert("http://hl7.org/fhirpath/System.String");
    m
});

fn evaluate_literal<'b>(
    literal: &Literal,
    context: Context<'b>,
) -> Result<Context<'b>, FHIRPathError> {
    match literal {
        Literal::String(string) => {
            Ok(context.new_context_from(vec![context.allocate(Box::new(string.clone()))]))
        }
        Literal::Integer(int) => {
            Ok(context.new_context_from(vec![context.allocate(Box::new(int.clone()))]))
        }
        Literal::Float(decimal) => {
            Ok(context.new_context_from(vec![context.allocate(Box::new(decimal.clone()))]))
        }
        Literal::Boolean(bool) => {
            Ok(context.new_context_from(vec![context.allocate(Box::new(bool.clone()))]))
        }
        Literal::Null => Ok(context.new_context_from(vec![])),
        _ => Err(FHIRPathError::InvalidLiteral(literal.to_owned())),
    }
}

fn evaluate_invocation<'a>(
    invocation: &Invocation,
    context: Context<'a>,
    config: &'a Option<Config<'a>>,
) -> Result<Context<'a>, FHIRPathError> {
    match invocation {
        Invocation::This => Ok(context),
        Invocation::Index(index_expression) => {
            let index = evaluate_expression(index_expression, context.clone(), config)?;
            if index.values.len() != 1 {
                return Err(FHIRPathError::OperationError(
                    OperationError::InvalidCardinality,
                ));
            }
            let index = downcast_number(index.values[0])? as usize;
            if let Some(value) = context.values.get(index) {
                Ok(context.new_context_from(vec![*value]))
            } else {
                Ok(context.new_context_from(vec![]))
            }
        }
        Invocation::IndexAccessor => Err(FHIRPathError::NotImplemented("index access".to_string())),
        Invocation::Total => Err(FHIRPathError::NotImplemented("total".to_string())),
        Invocation::Identifier(Identifier(id)) => Ok(context.new_context_from(
            context
                .values
                .iter()
                .flat_map(|v| {
                    v.get_field(id)
                        .map(|v| v.flatten())
                        .unwrap_or_else(|| vec![])
                })
                .collect(),
        )),
        Invocation::Function(function) => evaluate_function(function, context, config),
    }
}

fn evaluate_term<'a>(
    term: &Term,
    context: Context<'a>,
    config: &'a Option<Config<'a>>,
) -> Result<Context<'a>, FHIRPathError> {
    match term {
        Term::Literal(literal) => evaluate_literal(literal, context),
        Term::ExternalConstant(constant) => resolve_external_constant(
            constant,
            config.as_ref().and_then(|c| c.variable_resolver.as_ref()),
            context,
        ),
        Term::Parenthesized(expression) => evaluate_expression(expression, context, config),
        Term::Invocation(invocation) => evaluate_invocation(invocation, context, config),
    }
}

/// Need special handling as the first term could start with a type filter.
/// for example Patient.name
fn evaluate_first_term<'a>(
    term: &Term,
    context: Context<'a>,
    config: &'a Option<Config<'a>>,
) -> Result<Context<'a>, FHIRPathError> {
    match term {
        Term::Invocation(invocation) => match invocation {
            Invocation::Identifier(identifier) => {
                let type_filter = filter_by_type(&identifier.0, &context);
                if !type_filter.values.is_empty() {
                    Ok(type_filter)
                } else {
                    evaluate_invocation(invocation, context, config)
                }
            }
            _ => evaluate_invocation(invocation, context, config),
        },
        _ => evaluate_term(term, context, config),
    }
}

fn evaluate_singular<'a>(
    expression: &Vec<Term>,
    context: Context<'a>,
    config: &'a Option<Config<'a>>,
) -> Result<Context<'a>, FHIRPathError> {
    let mut current_context = context;

    let mut term_iterator = expression.iter();
    let first_term = term_iterator.next();
    if let Some(first_term) = first_term {
        current_context = evaluate_first_term(first_term, current_context, config)?;
    }

    for term in term_iterator {
        current_context = evaluate_term(term, current_context, config)?;
    }

    Ok(current_context)
}

fn operation_1<'a>(
    left: &Expression,
    right: &Expression,
    context: Context<'a>,
    config: &'a Option<Config<'a>>,
    executor: impl Fn(Context<'a>, Context<'a>) -> Result<Context<'a>, FHIRPathError>,
) -> Result<Context<'a>, FHIRPathError> {
    let left = evaluate_expression(left, context.clone(), config)?;
    let right = evaluate_expression(right, context, config)?;

    // If one of operands is empty per spec return an empty collection
    if (left.values.len() == 0 || right.values.len() == 0) {
        return Ok(left.new_context_from(vec![]));
    }

    if left.values.len() != 1 || right.values.len() != 1 {
        return Err(FHIRPathError::OperationError(
            OperationError::InvalidCardinality,
        ));
    }

    executor(left, right)
}

fn operation_n<'a>(
    left: &Expression,
    right: &Expression,
    context: Context<'a>,
    config: &'a Option<Config<'a>>,
    executor: impl Fn(Context<'a>, Context<'a>) -> Result<Context<'a>, FHIRPathError>,
) -> Result<Context<'a>, FHIRPathError> {
    let left = evaluate_expression(left, context.clone(), config)?;
    let right = evaluate_expression(right, context, config)?;
    executor(left, right)
}

fn downcast_bool(value: &dyn MetaValue) -> Result<bool, FHIRPathError> {
    match value.typename() {
        "http://hl7.org/fhirpath/System.Boolean" => value
            .as_any()
            .downcast_ref::<bool>()
            .map(|v| *v)
            .ok_or_else(|| FHIRPathError::FailedDowncast(value.typename().to_string())),
        "FHIRBoolean" => {
            let fp_bool = value
                .as_any()
                .downcast_ref::<FHIRBoolean>()
                .ok_or_else(|| FHIRPathError::FailedDowncast(value.typename().to_string()))?;
            downcast_bool(fp_bool.value.as_ref().unwrap_or(&false))
        }
        type_name => Err(FHIRPathError::FailedDowncast(type_name.to_string())),
    }
}

fn downcast_string(value: &dyn MetaValue) -> Result<String, FHIRPathError> {
    match value.typename() {
        "FHIRCanonical" | "FHIRBase64Binary" | "FHIRCode" | "FHIRString" | "FHIROid"
        | "FHIRUri" | "FHIRUrl" | "FHIRUuid" | "FHIRXhtml" => {
            downcast_string(value.get_field("value").unwrap_or(&"".to_string()))
        }

        "http://hl7.org/fhirpath/System.String" => value
            .as_any()
            .downcast_ref::<String>()
            .map(|v| v.clone())
            .ok_or_else(|| FHIRPathError::FailedDowncast(value.typename().to_string())),

        type_name => Err(FHIRPathError::FailedDowncast(type_name.to_string())),
    }
}

fn downcast_number(value: &dyn MetaValue) -> Result<f64, FHIRPathError> {
    match value.typename() {
        "FHIRInteger" => {
            let fp_integer = value
                .as_any()
                .downcast_ref::<FHIRInteger>()
                .ok_or_else(|| FHIRPathError::FailedDowncast(value.typename().to_string()))?;
            downcast_number(fp_integer.value.as_ref().unwrap_or(&0))
        }
        "FHIRDecimal" => {
            let fp_decimal = value
                .as_any()
                .downcast_ref::<FHIRDecimal>()
                .ok_or_else(|| FHIRPathError::FailedDowncast(value.typename().to_string()))?;
            downcast_number(fp_decimal.value.as_ref().unwrap_or(&0.0))
        }
        "FHIRPositiveInt" => {
            let fp_positive_int = value
                .as_any()
                .downcast_ref::<FHIRPositiveInt>()
                .ok_or_else(|| FHIRPathError::FailedDowncast(value.typename().to_string()))?;

            downcast_number(fp_positive_int.value.as_ref().unwrap_or(&0))
        }
        "FHIRUnsignedInt" => {
            let fp_unsigned_int = value
                .as_any()
                .downcast_ref::<FHIRUnsignedInt>()
                .ok_or_else(|| FHIRPathError::FailedDowncast(value.typename().to_string()))?;

            downcast_number(fp_unsigned_int.value.as_ref().unwrap_or(&0))
        }
        "http://hl7.org/fhirpath/System.Integer" => value
            .as_any()
            .downcast_ref::<i64>()
            .map(|v| *v as f64)
            .ok_or_else(|| FHIRPathError::FailedDowncast(value.typename().to_string())),

        "http://hl7.org/fhirpath/System.Decimal" => value
            .as_any()
            .downcast_ref::<f64>()
            .map(|v| *v)
            .ok_or_else(|| FHIRPathError::FailedDowncast(value.typename().to_string())),
        type_name => Err(FHIRPathError::FailedDowncast(type_name.to_string())),
    }
}

fn fp_func_0<'b>(
    ast_arguments: &Vec<Expression>,
    context: Context<'b>,
    executor: impl Fn(Context<'b>) -> Result<Context<'b>, FHIRPathError>,
) -> Result<Context<'b>, FHIRPathError> {
    if ast_arguments.len() != 0 {
        return Err(FHIRPathError::OperationError(
            OperationError::InvalidCardinality,
        ));
    }

    executor(context)
}

fn fp_func_1<'b>(
    ast_arguments: &Vec<Expression>,
    context: Context<'b>,
    executor: impl Fn(&Vec<Expression>, Context<'b>) -> Result<Context<'b>, FHIRPathError>,
) -> Result<Context<'b>, FHIRPathError> {
    if ast_arguments.len() != 1 {
        return Err(FHIRPathError::OperationError(
            OperationError::InvalidCardinality,
        ));
    }

    executor(ast_arguments, context)
}

fn fp_func_n<'b>(
    ast_arguments: &Vec<Expression>,
    context: Context<'b>,
    executor: impl Fn(&Vec<Expression>, Context<'b>) -> Result<Context<'b>, FHIRPathError>,
) -> Result<Context<'b>, FHIRPathError> {
    executor(ast_arguments, context)
}

fn derive_typename(expression_ast: &Expression) -> Result<String, FHIRPathError> {
    match expression_ast {
        Expression::Singular(ast) => match &ast[0] {
            Term::Invocation(Invocation::Identifier(type_id)) => Ok(type_id.0.clone()),
            _ => Err(FHIRPathError::FailedTypeNameDerivation),
        },
        _ => Err(FHIRPathError::FailedTypeNameDerivation),
    }
}

fn check_type_name(type_name: &str, type_to_check: &str) -> bool {
    match type_to_check {
        "Resource" | "DomainResource" => ResourceType::try_from(type_name).is_ok(),
        _ => type_name == type_to_check,
    }
}

fn check_type(value: &dyn MetaValue, type_to_check: &str) -> bool {
    match value.typename() {
        // Special handling for reference which is to check the reference type.
        "Reference" => {
            if type_to_check == "Reference" {
                return true;
            } else if let Some(reference) = value.as_any().downcast_ref::<Reference>() {
                if let Some(resource_type) = reference
                    .reference
                    .as_ref()
                    .and_then(|r| r.value.as_ref())
                    .and_then(|r| r.split("/").next())
                {
                    return check_type_name(resource_type, type_to_check);
                }
            }
            false
        }
        _ => check_type_name(value.typename(), type_to_check),
    }
}

fn filter_by_type<'a>(type_name: &str, context: &Context<'a>) -> Context<'a> {
    context.new_context_from(
        context
            .values
            .iter()
            .filter(|v| check_type(**v, type_name))
            .map(|v| *v)
            .collect(),
    )
}

#[derive(Debug, Reflect)]
struct Reflection {
    name: String,
}

fn evaluate_function<'b>(
    function: &FunctionInvocation,
    context: Context<'b>,
    config: &'b Option<Config<'b>>,
) -> Result<Context<'b>, FHIRPathError> {
    match function.name.0.as_str() {
        // Faking resolve to just return current context.
        "resolve" => Ok(context),
        "where" => fp_func_1(&function.arguments, context, |args, context| {
            let where_condition = &args[0];
            let mut new_context = vec![];
            for value in context.values.iter() {
                let result = evaluate_expression(
                    where_condition,
                    context.new_context_from(vec![*value]),
                    config,
                )?;

                if result.values.len() > 1 {
                    return Err(FHIRPathError::InternalError(
                        "Where condition did not return a single value".to_string(),
                    ));
                    // Empty set effectively means no match and treat as false.
                } else if !result.values.is_empty() && downcast_bool(result.values[0])? == true {
                    new_context.push(*value);
                }
            }
            Ok(context.new_context_from(new_context))
        }),
        "ofType" => fp_func_1(&function.arguments, context, |args, context| {
            let type_name = derive_typename(&args[0])?;
            Ok(filter_by_type(&type_name, &context))
        }),
        "as" => fp_func_1(&function.arguments, context, |args, context| {
            let type_name = derive_typename(&args[0])?;
            Ok(filter_by_type(&type_name, &context))
        }),
        "exists" => fp_func_n(&function.arguments, context, |args, context| {
            if args.len() > 1 {
                return Err(
                    FunctionError::InvalidCardinality("exists".to_string(), args.len()).into(),
                );
            }

            let context = if args.len() == 1 {
                evaluate_expression(&args[0], context, config)?
            } else {
                context
            };

            Ok(context
                .new_context_from(vec![context.allocate(Box::new(!context.values.is_empty()))]))
        }),
        "children" => fp_func_0(&function.arguments, context, |context| {
            Ok(context.new_context_from(
                context
                    .values
                    .iter()
                    .flat_map(|value| {
                        let result = value
                            .fields()
                            .iter()
                            .filter_map(|f| value.get_field(f).map(|v| v.flatten()))
                            .flatten()
                            .collect::<Vec<_>>();
                        result
                    })
                    .collect(),
            ))
        }),
        "repeat" => fp_func_1(&function.arguments, context, |args, context| {
            let projection = &args[0];
            let mut end_result = vec![];
            let mut cur = context;

            while (cur.values.len() != 0) {
                cur = evaluate_expression(projection, cur, config)?;
                end_result.extend_from_slice(cur.values.as_slice());
            }

            Ok(cur.new_context_from(end_result))
        }),
        "descendants" => fp_func_0(&function.arguments, context, |context| {
            // Descendants is shorthand for repeat(children()) see [https://hl7.org/fhirpath/N1/#descendants-collection].
            let result = evaluate_expression(
                &Expression::Singular(vec![Term::Invocation(Invocation::Function(
                    FunctionInvocation {
                        name: Identifier("repeat".to_string()),
                        arguments: vec![Expression::Singular(vec![Term::Invocation(
                            Invocation::Function(FunctionInvocation {
                                name: Identifier("children".to_string()),
                                arguments: vec![],
                            }),
                        )])],
                    },
                ))]),
                context,
                config,
            )?;

            Ok(result)
        }),
        "type" => fp_func_0(&function.arguments, context, |context| {
            Ok(context.new_context_from(
                context
                    .values
                    .iter()
                    .map(|value| {
                        let type_name = value.typename();
                        context.allocate(Box::new(Reflection {
                            name: type_name.to_string(),
                        }))
                    })
                    .collect(),
            ))
        }),
        _ => {
            return Err(FHIRPathError::NotImplemented(format!(
                "Function '{}' is not implemented",
                function.name.0
            )));
        }
    }
}

fn equal_check<'b>(left: &Context<'b>, right: &Context<'b>) -> Result<bool, FHIRPathError> {
    if NUMBER_TYPES.contains(left.values[0].typename())
        && NUMBER_TYPES.contains(right.values[0].typename())
    {
        let left_value = downcast_number(left.values[0])?;
        let right_value = downcast_number(right.values[0])?;
        Ok(left_value == right_value)
    } else if STRING_TYPES.contains(left.values[0].typename())
        && STRING_TYPES.contains(right.values[0].typename())
    {
        let left_value = downcast_string(left.values[0])?;
        let right_value = downcast_string(right.values[0])?;
        Ok(left_value == right_value)
    } else if BOOLEAN_TYPES.contains(left.values[0].typename())
        && BOOLEAN_TYPES.contains(right.values[0].typename())
    {
        let left_value = downcast_bool(left.values[0])?;
        let right_value = downcast_bool(right.values[0])?;
        Ok(left_value == right_value)
    } else {
        // https://hl7.org/fhirpath/N1/#conversion for implicit conversion rules todo.
        //
        // If types do not match return false.
        // Should consider implicit conversion rules here but for now
        // FPs like 'Patient.deceased.exists() and Patient.deceased != false' (deceased is either boolean or dateTime)
        // Should return false rather than error.
        Ok(false)
    }
}

fn evaluate_operation<'a>(
    operation: &Operation,
    context: Context<'a>,
    config: &'a Option<Config<'a>>,
) -> Result<Context<'a>, FHIRPathError> {
    match operation {
        Operation::Add(left, right) => operation_1(left, right, context, config, |left, right| {
            if NUMBER_TYPES.contains(left.values[0].typename())
                && NUMBER_TYPES.contains(right.values[0].typename())
            {
                let left_value = downcast_number(left.values[0])?;
                let right_value = downcast_number(right.values[0])?;
                Ok(left.new_context_from(vec![left.allocate(Box::new(left_value + right_value))]))
            } else if STRING_TYPES.contains(left.values[0].typename())
                && STRING_TYPES.contains(right.values[0].typename())
            {
                let left_string = downcast_string(left.values[0])?;
                let right_string = downcast_string(right.values[0])?;

                Ok(left
                    .new_context_from(vec![left.allocate(Box::new(left_string + &right_string))]))
            } else {
                Err(FHIRPathError::OperationError(OperationError::TypeMismatch(
                    left.values[0].typename(),
                    right.values[0].typename(),
                )))
            }
        }),
        Operation::Subtraction(left, right) => {
            operation_1(left, right, context, config, |left, right| {
                let left_value = downcast_number(left.values[0])?;
                let right_value = downcast_number(right.values[0])?;

                Ok(left.new_context_from(vec![left.allocate(Box::new(left_value - right_value))]))
            })
        }
        Operation::Multiplication(left, right) => {
            operation_1(left, right, context, config, |left, right| {
                let left_value = downcast_number(left.values[0])?;
                let right_value = downcast_number(right.values[0])?;

                Ok(left.new_context_from(vec![left.allocate(Box::new(left_value * right_value))]))
            })
        }
        Operation::Division(left, right) => {
            operation_1(left, right, context, config, |left, right| {
                let left_value = downcast_number(left.values[0])?;
                let right_value = downcast_number(right.values[0])?;

                Ok(left.new_context_from(vec![left.allocate(Box::new(left_value / right_value))]))
            })
        }
        Operation::Equal(left, right) => {
            operation_1(left, right, context, config, |left, right| {
                let are_equal = equal_check(&left, &right)?;
                Ok(left.new_context_from(vec![left.allocate(Box::new(are_equal))]))
            })
        }
        Operation::NotEqual(left, right) => {
            operation_1(left, right, context, config, |left, right| {
                let are_equal = equal_check(&left, &right)?;
                Ok(left.new_context_from(vec![left.allocate(Box::new(!are_equal))]))
            })
        }
        Operation::And(left, right) => operation_1(left, right, context, config, |left, right| {
            let left_value = downcast_bool(left.values[0])?;
            let right_value = downcast_bool(right.values[0])?;

            Ok(left.new_context_from(vec![left.allocate(Box::new(left_value && right_value))]))
        }),
        Operation::Or(left, right) => operation_1(left, right, context, config, |left, right| {
            let left_value = downcast_bool(left.values[0])?;
            let right_value = downcast_bool(right.values[0])?;

            Ok(left.new_context_from(vec![left.allocate(Box::new(left_value || right_value))]))
        }),
        Operation::Union(left, right) => {
            operation_n(left, right, context, config, |left, right| {
                let mut union = vec![];
                union.extend(left.values.iter());
                union.extend(right.values.iter());
                Ok(left.new_context_from(union))
            })
        }
        Operation::Polarity(_, _) => Err(FHIRPathError::NotImplemented("Polarity".to_string())),
        Operation::DivisionTruncated(_, _) => Err(FHIRPathError::NotImplemented(
            "DivisionTruncated".to_string(),
        )),
        Operation::Modulo(_, _) => Err(FHIRPathError::NotImplemented("Modulo".to_string())),
        Operation::Is(expression, type_name) => {
            let left = evaluate_expression(expression, context, config)?;
            if left.values.len() > 1 {
                Err(FHIRPathError::OperationError(
                    OperationError::InvalidCardinality,
                ))
            } else {
                if let Some(type_name) = type_name.0.get(0).as_ref().map(|k| &k.0) {
                    let next_context = filter_by_type(&type_name, &left);
                    Ok(left.new_context_from(vec![
                        left.allocate(Box::new(!next_context.values.is_empty())),
                    ]))
                } else {
                    Ok(left.new_context_from(vec![]))
                }
            }
        }
        Operation::As(expression, type_name) => {
            let left = evaluate_expression(expression, context, config)?;
            if left.values.len() > 1 {
                Err(FHIRPathError::OperationError(
                    OperationError::InvalidCardinality,
                ))
            } else {
                if let Some(type_name) = type_name.0.get(0).as_ref().map(|k| &k.0) {
                    Ok(filter_by_type(&type_name, &left))
                } else {
                    Ok(left.new_context_from(vec![]))
                }
            }
        }

        Operation::LessThan(_, _) => Err(FHIRPathError::NotImplemented("LessThan".to_string())),
        Operation::GreaterThan(_, _) => {
            Err(FHIRPathError::NotImplemented("GreaterThan".to_string()))
        }
        Operation::LessThanEqual(_, _) => {
            Err(FHIRPathError::NotImplemented("LessThanEqual".to_string()))
        }
        Operation::GreaterThanEqual(_, _) => Err(FHIRPathError::NotImplemented(
            "GreaterThanEqual".to_string(),
        )),
        Operation::LessThan(left, right) => {
            Err(FHIRPathError::NotImplemented("LessThan".to_string()))
        }
        Operation::GreaterThan(left, right) => {
            Err(FHIRPathError::NotImplemented("GreaterThan".to_string()))
        }
        Operation::Equivalent(_, _) => Err(FHIRPathError::NotImplemented("Equivalent".to_string())),

        Operation::NotEquivalent(_, _) => {
            Err(FHIRPathError::NotImplemented("NotEquivalent".to_string()))
        }
        Operation::In(left, right) => Err(FHIRPathError::NotImplemented("In".to_string())),
        Operation::Contains(left, right) => {
            Err(FHIRPathError::NotImplemented("Contains".to_string()))
        }
        Operation::XOr(left, right) => Err(FHIRPathError::NotImplemented("XOr".to_string())),
        Operation::Implies(left, right) => {
            Err(FHIRPathError::NotImplemented("Implies".to_string()))
        }
    }
}

fn evaluate_expression<'a>(
    ast: &Expression,
    context: Context<'a>,
    config: &'a Option<Config<'a>>,
) -> Result<Context<'a>, FHIRPathError> {
    match ast {
        Expression::Operation(operation) => evaluate_operation(operation, context, config),
        Expression::Singular(singular_ast) => evaluate_singular(singular_ast, context, config),
    }
}

/// Need a means to store objects that are created during evaluation.
///
struct Allocator<'a> {
    pub context: Vec<Box<dyn MetaValue>>,
    _marker: PhantomData<&'a dyn MetaValue>,
}

impl<'a> Allocator<'a> {
    pub fn new() -> Self {
        Allocator {
            context: Vec::new(),
            _marker: PhantomData,
        }
    }

    pub fn allocate(&mut self, value: Box<dyn MetaValue>) -> &'a dyn MetaValue {
        self.context.push(value);
        // Should be safe to unwrap as value guaranteed to be non-empty from above call.
        let ptr = &**self.context.last().unwrap() as *const _;
        unsafe { &*ptr }
    }
}

pub struct Context<'a> {
    allocator: Arc<Mutex<Allocator<'a>>>,
    values: Arc<Vec<&'a dyn MetaValue>>,
}

enum ExternalConstantResolver<'a> {
    Function(Box<dyn Fn(&str) -> Option<Box<dyn MetaValue>>>),
    Variable(HashMap<String, &'a dyn MetaValue>),
}

struct Config<'a> {
    variable_resolver: Option<ExternalConstantResolver<'a>>,
}

fn resolve_external_constant<'a>(
    name: &str,
    resolver: Option<&'a ExternalConstantResolver<'a>>,
    context: Context<'a>,
) -> Result<Context<'a>, FHIRPathError> {
    let external_constant = match resolver {
        Some(ExternalConstantResolver::Function(func)) => {
            let result = func(name);
            if let Some(result) = result {
                Some(context.allocate(result))
            } else {
                None
            }
        }
        Some(ExternalConstantResolver::Variable(map)) => map.get(name).map(|s| *s),
        None => None,
    };

    if let Some(result) = external_constant {
        return Ok(context.new_context_from(vec![result]));
    } else {
        return Ok(context.new_context_from(vec![]));
    }
}

impl<'a> Context<'a> {
    fn new(values: Vec<&'a dyn MetaValue>, allocator: Arc<Mutex<Allocator<'a>>>) -> Self {
        Self {
            allocator,
            values: Arc::new(values),
        }
    }
    fn new_context_from(&self, values: Vec<&'a dyn MetaValue>) -> Self {
        Self {
            allocator: self.allocator.clone(),
            values: Arc::new(values),
        }
    }
    fn allocate(&self, value: Box<dyn MetaValue>) -> &'a dyn MetaValue {
        self.allocator.lock().unwrap().allocate(value)
    }
    pub fn iter(&'a self) -> Box<dyn Iterator<Item = &'a dyn MetaValue> + 'a> {
        Box::new(self.values.iter().map(|v| *v))
    }
}

impl Clone for Context<'_> {
    fn clone(&self) -> Self {
        Self {
            allocator: self.allocator.clone(),
            values: self.values.clone(),
        }
    }
}

pub struct FPEngine {
    ast: Arc<DashMap<String, Expression>>,
}

static AST: LazyLock<Arc<DashMap<String, Expression>>> = LazyLock::new(|| Arc::new(DashMap::new()));

impl FPEngine {
    pub fn new() -> Self {
        Self { ast: AST.clone() }
    }

    /// Evaluate a FHIRPath expression against a context.
    /// The context is a vector of references to MetaValue objects.
    /// The path is a FHIRPath expression.
    /// The result is a vector of references to MetaValue objects.
    pub fn evaluate<'a, 'b>(
        &self,
        path: &str,
        values: Vec<&'a dyn MetaValue>,
    ) -> Result<Context<'b>, FHIRPathError>
    where
        'a: 'b,
    {
        let ast: dashmap::mapref::one::Ref<'_, String, Expression> =
            if let Some(ast) = self.ast.get(path) {
                ast
            } else {
                self.ast.insert(path.to_string(), parser::parse(path)?);
                let ast = self.ast.get(path).ok_or_else(|| {
                    FHIRPathError::InternalError("Failed to find path post insert".to_string())
                })?;
                ast
            };

        // Store created.
        let allocator: Arc<Mutex<Allocator<'b>>> = Arc::new(Mutex::new(Allocator::new()));

        let context = Context::new(values, allocator.clone());

        let result = evaluate_expression(&ast, context, &None)?;

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use oxidized_fhir_model::r4::generated::{
        resources::{
            Bundle, Patient, PatientDeceasedTypeChoice, PatientLink, Resource, SearchParameter,
        },
        types::{
            Extension, ExtensionValueTypeChoice, FHIRString, HumanName, Identifier, Reference,
        },
    };
    use oxidized_fhir_serialization_json;
    use oxidized_reflect_derive::Reflect;

    #[derive(Reflect, Debug)]
    struct C {
        c: String,
    }

    #[derive(Reflect, Debug)]
    struct B {
        b: Vec<Box<C>>,
    }

    #[derive(Reflect, Debug)]
    struct A {
        a: Vec<Box<B>>,
    }

    fn load_search_parameters() -> Vec<SearchParameter> {
        let json =
            include_str!("../../artifacts/artifacts/r4/hl7/minified/search-parameters.min.json");
        let bundle = oxidized_fhir_serialization_json::from_str::<Bundle>(json).unwrap();

        let search_parameters: Vec<SearchParameter> = bundle
            .entry
            .unwrap_or_else(|| Vec::new())
            .into_iter()
            .map(|e| e.resource)
            .filter(|e| e.is_some())
            .filter_map(|e| match e {
                Some(k) => match *k {
                    Resource::SearchParameter(sp) => Some(sp),
                    _ => None,
                },
                _ => None,
            })
            .collect();

        search_parameters
    }

    #[test]
    fn test_where_clause() {
        let engine = FPEngine::new();
        let mut patient = Patient::default();
        let mut identifier = Identifier::default();
        let mut extension = Extension {
            id: None,
            url: "test-extension".to_string(),
            extension: None,
            value: Some(ExtensionValueTypeChoice::String(Box::new(FHIRString {
                id: None,
                extension: None,
                value: Some("example value".to_string()),
            }))),
        };
        identifier.value = Some(Box::new(FHIRString {
            id: None,
            extension: Some(vec![Box::new(extension)]),
            value: Some("12345".to_string()),
        }));
        patient.identifier_ = Some(vec![Box::new(identifier)]);

        let context = engine.evaluate(
            "$this.identifier.value.where($this.extension.value.exists())",
            vec![&patient],
        );

        assert_eq!(context.unwrap().values.len(), 1);

        let context = engine.evaluate(
            "$this.identifier.value.where($this.extension.extension.exists())",
            vec![&patient],
        );
        assert_eq!(context.unwrap().values.len(), 0);
    }

    #[test]
    fn test_all_parameters() {
        let search_parameters = load_search_parameters();
        for param in search_parameters.iter() {
            if let Some(expression) = &param.expression {
                let engine = FPEngine::new();
                let context = engine.evaluate(expression.value.as_ref().unwrap().as_str(), vec![]);

                if let Err(err) = context {
                    panic!(
                        "Failed to evaluate search parameter '{}': {}",
                        expression.value.as_ref().unwrap(),
                        err
                    );
                }
            }
        }
    }

    fn test_patient() -> Patient {
        let mut patient = Patient::default();
        let mut name = HumanName::default();
        name.given = Some(vec![Box::new(FHIRString {
            id: None,
            extension: None,
            value: Some("Bob".to_string()),
        })]);

        let mut mrn_identifier = Identifier::default();
        mrn_identifier.value = Some(Box::new(FHIRString {
            id: None,
            extension: None,
            value: Some("mrn-12345".to_string()),
        }));
        mrn_identifier.system = Some(Box::new(FHIRUri {
            id: None,
            extension: None,
            value: Some("mrn".to_string()),
        }));

        let mut ssn_identifier = Identifier::default();
        ssn_identifier.value = Some(Box::new(FHIRString {
            id: None,
            extension: None,
            value: Some("ssn-12345".to_string()),
        }));
        ssn_identifier.system = Some(Box::new(FHIRUri {
            id: None,
            extension: None,
            value: Some("ssn".to_string()),
        }));

        mrn_identifier.system = Some(Box::new(FHIRUri {
            id: None,
            extension: None,
            value: Some("mrn".to_string()),
        }));

        patient.identifier_ = Some(vec![Box::new(mrn_identifier), Box::new(ssn_identifier)]);
        patient.name = Some(vec![Box::new(name)]);
        patient
    }

    #[test]
    fn indexing_tests() {
        let engine = FPEngine::new();
        let patient = test_patient();

        let given_name = engine
            .evaluate("$this.name.given[0]", vec![&patient])
            .unwrap();

        assert_eq!(given_name.values.len(), 1);
        let value = given_name.values[0];
        let name: &FHIRString = value
            .as_any()
            .downcast_ref::<FHIRString>()
            .expect("Failed to downcast to FHIRString");

        assert_eq!(name.value.as_deref(), Some("Bob"));

        let ssn_identifier = engine
            .evaluate("$this.identifier[1]", vec![&patient])
            .unwrap();

        assert_eq!(ssn_identifier.values.len(), 1);
        let value = ssn_identifier.values[0];
        let identifier: &Identifier = value
            .as_any()
            .downcast_ref::<Identifier>()
            .expect("Failed to downcast to Identifier");

        assert_eq!(
            identifier.value.as_ref().unwrap().value.as_deref(),
            Some("ssn-12345")
        );

        let all_identifiers = engine.evaluate("$this.identifier", vec![&patient]).unwrap();
        assert_eq!(all_identifiers.values.len(), 2);
    }

    #[test]
    fn where_testing() {
        let engine = FPEngine::new();
        let patient = test_patient();

        let name_where_clause = engine
            .evaluate(
                "$this.name.given.where($this.value = 'Bob')",
                vec![&patient],
            )
            .unwrap();

        assert_eq!(name_where_clause.values.len(), 1);
        let value = name_where_clause.values[0];
        let name: &FHIRString = value
            .as_any()
            .downcast_ref::<FHIRString>()
            .expect("Failed to downcast to FHIRString");

        assert_eq!(name.value.as_deref(), Some("Bob"));

        let ssn_identifier_clause = engine
            .evaluate(
                "$this.identifier.where($this.system.value = 'ssn')",
                vec![&patient],
            )
            .unwrap();
        assert_eq!(ssn_identifier_clause.values.len(), 1);

        let ssn_identifier = ssn_identifier_clause.values[0]
            .as_any()
            .downcast_ref::<Identifier>()
            .expect("Failed to downcast to Identifier");

        assert_eq!(
            ssn_identifier.value.as_ref().unwrap().value.as_deref(),
            Some("ssn-12345")
        );
    }

    #[test]
    fn test_equality() {
        let engine = FPEngine::new();

        // String tests
        let string_equal = engine.evaluate("'test' = 'test'", vec![]).unwrap();
        for r in string_equal.iter() {
            let b: bool = r.as_any().downcast_ref::<bool>().unwrap().clone();
            assert_eq!(b, true);
        }
        let string_unequal = engine.evaluate("'invalid' = 'test'", vec![]).unwrap();
        for r in string_unequal.iter() {
            let b: bool = r.as_any().downcast_ref::<bool>().unwrap().clone();
            assert_eq!(b, false);
        }

        // Number tests
        let number_equal = engine.evaluate("12 = 12", vec![]).unwrap();
        for r in number_equal.iter() {
            let b: bool = r.as_any().downcast_ref::<bool>().unwrap().clone();
            assert_eq!(b, true);
        }
        let number_unequal = engine.evaluate("13 = 12", vec![]).unwrap();
        for r in number_unequal.iter() {
            let b: bool = r.as_any().downcast_ref::<bool>().unwrap().clone();
            assert_eq!(b, false);
        }

        // Boolean tests
        let bool_equal = engine.evaluate("false = false", vec![]).unwrap();
        for r in bool_equal.iter() {
            let b: bool = r.as_any().downcast_ref::<bool>().unwrap().clone();
            assert_eq!(b, true);
        }
        let bool_unequal = engine.evaluate("false = true", vec![]).unwrap();
        for r in bool_unequal.iter() {
            let b: bool = r.as_any().downcast_ref::<bool>().unwrap().clone();
            assert_eq!(b, false);
        }

        // Nested Equality tests
        let bool_equal = engine.evaluate("12 = 13 = false", vec![]).unwrap();
        for r in bool_equal.iter() {
            let b: bool = r.as_any().downcast_ref::<bool>().unwrap().clone();
            assert_eq!(b, true);
        }
        let bool_unequal = engine.evaluate("12 = 13 = true", vec![]).unwrap();
        for r in bool_unequal.iter() {
            let b: bool = r.as_any().downcast_ref::<bool>().unwrap().clone();
            assert_eq!(b, false);
        }
        let bool_unequal = engine.evaluate("12 = (13 - 1)", vec![]).unwrap();
        for r in bool_unequal.iter() {
            let b: bool = r.as_any().downcast_ref::<bool>().unwrap().clone();
            assert_eq!(b, true);
        }
    }

    #[test]
    fn test_string_concat() {
        let engine = FPEngine::new();
        let patient = test_patient();

        let simple_result = engine.evaluate("'Hello' + ' World'", vec![]).unwrap();
        for r in simple_result.iter() {
            let s: String = r.as_any().downcast_ref::<String>().unwrap().clone();
            assert_eq!(s, "Hello World".to_string());
        }

        let simple_result = engine
            .evaluate("$this.name.given + ' Miller'", vec![&patient])
            .unwrap();
        for r in simple_result.iter() {
            let s: String = r.as_any().downcast_ref::<String>().unwrap().clone();
            assert_eq!(s, "Bob Miller".to_string());
        }
    }

    #[test]
    fn test_simple() {
        let root = A {
            a: vec![Box::new(B {
                b: vec![Box::new(C {
                    c: "whatever".to_string(),
                })],
            })],
        };

        let engine = FPEngine::new();
        let result = engine.evaluate("a.b.c", vec![&root]).unwrap();

        let strings: Vec<&String> = result
            .iter()
            .map(|r| r.as_any().downcast_ref::<String>().unwrap())
            .collect();

        assert_eq!(strings, vec!["whatever"]);
    }

    #[test]
    fn allocation() {
        let engine = FPEngine::new();
        let result = engine.evaluate("'asdf'", vec![]).unwrap();

        for r in result.iter() {
            let s: String = r.as_any().downcast_ref::<String>().unwrap().clone();

            assert_eq!(s, "asdf".to_string());
        }
    }

    #[test]
    fn order_operation() {
        let engine = FPEngine::new();
        let result = engine.evaluate("45 + 2  * 3", vec![]).unwrap();

        for r in result.iter() {
            let s = r.as_any().downcast_ref::<f64>().unwrap().clone();

            assert_eq!(s, 51.0);
        }
    }

    #[test]
    fn domain_resource_filter() {
        let engine = FPEngine::new();

        let patient = oxidized_fhir_serialization_json::from_str::<Resource>(
            r#"{"id": "patient-id", "resourceType": "Patient"}"#,
        )
        .unwrap();
        let result = engine.evaluate("Resource.id", vec![&patient]).unwrap();
        let ids: Vec<&String> = result
            .iter()
            .map(|r| r.as_any().downcast_ref::<String>().unwrap())
            .collect();

        assert_eq!(ids.len(), 1);
        assert_eq!(ids[0], "patient-id");

        let result2 = engine
            .evaluate("DomainResource.id", vec![&patient])
            .unwrap();
        let ids2: Vec<&String> = result2
            .iter()
            .map(|r| r.as_any().downcast_ref::<String>().unwrap())
            .collect();
        assert_eq!(ids2.len(), 1);
        assert_eq!(ids2[0], "patient-id");
    }

    #[test]
    fn type_test() {
        let engine = FPEngine::new();
        let patient = Patient::default();

        let result = engine
            .evaluate("$this.type().name", vec![&patient])
            .unwrap();
        let ids: Vec<&String> = result
            .iter()
            .map(|r| r.as_any().downcast_ref::<String>().unwrap())
            .collect();

        assert_eq!(ids.len(), 1);
        assert_eq!(ids[0], "Patient");
    }

    #[test]
    fn resolve_test() {
        let engine = FPEngine::new();
        let observation = oxidized_fhir_serialization_json::from_str::<Resource>(r#"
             {
                "resourceType": "Observation",
                "id": "f001",
                "text": {
                    "status": "generated",
                    "div": "<div xmlns=\"http://www.w3.org/1999/xhtml\"><p><b>Generated Narrative with Details</b></p><p><b>id</b>: f001</p><p><b>identifier</b>: 6323 (OFFICIAL)</p><p><b>status</b>: final</p><p><b>code</b>: Glucose [Moles/volume] in Blood <span>(Details : {LOINC code '15074-8' = 'Glucose [Moles/volume] in Blood', given as 'Glucose [Moles/volume] in Blood'})</span></p><p><b>subject</b>: <a>P. van de Heuvel</a></p><p><b>effective</b>: 02/04/2013 9:30:10 AM --&gt; (ongoing)</p><p><b>issued</b>: 03/04/2013 3:30:10 PM</p><p><b>performer</b>: <a>A. Langeveld</a></p><p><b>value</b>: 6.3 mmol/l<span> (Details: UCUM code mmol/L = 'mmol/L')</span></p><p><b>interpretation</b>: High <span>(Details : {http://terminology.hl7.org/CodeSystem/v3-ObservationInterpretation code 'H' = 'High', given as 'High'})</span></p><h3>ReferenceRanges</h3><table><tr><td>-</td><td><b>Low</b></td><td><b>High</b></td></tr><tr><td>*</td><td>3.1 mmol/l<span> (Details: UCUM code mmol/L = 'mmol/L')</span></td><td>6.2 mmol/l<span> (Details: UCUM code mmol/L = 'mmol/L')</span></td></tr></table></div>"
                },
                "identifier": [
                    {
                    "use": "official",
                    "system": "http://www.bmc.nl/zorgportal/identifiers/observations",
                    "value": "6323"
                    }
                ],
                "status": "final",
                "code": {
                    "coding": [
                    {
                        "system": "http://loinc.org",
                        "code": "15074-8",
                        "display": "Glucose [Moles/volume] in Blood"
                    }
                    ]
                },
                "subject": {
                    "reference": "Patient/f001",
                    "display": "P. van de Heuvel"
                },
                "effectivePeriod": {
                    "start": "2013-04-02T09:30:10+01:00"
                },
                "issued": "2013-04-03T15:30:10+01:00",
                "performer": [
                    {
                    "reference": "Practitioner/f005",
                    "display": "A. Langeveld"
                    }
                ],
                "valueQuantity": {
                    "value": 6.3,
                    "unit": "mmol/l",
                    "system": "http://unitsofmeasure.org",
                    "code": "mmol/L"
                },
                "interpretation": [
                    {
                    "coding": [
                        {
                        "system": "http://terminology.hl7.org/CodeSystem/v3-ObservationInterpretation",
                        "code": "H",
                        "display": "High"
                        }
                    ]
                    }
                ],
                "referenceRange": [
                    {
                    "low": {
                        "value": 3.1,
                        "unit": "mmol/l",
                        "system": "http://unitsofmeasure.org",
                        "code": "mmol/L"
                    },
                    "high": {
                        "value": 6.2,
                        "unit": "mmol/l",
                        "system": "http://unitsofmeasure.org",
                        "code": "mmol/L"
                    }
                    }
                ]
                }
            "#).unwrap();

        let result = engine
            .evaluate(
                "Observation.subject.where(resolve() is Patient)",
                vec![&observation],
            )
            .unwrap();

        let references: Vec<&Reference> = result
            .iter()
            .map(|r| r.as_any().downcast_ref::<Reference>().unwrap())
            .collect();

        assert_eq!(references.len(), 1);
        assert_eq!(
            references[0].reference.as_ref().unwrap().value,
            Some("Patient/f001".to_string())
        );
    }

    #[test]
    fn children_test() {
        let engine = FPEngine::new();
        let patient = Patient {
            name: Some(vec![Box::new(HumanName {
                given: Some(vec![Box::new(FHIRString {
                    value: Some("Alice".to_string()),
                    ..Default::default()
                })]),
                ..Default::default()
            })]),
            deceased: Some(PatientDeceasedTypeChoice::Boolean(Box::new(FHIRBoolean {
                value: Some(true),
                ..Default::default()
            }))),
            ..Default::default()
        };

        let result = engine.evaluate("$this.children()", vec![&patient]).unwrap();

        assert_eq!(result.values.len(), 2);
        assert_eq!(
            result
                .values
                .iter()
                .map(|v| v.typename())
                .collect::<Vec<_>>(),
            vec!["HumanName", "FHIRBoolean"]
        );
    }

    #[test]
    fn repeat_test() {
        let engine = FPEngine::new();
        let patient = Patient {
            name: Some(vec![Box::new(HumanName {
                given: Some(vec![Box::new(FHIRString {
                    value: Some("Alice".to_string()),
                    ..Default::default()
                })]),
                ..Default::default()
            })]),
            deceased: Some(PatientDeceasedTypeChoice::Boolean(Box::new(FHIRBoolean {
                value: Some(true),
                ..Default::default()
            }))),
            ..Default::default()
        };

        let result = engine.evaluate("$this.name.given", vec![&patient]).unwrap();

        assert_eq!(result.values.len(), 1);

        assert_eq!(result.values[0].typename(), "FHIRString");

        let result = engine
            .evaluate("$this.repeat(children())", vec![&patient])
            .unwrap();

        assert_eq!(
            result
                .values
                .iter()
                .map(|v| v.typename())
                .collect::<Vec<_>>(),
            vec![
                "HumanName",
                "FHIRBoolean",
                "FHIRString",
                "http://hl7.org/fhirpath/System.Boolean",
                "http://hl7.org/fhirpath/System.String"
            ]
        );
    }
    #[test]
    fn descendants_test() {
        let engine = FPEngine::new();
        let patient = Patient {
            name: Some(vec![Box::new(HumanName {
                given: Some(vec![Box::new(FHIRString {
                    value: Some("Alice".to_string()),
                    ..Default::default()
                })]),
                ..Default::default()
            })]),
            deceased: Some(PatientDeceasedTypeChoice::Boolean(Box::new(FHIRBoolean {
                value: Some(true),
                ..Default::default()
            }))),
            ..Default::default()
        };
        let result = engine.evaluate("descendants()", vec![&patient]).unwrap();

        assert_eq!(
            result
                .values
                .iter()
                .map(|v| v.typename())
                .collect::<Vec<_>>(),
            vec![
                "HumanName",
                "FHIRBoolean",
                "FHIRString",
                "http://hl7.org/fhirpath/System.Boolean",
                "http://hl7.org/fhirpath/System.String"
            ]
        );
    }

    #[test]
    fn descendants_test_filter() {
        let engine = FPEngine::new();
        let patient = Patient {
            link: Some(vec![PatientLink {
                other: Box::new(Reference {
                    reference: Some(Box::new(FHIRString {
                        value: Some("Patient/123".to_string()),
                        ..Default::default()
                    })),
                    ..Default::default()
                }),
                ..Default::default()
            }]),
            name: Some(vec![Box::new(HumanName {
                given: Some(vec![Box::new(FHIRString {
                    value: Some("Alice".to_string()),
                    ..Default::default()
                })]),
                ..Default::default()
            })]),
            deceased: Some(PatientDeceasedTypeChoice::Boolean(Box::new(FHIRBoolean {
                value: Some(true),
                ..Default::default()
            }))),
            ..Default::default()
        };
        let result = engine.evaluate("descendants()", vec![&patient]).unwrap();

        assert_eq!(
            result
                .values
                .iter()
                .map(|v| v.typename())
                .collect::<Vec<_>>(),
            vec![
                "HumanName",
                "FHIRBoolean",
                "PatientLink",
                "FHIRString",
                "http://hl7.org/fhirpath/System.Boolean",
                "Reference",
                "http://hl7.org/fhirpath/System.String",
                "FHIRString",
                "http://hl7.org/fhirpath/System.String"
            ]
        );

        let result = engine
            .evaluate("descendants().ofType(Reference)", vec![&patient])
            .unwrap();

        assert_eq!(
            result
                .values
                .iter()
                .map(|v| v.typename())
                .collect::<Vec<_>>(),
            vec!["Reference",]
        );

        let value = result.values[0]
            .as_any()
            .downcast_ref::<Reference>()
            .unwrap();

        assert_eq!(
            value.reference.as_ref().unwrap().value.as_ref().unwrap(),
            "Patient/123"
        );
    }

    #[test]
    fn try_unsafe_set_from_ref() {
        let engine = FPEngine::new();
        let patient = Patient {
            link: Some(vec![PatientLink {
                other: Box::new(Reference {
                    reference: Some(Box::new(FHIRString {
                        value: Some("Patient/123".to_string()),
                        ..Default::default()
                    })),
                    ..Default::default()
                }),
                ..Default::default()
            }]),
            name: Some(vec![Box::new(HumanName {
                given: Some(vec![Box::new(FHIRString {
                    value: Some("Alice".to_string()),
                    ..Default::default()
                })]),
                ..Default::default()
            })]),
            deceased: Some(PatientDeceasedTypeChoice::Boolean(Box::new(FHIRBoolean {
                value: Some(true),
                ..Default::default()
            }))),
            ..Default::default()
        };

        let result = engine
            .evaluate("descendants().ofType(Reference)", vec![&patient])
            .unwrap();

        assert_eq!(
            result
                .values
                .iter()
                .map(|v| v.typename())
                .collect::<Vec<_>>(),
            vec!["Reference",]
        );

        let value = result.values[0]
            .as_any()
            .downcast_ref::<Reference>()
            .unwrap();

        assert_eq!(
            value.reference.as_ref().unwrap().value.as_ref().unwrap(),
            "Patient/123"
        );

        // An example for use in transaction processing where we have a reference to an object
        // but need to modify it in place.
        unsafe {
            let r = value as *const Reference;
            let mut_ptr = r as *mut Reference;

            (*mut_ptr).reference = Some(Box::new(FHIRString {
                value: Some("Patient/456".to_string()),
                ..Default::default()
            }));
        }

        assert_eq!(
            value.reference.as_ref().unwrap().value.as_ref().unwrap(),
            "Patient/456"
        );

        assert_eq!(
            patient.link.as_ref().unwrap()[0]
                .other
                .reference
                .as_ref()
                .unwrap()
                .value
                .as_ref()
                .unwrap(),
            "Patient/456"
        );
    }
}
