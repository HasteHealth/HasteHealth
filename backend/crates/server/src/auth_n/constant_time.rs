//! Comparisons of secrets whose running time does not depend on where the
//! values first differ.
//!
//! `==` on strings stops at the first mismatching byte, so how long a
//! comparison takes tells a caller how many leading bytes they got right.
//! Client secrets and CSRF tokens are compared here instead.

use subtle::ConstantTimeEq;

/// Whether `a` and `b` are the same string, in time that depends only on
/// their lengths.
pub fn str_eq(a: &str, b: &str) -> bool {
    a.as_bytes().ct_eq(b.as_bytes()).into()
}

/// Whether an optional secret matches: both absent, or both present and
/// equal under [`str_eq`].
pub fn optional_str_eq(a: Option<&str>, b: Option<&str>) -> bool {
    match (a, b) {
        (None, None) => true,
        (Some(a), Some(b)) => str_eq(a, b),
        _ => false,
    }
}
