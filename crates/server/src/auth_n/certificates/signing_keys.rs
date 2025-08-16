use rand::rngs::OsRng;
use rsa::{RsaPrivateKey, RsaPublicKey};

pub fn get_decoding_key() -> jsonwebtoken::DecodingKey {
    let mut rng = OsRng;
    let bits = 2048;
    let priv_key = RsaPrivateKey::new(&mut rng, bits).expect("failed to generate a key");
    let pub_key = RsaPublicKey::from(&priv_key);
    todo!();
}
