//! Whether a resource's indexed values satisfy a parameter's values: what the
//! other backends' queries express, as pure functions over the values a
//! search parsed once.

use haste_fhir_client::url::Prefix;

use crate::{
    indexing_conversion::{DateRange, InsertableIndex, QuantityRange, ReferenceIndex, TokenIndex},
    query::{
        QueryBuildError,
        value::{
            DateValue, NumberValue, QuantityValue, ReferenceValue, ReferenceValues, StringValues,
            TokenSystem, TokenValue, TokenValues, UriValues, Values,
        },
    },
    search_ranges::{above, below},
};

/// A parameter's values, ready to test against each resource's. The strings
/// a prefix or `:contains` match compares are lowercased once here rather
/// than per resource.
pub(super) struct Predicate<'a> {
    values: Values<'a>,
    folded: Vec<String>,
}

impl<'a> Predicate<'a> {
    /// # Errors
    ///
    /// Returns an error for a modifier this backend doesn't search.
    pub(super) fn new(values: Values<'a>) -> Result<Self, QueryBuildError> {
        // Defined by the specification, not searched here.
        let refused = match &values {
            Values::String(values @ StringValues::Text(_)) => Some(values.modifier()),
            Values::Token(
                values @ (TokenValues::Above(_)
                | TokenValues::Below(_)
                | TokenValues::Text(_)
                | TokenValues::In(_)
                | TokenValues::NotIn(_)
                | TokenValues::OfType(_)),
            ) => Some(values.modifier()),
            Values::Uri(values @ (UriValues::Above(_) | UriValues::Below(_))) => {
                Some(values.modifier())
            }
            Values::Reference(
                values @ (ReferenceValues::Above(_)
                | ReferenceValues::Below(_)
                | ReferenceValues::Identifier(_)),
            ) => Some(values.modifier()),
            _ => None,
        };
        if let Some(modifier) = refused {
            return Err(QueryBuildError::unsupported_modifier(modifier));
        }

        let folded = match &values {
            Values::String(StringValues::Prefix(values) | StringValues::Contains(values)) => {
                values.iter().map(|value| value.to_lowercase()).collect()
            }
            _ => Vec::new(),
        };
        Ok(Predicate { values, folded })
    }

    /// Whether the indexed values satisfy the parameter's values.
    pub(super) fn holds(&self, index: &InsertableIndex) -> bool {
        match (&self.values, index) {
            (Values::Missing(missing), index) => is_empty(index) == *missing,
            (Values::String(values), InsertableIndex::String(strings)) => strings
                .iter()
                .any(|string| string_holds(values, &self.folded, string)),
            (Values::Token(TokenValues::Codes(wanted)), InsertableIndex::Token(tokens)) => {
                any_pair(wanted, tokens, token_holds)
            }
            // `:not` inverts the match.
            (Values::Token(TokenValues::Not(wanted)), InsertableIndex::Token(tokens)) => {
                !any_pair(wanted, tokens, token_holds)
            }
            (Values::Uri(UriValues::Uris(values)), InsertableIndex::URI(uris)) => {
                uris.iter().any(|uri| values.contains(uri))
            }
            (
                Values::Reference(ReferenceValues::References(wanted)),
                InsertableIndex::Reference(references),
            ) => any_pair(wanted, references, reference_holds),
            (Values::Date(dates), InsertableIndex::Date(ranges)) => {
                any_pair(dates, ranges, date_holds)
            }
            (Values::Number(numbers), InsertableIndex::Number(found)) => {
                any_pair(numbers, found, |number, found| number_holds(number, *found))
            }
            (Values::Quantity(quantities), InsertableIndex::Quantity(ranges)) => {
                any_pair(quantities, ranges, quantity_holds)
            }
            // The parameter's type chose the index's, and `new` refused the
            // rest, so nothing else occurs.
            _ => false,
        }
    }
}

/// Whether some `wanted` value holds for some `found` one.
fn any_pair<W, F>(wanted: &[W], found: &[F], holds: impl Fn(&W, &F) -> bool) -> bool {
    found
        .iter()
        .any(|found| wanted.iter().any(|wanted| holds(wanted, found)))
}

