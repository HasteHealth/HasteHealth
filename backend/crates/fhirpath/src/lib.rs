mod error;
mod parser;
use crate::{
    error::{FunctionError, OperationError},
    parser::{
        Expression, FunctionInvocation, Identifier, Invocation, Literal, Operation, Polarity,
        QualifiedIdentifier, Term, Unit,
    },
};
use dashmap::DashMap;
pub use error::FHIRPathError;
use haste_fhir_model::r4::{
    conversion::{
        BOOLEAN_TYPES, DATE_TIME_TYPES, NUMBER_TYPES, STRING_TYPES, downcast_bool,
        downcast_datetime, downcast_number, downcast_string,
    },
    datetime::{
        DATETIME_REGEX, Date, DateTime, Time, parse_date, parse_datetime, parse_partial_time,
        parse_time,
    },
    generated::{
        resources::ResourceType,
        types::{
            FHIRBoolean, FHIRCode, FHIRDate, FHIRDateTime, FHIRDecimal, FHIRId, FHIRInteger,
            FHIRPositiveInt, FHIRString, FHIRTime, FHIRUnsignedInt, FHIRUri, Quantity, Reference,
        },
    },
};
use haste_reflect::MetaValue;
use haste_reflect_derive::Reflect;
use regex::{Regex, RegexBuilder};
use std::{cmp::Ordering, pin::Pin};
use std::{
    collections::HashMap,
    sync::{Arc, LazyLock, Mutex},
};

mod allocators;

#[cfg(test)]
mod proptests;
#[cfg(test)]
mod spec_tests;
use allocators::AllocatorTrait;

fn evaluate_literal<'b>(
    literal: &Literal,
    context: &Context<'b>,
) -> Result<Context<'b>, FHIRPathError> {
    match literal {
        Literal::String(string) => Ok(context.new_context_from(vec![context.allocate_literal(
            FHIRString {
                value: Some(string.clone()),
                ..Default::default()
            },
        )])),
        Literal::Integer(int) => Ok(context.new_context_from(vec![context.allocate_literal(
            FHIRInteger {
                value: Some(*int),
                ..Default::default()
            },
        )])),
        Literal::Float(decimal) => Ok(context.new_context_from(vec![context.allocate_literal(
            FHIRDecimal {
                value: Some(*decimal),
                ..Default::default()
            },
        )])),
        Literal::Boolean(bool) => Ok(context.new_context_from(vec![context.allocate_literal(
            FHIRBoolean {
                value: Some(*bool),
                ..Default::default()
            },
        )])),
        Literal::Null => Ok(context.new_context_from(vec![])),
        Literal::Quantity(value, unit) => {
            let (system, code) = match unit {
                Unit::Ucum(code) => ("http://unitsofmeasure.org", code),
                Unit::Calendar(code) => ("http://hl7.org/fhirpath/CodeSystem/calendar-units", code),
            };
            Ok(
                context.new_context_from(vec![context.allocate_literal(Quantity {
                    value: Some(Box::new(FHIRDecimal {
                        value: Some(*value),
                        ..Default::default()
                    })),
                    unit: Some(Box::new(FHIRString {
                        value: Some(code.clone()),
                        ..Default::default()
                    })),
                    system: Some(Box::new(FHIRUri {
                        value: Some(system.to_string()),
                        ..Default::default()
                    })),
                    code: Some(Box::new(FHIRCode {
                        value: Some(code.clone()),
                        ..Default::default()
                    })),
                    ..Default::default()
                })]),
            )
        }
        Literal::Date(date) => temporal_literal(
            literal,
            context,
            parse_date(date).ok().map(|value| FHIRDate {
                value: Some(value),
                ..Default::default()
            }),
        ),
        Literal::DateTime(datetime) => temporal_literal(
            literal,
            context,
            parse_datetime_literal(datetime).map(|value| FHIRDateTime {
                value: Some(value),
                ..Default::default()
            }),
        ),
        Literal::Time(time) => temporal_literal(
            literal,
            context,
            parse_partial_time(time).ok().map(|value| FHIRTime {
                value: Some(value),
                ..Default::default()
            }),
        ),
    }
}

/// The singleton for a parsed `Date`, `DateTime` or `Time` literal.
fn temporal_literal<'b, T: MetaValue + 'static>(
    literal: &Literal,
    context: &Context<'b>,
    value: Option<T>,
) -> Result<Context<'b>, FHIRPathError> {
    value
        .map(|value| context.new_context_from(vec![context.allocate_literal(value)]))
        .ok_or_else(|| FHIRPathError::InvalidLiteral(literal.to_owned()))
}

/// A `DateTime` literal, whose time part may be cut short (`@2020-01-01T`) or lack
/// a timezone, which is read as UTC.
fn parse_datetime_literal(literal: &str) -> Option<DateTime> {
    let literal = literal.strip_suffix('T').unwrap_or(literal);
    let zoned = literal
        .split_once('T')
        .is_none_or(|(_, time)| time.ends_with('Z') || time.contains(['+', '-']));
    if zoned {
        parse_datetime(literal).ok()
    } else {
        parse_datetime(&format!("{literal}Z")).ok()
    }
}

async fn evaluate_invocation<'a>(
    invocation: &Invocation,
    context: Context<'a>,
    config: Option<Arc<Config<'a>>>,
) -> Result<Context<'a>, FHIRPathError> {
    match invocation {
        Invocation::This => Ok(context),
        Invocation::Index(index_expression) => {
            let index = evaluate_expression(index_expression, context.clone(), config).await?;
            if index.values.len() != 1 {
                return Err(FHIRPathError::OperationError(
                    OperationError::InvalidCardinality,
                ));
            }

            let float_index = downcast_number(index.values[0])?;

            // Ensure the index is non-negative and has no fractional part
            if float_index < 0.0 || float_index.fract() != 0.0 {
                return Err(FHIRPathError::OperationError(OperationError::InvalidIndex));
            }

            // Suppress clippy warnings because the float is guaranteed to be a valid,
            // non-negative integer index by the safety checks immediately above.
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let index: usize = (float_index as u64)
                .try_into()
                .map_err(|_| FHIRPathError::OperationError(OperationError::IndexOutOfBounds))?;

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
                .flat_map(|v| v.get_field(id).map_or_else(Vec::new, |v| v.flatten()))
                .collect(),
        )),
        Invocation::Function(function) => evaluate_function(function, context, config).await,
    }
}

async fn evaluate_term<'a>(
    term: &Term,
    context: Context<'a>,
    config: Option<Arc<Config<'a>>>,
) -> Result<Context<'a>, FHIRPathError> {
    match term {
        Term::Literal(literal) => evaluate_literal(literal, &context),
        Term::ExternalConstant(constant) => {
            resolve_external_constant(
                constant,
                config.as_ref().and_then(|c| c.variable_resolver.as_ref()),
                context,
            )
            .await
        }
        Term::Parenthesized(expression) => evaluate_expression(expression, context, config).await,
        Term::Invocation(invocation) => evaluate_invocation(invocation, context, config).await,
    }
}

/// Need special handling as the first term could start with a type filter.
/// for example Patient.name
async fn evaluate_first_term<'a>(
    term: &Term,
    context: Context<'a>,
    config: Option<Arc<Config<'a>>>,
) -> Result<Context<'a>, FHIRPathError> {
    match term {
        Term::Invocation(invocation) => match invocation {
            Invocation::Identifier(identifier) => {
                let type_filter = filter_by_type(&identifier.0, &context);
                if type_filter.values.is_empty() {
                    evaluate_invocation(invocation, context, config).await
                } else {
                    Ok(type_filter)
                }
            }
            _ => evaluate_invocation(invocation, context, config).await,
        },
        _ => evaluate_term(term, context, config).await,
    }
}

async fn evaluate_singular<'a>(
    expression: &[Term],
    context: Context<'a>,
    config: Option<Arc<Config<'a>>>,
) -> Result<Context<'a>, FHIRPathError> {
    let mut current_context = context;

    let mut term_iterator = expression.iter();
    let first_term = term_iterator.next();
    if let Some(first_term) = first_term {
        current_context = evaluate_first_term(first_term, current_context, config.clone()).await?;
    }

    for term in term_iterator {
        current_context = evaluate_term(term, current_context, config.clone()).await?;
    }

    Ok(current_context)
}

async fn operation_2<'a>(
    left: &Expression,
    right: &Expression,
    context: Context<'a>,
    config: Option<Arc<Config<'a>>>,
    executor: impl Fn(
        Context<'a>,
        Context<'a>,
    )
        -> Pin<Box<dyn Future<Output = Result<Context<'a>, FHIRPathError>> + Send + 'a>>,
) -> Result<Context<'a>, FHIRPathError> {
    let left = evaluate_expression(left, context.clone(), config.clone()).await?;
    let right = evaluate_expression(right, context, config).await?;

    // If one of operands is empty per spec return an empty collection
    if left.values.is_empty() || right.values.is_empty() {
        return Ok(left.new_context_from(vec![]));
    }

    if left.values.len() != 1 || right.values.len() != 1 {
        return Err(FHIRPathError::OperationError(
            OperationError::InvalidCardinality,
        ));
    }

    executor(left, right).await
}

async fn operation_n<'a>(
    left: &Expression,
    right: &Expression,
    context: Context<'a>,
    config: Option<Arc<Config<'a>>>,
    executor: impl Fn(Context<'a>, Context<'a>) -> Result<Context<'a>, FHIRPathError>,
) -> Result<Context<'a>, FHIRPathError> {
    let left = evaluate_expression(left, context.clone(), config.clone()).await?;
    let right = evaluate_expression(right, context, config).await?;
    executor(left, right)
}

enum Cardinality {
    Zero,
    One,
    Many,
    Custom(usize, usize),
}

fn validate_arguments(
    ast_arguments: &[Expression],
    cardinality: &Cardinality,
) -> Result<(), FHIRPathError> {
    match cardinality {
        Cardinality::Zero => {
            if !ast_arguments.is_empty() {
                return Err(FHIRPathError::OperationError(
                    OperationError::InvalidCardinality,
                ));
            }
        }
        Cardinality::Custom(min, max) => {
            if ast_arguments.len() < *min || ast_arguments.len() > *max {
                return Err(FHIRPathError::OperationError(
                    OperationError::InvalidCardinality,
                ));
            }
        }
        Cardinality::One => {
            if ast_arguments.len() != 1 {
                return Err(FHIRPathError::OperationError(
                    OperationError::InvalidCardinality,
                ));
            }
        }
        Cardinality::Many => {}
    }
    Ok(())
}

fn derive_typename(expression_ast: &Expression) -> Result<String, FHIRPathError> {
    match expression_ast {
        Expression::Singular(ast) => match &ast[0] {
            Term::Invocation(Invocation::Identifier(type_id)) => Ok(type_id.0.clone()),
            _ => Err(FHIRPathError::FailedTypeNameDerivation),
        },
        Expression::Operation(_) => Err(FHIRPathError::FailedTypeNameDerivation),
    }
}

fn check_type_name(type_name: &str, type_to_check: &str) -> bool {
    match type_to_check {
        "Resource" | "DomainResource" => ResourceType::try_from(type_name).is_ok(),
        _ => type_name == type_to_check,
    }
}

fn check_type(value: &dyn MetaValue, type_to_check: &str) -> bool {
    let fhir_type_name = value.fhir_type();

    match fhir_type_name {
        // Special handling for reference which is to check the reference type.
        "Reference" => {
            if type_to_check == "Reference" {
                return true;
            } else if let Some(reference) = value.as_any().downcast_ref::<Reference>()
                && let Some(resource_type) = reference
                    .reference
                    .as_ref()
                    .and_then(|r| r.value.as_ref())
                    .and_then(|r| r.split('/').next())
            {
                return check_type_name(resource_type, type_to_check);
            }
            false
        }
        fhir_type_name => check_type_name(fhir_type_name, type_to_check),
    }
}

fn filter_by_type<'a>(type_name: &str, context: &Context<'a>) -> Context<'a> {
    context.new_context_from(
        context
            .values
            .iter()
            .filter(|v| check_type(**v, type_name))
            .copied()
            .collect(),
    )
}

#[derive(Debug, Reflect)]
#[fhir_type = "Element"]
struct Reflection {
    name: String,
}

async fn evaluate_function<'a>(
    function: &FunctionInvocation,
    context: Context<'a>,
    config: Option<Arc<Config<'a>>>,
) -> Result<Context<'a>, FHIRPathError> {
    match function.name.0.as_str() {
        "resolve" => Ok(context),
        "where" => evaluate_where(function, context, config).await,
        // R4 search parameters apply as() to collections, so it filters like ofType.
        "ofType" | "as" => evaluate_of_type(function, &context),
        "is" => evaluate_is_function(function, &context),
        "count" => evaluate_count(function, &context),
        "upper" | "lower" => evaluate_case(function, &context),
        "empty" => evaluate_empty(function, &context),
        "join" => evaluate_join(function, context, config).await,
        "exists" => evaluate_exists(function, context, config).await,
        "children" => evaluate_children(function, &context),
        "repeat" => evaluate_repeat(function, context, config).await,
        "descendants" => evaluate_descendants(context, config).await,
        "type" => evaluate_type(function, &context),
        "first" => evaluate_first(function, &context),
        "single" | "last" | "tail" => evaluate_subsetting(function, &context),
        "skip" | "take" => evaluate_skip_take(function, context, config).await,
        "not" => evaluate_not(function, &context),
        "all" => evaluate_all(function, context, config).await,
        "allTrue" | "anyTrue" | "allFalse" | "anyFalse" => {
            evaluate_boolean_aggregate(function, &context)
        }
        "subsetOf" | "supersetOf" => evaluate_subset(function, context, config).await,
        "distinct" | "isDistinct" => evaluate_distinct(function, &context),
        "select" => evaluate_select(function, context, config).await,
        "union" | "combine" | "intersect" | "exclude" => {
            evaluate_combining(function, context, config).await
        }
        "iif" => evaluate_iif(function, context, config).await,
        "abs" | "ceiling" | "exp" | "floor" | "ln" | "log" | "power" | "round" | "sqrt"
        | "truncate" => evaluate_math(function, context, config).await,
        "startsWith" => evaluate_string_match(StringMatch::StartsWith, function, &context),
        "endsWith" => evaluate_string_match(StringMatch::EndsWith, function, &context),
        "contains" => evaluate_string_match(StringMatch::Contains, function, &context),
        "getReferenceKey" => evaluate_get_reference_key(function, &context),
        "getResourceKey" => evaluate_get_resource_key(function, &context, config),
        "indexOf" | "substring" | "length" | "toChars" | "replace" | "matches"
        | "replaceMatches" => evaluate_string_function(function, context, config).await,
        "toBoolean" | "convertsToBoolean" | "toInteger" | "convertsToInteger" | "toDecimal"
        | "convertsToDecimal" | "toString" | "convertsToString" | "toDate" | "convertsToDate"
        | "toDateTime" | "convertsToDateTime" | "toTime" | "convertsToTime" => {
            evaluate_conversion(function, &context)
        }
        "toQuantity" | "convertsToQuantity" => {
            evaluate_to_quantity(function, context, config).await
        }
        "trace" => evaluate_trace(function, context, config).await,
        "now" | "timeOfDay" | "today" => evaluate_current_time(function, &context),

        _ => Err(FHIRPathError::NotImplemented(format!(
            "Function '{}' is not implemented",
            function.name.0
        ))),
    }
}

