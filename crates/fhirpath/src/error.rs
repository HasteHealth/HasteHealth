use peg::str::LineCol;
use thiserror::Error;

use crate::parser::Literal;

#[derive(Debug, Error)]
pub enum OperationError {
    #[error("Left and right have different lengths")]
    LengthMismatch,
    #[error("Left and right have different types {0} and {1}")]
    TypeMismatch(&'static str, &'static str),
    #[error("Either Left or right have an invalid types {0} {1} ")]
    InvalidType(&'static str, &'static str),
    #[error("Operand has invalid cardinality")]
    InvalidCardinality,
}

#[derive(Debug, Error)]
pub enum FunctionError {
    #[error("Invalid function call: {0}")]
    InvalidFunctionCall(String),
    #[error("Invalid cardinality '{1}' for function '{0}'")]
    InvalidCardinality(String, usize),
}

#[derive(Debug, Error)]
pub enum FHIRPathError {
    #[error("Invalid FHIRPath expression: {0}")]
    ParseError(#[from] peg::error::ParseError<LineCol>),
    #[error("Invalid literal: {0:?}")]
    InvalidLiteral(Literal),
    #[error("Not implemented: {0}")]
    NotImplemented(String),
    #[error("Internal error: {0}")]
    InternalError(String),
    #[error("Operation error: {0}")]
    OperationError(OperationError),
    #[error("Failed to downcast value to type '{0}'")]
    FailedDowncast(String),
    #[error("Failed to derive type name")]
    FailedTypeNameDerivation,
    #[error("Function error: {0}")]
    FunctionError(#[from] FunctionError),
}
