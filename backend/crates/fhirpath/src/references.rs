//! The external constants (`%name`) an expression reads, for callers that check an
//! expression against a schema before it is ever evaluated.

use crate::{
    FHIRPathError, get_ast,
    parser::{Expression, Invocation, Operation, Term},
};

/// One `%name` in an expression and the member read from it, if any: `%request.type`
/// is `request` with member `type`, `%target` alone has no member.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExternalConstant {
    pub name: String,
    pub member: Option<String>,
}

/// Parses `expression` and lists its external constants in source order.
///
/// # Errors
///
/// Returns the parse error when `expression` is not valid `FHIRPath`.
pub fn external_constants(expression: &str) -> Result<Vec<ExternalConstant>, FHIRPathError> {
    let ast = get_ast(expression)?;
    let mut found = Vec::new();
    walk_expression(&ast, &mut found);
    Ok(found)
}

fn walk_expression(expression: &Expression, found: &mut Vec<ExternalConstant>) {
    match expression {
        Expression::Singular(terms) => walk_terms(terms, found),
        Expression::Operation(operation) => {
            let (left, right) = operands(operation);
            walk_expression(left, found);
            if let Some(right) = right {
                walk_expression(right, found);
            }
        }
    }
}

/// A singular expression is a term followed by its invocations, so the identifier
/// right after a `%name` is the member read from it.
fn walk_terms(terms: &[Term], found: &mut Vec<ExternalConstant>) {
    for (index, term) in terms.iter().enumerate() {
        match term {
            Term::ExternalConstant(name) => {
                let member = match terms.get(index + 1) {
                    Some(Term::Invocation(Invocation::Identifier(identifier))) => {
                        Some(identifier.0.clone())
                    }
                    _ => None,
                };
                found.push(ExternalConstant {
                    name: name.clone(),
                    member,
                });
            }
            Term::Invocation(Invocation::Function(function)) => {
                for argument in &function.arguments {
                    walk_expression(argument, found);
                }
            }
            Term::Invocation(Invocation::Index(index)) => walk_expression(index, found),
            Term::Parenthesized(inner) => walk_expression(inner, found),
            Term::Invocation(_) | Term::Literal(_) => {}
        }
    }
}

/// The sub-expressions of an operation: one for a unary or type operation, else two.
fn operands(operation: &Operation) -> (&Expression, Option<&Expression>) {
    match operation {
        Operation::Polarity(_, operand) | Operation::Is(operand, _) | Operation::As(operand, _) => {
            (operand, None)
        }
        Operation::Add(left, right)
        | Operation::Subtraction(left, right)
        | Operation::Concatenate(left, right)
        | Operation::Multiplication(left, right)
        | Operation::Division(left, right)
        | Operation::DivisionTruncated(left, right)
        | Operation::Modulo(left, right)
        | Operation::Union(left, right)
        | Operation::LessThanEqual(left, right)
        | Operation::GreaterThanEqual(left, right)
        | Operation::LessThan(left, right)
        | Operation::GreaterThan(left, right)
        | Operation::Equal(left, right)
        | Operation::NotEqual(left, right)
        | Operation::Equivalent(left, right)
        | Operation::NotEquivalent(left, right)
        | Operation::In(left, right)
        | Operation::Contains(left, right)
        | Operation::And(left, right)
        | Operation::Or(left, right)
        | Operation::XOr(left, right)
        | Operation::Implies(left, right) => (left, Some(right)),
    }
}