async fn evaluate_where<'a>(
    function: &FunctionInvocation,
    context: Context<'a>,
    config: Option<Arc<Config<'a>>>,
) -> Result<Context<'a>, FHIRPathError> {
    validate_arguments(&function.arguments, &Cardinality::One)?;

    let where_condition = &function.arguments[0];
    let mut new_context = vec![];

    for value in &context.values {
        let result = evaluate_expression(
            where_condition,
            context.new_context_from(vec![*value]),
            config.clone(),
        )
        .await?;

        if result.values.len() > 1 {
            return Err(FHIRPathError::InternalError(
                "Where condition did not return a single value".to_string(),
            ));
        }

        // Empty result is treated as false.
        if !result.values.is_empty() && downcast_bool(result.values[0])? {
            new_context.push(*value);
        }
    }

    Ok(context.new_context_from(new_context))
}

fn evaluate_of_type<'a>(
    function: &FunctionInvocation,
    context: &Context<'a>,
) -> Result<Context<'a>, FHIRPathError> {
    validate_arguments(&function.arguments, &Cardinality::One)?;

    let type_name = derive_typename(&function.arguments[0])?;

    Ok(filter_by_type(&type_name, context))
}

fn evaluate_count<'a>(
    function: &FunctionInvocation,
    context: &Context<'a>,
) -> Result<Context<'a>, FHIRPathError> {
    validate_arguments(&function.arguments, &Cardinality::Zero)?;

    let count: i64 = context
        .values
        .len()
        .try_into()
        .map_err(|_| FHIRPathError::OperationError(OperationError::SizeOverflow))?;

    Ok(
        context.new_context_from(vec![context.allocate_literal(FHIRInteger {
            value: Some(count),
            ..Default::default()
        })]),
    )
}

fn evaluate_case<'a>(
    function: &FunctionInvocation,
    context: &Context<'a>,
) -> Result<Context<'a>, FHIRPathError> {
    validate_arguments(&function.arguments, &Cardinality::Zero)?;

    let op = function.name.0.as_str();

    if context.values.is_empty() {
        return Ok(context.new_context_from(vec![]));
    }

    if context.values.len() > 1 {
        return Err(FunctionError::InvalidCardinality(op.to_string(), context.values.len()).into());
    }

    let input = downcast_string(context.values[0])?;

    let transformed = match op {
        "upper" => input.to_uppercase(),
        "lower" => input.to_lowercase(),
        _ => unreachable!(),
    };

    Ok(
        context.new_context_from(vec![context.allocate_literal(FHIRString {
            value: Some(transformed),
            ..Default::default()
        })]),
    )
}

fn evaluate_empty<'a>(
    function: &FunctionInvocation,
    context: &Context<'a>,
) -> Result<Context<'a>, FHIRPathError> {
    validate_arguments(&function.arguments, &Cardinality::Zero)?;

    Ok(
        context.new_context_from(vec![context.allocate_literal(FHIRBoolean {
            value: Some(context.values.is_empty()),
            ..Default::default()
        })]),
    )
}

async fn evaluate_join<'a>(
    function: &FunctionInvocation,
    context: Context<'a>,
    config: Option<Arc<Config<'a>>>,
) -> Result<Context<'a>, FHIRPathError> {
    validate_arguments(&function.arguments, &Cardinality::Custom(0, 1))?;

    let separator = if let Some(separator_expression) = function.arguments.first() {
        let separator_context =
            evaluate_expression(separator_expression, context.clone(), config).await?;

        if separator_context.values.len() != 1 {
            return Err(FHIRPathError::OperationError(
                OperationError::InvalidCardinality,
            ));
        }

        downcast_string(separator_context.values[0])?
    } else {
        String::new()
    };

    let joined = context
        .values
        .iter()
        .map(|v| downcast_string(*v))
        .collect::<Result<Vec<_>, _>>()?
        .join(&separator);

    Ok(
        context.new_context_from(vec![context.allocate_literal(FHIRString {
            value: Some(joined),
            ..Default::default()
        })]),
    )
}

async fn evaluate_exists<'a>(
    function: &FunctionInvocation,
    context: Context<'a>,
    config: Option<Arc<Config<'a>>>,
) -> Result<Context<'a>, FHIRPathError> {
    validate_arguments(&function.arguments, &Cardinality::Many)?;

    if function.arguments.len() > 1 {
        return Err(FunctionError::InvalidCardinality(
            "exists".to_string(),
            function.arguments.len(),
        )
        .into());
    }

    if let Some(condition) = function.arguments.first() {
        for value in &context.values {
            let result = evaluate_expression(
                condition,
                context.new_context_from(vec![*value]),
                config.clone(),
            )
            .await?;

            if result.values.len() > 1 {
                return Err(FHIRPathError::InternalError(
                    "Exists condition did not return a single value".to_string(),
                ));
            }

            if !result.values.is_empty() && downcast_bool(result.values[0])? {
                return Ok(
                    context.new_context_from(vec![context.allocate_literal(FHIRBoolean {
                        value: Some(true),
                        ..Default::default()
                    })]),
                );
            }
        }

        return Ok(
            context.new_context_from(vec![context.allocate_literal(FHIRBoolean {
                value: Some(false),
                ..Default::default()
            })]),
        );
    }

    Ok(
        context.new_context_from(vec![context.allocate_literal(FHIRBoolean {
            value: Some(!context.values.is_empty()),
            ..Default::default()
        })]),
    )
}

fn evaluate_children<'a>(
    function: &FunctionInvocation,
    context: &Context<'a>,
) -> Result<Context<'a>, FHIRPathError> {
    validate_arguments(&function.arguments, &Cardinality::Zero)?;

    let children = context
        .values
        .iter()
        .flat_map(|value| {
            value
                .fields()
                .iter()
                .filter_map(|f| value.get_field(f).map(|v| v.flatten()))
                .flatten()
                .collect::<Vec<_>>()
        })
        .collect();

    Ok(context.new_context_from(children))
}

async fn evaluate_repeat<'a>(
    function: &FunctionInvocation,
    context: Context<'a>,
    config: Option<Arc<Config<'a>>>,
) -> Result<Context<'a>, FHIRPathError> {
    validate_arguments(&function.arguments, &Cardinality::One)?;

    let projection = &function.arguments[0];
    let mut end_result = vec![];
    let mut cur = context;

    while !cur.values.is_empty() {
        cur = evaluate_expression(projection, cur, config.clone()).await?;
        end_result.extend_from_slice(cur.values.as_slice());
    }

    Ok(cur.new_context_from(end_result))
}

async fn evaluate_descendants<'a>(
    context: Context<'a>,
    config: Option<Arc<Config<'a>>>,
) -> Result<Context<'a>, FHIRPathError> {
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
    )
    .await?;

    Ok(result)
}

fn evaluate_type<'a>(
    function: &FunctionInvocation,
    context: &Context<'a>,
) -> Result<Context<'a>, FHIRPathError> {
    validate_arguments(&function.arguments, &Cardinality::Zero)?;

    let mut next_ctx = Vec::with_capacity(context.values.len());

    for value in &context.values {
        let type_name = value.fhir_type();

        next_ctx.push(context.allocate_literal(Reflection {
            name: type_name.to_string(),
        }));
    }

    Ok(context.new_context_from(next_ctx))
}

fn evaluate_first<'a>(
    function: &FunctionInvocation,
    context: &Context<'a>,
) -> Result<Context<'a>, FHIRPathError> {
    validate_arguments(&function.arguments, &Cardinality::Zero)?;

    match context.values.first() {
        Some(value) => Ok(context.new_context_from(vec![*value])),
        None => Ok(context.new_context_from(vec![])),
    }
}

fn boolean_result<'a>(context: &Context<'a>, value: bool) -> Context<'a> {
    context.new_context_from(vec![context.allocate_literal(FHIRBoolean {
        value: Some(value),
        ..Default::default()
    })])
}

fn optional_boolean_result<'a>(context: &Context<'a>, value: Option<bool>) -> Context<'a> {
    match value {
        Some(value) => boolean_result(context, value),
        None => context.new_context_from(vec![]),
    }
}

/// A number that keeps its type, so Integer arithmetic gives an Integer.
#[derive(Clone, Copy)]
enum Number {
    Integer(i64),
    Decimal(f64),
}

impl Number {
    fn of(value: &dyn MetaValue) -> Result<Number, FHIRPathError> {
        let any = value.as_any();
        let integer = if let Some(integer) = any.downcast_ref::<FHIRInteger>() {
            integer.value
        } else if let Some(integer) = any.downcast_ref::<FHIRPositiveInt>() {
            integer.value.and_then(|v| i64::try_from(v).ok())
        } else if let Some(integer) = any.downcast_ref::<FHIRUnsignedInt>() {
            integer.value.and_then(|v| i64::try_from(v).ok())
        } else {
            any.downcast_ref::<i64>().copied()
        };

        match integer {
            Some(integer) => Ok(Number::Integer(integer)),
            None => Ok(Number::Decimal(downcast_number(value)?)),
        }
    }

    #[allow(clippy::cast_precision_loss)]
    fn as_f64(self) -> f64 {
        match self {
            Number::Integer(value) => value as f64,
            Number::Decimal(value) => value,
        }
    }

    /// `None` when the whole number does not fit an Integer.
    #[allow(clippy::cast_possible_truncation)]
    fn integer_from(value: f64) -> Option<Number> {
        // 2^63, exact as an f64. The range also rejects NaN and infinity.
        const BOUND: f64 = 9_223_372_036_854_775_808.0;
        (-BOUND..BOUND)
            .contains(&value)
            .then_some(Number::Integer(value as i64))
    }

    /// `None` for NaN and infinity, which a Decimal cannot represent.
    fn decimal_from(value: f64) -> Option<Number> {
        value.is_finite().then_some(Number::Decimal(value))
    }
}

fn number_result<'a>(context: &Context<'a>, number: Option<Number>) -> Context<'a> {
    let value = match number {
        Some(Number::Integer(value)) => context.allocate_literal(FHIRInteger {
            value: Some(value),
            ..Default::default()
        }),
        Some(Number::Decimal(value)) => context.allocate_literal(FHIRDecimal {
            value: Some(value),
            ..Default::default()
        }),
        None => return context.new_context_from(vec![]),
    };

    context.new_context_from(vec![value])
}

/// The input's only item: `None` when empty, an error for more than one.
fn singleton<'a>(
    name: &str,
    context: &Context<'a>,
) -> Result<Option<&'a dyn MetaValue>, FHIRPathError> {
    match context.values.as_slice() {
        [] => Ok(None),
        [value] => Ok(Some(*value)),
        values => Err(FunctionError::InvalidCardinality(name.to_string(), values.len()).into()),
    }
}

/// <https://hl7.org/fhirpath/N1/#singleton-evaluation-of-collections>: a
/// single item that is not a Boolean counts as true.
fn singleton_boolean(name: &str, context: &Context<'_>) -> Result<Option<bool>, FHIRPathError> {
    match singleton(name, context)? {
        None => Ok(None),
        Some(value) if BOOLEAN_TYPES.contains(value.fhir_type()) => Ok(Some(downcast_bool(value)?)),
        Some(_) => Ok(Some(true)),
    }
}

fn contains_value(
    collection: &[&dyn MetaValue],
    value: &dyn MetaValue,
) -> Result<bool, FHIRPathError> {
    for item in collection {
        if value_equal(*item, value)? {
            return Ok(true);
        }
    }
    Ok(false)
}

/// The items of `values` whose membership in `other` is `member`.
fn filter_by_membership<'a>(
    values: impl IntoIterator<Item = &'a dyn MetaValue>,
    other: &[&dyn MetaValue],
    member: bool,
) -> Result<Vec<&'a dyn MetaValue>, FHIRPathError> {
    let mut filtered = Vec::new();
    for value in values {
        if contains_value(other, value)? == member {
            filtered.push(value);
        }
    }
    Ok(filtered)
}

fn distinct_values<'a>(
    values: impl IntoIterator<Item = &'a dyn MetaValue>,
) -> Result<Vec<&'a dyn MetaValue>, FHIRPathError> {
    let mut distinct = Vec::new();
    for value in values {
        if !contains_value(&distinct, value)? {
            distinct.push(value);
        }
    }
    Ok(distinct)
}

/// <https://hl7.org/fhirpath/N1/#not-boolean>
fn evaluate_not<'a>(
    function: &FunctionInvocation,
    context: &Context<'a>,
) -> Result<Context<'a>, FHIRPathError> {
    validate_arguments(&function.arguments, &Cardinality::Zero)?;

    let value = singleton_boolean("not", context)?;
    Ok(optional_boolean_result(context, value.map(|value| !value)))
}

/// <https://hl7.org/fhirpath/N1/#allcriteria-expression-boolean>
async fn evaluate_all<'a>(
    function: &FunctionInvocation,
    context: Context<'a>,
    config: Option<Arc<Config<'a>>>,
) -> Result<Context<'a>, FHIRPathError> {
    validate_arguments(&function.arguments, &Cardinality::One)?;

    for value in &context.values {
        let item = context.new_context_from(vec![*value]);
        let result = evaluate_expression(&function.arguments[0], item, config.clone()).await?;

        if singleton_boolean("all", &result)? != Some(true) {
            return Ok(boolean_result(&context, false));
        }
    }

    Ok(boolean_result(&context, true))
}

/// <https://hl7.org/fhirpath/N1/#selectprojection-expression-collection>
async fn evaluate_select<'a>(
    function: &FunctionInvocation,
    context: Context<'a>,
    config: Option<Arc<Config<'a>>>,
) -> Result<Context<'a>, FHIRPathError> {
    validate_arguments(&function.arguments, &Cardinality::One)?;

    let mut selected = vec![];
    for value in &context.values {
        let item = context.new_context_from(vec![*value]);
        let result = evaluate_expression(&function.arguments[0], item, config.clone()).await?;
        selected.extend(result.values);
    }

    Ok(context.new_context_from(selected))
}

/// <https://hl7.org/fhirpath/N1/#alltrue-boolean> and the three that follow it.
fn evaluate_boolean_aggregate<'a>(
    function: &FunctionInvocation,
    context: &Context<'a>,
) -> Result<Context<'a>, FHIRPathError> {
    validate_arguments(&function.arguments, &Cardinality::Zero)?;

    let values = context
        .values
        .iter()
        .map(|value| downcast_bool(*value))
        .collect::<Result<Vec<_>, _>>()?;

    let result = match function.name.0.as_str() {
        "allTrue" => values.iter().all(|value| *value),
        "anyTrue" => values.iter().any(|value| *value),
        "allFalse" => values.iter().all(|value| !*value),
        "anyFalse" => values.iter().any(|value| !*value),
        _ => unreachable!(),
    };

    Ok(boolean_result(context, result))
}

/// <https://hl7.org/fhirpath/N1/#subsetofother-collection-boolean> and
/// <https://hl7.org/fhirpath/N1/#supersetofother-collection-boolean>
async fn evaluate_subset<'a>(
    function: &FunctionInvocation,
    context: Context<'a>,
    config: Option<Arc<Config<'a>>>,
) -> Result<Context<'a>, FHIRPathError> {
    validate_arguments(&function.arguments, &Cardinality::One)?;

    let other = evaluate_expression(&function.arguments[0], context.clone(), config).await?;
    let (subset, superset) = if function.name.0 == "subsetOf" {
        (&context.values, &other.values)
    } else {
        (&other.values, &context.values)
    };

    let missing = filter_by_membership(subset.iter().copied(), superset, false)?;
    Ok(boolean_result(&context, missing.is_empty()))
}

