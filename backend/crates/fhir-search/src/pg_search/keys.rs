//! Hashed keys that scope rows in the search tables.
//!
//! Each key is SHA-256 over length-prefixed parts, truncated to 64 bits.
//! Stored rows carry these values, so changing the hash requires a full
//! reindex.
//!
//! Collisions are mostly harmless: searches filter the anchor on its real
//! `tenant`/`project` columns and join other tables by the unique `res_key`, so
//! a collision only makes an index scan read rows that get discarded. The one
//! harmful case, two parameters in one project sharing an identity, is caught
//! by [`check_identities`].

use std::collections::HashMap;

use sha2::{Digest, Sha256};

/// Scope of a resource type table row (one tenant's project). Leads every
/// index on those tables.
#[must_use]
pub fn scope_key(tenant: &str, project: &str) -> i64 {
    hash_parts(&[b"scope", tenant.as_bytes(), project.as_bytes()])
}

/// Identifies a parameter's rows in the shared tables, in place of its URL.
#[must_use]
pub fn param_identity(tenant: &str, project: &str, param_url: &str) -> i64 {
    hash_parts(&[
        b"param",
        tenant.as_bytes(),
        project.as_bytes(),
        param_url.as_bytes(),
    ])
}

/// Fails if two distinct `param_urls` hash to the same identity.
///
/// # Errors
///
/// Returns the two colliding URLs.
pub fn check_identities<'a>(
    tenant: &str,
    project: &str,
    param_urls: impl IntoIterator<Item = &'a str>,
) -> Result<(), (String, String)> {
    param_urls
        .into_iter()
        .try_fold(HashMap::<i64, &str>::new(), |mut seen, url| {
            match seen.insert(param_identity(tenant, project, url), url) {
                Some(existing) if existing != url => Err((existing.to_string(), url.to_string())),
                _ => Ok(seen),
            }
        })
        .map(|_| ())
}

/// Length-prefixes each part, so `("ab", "c")` and `("a", "bc")` differ.
fn hash_parts(parts: &[&[u8]]) -> i64 {
    let digest = parts
        .iter()
        .fold(Sha256::new(), |hasher, part| {
            hasher
                .chain_update((part.len() as u64).to_be_bytes())
                .chain_update(part)
        })
        .finalize();

    let mut first = [0u8; 8];
    first.copy_from_slice(&digest[..8]);
    i64::from_be_bytes(first)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Stored rows carry these values, so they must be deterministic.
    #[test]
    fn hashes_are_stable() {
        assert_eq!(scope_key("t", "p"), scope_key("t", "p"));
        assert_eq!(
            param_identity("t", "p", "http://hl7.org/fhir/SearchParameter/Patient-name"),
            param_identity("t", "p", "http://hl7.org/fhir/SearchParameter/Patient-name"),
        );
    }

    #[test]
    fn part_boundaries_are_part_of_the_input() {
        assert_ne!(scope_key("ab", "c"), scope_key("a", "bc"));
        assert_ne!(
            param_identity("t", "pa", "b"),
            param_identity("t", "p", "ab")
        );
    }

    #[test]
    fn identities_are_scoped_to_the_project() {
        let url = "http://hl7.org/fhir/SearchParameter/Patient-name";
        assert_ne!(
            param_identity("t", "p1", url),
            param_identity("t", "p2", url)
        );
        assert_ne!(
            param_identity("t1", "p", url),
            param_identity("t2", "p", url)
        );
    }

    #[test]
    fn scopes_and_identities_do_not_share_a_space() {
        assert_ne!(scope_key("t", "p"), param_identity("t", "p", ""));
    }

    #[test]
    fn repeated_urls_are_not_collisions() {
        assert!(check_identities("t", "p", ["a", "b", "a"]).is_ok());
    }
}
