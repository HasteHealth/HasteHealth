//! Rules are transforms written in Rust that apply across many resources, for
//! changes that are impractical as targeted patches. A manifest enables a rule
//! by its [`Rule::name`].

mod clippy_doc_markdown;

use serde_json::Value;

/// A single string rewritten by a rule.
#[derive(Debug, Clone)]
pub struct Edit {
    /// JSON pointer into the resource.
    pub pointer: String,
    pub before: Value,
    pub after: Value,
}

pub trait Rule: Sync {
    /// Name used to enable the rule in a manifest.
    fn name(&self) -> &'static str;
    /// Rewrites `resource` in place and returns what changed.
    fn apply(&self, resource: &mut Value) -> Vec<Edit>;
}

static RULES: &[&dyn Rule] = &[&clippy_doc_markdown::ClippyDocMarkdown];

/// Looks up a rule by name.
#[must_use]
pub fn find(name: &str) -> Option<&'static dyn Rule> {
    RULES.iter().copied().find(|rule| rule.name() == name)
}

/// Names of every available rule.
#[must_use]
pub fn names() -> Vec<&'static str> {
    RULES.iter().map(|rule| rule.name()).collect()
}