/// <https://hl7.org/fhirpath/N1/#distinct-collection> and
/// <https://hl7.org/fhirpath/N1/#isdistinct-boolean>
fn evaluate_distinct<'a>(
    function: &FunctionInvocation,
    context: &Context<'a>,
) -> Result<Context<'a>, FHIRPathError> {
    validate_arguments(&function.arguments, &Cardinality::Zero)?;

    let distinct = distinct_values(context.values.iter().copied())?;

    if function.name.0 == "isDistinct" {
        let is_distinct = distinct.len() == context.values.len();
        Ok(boolean_result(context, is_distinct))
    } else {
        Ok(context.new_context_from(distinct))
    }
}

/// <https://hl7.org/fhirpath/N1/#single-collection>,
/// <https://hl7.org/fhirpath/N1/#last-collection> and
/// <https://hl7.org/fhirpath/N1/#tail-collection>
fn evaluate_subsetting<'a>(
    function: &FunctionInvocation,
    context: &Context<'a>,
) -> Result<Context<'a>, FHIRPathError> {
    validate_arguments(&function.arguments, &Cardinality::Zero)?;

    let values = match function.name.0.as_str() {
        "single" => singleton("single", context)?.into_iter().collect(),
        "last" => context.values.last().copied().into_iter().collect(),
        "tail" => context.values.iter().skip(1).copied().collect(),
        _ => unreachable!(),
    };

    Ok(context.new_context_from(values))
}

/// <https://hl7.org/fhirpath/N1/#skipnum-integer-collection> and
/// <https://hl7.org/fhirpath/N1/#takenum-integer-collection>
async fn evaluate_skip_take<'a>(
    function: &FunctionInvocation,
    context: Context<'a>,
    config: Option<Arc<Config<'a>>>,
) -> Result<Context<'a>, FHIRPathError> {
    validate_arguments(&function.arguments, &Cardinality::One)?;

    let name = function.name.0.as_str();
    let num = evaluate_expression(&function.arguments[0], context.clone(), config).await?;
    let num = match singleton(name, &num)?.map(Number::of).transpose()? {
        Some(Number::Integer(num)) => num,
        None => return Ok(context.new_context_from(vec![])),
        Some(Number::Decimal(_)) => {
            let message = format!("{name} requires an Integer argument");
            return Err(FunctionError::InvalidFunctionCall(message).into());
        }
    };

    // Zero or less skips nothing and takes nothing.
    let num = usize::try_from(num.max(0)).unwrap_or(usize::MAX);
    let values = context.values.iter().copied();
    let values = if name == "skip" {
        values.skip(num).collect()
    } else {
        values.take(num).collect()
    };

    Ok(context.new_context_from(values))
}

/// <https://hl7.org/fhirpath/N1/#unionother-collection>,
/// <https://hl7.org/fhirpath/N1/#combineother-collection-collection>,
/// <https://hl7.org/fhirpath/N1/#intersectother-collection-collection> and
/// <https://hl7.org/fhirpath/N1/#excludeother-collection-collection>
async fn evaluate_combining<'a>(
    function: &FunctionInvocation,
    context: Context<'a>,
    config: Option<Arc<Config<'a>>>,
) -> Result<Context<'a>, FHIRPathError> {
    validate_arguments(&function.arguments, &Cardinality::One)?;

    let other = evaluate_expression(&function.arguments[0], context.clone(), config).await?;
    let input = context.values.iter().copied();
    let both = input.clone().chain(other.values.iter().copied());

    let values = match function.name.0.as_str() {
        "union" => distinct_values(both)?,
        "combine" => both.collect(),
        "intersect" => filter_by_membership(distinct_values(input)?, &other.values, true)?,
        "exclude" => filter_by_membership(input, &other.values, false)?,
        _ => unreachable!(),
    };

    Ok(context.new_context_from(values))
}

/// <https://hl7.org/fhirpath/N1/#iifcriterion-expression-true-result-collection-otherwise-result-collection-collection>
async fn evaluate_iif<'a>(
    function: &FunctionInvocation,
    context: Context<'a>,
    config: Option<Arc<Config<'a>>>,
) -> Result<Context<'a>, FHIRPathError> {
    validate_arguments(&function.arguments, &Cardinality::Custom(2, 3))?;

    let criterion =
        evaluate_expression(&function.arguments[0], context.clone(), config.clone()).await?;

    // Only the chosen branch is evaluated.
    let branch = if singleton_boolean("iif", &criterion)? == Some(true) {
        function.arguments.get(1)
    } else {
        function.arguments.get(2)
    };

    match branch {
        Some(branch) => evaluate_expression(branch, context, config).await,
        None => Ok(context.new_context_from(vec![])),
    }
}

/// <https://hl7.org/fhirpath/N1/#math>: empty for an empty input or a result
/// that cannot be represented, such as `(-1).sqrt()`.
async fn evaluate_math<'a>(
    function: &FunctionInvocation,
    context: Context<'a>,
    config: Option<Arc<Config<'a>>>,
) -> Result<Context<'a>, FHIRPathError> {
    let name = function.name.0.as_str();
    let cardinality = match name {
        "log" | "power" => Cardinality::One,
        "round" => Cardinality::Custom(0, 1),
        _ => Cardinality::Zero,
    };
    validate_arguments(&function.arguments, &cardinality)?;

    let Some(input) = singleton(name, &context)? else {
        return Ok(context.new_context_from(vec![]));
    };

    // abs keeps a Quantity's unit.
    if name == "abs"
        && let Some(quantity) = input.as_any().downcast_ref::<Quantity>()
    {
        let mut quantity = quantity.clone();
        if let Some(value) = quantity.value.as_mut() {
            value.value = value.value.map(f64::abs);
        }
        return Ok(context.new_context_from(vec![context.allocate_literal(quantity)]));
    }

    let argument = match function.arguments.first() {
        Some(argument) => {
            let argument = evaluate_expression(argument, context.clone(), config).await?;
            singleton(name, &argument)?.map(Number::of).transpose()?
        }
        None => None,
    };

    let number = Number::of(input)?;
    let value = number.as_f64();
    let result = match (name, number, argument) {
        ("abs", Number::Integer(integer), _) => integer.checked_abs().map(Number::Integer),
        ("abs", Number::Decimal(decimal), _) => Some(Number::Decimal(decimal.abs())),
        ("ceiling", _, _) => Number::integer_from(value.ceil()),
        ("floor", _, _) => Number::integer_from(value.floor()),
        ("truncate", _, _) => Number::integer_from(value.trunc()),
        ("exp", _, _) => Number::decimal_from(value.exp()),
        ("ln", _, _) => Number::decimal_from(value.ln()),
        ("sqrt", _, _) => Number::decimal_from(value.sqrt()),
        ("log", _, Some(base)) => Number::decimal_from(value.log(base.as_f64())),
        ("power", Number::Integer(base), Some(Number::Integer(exponent))) if exponent >= 0 => {
            u32::try_from(exponent)
                .ok()
                .and_then(|exponent| base.checked_pow(exponent))
                .map(Number::Integer)
        }
        ("power", _, Some(exponent)) => Number::decimal_from(value.powf(exponent.as_f64())),
        ("round", _, None) => Number::decimal_from(value.round()),
        ("round", _, Some(Number::Integer(precision @ 0..))) => {
            // An f64 holds about 15 decimal places, so more changes nothing.
            let scale = 10f64.powi(i32::try_from(precision.min(15)).unwrap_or(15));
            Number::decimal_from((value * scale).round() / scale)
        }
        ("round", _, Some(_)) => {
            let message = "round precision must be an Integer >= 0".to_string();
            return Err(FunctionError::InvalidFunctionCall(message).into());
        }
        // log or power with an empty argument.
        _ => None,
    };

    Ok(number_result(&context, result))
}

fn string_result<'a>(context: &Context<'a>, value: String) -> Context<'a> {
    context.new_context_from(vec![context.allocate_literal(FHIRString {
        value: Some(value),
        ..Default::default()
    })])
}

/// Compiled patterns, which come from expressions and are cached like their ASTs.
static REGEX: LazyLock<DashMap<String, Arc<Regex>>> = LazyLock::new(DashMap::new);

fn compile_regex(pattern: &str) -> Result<Arc<Regex>, FHIRPathError> {
    if let Some(regex) = REGEX.get(pattern) {
        return Ok(regex.clone());
    }

    // The spec's 'single line' mode, where '.' also matches a newline.
    let regex = RegexBuilder::new(pattern)
        .dot_matches_new_line(true)
        .build()
        .map_err(|e| FunctionError::InvalidFunctionCall(format!("invalid regex: {e}")))?;
    let regex = Arc::new(regex);
    REGEX.insert(pattern.to_string(), regex.clone());
    Ok(regex)
}

fn char_count(value: &str) -> Result<i64, FHIRPathError> {
    i64::try_from(value.chars().count())
        .map_err(|_| FHIRPathError::OperationError(OperationError::SizeOverflow))
}

/// <https://hl7.org/fhirpath/N1/#indexofsubstring-string-integer> through
/// <https://hl7.org/fhirpath/N1/#tochars-collection>: empty when the input or
/// a required argument is empty.
async fn evaluate_string_function<'a>(
    function: &FunctionInvocation,
    context: Context<'a>,
    config: Option<Arc<Config<'a>>>,
) -> Result<Context<'a>, FHIRPathError> {
    let name = function.name.0.as_str();
    let cardinality = match name {
        "length" | "toChars" => Cardinality::Zero,
        "indexOf" | "matches" => Cardinality::One,
        "substring" => Cardinality::Custom(1, 2),
        _ => Cardinality::Custom(2, 2),
    };
    validate_arguments(&function.arguments, &cardinality)?;

    let empty = context.new_context_from(vec![]);
    let Some(input) = singleton(name, &context)? else {
        return Ok(empty);
    };
    let input = downcast_string(input)?;

    let mut arguments = Vec::with_capacity(function.arguments.len());
    for argument in &function.arguments {
        let argument = evaluate_expression(argument, context.clone(), config.clone()).await?;
        arguments.push(singleton(name, &argument)?);
    }
    let string_argument = |index: usize| -> Result<Option<String>, FHIRPathError> {
        Ok(arguments[index].map(downcast_string).transpose()?)
    };
    let integer_argument = |index: usize| -> Result<Option<i64>, FHIRPathError> {
        match arguments
            .get(index)
            .copied()
            .flatten()
            .map(Number::of)
            .transpose()?
        {
            Some(Number::Integer(value)) => Ok(Some(value)),
            None => Ok(None),
            Some(Number::Decimal(_)) => {
                let message = format!("{name} requires Integer arguments");
                Err(FunctionError::InvalidFunctionCall(message).into())
            }
        }
    };

    let result = match name {
        "length" => number_result(&context, Some(Number::Integer(char_count(&input)?))),
        "toChars" => {
            let chars = input.chars().map(|c| {
                context.allocate_literal(FHIRString {
                    value: Some(c.to_string()),
                    ..Default::default()
                })
            });
            context.new_context_from(chars.collect())
        }
        "indexOf" => {
            let Some(substring) = string_argument(0)? else {
                return Ok(empty);
            };
            let index = match input.find(&substring) {
                Some(byte_index) => char_count(&input[..byte_index])?,
                None => -1,
            };
            number_result(&context, Some(Number::Integer(index)))
        }
        "substring" => {
            let Some(start) = integer_argument(0)? else {
                return Ok(empty);
            };
            let Ok(start) = usize::try_from(start) else {
                return Ok(empty);
            };
            if start >= input.chars().count() {
                return Ok(empty);
            }
            // An empty length is the same as none.
            let length = integer_argument(1)?
                .map_or(usize::MAX, |length| usize::try_from(length).unwrap_or(0));
            string_result(&context, input.chars().skip(start).take(length).collect())
        }
        "replace" => {
            let (Some(pattern), Some(substitution)) = (string_argument(0)?, string_argument(1)?)
            else {
                return Ok(empty);
            };
            string_result(&context, input.replace(&pattern, &substitution))
        }
        "matches" => {
            let Some(pattern) = string_argument(0)? else {
                return Ok(empty);
            };
            boolean_result(&context, compile_regex(&pattern)?.is_match(&input))
        }
        "replaceMatches" => {
            let (Some(pattern), Some(substitution)) = (string_argument(0)?, string_argument(1)?)
            else {
                return Ok(empty);
            };
            let replaced = compile_regex(&pattern)?.replace_all(&input, substitution.as_str());
            string_result(&context, replaced.into_owned())
        }
        _ => unreachable!(),
    };

    Ok(result)
}

/// The kinds of item the conversion functions accept.
enum Convertible<'v> {
    Boolean(bool),
    Number(Number),
    String(String),
    DateTime(String),
    Quantity(&'v Quantity),
}

impl<'v> Convertible<'v> {
    fn of(value: &'v dyn MetaValue) -> Result<Option<Self>, FHIRPathError> {
        let fhir_type = value.fhir_type();
        let convertible = if BOOLEAN_TYPES.contains(fhir_type) {
            Convertible::Boolean(downcast_bool(value)?)
        } else if NUMBER_TYPES.contains(fhir_type) {
            Convertible::Number(Number::of(value)?)
        } else if STRING_TYPES.contains(fhir_type) {
            Convertible::String(downcast_string(value)?)
        } else if DATE_TIME_TYPES.contains(fhir_type) {
            Convertible::DateTime(downcast_datetime(value)?)
        } else if let Some(quantity) = value.as_any().downcast_ref::<Quantity>() {
            Convertible::Quantity(quantity)
        } else {
            return Ok(None);
        };
        Ok(Some(convertible))
    }
}

/// <https://hl7.org/fhirpath/N1/#boolean-conversion-functions>
fn to_boolean(value: Convertible) -> Option<bool> {
    match value {
        Convertible::Boolean(value) => Some(value),
        Convertible::Number(number) => match number.as_f64() {
            1.0 => Some(true),
            0.0 => Some(false),
            _ => None,
        },
        Convertible::String(value) => {
            let is = |words: &[&str]| words.iter().any(|word| value.eq_ignore_ascii_case(word));
            if is(&["true", "t", "yes", "y", "1", "1.0"]) {
                Some(true)
            } else if is(&["false", "f", "no", "n", "0", "0.0"]) {
                Some(false)
            } else {
                None
            }
        }
        Convertible::DateTime(_) | Convertible::Quantity(_) => None,
    }
}

/// <https://hl7.org/fhirpath/N1/#integer-conversion-functions>
fn to_integer(value: Convertible) -> Option<i64> {
    match value {
        Convertible::Boolean(value) => Some(i64::from(value)),
        Convertible::Number(Number::Integer(value)) => Some(value),
        // Rust's parser accepts exactly (\+|-)?\d+, failing only on overflow.
        Convertible::String(value) => value.parse().ok(),
        _ => None,
    }
}

