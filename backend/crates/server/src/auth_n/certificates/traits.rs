use std::sync::Arc;

use crate::auth_n::certificates::JSONWebKeySet;

pub trait CertificationProvider: Sync + Send {
    fn decoding_key(&self) -> Arc<jsonwebtoken::DecodingKey>;
    fn encoding_key(&self) -> Arc<jsonwebtoken::EncodingKey>;
    fn jwk_set(&self) -> Arc<JSONWebKeySet>;
}
