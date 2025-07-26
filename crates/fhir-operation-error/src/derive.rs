use fhir_operation_error_derive::*;

#[cfg(test)]
mod test {
    use super::*;

    #[derive(OperationOutcomeError)]
    pub enum CustomOpError {
        #[fatal(code = "invalid", diagnostic = "Invalid operation")]
        NotFound,
        #[error(code = "not-found", diagnostic = "Resource not found")]
        InvalidInput,
    }

    #[test]
    fn test_operation_error() {
        let error = CustomOpError::NotFound;
        let outcome = error.outcome();

        assert_eq!(outcome.code, "invalid");
        assert_eq!(outcome.diagnostic, Some("Invalid operation".into()));
    }
}