/// Whether `value` has the spec's Decimal format, (\+|-)?\d+(\.\d+)?.
fn is_decimal(value: &str) -> bool {
    let unsigned = value.strip_prefix(['+', '-']).unwrap_or(value);
    let (whole, fraction) = unsigned.split_once('.').unwrap_or((unsigned, "0"));
    let digits = |part: &str| !part.is_empty() && part.bytes().all(|b| b.is_ascii_digit());
    digits(whole) && digits(fraction)
}

/// <https://hl7.org/fhirpath/N1/#decimal-conversion-functions>
fn to_decimal(value: Convertible) -> Option<f64> {
    match value {
        Convertible::Boolean(value) => Some(f64::from(u8::from(value))),
        Convertible::Number(number) => Some(number.as_f64()),
        Convertible::String(value) if is_decimal(&value) => value.parse().ok(),
        _ => None,
    }
}

/// <https://hl7.org/fhirpath/N1/#date-conversion-functions>: the date part of a
/// `Date`, `DateTime` or date-formatted `String`, keeping its precision.
fn to_date(value: Convertible) -> Option<Date> {
    let (Convertible::String(value) | Convertible::DateTime(value)) = value else {
        return None;
    };
    if !DATETIME_REGEX.is_match(&value) {
        return None;
    }
    let date = value
        .split_once('T')
        .map_or(value.as_str(), |(date, _)| date);
    parse_date(date).ok()
}

/// <https://hl7.org/fhirpath/N1/#datetime-conversion-functions>: a `Date`,
/// `DateTime` or datetime-formatted `String`, keeping its precision.
fn to_date_time(value: Convertible) -> Option<DateTime> {
    let (Convertible::String(value) | Convertible::DateTime(value)) = value else {
        return None;
    };
    parse_datetime(&value).ok()
}

/// <https://hl7.org/fhirpath/N1/#time-conversion-functions>: a `Time` or
/// time-formatted `String`, keeping its precision.
fn to_time(value: Convertible) -> Option<Time> {
    let (Convertible::String(value) | Convertible::DateTime(value)) = value else {
        return None;
    };
    parse_partial_time(&value).ok()
}

/// A Decimal's String form, which keeps a fraction so it reads as a Decimal.
fn decimal_string(value: f64) -> String {
    let string = value.to_string();
    if string.contains('.') {
        string
    } else {
        string + ".0"
    }
}

/// <https://hl7.org/fhirpath/N1/#string-conversion-functions>
fn to_string(value: Convertible) -> Option<String> {
    match value {
        Convertible::Boolean(value) => Some(value.to_string()),
        Convertible::Number(Number::Integer(value)) => Some(value.to_string()),
        Convertible::Number(Number::Decimal(value)) => Some(decimal_string(value)),
        Convertible::String(value) | Convertible::DateTime(value) => Some(value),
        Convertible::Quantity(quantity) => {
            let value = quantity.value.as_ref()?.value?;
            Some(format!(
                "{} '{}'",
                decimal_string(value),
                quantity_unit(quantity)
            ))
        }
    }
}

/// The UCUM code when there is one, as `FHIRPath` quantities use UCUM units.
fn quantity_unit(quantity: &Quantity) -> &str {
    let code = quantity
        .code
        .as_ref()
        .and_then(|code| code.value.as_deref());
    let unit = quantity
        .unit
        .as_ref()
        .and_then(|unit| unit.value.as_deref());
    code.or(unit).unwrap_or("1")
}

const UCUM_SYSTEM: &str = "http://unitsofmeasure.org";

/// The spec's calendar duration keywords, which are not UCUM units.
const CALENDAR_UNITS: [&str; 16] = [
    "year",
    "years",
    "month",
    "months",
    "week",
    "weeks",
    "day",
    "days",
    "hour",
    "hours",
    "minute",
    "minutes",
    "second",
    "seconds",
    "millisecond",
    "milliseconds",
];

/// The spec's Quantity string format: a number then an optional quoted UCUM
/// unit or calendar duration keyword.
static QUANTITY_STRING: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^([+-]?\d+(?:\.\d+)?)\s*(?:'([^']+)'|([a-zA-Z]+))?$")
        .expect("quantity regex is valid")
});

fn new_quantity(value: f64, unit: &str) -> Quantity {
    let ucum = !CALENDAR_UNITS.contains(&unit);
    Quantity {
        value: Some(Box::new(FHIRDecimal {
            value: Some(value),
            ..Default::default()
        })),
        unit: Some(Box::new(unit.to_string().into())),
        system: ucum.then(|| Box::new(FHIRUri::from(UCUM_SYSTEM.to_string()))),
        code: Some(Box::new(FHIRCode::from(unit.to_string()))),
        ..Default::default()
    }
}

/// <https://hl7.org/fhirpath/N1/#toquantityunit-string-quantity>. Without a
/// UCUM engine, a `unit` is only satisfied by the item's own unit.
fn to_quantity(value: Convertible, unit: Option<&str>) -> Option<Quantity> {
    let quantity = match value {
        Convertible::Quantity(quantity) => {
            quantity.value.as_ref()?.value?;
            quantity.clone()
        }
        Convertible::Boolean(value) => new_quantity(f64::from(u8::from(value)), "1"),
        Convertible::Number(number) => new_quantity(number.as_f64(), "1"),
        Convertible::String(value) => {
            let captures = QUANTITY_STRING.captures(&value)?;
            let number = captures[1].parse().ok()?;
            let unit = captures.get(2).or_else(|| captures.get(3));
            new_quantity(number, unit.map_or("1", |unit| unit.as_str()))
        }
        Convertible::DateTime(_) => return None,
    };
    unit.is_none_or(|unit| unit == quantity_unit(&quantity))
        .then_some(quantity)
}

/// `toQuantity([unit])` and `convertsToQuantity([unit])`.
async fn evaluate_to_quantity<'a>(
    function: &FunctionInvocation,
    context: Context<'a>,
    config: Option<Arc<Config<'a>>>,
) -> Result<Context<'a>, FHIRPathError> {
    validate_arguments(&function.arguments, &Cardinality::Custom(0, 1))?;

    let name = function.name.0.as_str();
    let converts = name == "convertsToQuantity";
    let empty = context.new_context_from(vec![]);
    let Some(value) = singleton(name, &context)? else {
        return Ok(empty);
    };

    let mut unit = None;
    if let Some(argument) = function.arguments.first() {
        let argument = evaluate_expression(argument, context.clone(), config).await?;
        match singleton(name, &argument)? {
            Some(argument) => unit = Some(downcast_string(argument)?),
            None => return Ok(empty),
        }
    }

    let quantity = Convertible::of(value)?.and_then(|value| to_quantity(value, unit.as_deref()));
    Ok(match quantity {
        _ if converts => boolean_result(&context, quantity.is_some()),
        Some(quantity) => context.new_context_from(vec![context.allocate_literal(quantity)]),
        None => empty,
    })
}

/// `toX()` gives the converted item or empty, `convertsToX()` whether there is
/// one, per <https://hl7.org/fhirpath/N1/#conversion>.
fn evaluate_conversion<'a>(
    function: &FunctionInvocation,
    context: &Context<'a>,
) -> Result<Context<'a>, FHIRPathError> {
    validate_arguments(&function.arguments, &Cardinality::Zero)?;

    let name = function.name.0.as_str();
    let Some(value) = singleton(name, context)? else {
        return Ok(context.new_context_from(vec![]));
    };
    let (converts, target) = match name.strip_prefix("convertsTo") {
        Some(target) => (true, target),
        None => (false, &name["to".len()..]),
    };

    let converted = Convertible::of(value)?.and_then(|value| match target {
        "Boolean" => to_boolean(value).map(|value| boolean_result(context, value)),
        "Integer" => {
            to_integer(value).map(|value| number_result(context, Some(Number::Integer(value))))
        }
        "Decimal" => {
            to_decimal(value).map(|value| number_result(context, Some(Number::Decimal(value))))
        }
        "String" => to_string(value).map(|value| string_result(context, value)),
        "Date" => to_date(value).map(|value| {
            context.new_context_from(vec![context.allocate_literal(FHIRDate {
                value: Some(value),
                ..Default::default()
            })])
        }),
        "DateTime" => to_date_time(value).map(|value| {
            context.new_context_from(vec![context.allocate_literal(FHIRDateTime {
                value: Some(value),
                ..Default::default()
            })])
        }),
        "Time" => to_time(value).map(|value| {
            context.new_context_from(vec![context.allocate_literal(FHIRTime {
                value: Some(value),
                ..Default::default()
            })])
        }),
        _ => unreachable!(),
    });

    if converts {
        Ok(boolean_result(context, converted.is_some()))
    } else {
        Ok(converted.unwrap_or_else(|| context.new_context_from(vec![])))
    }
}

/// `now`, `timeOfDay` and `today`, per
/// <https://hl7.org/fhirpath/N1/#current-date-and-time-functions>.
fn evaluate_current_time<'a>(
    function: &FunctionInvocation,
    context: &Context<'a>,
) -> Result<Context<'a>, FHIRPathError> {
    validate_arguments(&function.arguments, &Cardinality::Zero)?;

    let now = context.now();
    let value = match function.name.0.as_str() {
        "now" => context.allocate_literal(FHIRDateTime {
            value: Some(DateTime::Iso8601(now)),
            ..Default::default()
        }),
        "timeOfDay" => context.allocate_literal(FHIRTime {
            value: Some(
                parse_time(&now.format("%H:%M:%S%.3f").to_string())
                    .map_err(|e| FHIRPathError::InternalError(format!("{e:?}")))?,
            ),
            ..Default::default()
        }),
        _ => context.allocate_literal(FHIRDate {
            value: Some(
                parse_date(&now.format("%Y-%m-%d").to_string())
                    .map_err(|e| FHIRPathError::InternalError(format!("{e:?}")))?,
            ),
            ..Default::default()
        }),
    };
    Ok(context.new_context_from(vec![value]))
}

/// <https://hl7.org/fhirpath/N1/#tracename-string-projection-expression-collection>:
/// logs at debug level and returns the input. The arguments are only evaluated
/// when that level is on.
async fn evaluate_trace<'a>(
    function: &FunctionInvocation,
    context: Context<'a>,
    config: Option<Arc<Config<'a>>>,
) -> Result<Context<'a>, FHIRPathError> {
    validate_arguments(&function.arguments, &Cardinality::Custom(1, 2))?;

    if tracing::enabled!(tracing::Level::DEBUG) {
        let name = evaluate_expression(&function.arguments[0], context.clone(), config.clone());
        let name = singleton("trace", &name.await?)?.map(downcast_string);
        let name = name.transpose()?.unwrap_or_default();

        let traced = match function.arguments.get(1) {
            Some(projection) => evaluate_expression(projection, context.clone(), config).await?,
            None => context.clone(),
        };
        tracing::debug!(name, values = ?traced.values, "FHIRPath trace");
    }

    Ok(context)
}

/// `is` as an operator or function, per <https://hl7.org/fhirpath/N1/#is-type-specifier>:
/// empty for an empty input and an error for more than one item.
fn is_type<'a>(input: &Context<'a>, type_name: &str) -> Result<Context<'a>, FHIRPathError> {
    let value = singleton("is", input)?;
    Ok(optional_boolean_result(
        input,
        value.map(|value| check_type(value, type_name)),
    ))
}

/// <https://hl7.org/fhirpath/N1/#istype-type-specifier>
fn evaluate_is_function<'a>(
    function: &FunctionInvocation,
    context: &Context<'a>,
) -> Result<Context<'a>, FHIRPathError> {
    validate_arguments(&function.arguments, &Cardinality::One)?;

    is_type(context, &derive_typename(&function.arguments[0])?)
}

fn expression_to_literal(expression: &Expression) -> Result<&Literal, FHIRPathError> {
    match expression {
        Expression::Singular(vec) if vec.len() == 1 => match &vec[0] {
            Term::Literal(lit) => Ok(lit),
            _ => Err(FHIRPathError::InternalError(
                "Expected a literal expression".to_string(),
            )),
        },
        _ => Err(FHIRPathError::InternalError(
            "Expected a singular expression".to_string(),
        )),
    }
}

/// The string functions that test their input against one string argument.
#[derive(Clone, Copy)]
enum StringMatch {
    StartsWith,
    EndsWith,
    Contains,
}

impl StringMatch {
    /// The function's name, as written in an expression.
    fn name(self) -> &'static str {
        match self {
            StringMatch::StartsWith => "startsWith",
            StringMatch::EndsWith => "endsWith",
            StringMatch::Contains => "contains",
        }
    }

    /// What the function calls its argument, for error messages.
    fn argument(self) -> &'static str {
        match self {
            StringMatch::StartsWith => "prefix",
            StringMatch::EndsWith => "suffix",
            StringMatch::Contains => "substring",
        }
    }

    fn matches(self, value: &str, argument: &str) -> bool {
        match self {
            StringMatch::StartsWith => value.starts_with(argument),
            StringMatch::EndsWith => value.ends_with(argument),
            StringMatch::Contains => value.contains(argument),
        }
    }
}

/// `startsWith`, `endsWith` and `contains`, per
/// <https://hl7.org/fhirpath/N1/#startswithprefix-string-boolean> and the two
/// that follow it: empty when the input is empty, an error when it has more
/// than one value.
fn evaluate_string_match<'a>(
    string_match: StringMatch,
    function: &FunctionInvocation,
    context: &Context<'a>,
) -> Result<Context<'a>, FHIRPathError> {
    let name = string_match.name();
    let argument = string_match.argument();

    if context.values.is_empty() {
        return Ok(context.new_context_from(vec![]));
    }

    if context.values.len() > 1 {
        return Err(FHIRPathError::InternalError(format!(
            "{name} function requires no more than one context value"
        )));
    }

    if function.arguments.is_empty() {
        return Ok(context.new_context_from(vec![]));
    }

    validate_arguments(&function.arguments, &Cardinality::Custom(1, 1))?;

    let literal = if let Some(arg) = function.arguments.first() {
        expression_to_literal(arg)?
    } else {
        return Err(FHIRPathError::InternalError(format!(
            "{name} function requires a {argument} argument"
        )));
    };

    let Literal::String(literal) = literal else {
        return Err(FHIRPathError::InternalError(format!(
            "{name} function requires a string {argument}"
        )));
    };

    let value_string = context
        .values
        .first()
        .and_then(|k| downcast_string(*k).ok())
        .unwrap_or_default();

    Ok(
        context.new_context_from(vec![context.allocate_literal(FHIRBoolean {
            value: Some(string_match.matches(&value_string, literal)),
            ..Default::default()
        })]),
    )
}

fn evaluate_get_reference_key<'a>(
    function: &FunctionInvocation,
    context: &Context<'a>,
) -> Result<Context<'a>, FHIRPathError> {
    validate_arguments(&function.arguments, &Cardinality::Custom(0, 1))?;

    let type_to_filter_by = if let Some(resource_type) = function.arguments.first() {
        Some(derive_typename(resource_type)?)
    } else {
        None
    };

    let ids = context
        .iter()
        .filter_map(|value| {
            let reference = value.as_any().downcast_ref::<Reference>()?;

            let mut pieces = reference
                .reference
                .as_ref()
                .and_then(|r| r.value.as_ref())
                .map(|r| r.split('/'))?;

            let resource_type = pieces.next()?;
            let id = pieces.next()?;

            if let Some(type_to_filter_by) = &type_to_filter_by
                && !check_type_name(resource_type, type_to_filter_by)
            {
                return None;
            }

            Some(FHIRId {
                value: Some(id.to_string()),
                ..Default::default()
            })
        })
        .collect::<Vec<_>>();

    let mut next_context = Vec::with_capacity(ids.len());

    for id in ids {
        next_context.push(context.allocate_literal(id));
    }

    Ok(context.new_context_from(next_context))
}

