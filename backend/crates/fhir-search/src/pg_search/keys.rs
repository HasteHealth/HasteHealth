//! The hashed keys the search tables are scoped by.
//!
//! Both are SHA-256 over length-prefixed parts, truncated to 64 bits. Every
//! stored row carries these values, so changing the function means reindexing
//! everything.
//!
//! Neither key has to be collision-free. Searches filter the anchor table by
//! its real `tenant` and `project` columns and reach the other tables only
//! through the never-colliding `res_key`, so a collision across scopes or
//! projects only makes an index scan read rows the anchor then discards. The
//! one collision that would change results — two parameters of the same
//! project sharing an identity — is caught by [`check_identities`].

use std::collections::HashMap;

use sha2::{Digest, Sha256};

/// The scope of a per-resource-type table row: one tenant's project. Leads
/// every index on those tables, so a lookup stays within one project.
#[must_use]
pub fn scope_key(tenant: &str, project: &str) -> i64 {
    hash_parts(&[b"scope", tenant.as_bytes(), project.as_bytes()])
}

/// What a shared-table row is discriminated by, in place of the parameter's
/// canonical URL.
#[must_use]
pub fn param_identity(tenant: &str, project: &str, param_url: &str) -> i64 {
    hash_parts(&[
        b"param",
        tenant.as_bytes(),
        project.as_bytes(),
        param_url.as_bytes(),
    ])
}

/// Fails if two of `param_urls` share an identity, which would let one
/// parameter's rows answer a search for the other.
///
/// # Errors
///
/// Returns the two colliding URLs.
pub fn check_identities<'a>(
    tenant: &str,
    project: &str,
    param_urls: impl IntoIterator<Item = &'a str>,
) -> Result<(), (String, String)> {
    let mut seen: HashMap<i64, &str> = HashMap::new();

    for url in param_urls {
        let identity = param_identity(tenant, project, url);
        if let Some(existing) = seen.insert(identity, url)
            && existing != url
        {
            return Err((existing.to_string(), url.to_string()));
        }
    }

    Ok(())
}

/// Length-prefixes each part, so `("ab", "c")` and `("a", "bc")` differ.
fn hash_parts(parts: &[&[u8]]) -> i64 {
    let mut hasher = Sha256::new();
    for part in parts {
        hasher.update((part.len() as u64).to_be_bytes());
        hasher.update(part);
    }

    let digest = hasher.finalize();
    let mut first = [0u8; 8];
    first.copy_from_slice(&digest[..8]);
    i64::from_be_bytes(first)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Stored rows carry these values, so the function must not drift.
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
