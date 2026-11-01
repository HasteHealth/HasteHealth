use haste_fhir_operation_error::derive::OperationOutcomeError;

/// Why a package could not be read or fetched.
#[derive(OperationOutcomeError, Debug)]
pub enum PackageError {
    #[error(
        code = "invalid",
        diagnostic = "'{arg0}' is not a valid package name. Names use letters, digits, '.', '-' and '_'."
    )]
    InvalidName(String),

    #[error(
        code = "invalid",
        diagnostic = "'{arg0}' is not a valid package version."
    )]
    InvalidVersion(String),

    #[error(code = "invalid", diagnostic = "The package could not be read: {arg0}")]
    Unreadable(String),

    #[error(
        code = "invalid",
        diagnostic = "The package has no package.json, so it is not a FHIR package."
    )]
    MissingManifest,

    #[error(
        code = "invalid",
        diagnostic = "The package's package.json is invalid: {arg0}"
    )]
    InvalidManifest(String),

    #[error(
        code = "too-costly",
        diagnostic = "The package is larger than the {arg0} byte limit."
    )]
    TooLarge(u64),

    #[error(
        code = "too-costly",
        diagnostic = "The package has more than {arg0} files."
    )]
    TooManyFiles(usize),

    #[error(
        code = "not-found",
        diagnostic = "Package '{arg0}' was not found in the package registry."
    )]
    NotFound(String),

    #[error(
        code = "not-found",
        diagnostic = "No published version of '{arg0}' matches '{arg1}'."
    )]
    NoMatchingVersion(String, String),

    #[error(
        code = "transient",
        diagnostic = "The package registry could not be reached: {arg0}"
    )]
    Registry(String),

    #[error(
        code = "not-supported",
        diagnostic = "This server does not fetch packages from a registry. Upload the package instead."
    )]
    RegistryDisabled,

    #[error(
        code = "exception",
        diagnostic = "The server's package registry setting (packages.registry_url) is not an https URL."
    )]
    InvalidRegistry,
}