fn evaluate_get_resource_key<'a>(
    function: &FunctionInvocation,
    context: &Context<'a>,
    config: Option<Arc<Config<'a>>>,
) -> Result<Context<'a>, FHIRPathError> {
    validate_arguments(&function.arguments, &Cardinality::Zero)?;

    let Some(id) = config.and_then(|c| c.resource_id.clone()) else {
        return Err(FHIRPathError::InternalError(
            "getResourceKey function requires resource_id in config".to_string(),
        ));
    };

    let resource_key = FHIRId {
        value: Some(id),
        ..Default::default()
    };

    Ok(context.new_context_from(vec![context.allocate_literal(resource_key)]))
}

fn equal_check<'b>(left: &Context<'b>, right: &Context<'b>) -> Result<bool, FHIRPathError> {
    value_equal(left.values[0], right.values[0])
}

fn value_equal(left: &dyn MetaValue, right: &dyn MetaValue) -> Result<bool, FHIRPathError> {
    // An element equals itself. Other complex values are unequal until
    // member-wise equality is implemented.
    if std::ptr::addr_eq(left, right) && left.fhir_type() == right.fhir_type() {
        return Ok(true);
    }

    if NUMBER_TYPES.contains(left.fhir_type()) && NUMBER_TYPES.contains(right.fhir_type()) {
        let left_value = downcast_number(left)?;
        let right_value = downcast_number(right)?;
        #[allow(clippy::float_cmp)]
        Ok(left_value == right_value)
    } else if STRING_TYPES.contains(left.fhir_type()) && STRING_TYPES.contains(right.fhir_type()) {
        let left_value = downcast_string(left)?;
        let right_value = downcast_string(right)?;
        Ok(left_value == right_value)
    } else if BOOLEAN_TYPES.contains(left.fhir_type()) && BOOLEAN_TYPES.contains(right.fhir_type())
    {
        let left_value = downcast_bool(left)?;
        let right_value = downcast_bool(right)?;
        // Suppress clippy warning because strict equality comparison for float values
        // is intentional and required by the FHIRPath specification.
        #[allow(clippy::float_cmp)]
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

async fn evaluate_operation<'a>(
    operation: &Operation,
    context: Context<'a>,
    config: Option<Arc<Config<'a>>>,
) -> Result<Context<'a>, FHIRPathError> {
    match operation {
        Operation::Add(left, right) => evaluate_add(left, right, context, config).await,
        Operation::Subtraction(left, right) => {
            evaluate_subtraction(left, right, context, config).await
        }
        Operation::Multiplication(left, right) => {
            evaluate_multiplication(left, right, context, config).await
        }
        Operation::Division(left, right) => evaluate_division(left, right, context, config).await,
        Operation::Equal(left, right) => evaluate_equal(left, right, context, config).await,
        Operation::NotEqual(left, right) => evaluate_not_equal(left, right, context, config).await,
        Operation::And(left, right) => evaluate_and(left, right, context, config).await,
        Operation::Or(left, right) => evaluate_or(left, right, context, config).await,
        Operation::Union(left, right) => evaluate_union(left, right, context, config).await,
        Operation::Is(expr, ty) => evaluate_is(expr, ty, context, config).await,
        Operation::As(expr, ty) => evaluate_as(expr, ty, context, config).await,
        Operation::XOr(left, right) => evaluate_xor(left, right, context, config).await,

        Operation::Modulo(left, right) => {
            evaluate_truncated_division(left, right, context, config, true).await
        }
        Operation::Polarity(polarity, expression) => {
            evaluate_polarity(polarity, expression, context, config).await
        }
        Operation::DivisionTruncated(left, right) => {
            evaluate_truncated_division(left, right, context, config, false).await
        }
        Operation::Concatenate(left, right) => {
            evaluate_concatenate(left, right, context, config).await
        }
        Operation::LessThan(left, right) => evaluate_less_than(left, right, context, config).await,
        Operation::GreaterThan(left, right) => {
            evaluate_greater_than(left, right, context, config).await
        }
        Operation::LessThanEqual(left, right) => {
            evaluate_less_than_equal(left, right, context, config).await
        }
        Operation::GreaterThanEqual(left, right) => {
            evaluate_greater_than_equal(left, right, context, config).await
        }
        Operation::Equivalent(left, right) => {
            evaluate_equivalence(left, right, context, config, false).await
        }
        Operation::NotEquivalent(left, right) => {
            evaluate_equivalence(left, right, context, config, true).await
        }
        Operation::In(element, collection) | Operation::Contains(collection, element) => {
            evaluate_membership(element, collection, context, config).await
        }
        Operation::Implies(left, right) => evaluate_implies(left, right, context, config).await,
    }
}

async fn evaluate_add<'a>(
    left: &Expression,
    right: &Expression,
    context: Context<'a>,
    config: Option<Arc<Config<'a>>>,
) -> Result<Context<'a>, FHIRPathError> {
    operation_2(left, right, context, config, |left, right| {
        Box::pin(async move {
            if NUMBER_TYPES.contains(left.values[0].fhir_type())
                && NUMBER_TYPES.contains(right.values[0].fhir_type())
            {
                let left_value = downcast_number(left.values[0])?;
                let right_value = downcast_number(right.values[0])?;

                Ok(
                    left.new_context_from(vec![left.allocate_literal(FHIRDecimal {
                        value: Some(left_value + right_value),
                        ..Default::default()
                    })]),
                )
            } else if STRING_TYPES.contains(left.values[0].fhir_type())
                && STRING_TYPES.contains(right.values[0].fhir_type())
            {
                let left_string = downcast_string(left.values[0])?;
                let right_string = downcast_string(right.values[0])?;

                Ok(
                    left.new_context_from(vec![left.allocate_literal(FHIRString {
                        value: Some(left_string + &right_string),
                        ..Default::default()
                    })]),
                )
            } else {
                Err(FHIRPathError::OperationError(OperationError::TypeMismatch(
                    left.values[0].fhir_type(),
                    right.values[0].fhir_type(),
                )))
            }
        })
    })
    .await
}

async fn evaluate_numeric_binary<'a, F>(
    left: &Expression,
    right: &Expression,
    context: Context<'a>,
    config: Option<Arc<Config<'a>>>,
    op: F,
) -> Result<Context<'a>, FHIRPathError>
where
    F: FnOnce(f64, f64) -> f64 + Copy + Send + 'static,
{
    operation_2(left, right, context, config, move |left, right| {
        Box::pin(async move {
            let left_value = downcast_number(left.values[0])?;
            let right_value = downcast_number(right.values[0])?;

            Ok(
                left.new_context_from(vec![left.allocate_literal(FHIRDecimal {
                    value: Some(op(left_value, right_value)),
                    ..Default::default()
                })]),
            )
        })
    })
    .await
}

/// Ordered comparison of two numbers or two strings (code point order).
/// Empty when unordered (NaN); errors on any other operand types.
async fn evaluate_ordered_comparison<'a>(
    left: &Expression,
    right: &Expression,
    context: Context<'a>,
    config: Option<Arc<Config<'a>>>,
    accept: fn(Ordering) -> bool,
) -> Result<Context<'a>, FHIRPathError> {
    operation_2(left, right, context, config, move |left, right| {
        Box::pin(async move {
            let (l, r) = (left.values[0], right.values[0]);
            let ordering = if let (Ok(l), Ok(r)) = (downcast_number(l), downcast_number(r)) {
                l.partial_cmp(&r)
            } else if let (Ok(l), Ok(r)) = (downcast_string(l), downcast_string(r)) {
                Some(l.cmp(&r))
            } else {
                return Err(FHIRPathError::OperationError(OperationError::InvalidType(
                    l.fhir_type(),
                    r.fhir_type(),
                )));
            };

            Ok(left.new_context_from(
                ordering
                    .map(|o| {
                        left.allocate_literal(FHIRBoolean {
                            value: Some(accept(o)),
                            ..Default::default()
                        })
                    })
                    .into_iter()
                    .collect(),
            ))
        })
    })
    .await
}

async fn evaluate_subtraction<'a>(
    left: &Expression,
    right: &Expression,
    context: Context<'a>,
    config: Option<Arc<Config<'a>>>,
) -> Result<Context<'a>, FHIRPathError> {
    evaluate_numeric_binary(left, right, context, config, |l, r| l - r).await
}

async fn evaluate_multiplication<'a>(
    left: &Expression,
    right: &Expression,
    context: Context<'a>,
    config: Option<Arc<Config<'a>>>,
) -> Result<Context<'a>, FHIRPathError> {
    evaluate_numeric_binary(left, right, context, config, |l, r| l * r).await
}

async fn evaluate_division<'a>(
    left: &Expression,
    right: &Expression,
    context: Context<'a>,
    config: Option<Arc<Config<'a>>>,
) -> Result<Context<'a>, FHIRPathError> {
    evaluate_numeric_binary(left, right, context, config, |l, r| l / r).await
}

async fn evaluate_less_than<'a>(
    left: &Expression,
    right: &Expression,
    context: Context<'a>,
    config: Option<Arc<Config<'a>>>,
) -> Result<Context<'a>, FHIRPathError> {
    evaluate_ordered_comparison(left, right, context, config, Ordering::is_lt).await
}

async fn evaluate_less_than_equal<'a>(
    left: &Expression,
    right: &Expression,
    context: Context<'a>,
    config: Option<Arc<Config<'a>>>,
) -> Result<Context<'a>, FHIRPathError> {
    evaluate_ordered_comparison(left, right, context, config, Ordering::is_le).await
}

async fn evaluate_greater_than<'a>(
    left: &Expression,
    right: &Expression,
    context: Context<'a>,
    config: Option<Arc<Config<'a>>>,
) -> Result<Context<'a>, FHIRPathError> {
    evaluate_ordered_comparison(left, right, context, config, Ordering::is_gt).await
}

async fn evaluate_greater_than_equal<'a>(
    left: &Expression,
    right: &Expression,
    context: Context<'a>,
    config: Option<Arc<Config<'a>>>,
) -> Result<Context<'a>, FHIRPathError> {
    evaluate_ordered_comparison(left, right, context, config, Ordering::is_ge).await
}

async fn evaluate_boolean_binary<'a, F>(
    left: &Expression,
    right: &Expression,
    context: Context<'a>,
    config: Option<Arc<Config<'a>>>,
    op: F,
) -> Result<Context<'a>, FHIRPathError>
where
    F: FnOnce(bool, bool) -> bool + Copy + Send + 'static,
{
    operation_2(left, right, context, config, move |left, right| {
        Box::pin(async move {
            let l = downcast_bool(left.values[0])?;
            let r = downcast_bool(right.values[0])?;

            Ok(
                left.new_context_from(vec![left.allocate_literal(FHIRBoolean {
                    value: Some(op(l, r)),
                    ..Default::default()
                })]),
            )
        })
    })
    .await
}

async fn evaluate_and<'a>(
    left: &Expression,
    right: &Expression,
    context: Context<'a>,
    config: Option<Arc<Config<'a>>>,
) -> Result<Context<'a>, FHIRPathError> {
    evaluate_boolean_binary(left, right, context, config, |l, r| l && r).await
}

async fn evaluate_or<'a>(
    left: &Expression,
    right: &Expression,
    context: Context<'a>,
    config: Option<Arc<Config<'a>>>,
) -> Result<Context<'a>, FHIRPathError> {
    evaluate_boolean_binary(left, right, context, config, |l, r| l || r).await
}

async fn evaluate_xor<'a>(
    left: &Expression,
    right: &Expression,
    context: Context<'a>,
    config: Option<Arc<Config<'a>>>,
) -> Result<Context<'a>, FHIRPathError> {
    evaluate_boolean_binary(left, right, context, config, |l, r| l ^ r).await
}

async fn evaluate_equality<'a>(
    left: &Expression,
    right: &Expression,
    context: Context<'a>,
    config: Option<Arc<Config<'a>>>,
    negate: bool,
) -> Result<Context<'a>, FHIRPathError> {
    operation_2(left, right, context, config, move |left, right| {
        Box::pin(async move {
            let mut result = equal_check(&left, &right)?;

            if negate {
                result = !result;
            }

            Ok(
                left.new_context_from(vec![left.allocate_literal(FHIRBoolean {
                    value: Some(result),
                    ..Default::default()
                })]),
            )
        })
    })
    .await
}

async fn evaluate_equal<'a>(
    left: &Expression,
    right: &Expression,
    context: Context<'a>,
    config: Option<Arc<Config<'a>>>,
) -> Result<Context<'a>, FHIRPathError> {
    evaluate_equality(left, right, context, config, false).await
}

async fn evaluate_not_equal<'a>(
    left: &Expression,
    right: &Expression,
    context: Context<'a>,
    config: Option<Arc<Config<'a>>>,
) -> Result<Context<'a>, FHIRPathError> {
    evaluate_equality(left, right, context, config, true).await
}

/// Strings are equivalent ignoring case and whitespace differences; other
/// types follow `=`.
fn value_equivalent(left: &dyn MetaValue, right: &dyn MetaValue) -> Result<bool, FHIRPathError> {
    if STRING_TYPES.contains(left.fhir_type()) && STRING_TYPES.contains(right.fhir_type()) {
        let normalize = |value: &str| {
            value
                .split_whitespace()
                .map(str::to_lowercase)
                .collect::<Vec<_>>()
        };
        Ok(normalize(&downcast_string(left)?) == normalize(&downcast_string(right)?))
    } else {
        value_equal(left, right)
    }
}

/// <https://hl7.org/fhirpath/N1/#equivalent> and
/// <https://hl7.org/fhirpath/N1/#not-equivalent>. Order is ignored.
async fn evaluate_equivalence<'a>(
    left: &Expression,
    right: &Expression,
    context: Context<'a>,
    config: Option<Arc<Config<'a>>>,
    negate: bool,
) -> Result<Context<'a>, FHIRPathError> {
    operation_n(left, right, context, config, |left, right| {
        let mut equivalent = left.values.len() == right.values.len();
        if equivalent {
            let mut matched = vec![false; right.values.len()];
            'left: for item in &left.values {
                for (index, candidate) in right.values.iter().enumerate() {
                    if !matched[index] && value_equivalent(*item, *candidate)? {
                        matched[index] = true;
                        continue 'left;
                    }
                }
                equivalent = false;
                break;
            }
        }
        Ok(boolean_result(&left, equivalent != negate))
    })
    .await
}

async fn evaluate_union<'a>(
    left: &Expression,
    right: &Expression,
    context: Context<'a>,
    config: Option<Arc<Config<'a>>>,
) -> Result<Context<'a>, FHIRPathError> {
    // <https://hl7.org/fhirpath/N1/#union-collections> removes duplicates.
    operation_n(left, right, context, config, |left, right| {
        let union = distinct_values(left.values.iter().chain(right.values.iter()).copied())?;

        Ok(left.new_context_from(union))
    })
    .await
}

