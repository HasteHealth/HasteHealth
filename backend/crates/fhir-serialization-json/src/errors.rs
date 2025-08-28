use thiserror::Error;

#[derive(Error, Debug)]
pub enum DeserializeError {
    #[error("Missing required field: {0}")]
    MissingRequiredField(String),
    #[error("Failed to convert type '{0}'")]
    FailedToConvertType(String),
    #[error("Failed to parse JSON {0}")]
    FailedToParseJSON(#[from] serde_json::Error),
    #[error("Cannot deserialize as value a typechoice variant")]
    CannotDeserializeTypeChoiceAsValue,
    #[error("Unknown field encountered: {0}")]
    UnknownField(String),
    #[error("Invalid type encountered: {0}")]
    InvalidType(String),
    #[error("Duplicate type choice variant: {0}")]
    DuplicateTypeChoiceVariant(String),
    #[error("Invalid resource type: expected '{0}', found '{1}'")]
    InvalidResourceType(String, String),
    #[error("Invalid '{0}' found '{1}'")]
    InvalidEnumVariant(String, String),
    #[error("Invalid type choice variant: {0}")]
    InvalidTypeChoiceVariant(String),
}
