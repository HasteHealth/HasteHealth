use crate::auth_n::certificates::JSONWebKeySet;
use haste_fhir_operation_error::OperationOutcomeError;
use std::sync::Arc;

pub struct DecodingKey {
    pub kid: String,
    pub decoding_key: jsonwebtoken::DecodingKey,
}

pub struct EncodingKey {
    pub kid: String,
    pub encoding_key: jsonwebtoken::EncodingKey,
}

pub trait CertificationProvider: Sync + Send {
    fn decoding_key<'a>(&'a self, kid: &str) -> Result<&'a DecodingKey, OperationOutcomeError>;
    fn encoding_key<'a>(&'a self) -> Result<&'a EncodingKey, OperationOutcomeError>;
    fn jwk_set(&self) -> Arc<JSONWebKeySet>;
}