/// <https://hl7.org/fhirpath/N1/#div> and <https://hl7.org/fhirpath/N1/#mod>:
/// empty when dividing by 0.
async fn evaluate_truncated_division<'a>(
    left: &Expression,
    right: &Expression,
    context: Context<'a>,
    config: Option<Arc<Config<'a>>>,
    remainder: bool,
) -> Result<Context<'a>, FHIRPathError> {
    operation_2(left, right, context, config, move |left, right| {
        Box::pin(async move {
            let result = match (Number::of(left.values[0])?, Number::of(right.values[0])?) {
                (Number::Integer(dividend), Number::Integer(divisor)) => {
                    let result = if remainder {
                        dividend.checked_rem(divisor)
                    } else {
                        dividend.checked_div(divisor)
                    };
                    result.map(Number::Integer)
                }
                // Dividing by 0.0 gives NaN or infinity, which decimal_from turns into empty.
                (dividend, divisor) => {
                    let (dividend, divisor) = (dividend.as_f64(), divisor.as_f64());
                    Number::decimal_from(if remainder {
                        dividend % divisor
                    } else {
                        (dividend / divisor).trunc()
                    })
                }
            };

            Ok(number_result(&left, result))
        })
    })
    .await
}

/// Unary `+` and `-`, per <https://hl7.org/fhirpath/N1/#operator-precedence>.
async fn evaluate_polarity<'a>(
    polarity: &Polarity,
    expression: &Expression,
    context: Context<'a>,
    config: Option<Arc<Config<'a>>>,
) -> Result<Context<'a>, FHIRPathError> {
    let operand = evaluate_expression(expression, context, config).await?;
    let Some(value) = singleton("polarity", &operand)? else {
        return Ok(operand);
    };

    let result = match (polarity, Number::of(value)?) {
        (Polarity::Positive, number) => Some(number),
        (Polarity::Negative, Number::Integer(value)) => value.checked_neg().map(Number::Integer),
        (Polarity::Negative, Number::Decimal(value)) => Some(Number::Decimal(-value)),
    };

    Ok(number_result(&operand, result))
}

/// <https://hl7.org/fhirpath/N1/#string-concatenation>: an empty operand
/// counts as `''`.
async fn evaluate_concatenate<'a>(
    left: &Expression,
    right: &Expression,
    context: Context<'a>,
    config: Option<Arc<Config<'a>>>,
) -> Result<Context<'a>, FHIRPathError> {
    operation_n(left, right, context, config, |left, right| {
        let mut concatenated = String::new();
        for operand in [&left, &right] {
            if let Some(value) = singleton("&", operand)? {
                concatenated.push_str(&downcast_string(value)?);
            }
        }

        Ok(string_result(&left, concatenated))
    })
    .await
}

/// `in` and `contains`, per <https://hl7.org/fhirpath/N1/#membership>: empty
/// for an empty element, false for an empty collection.
async fn evaluate_membership<'a>(
    element: &Expression,
    collection: &Expression,
    context: Context<'a>,
    config: Option<Arc<Config<'a>>>,
) -> Result<Context<'a>, FHIRPathError> {
    operation_n(
        element,
        collection,
        context,
        config,
        |element, collection| {
            let Some(value) = singleton("membership", &element)? else {
                return Ok(element);
            };

            let contained = contains_value(&collection.values, value)?;
            Ok(boolean_result(&element, contained))
        },
    )
    .await
}

/// <https://hl7.org/fhirpath/N1/#implies>
async fn evaluate_implies<'a>(
    left: &Expression,
    right: &Expression,
    context: Context<'a>,
    config: Option<Arc<Config<'a>>>,
) -> Result<Context<'a>, FHIRPathError> {
    let left = evaluate_expression(left, context.clone(), config.clone()).await?;
    let left = singleton_boolean("implies", &left)?;

    // A false left operand makes the right one irrelevant.
    if left == Some(false) {
        return Ok(boolean_result(&context, true));
    }

    let right = evaluate_expression(right, context.clone(), config).await?;
    let right = singleton_boolean("implies", &right)?;

    let result = match (left, right) {
        (Some(_), right) => right,
        (None, Some(true)) => Some(true),
        (None, _) => None,
    };

    Ok(optional_boolean_result(&context, result))
}

async fn evaluate_type_operation<'a>(
    expression: &Expression,
    type_name: &QualifiedIdentifier,
    context: Context<'a>,
    config: Option<Arc<Config<'a>>>,
    return_context: bool,
) -> Result<Context<'a>, FHIRPathError> {
    let left = evaluate_expression(expression, context, config).await?;

    // The last part names the type, as in FHIR.Patient.
    let Some(Identifier(type_name)) = type_name.0.last() else {
        return Ok(left.new_context_from(vec![]));
    };

    if return_context {
        // <https://hl7.org/fhirpath/N1/#as-type-specifier>
        let value = singleton("as", &left)?.filter(|value| check_type(*value, type_name));
        Ok(left.new_context_from(value.into_iter().collect()))
    } else {
        is_type(&left, type_name)
    }
}

async fn evaluate_is<'a>(
    expression: &Expression,
    type_name: &QualifiedIdentifier,
    context: Context<'a>,
    config: Option<Arc<Config<'a>>>,
) -> Result<Context<'a>, FHIRPathError> {
    evaluate_type_operation(expression, type_name, context, config, false).await
}

async fn evaluate_as<'a>(
    expression: &Expression,
    type_name: &QualifiedIdentifier,
    context: Context<'a>,
    config: Option<Arc<Config<'a>>>,
) -> Result<Context<'a>, FHIRPathError> {
    evaluate_type_operation(expression, type_name, context, config, true).await
}

fn evaluate_expression<'a>(
    ast: &Expression,
    context: Context<'a>,
    config: Option<Arc<Config<'a>>>,
) -> Pin<Box<impl Future<Output = Result<Context<'a>, FHIRPathError>>>> {
    Box::pin(async move {
        match ast {
            Expression::Operation(operation) => {
                evaluate_operation(operation, context, config).await
            }
            Expression::Singular(singular_ast) => {
                evaluate_singular(singular_ast, context, config).await
            }
        }
    })
}

#[derive(Debug)]
pub enum ResolvedValue {
    Box(Box<dyn MetaValue>),
    Arc(Arc<dyn MetaValue>),
}

impl ResolvedValue {
    #[must_use]
    pub fn as_meta_value(&self) -> &dyn MetaValue {
        match self {
            ResolvedValue::Box(b) => &**b,
            ResolvedValue::Arc(a) => &**a,
        }
    }
}

pub struct Context<'a> {
    allocator: Arc<Mutex<allocators::bumpalo::Allocator>>,
    values: Vec<&'a dyn MetaValue>,
}

pub type ResolvedValueFuture = Pin<Box<dyn Future<Output = Option<ResolvedValue>> + Send>>;
pub type ResolveValueCallback = Box<dyn Fn(String) -> ResolvedValueFuture + Send + Sync>;

pub enum ExternalConstantResolver<'a> {
    Function(ResolveValueCallback),
    Variable(Arc<HashMap<String, &'a dyn MetaValue>>),
}

#[derive(Default)]
pub struct Config<'a> {
    // Used to resolve resource_id in getResourceKey for sql-on-fhir.
    resource_id: Option<String>,
    variable_resolver: Option<ExternalConstantResolver<'a>>,
}

impl<'a> Config<'a> {
    #[must_use]
    pub fn builder() -> Self {
        Config::default()
    }

    #[must_use]
    pub fn with_variable_resolver(mut self, resolver: ExternalConstantResolver<'a>) -> Self {
        self.variable_resolver = Some(resolver);
        self
    }

    #[must_use]
    pub fn with_resource_id(mut self, resource_id: String) -> Self {
        self.resource_id = Some(resource_id);
        self
    }
}

async fn resolve_external_constant<'a>(
    name: &str,
    resolver: Option<&ExternalConstantResolver<'a>>,
    context: Context<'a>,
) -> Result<Context<'a>, FHIRPathError> {
    let external_constant = match resolver {
        Some(ExternalConstantResolver::Function(func)) => {
            let result = func(name.to_string()).await;

            result.map(|result| context.allocate(result))
        }
        Some(ExternalConstantResolver::Variable(map)) => map.get(name).copied(),
        None => None,
    };

    if let Some(result) = external_constant {
        return Ok(context.new_context_from(vec![result]));
    }
    Ok(context.new_context_from(vec![]))
}