fn is_empty(index: &InsertableIndex) -> bool {
    match index {
        InsertableIndex::Meta(_) => false,
        InsertableIndex::String(values)
        | InsertableIndex::URI(values)
        | InsertableIndex::Composite(values)
        | InsertableIndex::Special(values) => values.is_empty(),
        InsertableIndex::Number(values) => values.is_empty(),
        InsertableIndex::Token(values) => values.is_empty(),
        InsertableIndex::Date(values) => values.is_empty(),
        InsertableIndex::Reference(values) => values.is_empty(),
        InsertableIndex::Quantity(values) => values.is_empty(),
        InsertableIndex::DynamicParameters(values) => values.is_empty(),
    }
}

/// `folded` is the values lowercased, for the matches that ignore case.
fn string_holds(values: &StringValues<'_>, folded: &[String], string: &str) -> bool {
    match values {
        // Case-sensitive equality.
        StringValues::Exact(values) => values.iter().any(|value| value == string),
        // Case-insensitive substring.
        StringValues::Contains(_) => {
            let string = string.to_lowercase();
            folded.iter().any(|value| string.contains(value.as_str()))
        }
        // Case-insensitive prefix; `:text` is refused by [`Predicate::new`].
        StringValues::Prefix(_) | StringValues::Text(_) => {
            let string = string.to_lowercase();
            folded
                .iter()
                .any(|value| string.starts_with(value.as_str()))
        }
    }
}

fn token_holds(wanted: &TokenValue<'_>, token: &TokenIndex) -> bool {
    let system_holds = match wanted.system {
        TokenSystem::Any => true,
        TokenSystem::Absent => token.system().is_none(),
        TokenSystem::Is(system) => token.system() == Some(system),
    };
    system_holds && wanted.code.is_none_or(|code| token.code() == Some(code))
}

fn reference_holds(wanted: &ReferenceValue<'_>, reference: &ReferenceIndex) -> bool {
    reference.id() == Some(wanted.id)
        && wanted
            .resource_type
            .is_none_or(|resource_type| reference.resource_type() == Some(resource_type))
}

/// Compares the indexed period to the searched range, by prefix.
fn date_holds(date: &DateValue, indexed: &DateRange) -> bool {
    let (start, end) = (date.range.start, date.range.end);
    match date.prefix {
        Prefix::Gt | Prefix::Sa => indexed.start > end,
        Prefix::Lt => indexed.start < start,
        Prefix::Eb => indexed.end < start,
        Prefix::Ge => indexed.start >= start,
        Prefix::Le => indexed.end <= end,
        Prefix::Ne => !(indexed.start <= end && indexed.end >= start),
        // `ap` was widened when it was parsed.
        Prefix::Eq | Prefix::Ap => indexed.start <= end && indexed.end >= start,
    }
}

/// A number is indexed as a point, so `eq` is containment in the search's
/// precision range.
fn number_holds(number: &NumberValue, found: f64) -> bool {
    let (start, end) = (number.range.start, number.range.end);
    match number.prefix {
        // `ap` was widened when it was parsed.
        Prefix::Eq | Prefix::Ap => (start..=end).contains(&found),
        Prefix::Ne => !(start..=end).contains(&found),
        Prefix::Gt | Prefix::Sa => found > end,
        Prefix::Lt | Prefix::Eb => found < start,
        Prefix::Ge => found >= start,
        Prefix::Le => found <= end,
    }
}

/// A quantity is indexed as a range, so `eq` is the search value falling
/// inside it.
fn range_holds(amount: &NumberValue, start: f64, end: f64) -> bool {
    match amount.prefix {
        Prefix::Eq => (start..=end).contains(&amount.value),
        Prefix::Ne => (start..=end).contains(&amount.value),
        Prefix::Gt | Prefix::Sa => start >= above(&amount.range),
        Prefix::Lt | Prefix::Eb => end <= below(&amount.range),
        Prefix::Ge => start >= amount.range.start,
        Prefix::Le => end <= amount.range.end,
        // Overlaps the range, widened when it was parsed.
        Prefix::Ap => start <= amount.range.end && end >= amount.range.start,
    }
}

fn quantity_holds(quantity: &QuantityValue<'_>, indexed: &QuantityRange) -> bool {
    let matches = quantity
        .amount
        .as_ref()
        .is_none_or(|amount| range_holds(amount, indexed.start_value(), indexed.end_value()))
        && quantity
            .system
            .is_none_or(|system| indexed.start_system() == Some(system))
        && quantity
            .code
            .is_none_or(|code| indexed.start_code() == Some(code));

    // `ne` negates the whole match, so another unit also qualifies.
    match &quantity.amount {
        Some(amount) if amount.prefix == Prefix::Ne => !matches,
        _ => matches,
    }
}