impl<'a> IntoIterator for &'a Context<'_> {
    type Item = &'a dyn haste_reflect::MetaValue;

    type IntoIter = std::boxed::Box<
        dyn std::iter::Iterator<Item = &'a (dyn haste_reflect::MetaValue + 'static)> + 'a,
    >;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a> Context<'a> {
    fn new(
        values: Vec<&'a dyn MetaValue>,
        allocator: Arc<Mutex<allocators::bumpalo::Allocator>>,
    ) -> Self {
        Self { allocator, values }
    }
    fn new_context_from(&self, values: Vec<&'a dyn MetaValue>) -> Self {
        Self {
            allocator: self.allocator.clone(),
            values,
        }
    }
    fn allocate(&self, value: ResolvedValue) -> &'a dyn MetaValue {
        self.allocator
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .allocate_resolved(value)
    }

    fn now(&self) -> chrono::DateTime<chrono::Utc> {
        self.allocator
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .now()
    }

    fn allocate_literal<T: MetaValue>(&self, value: T) -> &'a dyn MetaValue {
        self.allocator
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .allocate_literal(value)
    }

    #[must_use]
    pub fn iter(&'a self) -> Box<dyn Iterator<Item = &'a dyn MetaValue> + 'a> {
        Box::new(self.values.iter().copied())
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

pub struct FPEngine {}

static AST: LazyLock<DashMap<String, Arc<Expression>>> = LazyLock::new(DashMap::new);

fn get_ast(path: &str) -> Result<Arc<Expression>, FHIRPathError> {
    if let Some(expression_ast) = AST.get(path) {
        return Ok(expression_ast.clone());
    }

    let expression_ast = Arc::new(parser::parse(path)?);
    AST.insert(path.to_string(), expression_ast.clone());

    Ok(expression_ast)
}

impl Default for FPEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl FPEngine {
    #[must_use]
    pub fn new() -> Self {
        Self {}
    }

    /// Evaluate a `FHIRPath` expression against a context.
    /// The context is a vector of references to `MetaValue` objects.
    /// The path is a `FHIRPath` expression.
    /// The result is a vector of references to `MetaValue` objects.
    ///
    /// # Errors
    ///
    /// Returns a [`FHIRPathError`] if the expression parsing fails (bad AST)
    /// or if the evaluation engine encounters an error during execution.
    pub async fn evaluate<'a, 'b>(
        &self,
        path: &str,
        values: Vec<&'a dyn MetaValue>,
    ) -> Result<Context<'b>, FHIRPathError>
    where
        'a: 'b,
    {
        let ast = get_ast(path)?;

        // Store created.
        let allocator: Arc<Mutex<allocators::bumpalo::Allocator>> =
            Arc::new(Mutex::new(allocators::bumpalo::Allocator::new()));

        let context = Context::new(values, allocator.clone());

        let result = evaluate_expression(&ast, context, None).await?;
        Ok(result)
    }

    /// Evaluate a `FHIRPath` expression against a context.
    /// The context is a vector of references to `MetaValue` objects.
    /// The path is a `FHIRPath` expression.
    /// The result is a vector of references to `MetaValue` objects.
    ///
    /// # Errors
    ///
    /// Returns a [`FHIRPathError`] if the expression parsing fails (bad AST)
    /// or if the evaluation engine encounters an error during execution.
    pub async fn evaluate_with_config<'a, 'b>(
        &self,
        path: &str,
        values: Vec<&'a dyn MetaValue>,
        config: Arc<Config<'b>>,
    ) -> Result<Context<'b>, FHIRPathError>
    where
        'a: 'b,
    {
        let ast = get_ast(path)?;

        // Store created.
        let allocator = Arc::new(Mutex::new(allocators::bumpalo::Allocator::new()));

        let context = Context::new(values, allocator.clone());

        let result = evaluate_expression(&ast, context, Some(config)).await?;

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use haste_fhir_model::r4::{
        datetime::DateTime,
        generated::{
            resources::{
                Bundle, Group, GroupMember, Patient, PatientDeceasedTypeChoice, PatientLink,
                Resource, SearchParameter,
            },
            types::{
                Extension, ExtensionValueTypeChoice, FHIRDateTime, FHIRString, FHIRUri, HumanName,
                Identifier, Reference,
            },
        },
    };

    use haste_reflect_derive::Reflect;

    #[derive(Reflect, Debug)]
    #[fhir_type = "BackboneElement"]
    struct C {
        c: String,
    }

    #[derive(Reflect, Debug)]
    #[fhir_type = "BackboneElement"]
    struct B {
        b: Vec<C>,
    }

    #[derive(Reflect, Debug)]
    #[fhir_type = "BackboneElement"]
    struct A {
        a: Vec<B>,
    }

    fn load_search_parameters() -> Vec<SearchParameter> {
        let json = include_str!(
            "../../../../artifacts/r4/hl7-core/definitions/hl7/search-parameters.min.json"
        );
        let bundle = serde_json::from_str::<Bundle>(json).unwrap();

        bundle
            .entry
            .unwrap_or_default()
            .into_iter()
            .filter_map(|e| e.resource)
            .filter_map(|e| match *e {
                Resource::SearchParameter(sp) => Some(sp),
                _ => None,
            })
            .collect()
    }

    #[tokio::test]
    async fn filter_typechoice_test() {
        let patient = Patient {
            id: Some("patient-id".to_string()),
            deceased: Some(PatientDeceasedTypeChoice::Boolean(Box::new(FHIRBoolean {
                value: Some(true),
                ..Default::default()
            }))),
            ..Default::default()
        };

        let engine = FPEngine::new();
        let result = engine
            .evaluate("(Patient.deceased.ofType(dateTime))", vec![&patient])
            .await
            .unwrap();

        assert_eq!(result.values.len(), 0);

        let result = engine
            .evaluate("(Patient.deceased.ofType(boolean))", vec![&patient])
            .await
            .unwrap();

        let value = result.values[0];
        let boolean_value: &FHIRBoolean = value
            .as_any()
            .downcast_ref::<FHIRBoolean>()
            .expect("Failed to downcast to FHIRBoolean");

        assert_eq!(boolean_value.value, Some(true));

        let patient = Patient {
            id: Some("patient-id".to_string()),
            deceased: Some(PatientDeceasedTypeChoice::DateTime(Box::new(
                FHIRDateTime {
                    value: Some(DateTime::Year(1980)),
                    ..Default::default()
                },
            ))),
            ..Default::default()
        };

        let result = engine
            .evaluate("(Patient.deceased.ofType(boolean))", vec![&patient])
            .await
            .unwrap();

        assert_eq!(result.values.len(), 0);

        let result = engine
            .evaluate("(Patient.deceased.ofType(dateTime))", vec![&patient])
            .await
            .unwrap();

        assert_eq!(result.values.len(), 1);

        let value = result.values[0];
        let datetime_value: &FHIRDateTime = value
            .as_any()
            .downcast_ref::<FHIRDateTime>()
            .expect("Failed to downcast to FHIRDateTime");

        assert_eq!(datetime_value.value, Some(DateTime::Year(1980)));
    }

    #[tokio::test]
    async fn test_current_date_and_time() {
        let engine = FPEngine::new();
        let result = engine
            .evaluate(
                "now().toString().substring(0, 10) = today().toString() \
                 and now().toString().substring(11, 8) = timeOfDay().toString().substring(0, 8) \
                 and today().toString() = today().toString()",
                vec![],
            )
            .await
            .unwrap();
        assert_eq!(result.values.len(), 1);
        assert!(downcast_bool(result.values[0]).unwrap());

        for name in ["now", "today", "timeOfDay"] {
            assert!(
                engine
                    .evaluate(&format!("{name}(1)"), vec![])
                    .await
                    .is_err()
            );
        }
    }

    #[tokio::test]
    async fn test_variable_resolution() {
        let engine = FPEngine::new();
        let patient = Patient {
            id: Some("my-patient".to_string()),
            ..Default::default()
        };
        let config = Arc::new(
            Config::builder().with_variable_resolver(ExternalConstantResolver::Variable(Arc::new(
                vec![("patient".to_string(), &patient as &dyn MetaValue)]
                    .into_iter()
                    .collect(),
            ))),
        );

        let result = engine
            .evaluate_with_config("%patient", vec![], config.clone())
            .await
            .unwrap();

        assert_eq!(result.values.len(), 1);
        let p = result.values[0].as_any().downcast_ref::<Patient>().unwrap();

        assert_eq!(p.id, patient.id);

        let result_failed = engine
            .evaluate_with_config("%nobody", vec![], config)
            .await
            .unwrap();

        assert_eq!(result_failed.values.len(), 0);
    }

    #[tokio::test]
    async fn test_where_clause() {
        let engine = FPEngine::new();
        let mut patient = Patient::default();
        let mut identifier = Identifier::default();
        let extension = Extension {
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
            extension: Some(vec![extension]),
            value: Some("12345".to_string()),
        }));
        patient.identifier_ = Some(vec![identifier]);

        let context = engine
            .evaluate(
                "$this.identifier.value.where($this.extension.value.exists())",
                vec![&patient],
            )
            .await;

        assert_eq!(context.unwrap().values.len(), 1);

        let context = engine
            .evaluate(
                "$this.identifier.value.where($this.extension.extension.exists())",
                vec![&patient],
            )
            .await;
        assert_eq!(context.unwrap().values.len(), 0);
    }

    #[tokio::test]
    async fn test_all_parameters() {
        let search_parameters = load_search_parameters();
        for param in &search_parameters {
            if let Some(expression) = &param.expression {
                let engine = FPEngine::new();
                let context = engine
                    .evaluate(expression.value.as_ref().unwrap().as_str(), vec![])
                    .await;

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
        let name = HumanName {
            given: Some(vec![FHIRString {
                id: None,
                extension: None,
                value: Some("Bob".to_string()),
            }]),
            ..Default::default()
        };
        let mrn_identifier = Identifier {
            value: Some(Box::new(FHIRString {
                id: None,
                extension: None,
                value: Some("mrn-12345".to_string()),
            })),
            system: Some(Box::new(FHIRUri {
                id: None,
                extension: None,
                value: Some("mrn".to_string()),
            })),
            ..Default::default()
        };
        let ssn_identifier = Identifier {
            value: Some(Box::new(FHIRString {
                id: None,
                extension: None,
                value: Some("ssn-12345".to_string()),
            })),
            system: Some(Box::new(FHIRUri {
                id: None,
                extension: None,
                value: Some("ssn".to_string()),
            })),
            ..Default::default()
        };

        patient.identifier_ = Some(vec![mrn_identifier, ssn_identifier]);
        patient.name = Some(vec![name]);
        patient
    }

    #[tokio::test]
    async fn indexing_tests() {
        let engine = FPEngine::new();
        let patient = test_patient();

        let given_name = engine
            .evaluate("$this.name.given[0]", vec![&patient])
            .await
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
            .await
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

        let all_identifiers = engine
            .evaluate("$this.identifier", vec![&patient])
            .await
            .unwrap();
        assert_eq!(all_identifiers.values.len(), 2);
    }

    #[tokio::test]
    async fn where_testing() {
        let engine = FPEngine::new();
        let patient = test_patient();

        let name_where_clause = engine
            .evaluate(
                "$this.name.given.where($this.value = 'Bob')",
                vec![&patient],
            )
            .await
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
            .await
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

    #[tokio::test]
    async fn test_equality() {
        let engine = FPEngine::new();

        // String tests
        let string_equal = engine.evaluate("'test' = 'test'", vec![]).await.unwrap();
        for r in &string_equal {
            let b = r
                .as_any()
                .downcast_ref::<FHIRBoolean>()
                .unwrap()
                .value
                .unwrap();
            assert!(b);
        }
        let string_unequal = engine.evaluate("'invalid' = 'test'", vec![]).await.unwrap();
        for r in &string_unequal {
            let b = r
                .as_any()
                .downcast_ref::<FHIRBoolean>()
                .unwrap()
                .value
                .unwrap();
            assert!(!b);
        }

        // Number tests
        let number_equal = engine.evaluate("12 = 12", vec![]).await.unwrap();
        for r in &number_equal {
            let b = r
                .as_any()
                .downcast_ref::<FHIRBoolean>()
                .unwrap()
                .value
                .unwrap();
            assert!(b);
        }
        let number_unequal = engine.evaluate("13 = 12", vec![]).await.unwrap();
        for r in &number_unequal {
            let b = r
                .as_any()
                .downcast_ref::<FHIRBoolean>()
                .unwrap()
                .value
                .unwrap();
            assert!(!b);
        }

        // Boolean tests
        let bool_equal = engine.evaluate("false = false", vec![]).await.unwrap();
        for r in &bool_equal {
            let b = r
                .as_any()
                .downcast_ref::<FHIRBoolean>()
                .unwrap()
                .value
                .unwrap();
            assert!(b);
        }
        let bool_unequal = engine.evaluate("false = true", vec![]).await.unwrap();
        for r in &bool_unequal {
            let b = r
                .as_any()
                .downcast_ref::<FHIRBoolean>()
                .unwrap()
                .value
                .unwrap();
            assert!(!b);
        }

        // Nested Equality tests
        let bool_equal = engine.evaluate("12 = 13 = false", vec![]).await.unwrap();
        for r in &bool_equal {
            let b = r
                .as_any()
                .downcast_ref::<FHIRBoolean>()
                .unwrap()
                .value
                .unwrap();
            assert!(b);
        }
        let bool_unequal = engine.evaluate("12 = 13 = true", vec![]).await.unwrap();
        for r in &bool_unequal {
            let b = r
                .as_any()
                .downcast_ref::<FHIRBoolean>()
                .unwrap()
                .value
                .unwrap();
            assert!(!b);
        }
        let bool_unequal = engine.evaluate("12 = (13 - 1)", vec![]).await.unwrap();
        for r in &bool_unequal {
            let b = r
                .as_any()
                .downcast_ref::<FHIRBoolean>()
                .unwrap()
                .value
                .unwrap();
            assert!(b);
        }
    }

    #[tokio::test]
    async fn test_string_concat() {
        let engine = FPEngine::new();
        let patient = test_patient();

        let simple_result = engine.evaluate("'Hello' + ' World'", vec![]).await.unwrap();
        for r in &simple_result {
            let s = r.as_any().downcast_ref::<FHIRString>().unwrap().clone();
            assert_eq!(s.value, Some("Hello World".to_string()));
        }

        let simple_result = engine
            .evaluate("$this.name.given + ' Miller'", vec![&patient])
            .await
            .unwrap();
        for r in &simple_result {
            let s = r.as_any().downcast_ref::<FHIRString>().unwrap().clone();
            assert_eq!(s.value, Some("Bob Miller".to_string()));
        }
    }

    #[tokio::test]
    async fn test_simple() {
        let root = A {
            a: vec![B {
                b: vec![C {
                    c: "whatever".to_string(),
                }],
            }],
        };

        let engine = FPEngine::new();
        let result = engine.evaluate("a.b.c", vec![&root]).await.unwrap();

        let strings: Vec<&String> = result
            .iter()
            .map(|r| r.as_any().downcast_ref::<String>().unwrap())
            .collect();

        assert_eq!(strings, vec!["whatever"]);
    }

    #[tokio::test]
    async fn allocation() {
        let engine = FPEngine::new();
        let result = engine.evaluate("'asdf'", vec![]).await.unwrap();

        for r in &result {
            let s = r.as_any().downcast_ref::<FHIRString>().unwrap().clone();

            assert_eq!(s.value, Some("asdf".to_string()));
        }
    }

    #[tokio::test]
    async fn order_operation() {
        let engine = FPEngine::new();
        let result = engine.evaluate("45 + 2  * 3", vec![]).await.unwrap();

        for r in &result {
            let s = r.as_any().downcast_ref::<FHIRDecimal>().unwrap().clone();

            assert_eq!(s.value, Some(51.0));
        }
    }

    #[tokio::test]
    async fn xor_operation() {
        let engine = FPEngine::new();
        let result = engine.evaluate("true xor true", vec![]).await.unwrap();

        for r in &result {
            let b = r
                .as_any()
                .downcast_ref::<FHIRBoolean>()
                .unwrap()
                .value
                .unwrap();

            assert!(!b);
        }
    }

    #[tokio::test]
    async fn domain_resource_filter() {
        let engine = FPEngine::new();

        let patient =
            serde_json::from_str::<Resource>(r#"{"id": "patient-id", "resourceType": "Patient"}"#)
                .unwrap();
        let result = engine
            .evaluate("Resource.id", vec![&patient])
            .await
            .unwrap();
        let ids: Vec<&String> = result
            .iter()
            .map(|r| r.as_any().downcast_ref::<String>().unwrap())
            .collect();

        assert_eq!(ids.len(), 1);
        assert_eq!(ids[0], "patient-id");

        let result2 = engine
            .evaluate("DomainResource.id", vec![&patient])
            .await
            .unwrap();
        let ids2: Vec<&String> = result2
            .iter()
            .map(|r| r.as_any().downcast_ref::<String>().unwrap())
            .collect();
        assert_eq!(ids2.len(), 1);
        assert_eq!(ids2[0], "patient-id");
    }

    #[tokio::test]
    async fn type_test() {
        let engine = FPEngine::new();
        let patient = Patient::default();

        let result = engine
            .evaluate("$this.type().name", vec![&patient])
            .await
            .unwrap();
        let ids: Vec<&String> = result
            .iter()
            .map(|r| r.as_any().downcast_ref::<String>().unwrap())
            .collect();

        assert_eq!(ids.len(), 1);
        assert_eq!(ids[0], "Patient");
    }

    #[tokio::test]
    async fn resolve_test() {
        let engine = FPEngine::new();
        let observation = serde_json::from_str::<Resource>(r#"
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
            .await
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

    #[tokio::test]
    async fn children_test() {
        let engine = FPEngine::new();
        let patient = Patient {
            name: Some(vec![HumanName {
                given: Some(vec![FHIRString {
                    value: Some("Alice".to_string()),
                    ..Default::default()
                }]),
                ..Default::default()
            }]),
            deceased: Some(PatientDeceasedTypeChoice::Boolean(Box::new(FHIRBoolean {
                value: Some(true),
                ..Default::default()
            }))),
            ..Default::default()
        };

        let result = engine
            .evaluate("$this.children()", vec![&patient])
            .await
            .unwrap();

        assert_eq!(result.values.len(), 2);
        assert_eq!(
            result
                .values
                .iter()
                .map(|v| v.fhir_type())
                .collect::<Vec<_>>(),
            vec!["HumanName", "boolean"]
        );
    }

    #[tokio::test]
    async fn repeat_test() {
        let engine = FPEngine::new();
        let patient = Patient {
            name: Some(vec![HumanName {
                given: Some(vec![FHIRString {
                    value: Some("Alice".to_string()),
                    ..Default::default()
                }]),
                ..Default::default()
            }]),
            deceased: Some(PatientDeceasedTypeChoice::Boolean(Box::new(FHIRBoolean {
                value: Some(true),
                ..Default::default()
            }))),
            ..Default::default()
        };

        let result = engine
            .evaluate("$this.name.given", vec![&patient])
            .await
            .unwrap();

        assert_eq!(result.values.len(), 1);

        assert_eq!(result.values[0].fhir_type(), "string");

        let result = engine
            .evaluate("$this.repeat(children())", vec![&patient])
            .await
            .unwrap();

        assert_eq!(
            result
                .values
                .iter()
                .map(|v| v.fhir_type())
                .collect::<Vec<_>>(),
            vec![
                "HumanName",
                "boolean",
                "string",
                "http://hl7.org/fhirpath/System.Boolean",
                "http://hl7.org/fhirpath/System.String"
            ]
        );
    }
    #[tokio::test]
    async fn descendants_test() {
        let engine = FPEngine::new();
        let patient = Patient {
            name: Some(vec![HumanName {
                given: Some(vec![FHIRString {
                    value: Some("Alice".to_string()),
                    ..Default::default()
                }]),
                ..Default::default()
            }]),
            deceased: Some(PatientDeceasedTypeChoice::Boolean(Box::new(FHIRBoolean {
                value: Some(true),
                ..Default::default()
            }))),
            ..Default::default()
        };
        let result = engine
            .evaluate("descendants()", vec![&patient])
            .await
            .unwrap();

        assert_eq!(
            result
                .values
                .iter()
                .map(|v| v.fhir_type())
                .collect::<Vec<_>>(),
            vec![
                "HumanName",
                "boolean",
                "string",
                "http://hl7.org/fhirpath/System.Boolean",
                "http://hl7.org/fhirpath/System.String"
            ]
        );
    }

    #[tokio::test]
    async fn descendants_test_filter() {
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
            name: Some(vec![HumanName {
                given: Some(vec![FHIRString {
                    value: Some("Alice".to_string()),
                    ..Default::default()
                }]),
                ..Default::default()
            }]),
            deceased: Some(PatientDeceasedTypeChoice::Boolean(Box::new(FHIRBoolean {
                value: Some(true),
                ..Default::default()
            }))),
            ..Default::default()
        };
        let result = engine
            .evaluate("descendants()", vec![&patient])
            .await
            .unwrap();

        assert_eq!(
            result
                .values
                .iter()
                .map(|v| v.fhir_type())
                .collect::<Vec<_>>(),
            vec![
                "HumanName",
                "boolean",
                "BackboneElement",
                "string",
                "http://hl7.org/fhirpath/System.Boolean",
                "Reference",
                "http://hl7.org/fhirpath/System.String",
                "string",
                "http://hl7.org/fhirpath/System.String"
            ]
        );

        let result = engine
            .evaluate("descendants().ofType(Reference)", vec![&patient])
            .await
            .unwrap();

        assert_eq!(
            result
                .values
                .iter()
                .map(|v| v.fhir_type())
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

    #[tokio::test]
    async fn try_unsafe_set_from_ref() {
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
            name: Some(vec![HumanName {
                given: Some(vec![FHIRString {
                    value: Some("Alice".to_string()),
                    ..Default::default()
                }]),
                ..Default::default()
            }]),
            deceased: Some(PatientDeceasedTypeChoice::Boolean(Box::new(FHIRBoolean {
                value: Some(true),
                ..Default::default()
            }))),
            ..Default::default()
        };

        let result = engine
            .evaluate("descendants().ofType(Reference)", vec![&patient])
            .await
            .unwrap();

        assert_eq!(
            result
                .values
                .iter()
                .map(|v| v.fhir_type())
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
            let r = std::ptr::from_ref::<Reference>(value);
            let mut_ptr = r.cast_mut();

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

    #[tokio::test]
    async fn test_external_constant_function() {
        let engine = FPEngine::new();

        let config = Arc::new(Config::builder().with_variable_resolver(
            ExternalConstantResolver::Function(Box::new(|v| {
                Box::pin(async move {
                    match v.as_ref() {
                        "test_variable" => Some(ResolvedValue::Box(Box::new(Patient {
                            name: Some(vec![HumanName {
                                given: Some(vec![FHIRString {
                                    value: Some("Paul".to_string()),
                                    ..Default::default()
                                }]),
                                ..Default::default()
                            }]),
                            ..Default::default()
                        })
                            as Box<dyn MetaValue>)),
                        _ => None,
                    }
                })
            })),
        ));

        let result = engine
            .evaluate_with_config("%test_variable.name.given", vec![], config)
            .await
            .unwrap();

        let value = result.values[0]
            .as_any()
            .downcast_ref::<FHIRString>()
            .unwrap();

        assert_eq!(value.value.as_ref(), Some(&"Paul".to_string()));
    }

    #[tokio::test]
    async fn test_external_constant_function_reference() {
        let engine = FPEngine::new();

        let patient = Arc::new(Patient {
            name: Some(vec![HumanName {
                given: Some(vec![FHIRString {
                    value: Some("Paul".to_string()),
                    ..Default::default()
                }]),
                ..Default::default()
            }]),
            ..Default::default()
        });

        let resolver = {
            let patient = patient.clone();
            ExternalConstantResolver::Function(Box::new(move |v| {
                let patient = patient.clone();
                Box::pin(async move {
                    // let patient = patient.clone();
                    match v.as_ref() {
                        "test_variable" => Some(ResolvedValue::Arc(patient.clone())),
                        _ => None,
                    }
                })
            }))
        };

        let config = Arc::new(Config::builder().with_variable_resolver(resolver));

        let result = engine
            .evaluate_with_config("%test_variable.name.given", vec![], config)
            .await
            .unwrap();

        let value = result.values[0]
            .as_any()
            .downcast_ref::<FHIRString>()
            .unwrap();

        assert_eq!(value.value.as_ref(), Some(&"Paul".to_string()));
    }

    #[tokio::test]
    async fn test_upper_function() {
        let engine = FPEngine::new();

        let result = engine.evaluate("'hello'.upper()", vec![]).await.unwrap();
        assert_eq!(result.values.len(), 1);
        let value = result.values[0]
            .as_any()
            .downcast_ref::<FHIRString>()
            .unwrap();
        assert_eq!(value.value.as_deref(), Some("HELLO"));

        let result = engine.evaluate("'AbCd'.upper()", vec![]).await.unwrap();
        let value = result.values[0]
            .as_any()
            .downcast_ref::<FHIRString>()
            .unwrap();
        assert_eq!(value.value.as_deref(), Some("ABCD"));

        let result = engine.evaluate("'XYZ'.upper()", vec![]).await.unwrap();
        let value = result.values[0]
            .as_any()
            .downcast_ref::<FHIRString>()
            .unwrap();
        assert_eq!(value.value.as_deref(), Some("XYZ"));
    }

    #[tokio::test]
    async fn test_lower_function() {
        let engine = FPEngine::new();

        let result = engine.evaluate("'HELLO'.lower()", vec![]).await.unwrap();
        assert_eq!(result.values.len(), 1);
        let value = result.values[0]
            .as_any()
            .downcast_ref::<FHIRString>()
            .unwrap();
        assert_eq!(value.value.as_deref(), Some("hello"));

        let result = engine.evaluate("'AbCd'.lower()", vec![]).await.unwrap();
        let value = result.values[0]
            .as_any()
            .downcast_ref::<FHIRString>()
            .unwrap();
        assert_eq!(value.value.as_deref(), Some("abcd"));

        let result = engine.evaluate("'xyz'.lower()", vec![]).await.unwrap();
        let value = result.values[0]
            .as_any()
            .downcast_ref::<FHIRString>()
            .unwrap();
        assert_eq!(value.value.as_deref(), Some("xyz"));
    }

    #[tokio::test]
    async fn get_resource_key() {
        let engine = FPEngine::new();
        let fp_config = Config::builder().with_resource_id("asdf".to_string());

        let result = engine
            .evaluate_with_config("getResourceKey()", vec![], Arc::new(fp_config))
            .await
            .unwrap();

        let k = result.iter().collect::<Vec<_>>();

        assert_eq!(k.len(), 1);

        let s = k[0].as_any().downcast_ref::<FHIRId>().unwrap();
        assert_eq!(s.value.as_deref(), Some("asdf"));
    }

    #[tokio::test]
    async fn get_reference_key() {
        let engine = FPEngine::new();

        let group = Group {
            member: Some(vec![GroupMember {
                entity: Box::new(Reference {
                    reference: Some(Box::new("Patient/123".to_string().into())),
                    ..Default::default()
                }),
                ..Default::default()
            }]),
            ..Default::default()
        };

        let result = engine
            .evaluate("$this.member.entity.getReferenceKey(Patient)", vec![&group])
            .await
            .expect("Failed to evaluate getReferenceKey");

        let ids = result.iter().collect::<Vec<_>>();

        assert_eq!(ids.len(), 1);

        let s = ids[0].as_any().downcast_ref::<FHIRId>().unwrap();
        assert_eq!(s.value.as_deref(), Some("123"));

        let result = engine
            .evaluate("$this.member.entity.getReferenceKey(Group)", vec![&group])
            .await
            .expect("Failed to evaluate getReferenceKey");

        let ids = result.iter().collect::<Vec<_>>();
        assert_eq!(ids.len(), 0);

        let result = engine
            .evaluate("$this.member.entity.getReferenceKey()", vec![&group])
            .await
            .expect("Failed to evaluate getReferenceKey");

        let ids = result.iter().collect::<Vec<_>>();
        assert_eq!(ids.len(), 1);
        let s = ids[0].as_any().downcast_ref::<FHIRId>().unwrap();
        assert_eq!(s.value.as_deref(), Some("123"));
    }

    #[tokio::test]
    async fn exists_with_clause() {
        let patient = Patient {
            name: Some(vec![
                HumanName {
                    given: Some(vec![FHIRString {
                        value: Some("Alice".to_string()),
                        ..Default::default()
                    }]),
                    ..Default::default()
                },
                HumanName {
                    given: Some(vec![FHIRString {
                        value: Some("Matilda".to_string()),
                        ..Default::default()
                    }]),
                    ..Default::default()
                },
            ]),
            ..Default::default()
        };

        let engine = FPEngine::new();
        let result = engine
            .evaluate("$this.name.exists(given.exists())", vec![&patient])
            .await
            .unwrap();

        let result = result.iter().collect::<Vec<_>>();

        assert_eq!(result.len(), 1);
        let s = result[0].as_any().downcast_ref::<FHIRBoolean>().unwrap();
        assert_eq!(s.value, Some(true));

        let result = engine
            .evaluate("$this.name.exists(given.empty())", vec![&patient])
            .await
            .unwrap();

        let result = result.iter().collect::<Vec<_>>();
        assert_eq!(result.len(), 1);
        let s = result[0].as_any().downcast_ref::<FHIRBoolean>().unwrap();
        assert_eq!(s.value, Some(false));

        let result = engine
            .evaluate("$this.name.exists(given = 'Matilda')", vec![&patient])
            .await
            .unwrap();

        let result = result.iter().collect::<Vec<_>>();
        assert_eq!(result.len(), 1);
        let s = result[0].as_any().downcast_ref::<FHIRBoolean>().unwrap();
        assert_eq!(s.value, Some(true));

        let result = engine
            .evaluate("$this.name.exists(given = 'Jane')", vec![&patient])
            .await
            .unwrap();

        let result = result.iter().collect::<Vec<_>>();
        assert_eq!(result.len(), 1);
        let s = result[0].as_any().downcast_ref::<FHIRBoolean>().unwrap();
        assert_eq!(s.value, Some(false));
    }
    #[tokio::test]
    async fn test_first() {
        let engine = FPEngine::new();

        let group = Group {
            member: Some(vec![
                GroupMember {
                    entity: Box::new(Reference {
                        reference: Some(Box::new("Patient/1".to_string().into())),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
                GroupMember {
                    entity: Box::new(Reference {
                        reference: Some(Box::new("Patient/2".to_string().into())),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
            ]),
            ..Default::default()
        };

        let result = engine
            .evaluate("$this.member.entity.first()", vec![&group])
            .await
            .expect("Failed to evaluate first()");

        let references = result.iter().collect::<Vec<_>>();
        assert_eq!(references.len(), 1);

        let s = references[0].as_any().downcast_ref::<Reference>().unwrap();
        assert_eq!(
            s.reference.as_ref().unwrap().value,
            Some("Patient/1".to_string())
        );
    }

    #[tokio::test]
    async fn test_join() {
        let engine = FPEngine::new();

        let mut patient = test_patient();
        patient.name.as_mut().unwrap()[0]
            .given
            .as_mut()
            .unwrap()
            .push(FHIRString {
                value: Some("David".to_string()),
                ..Default::default()
            });

        let result = engine
            .evaluate("$this.name.given.join(',')", vec![&patient])
            .await
            .expect("Failed to evaluate join()");

        let joined_ = result.iter().collect::<Vec<_>>();
        assert_eq!(joined_.len(), 1);
        let s = joined_[0].as_any().downcast_ref::<FHIRString>().unwrap();
        assert_eq!(s.value, Some("Bob,David".to_string()));

        let result = engine
            .evaluate("$this.name.given.join()", vec![&patient])
            .await
            .expect("Failed to evaluate join()");

        let joined_ = result.iter().collect::<Vec<_>>();
        assert_eq!(joined_.len(), 1);
        let s = joined_[0].as_any().downcast_ref::<FHIRString>().unwrap();
        assert_eq!(s.value, Some("BobDavid".to_string()));

        let result = engine.evaluate("$this.name.join()", vec![&patient]).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn numerical_comparisons() {
        let engine = FPEngine::new();

        let result = engine
            .evaluate("5 > 5", vec![])
            .await
            .expect("Failed to evaluate join()");

        assert_eq!(result.values.len(), 1);
        let b = result.values[0]
            .as_any()
            .downcast_ref::<FHIRBoolean>()
            .unwrap();
        assert_eq!(b.value, Some(false));

        let result = engine
            .evaluate("5 > 4", vec![])
            .await
            .expect("Failed to evaluate join()");

        assert_eq!(result.values.len(), 1);
        let b = result.values[0]
            .as_any()
            .downcast_ref::<FHIRBoolean>()
            .unwrap();
        assert_eq!(b.value, Some(true));

        let result = engine
            .evaluate("5 >= 5", vec![])
            .await
            .expect("Failed to evaluate join()");

        assert_eq!(result.values.len(), 1);
        let b = result.values[0]
            .as_any()
            .downcast_ref::<FHIRBoolean>()
            .unwrap();
        assert_eq!(b.value, Some(true));

        let result = engine
            .evaluate("4 >= 5", vec![])
            .await
            .expect("Failed to evaluate join()");

        assert_eq!(result.values.len(), 1);
        let b = result.values[0]
            .as_any()
            .downcast_ref::<FHIRBoolean>()
            .unwrap();
        assert_eq!(b.value, Some(false));

        let result = engine
            .evaluate("6 <= 5", vec![])
            .await
            .expect("Failed to evaluate join()");

        assert_eq!(result.values.len(), 1);
        let b = result.values[0]
            .as_any()
            .downcast_ref::<FHIRBoolean>()
            .unwrap();
        assert_eq!(b.value, Some(false));

        let result = engine
            .evaluate("6 <= 6", vec![])
            .await
            .expect("Failed to evaluate join()");

        assert_eq!(result.values.len(), 1);
        let b = result.values[0]
            .as_any()
            .downcast_ref::<FHIRBoolean>()
            .unwrap();
        assert_eq!(b.value, Some(true));

        let result = engine
            .evaluate("6 < 6", vec![])
            .await
            .expect("Failed to evaluate join()");

        assert_eq!(result.values.len(), 1);
        let b = result.values[0]
            .as_any()
            .downcast_ref::<FHIRBoolean>()
            .unwrap();
        assert_eq!(b.value, Some(false));

        let result = engine
            .evaluate("6 < 7", vec![])
            .await
            .expect("Failed to evaluate join()");

        assert_eq!(result.values.len(), 1);
        let b = result.values[0]
            .as_any()
            .downcast_ref::<FHIRBoolean>()
            .unwrap();
        assert_eq!(b.value, Some(true));
    }

    #[tokio::test]
    async fn string_match() {
        let engine = FPEngine::new();

        let result = engine
            .evaluate("'test'.startsWith('te')", vec![])
            .await
            .expect("Failed to evaluate join()");

        assert_eq!(result.values.len(), 1);
        let b = result.values[0]
            .as_any()
            .downcast_ref::<FHIRBoolean>()
            .unwrap();
        assert_eq!(b.value, Some(true));

        let context: &FHIRString = &"Hello".to_string().into();

        let result = engine
            .evaluate("$this.startsWith('He')", vec![context])
            .await
            .expect("Failed to evaluate join()");

        assert_eq!(result.values.len(), 1);
        let b = result.values[0]
            .as_any()
            .downcast_ref::<FHIRBoolean>()
            .unwrap();
        assert_eq!(b.value, Some(true));

        let result = engine
            .evaluate("$this.startsWith('EH')", vec![context])
            .await
            .expect("Failed to evaluate join()");

        assert_eq!(result.values.len(), 1);
        let b = result.values[0]
            .as_any()
            .downcast_ref::<FHIRBoolean>()
            .unwrap();
        assert_eq!(b.value, Some(false));

        let result = engine
            .evaluate("$this.endsWith('lo')", vec![context])
            .await
            .expect("Failed to evaluate join()");

        assert_eq!(result.values.len(), 1);
        let b = result.values[0]
            .as_any()
            .downcast_ref::<FHIRBoolean>()
            .unwrap();
        assert_eq!(b.value, Some(true));

        let result = engine
            .evaluate("$this.endsWith('ll')", vec![context])
            .await
            .expect("Failed to evaluate join()");

        assert_eq!(result.values.len(), 1);
        let b = result.values[0]
            .as_any()
            .downcast_ref::<FHIRBoolean>()
            .unwrap();
        assert_eq!(b.value, Some(false));

        let result = engine
            .evaluate("$this.contains('ll')", vec![context])
            .await
            .expect("Failed to evaluate join()");

        assert_eq!(result.values.len(), 1);
        let b = result.values[0]
            .as_any()
            .downcast_ref::<FHIRBoolean>()
            .unwrap();
        assert_eq!(b.value, Some(true));

        let result = engine
            .evaluate("$this.contains('ele')", vec![context])
            .await
            .expect("Failed to evaluate join()");

        assert_eq!(result.values.len(), 1);
        let b = result.values[0]
            .as_any()
            .downcast_ref::<FHIRBoolean>()
            .unwrap();
        assert_eq!(b.value, Some(false));
    }
}
