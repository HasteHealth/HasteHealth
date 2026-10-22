//! A parameter's values parsed by type and modifier, the way every backend
//! reads them (<https://hl7.org/fhir/R4/search.html>).
//!
//! Every modifier the specification defines parses here, into the shape it
//! gives the values; a backend refuses the ones it doesn't search. A decimal
//! is parsed as the precision range it implies and `ap` as the widened range
//! it overlaps, so a backend only writes the comparison in its own language.
//!
//! Text is borrowed from the parameter, and copied once, by the backend, into
//! its bind list or query. Only a value with an escape is copied sooner, by
//! the parameter's accessors, which resolve it: nothing here reads a
//! backslash.

use std::borrow::Cow;

use haste_fhir_client::url::{Parameter, Prefix, Segments, parse_prefix};
use haste_fhir_model::r4::{
    datetime::parse_datetime,
    generated::{
        resources::ResourceType,
        terminology::{BoundCode, SearchParamType},
    },
};
use haste_repository::utilities::parse_haste_canonical;

use crate::{
    indexing_conversion::{DateRange, DecimalRange, date_time_range, get_decimal_range},
    query::QueryBuildError,
    search_ranges::{approximate_date, approximate_decimal, now_ms},
};

/// A parameter's values, by type and then by modifier
/// (<https://hl7.org/fhir/R4/search.html#modifiers>). A resource matches on
/// any of the values.
#[derive(Debug)]
pub enum Values<'a> {
    /// `:missing=true` (`true`) or `:missing=false` (`false`); any type.
    Missing(bool),
    String(StringValues<'a>),
    Token(TokenValues<'a>),
    Uri(UriValues<'a>),
    Reference(ReferenceValues<'a>),
    Date(Vec<DateValue>),
    Number(Vec<NumberValue>),
    Quantity(Vec<QuantityValue<'a>>),
}

/// A string parameter's values, by modifier.
#[derive(Debug)]
pub enum StringValues<'a> {
    /// No modifier: a prefix, ignoring case.
    Prefix(Cow<'a, [String]>),
    /// `:exact`: the whole value, case and accents included.
    Exact(Cow<'a, [String]>),
    /// `:contains`: anywhere in the value, ignoring case.
    Contains(Cow<'a, [String]>),
    /// `:text`: advanced text handling.
    Text(Cow<'a, [String]>),
}

/// A token parameter's values, by modifier.
#[derive(Debug)]
#[expect(
    dead_code,
    reason = "modifiers the specification defines that no backend searches yet"
)]
pub enum TokenValues<'a> {
    /// No modifier: `[system|]code`.
    Codes(Vec<TokenValue<'a>>),
    /// `:not`: resources without a matching token.
    Not(Vec<TokenValue<'a>>),
    /// `:above`: tokens that subsume the searched code.
    Above(Vec<TokenValue<'a>>),
    /// `:below`: tokens the searched code subsumes.
    Below(Vec<TokenValue<'a>>),
    /// `:text`: free text matched against a coding's display or a concept's
    /// text.
    Text(Cow<'a, [String]>),
    /// `:in`: tokens in any of the value sets, by canonical URI.
    In(Cow<'a, [String]>),
    /// `:not-in`: tokens in none of the value sets, by canonical URI.
    NotIn(Cow<'a, [String]>),
    /// `:of-type`: `system|code|value`, the type and value of an identifier.
    OfType(Vec<IdentifierTypeValue<'a>>),
}

/// A uri parameter's values, by modifier.
#[derive(Debug)]
#[expect(
    dead_code,
    reason = "modifiers the specification defines that no backend searches yet"
)]
pub enum UriValues<'a> {
    /// No modifier: the URI itself.
    Uris(Cow<'a, [String]>),
    /// `:above`: URIs above the searched one in a hierarchy.
    Above(Cow<'a, [String]>),
    /// `:below`: URIs below the searched one in a hierarchy.
    Below(Cow<'a, [String]>),
}

/// A reference parameter's values, by modifier.
#[derive(Debug)]
#[expect(
    dead_code,
    reason = "modifiers the specification defines that no backend searches yet"
)]
pub enum ReferenceValues<'a> {
    /// No modifier, or `:[type]` applied to the values: `subject:Patient=23`
    /// is `subject=Patient/23`.
    References(Vec<ReferenceValue<'a>>),
    /// `:above`: references transitively up a hierarchy.
    Above(Vec<ReferenceValue<'a>>),
    /// `:below`: references transitively down a hierarchy.
    Below(Vec<ReferenceValue<'a>>),
    /// `:identifier`: `[system|]value` of the target's identifier.
    Identifier(Vec<TokenValue<'a>>),
}

/// `code`, `system|code`, `|code`, `system|` or `|`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TokenValue<'a> {
    pub system: TokenSystem<'a>,
    /// `None` for an empty code (`system|`, `|`): any code.
    pub code: Option<Cow<'a, str>>,
}

/// What a token value says of the system.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenSystem<'a> {
    /// Nothing (`code`, `|`): any system.
    Any,
    /// `|code`: there is none.
    Absent,
    /// `system|code` or `system|`.
    Is(Cow<'a, str>),
}

/// `system|code|value`: an identifier's type and value, for `:of-type`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IdentifierTypeValue<'a> {
    pub system: Cow<'a, str>,
    pub code: Cow<'a, str>,
    pub value: Cow<'a, str>,
}

/// `id` or `Type/id`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReferenceValue<'a> {
    pub resource_type: Option<Cow<'a, str>>,
    pub id: Cow<'a, str>,
}

/// `[prefix]date`, as the range the prefix compares. For `ap` the range is
/// already widened, so it compares as `eq` does.
#[derive(Debug)]
pub struct DateValue {
    pub prefix: Prefix,
    pub range: DateRange,
}

/// `[prefix]number`: the value, and the precision range it implies (`45` is
/// `[44.5, 45.5]`). For `ap` the range is already widened, so it compares as
/// `eq` does.
#[derive(Debug)]
pub struct NumberValue {
    pub prefix: Prefix,
    pub value: f64,
    pub range: DecimalRange,
}

/// `[prefix]value[|system|code]`; an empty segment is unconstrained.
#[derive(Debug)]
pub struct QuantityValue<'a> {
    pub amount: Option<NumberValue>,
    pub system: Option<Cow<'a, str>>,
    pub code: Option<Cow<'a, str>>,
}

/// The parameter types every backend indexes: all but composite and special.
#[derive(Clone, Copy)]
enum Indexed {
    String,
    Token,
    Uri,
    Reference,
    Date,
    Number,
    Quantity,
}

impl Indexed {
    fn of(param_type: &BoundCode<SearchParamType>) -> Option<Self> {
        Some(match param_type.as_str()? {
            "string" => Indexed::String,
            "token" => Indexed::Token,
            "uri" => Indexed::Uri,
            "reference" => Indexed::Reference,
            "date" => Indexed::Date,
            "number" => Indexed::Number,
            "quantity" => Indexed::Quantity,
            _ => return None,
        })
    }
}

impl<'a> Values<'a> {
    /// `parameter`'s values parsed by `param_type`, under its modifier.
    ///
    /// # Errors
    ///
    /// Returns an error for a type no backend indexes, a modifier the
    /// specification doesn't define for the type, no values, and a value the
    /// type or modifier can't read.
    pub fn parse(
        parameter: &'a Parameter,
        param_type: &BoundCode<SearchParamType>,
    ) -> Result<Self, QueryBuildError> {
        let Some(param_type) = Indexed::of(param_type) else {
            return Err(QueryBuildError::UnsupportedParameter(
                parameter.name.clone(),
            ));
        };
        let modifier = parameter.modifier.as_deref();
        // `:missing` applies to every type.
        if modifier == Some("missing") {
            return Ok(Values::Missing(missing_value(parameter)?));
        }
        if parameter.value.is_empty() {
            return Err(QueryBuildError::InvalidParameterValue(
                parameter.name.clone(),
            ));
        }

        Ok(match (param_type, modifier) {
            (Indexed::String, _) => Values::String(StringValues::parse(modifier, parameter)?),
            (Indexed::Token, _) => Values::Token(TokenValues::parse(modifier, parameter)?),
            (Indexed::Uri, _) => Values::Uri(UriValues::parse(modifier, parameter)?),
            (Indexed::Reference, _) => {
                Values::Reference(ReferenceValues::parse(modifier, parameter)?)
            }
            (Indexed::Date, None) => Values::Date(each(parameter, |v| DateValue::parse(&v))?),
            (Indexed::Number, None) => Values::Number(each(parameter, |v| NumberValue::parse(&v))?),
            (Indexed::Quantity, None) => {
                Values::Quantity(segments(parameter, QuantityValue::parse)?)
            }
            // A date, number or quantity is qualified by a prefix instead.
            (Indexed::Date | Indexed::Number | Indexed::Quantity, Some(modifier)) => {
                return Err(invalid_modifier(modifier));
            }
        })
    }
}

/// `:missing`'s value: exactly `true` or `false`.
fn missing_value(parameter: &Parameter) -> Result<bool, QueryBuildError> {
    match &*parameter.values() {
        [value] if value == "true" => Ok(true),
        [value] if value == "false" => Ok(false),
        _ => Err(QueryBuildError::InvalidParameterValue(
            parameter.name.clone(),
        )),
    }
}

/// Every value parsed, or the first error.
fn each<'a, T>(
    parameter: &'a Parameter,
    parse: impl Fn(Cow<'a, str>) -> Result<T, QueryBuildError>,
) -> Result<Vec<T>, QueryBuildError> {
    parameter.each_value().map(parse).collect()
}

/// Every value parsed from its `|` segments, or the first error.
fn segments<'a, T>(
    parameter: &'a Parameter,
    parse: impl Fn(Segments<'a>) -> Result<T, QueryBuildError>,
) -> Result<Vec<T>, QueryBuildError> {
    parameter.segmented('|').map(parse).collect()
}

fn invalid_modifier(modifier: &str) -> QueryBuildError {
    QueryBuildError::InvalidModifier(modifier.to_string())
}

impl<'a> StringValues<'a> {
    fn parse(modifier: Option<&str>, parameter: &'a Parameter) -> Result<Self, QueryBuildError> {
        Ok(match modifier {
            None => StringValues::Prefix(parameter.values()),
            Some("exact") => StringValues::Exact(parameter.values()),
            Some("contains") => StringValues::Contains(parameter.values()),
            Some("text") => StringValues::Text(parameter.values()),
            Some(other) => return Err(invalid_modifier(other)),
        })
    }

    /// The modifier as written after the colon; empty when there is none.
    #[must_use]
    pub fn modifier(&self) -> &'static str {
        match self {
            StringValues::Prefix(_) => "",
            StringValues::Exact(_) => "exact",
            StringValues::Contains(_) => "contains",
            StringValues::Text(_) => "text",
        }
    }

    /// The strings, whichever modifier qualifies them.
    #[must_use]
    pub fn values(&self) -> &[String] {
        match self {
            StringValues::Prefix(values)
            | StringValues::Exact(values)
            | StringValues::Contains(values)
            | StringValues::Text(values) => values,
        }
    }
}

impl<'a> TokenValues<'a> {
    fn parse(modifier: Option<&str>, parameter: &'a Parameter) -> Result<Self, QueryBuildError> {
        let codes = || segments(parameter, TokenValue::parse);
        Ok(match modifier {
            None => TokenValues::Codes(codes()?),
            Some("not") => TokenValues::Not(codes()?),
            Some("above") => TokenValues::Above(codes()?),
            Some("below") => TokenValues::Below(codes()?),
            Some("text") => TokenValues::Text(parameter.values()),
            Some("in") => TokenValues::In(parameter.values()),
            Some("not-in") => TokenValues::NotIn(parameter.values()),
            Some("of-type") => {
                TokenValues::OfType(segments(parameter, IdentifierTypeValue::parse)?)
            }
            Some(other) => return Err(invalid_modifier(other)),
        })
    }

    /// The modifier as written after the colon; empty when there is none.
    #[must_use]
    pub fn modifier(&self) -> &'static str {
        match self {
            TokenValues::Codes(_) => "",
            TokenValues::Not(_) => "not",
            TokenValues::Above(_) => "above",
            TokenValues::Below(_) => "below",
            TokenValues::Text(_) => "text",
            TokenValues::In(_) => "in",
            TokenValues::NotIn(_) => "not-in",
            TokenValues::OfType(_) => "of-type",
        }
    }
}

impl<'a> UriValues<'a> {
    fn parse(modifier: Option<&str>, parameter: &'a Parameter) -> Result<Self, QueryBuildError> {
        Ok(match modifier {
            None => UriValues::Uris(parameter.values()),
            Some("above") => UriValues::Above(parameter.values()),
            Some("below") => UriValues::Below(parameter.values()),
            Some(other) => return Err(invalid_modifier(other)),
        })
    }

    /// The modifier as written after the colon; empty when there is none.
    #[must_use]
    pub fn modifier(&self) -> &'static str {
        match self {
            UriValues::Uris(_) => "",
            UriValues::Above(_) => "above",
            UriValues::Below(_) => "below",
        }
    }
}

impl<'a> ReferenceValues<'a> {
    /// `:above`, `:below`, `:identifier` or `:[type]`, the name of a resource
    /// type.
    fn parse(modifier: Option<&'a str>, parameter: &'a Parameter) -> Result<Self, QueryBuildError> {
        let references =
            |type_name| each(parameter, |value| ReferenceValue::parse(value, type_name));
        Ok(match modifier {
            None => ReferenceValues::References(references(None)?),
            Some("above") => ReferenceValues::Above(references(None)?),
            Some("below") => ReferenceValues::Below(references(None)?),
            Some("identifier") => {
                ReferenceValues::Identifier(segments(parameter, TokenValue::parse)?)
            }
            Some(name) if ResourceType::try_from(name).is_ok() => {
                ReferenceValues::References(references(Some(name))?)
            }
            Some(other) => return Err(invalid_modifier(other)),
        })
    }

    /// The modifier as written after the colon; empty when there is none or
    /// it was `:[type]`, which the values carry.
    #[must_use]
    pub fn modifier(&self) -> &'static str {
        match self {
            ReferenceValues::References(_) => "",
            ReferenceValues::Above(_) => "above",
            ReferenceValues::Below(_) => "below",
            ReferenceValues::Identifier(_) => "identifier",
        }
    }
}

impl<'a> TokenValue<'a> {
    /// Parses a value's `|` segments.
    ///
    /// # Errors
    ///
    /// Returns an error for a value with three or more segments, which names
    /// no token.
    pub fn parse(mut segments: Segments<'a>) -> Result<Self, QueryBuildError> {
        let source = segments.source();
        let invalid = || QueryBuildError::InvalidParameterValue(source.to_string());
        let (first, second, third) = (segments.next(), segments.next(), segments.next());
        Ok(match (first, second, third) {
            (Some(code), None, _) => TokenValue {
                system: TokenSystem::Any,
                code: Some(code),
            },
            (Some(_), Some(_), Some(_)) | (None, ..) => return Err(invalid()),
            (Some(system), Some(code), None) => {
                let code = (!code.is_empty()).then_some(code);
                let system = match (system.is_empty(), code.is_some()) {
                    // `|`: nothing is said of either.
                    (true, false) => TokenSystem::Any,
                    (true, true) => TokenSystem::Absent,
                    (false, _) => TokenSystem::Is(system),
                };
                TokenValue { system, code }
            }
        })
    }
}

impl<'a> IdentifierTypeValue<'a> {
    /// Parses a value's `|` segments.
    ///
    /// # Errors
    ///
    /// Returns an error unless all three of `system|code|value` are present.
    pub fn parse(mut segments: Segments<'a>) -> Result<Self, QueryBuildError> {
        match (
            segments.next(),
            segments.next(),
            segments.next(),
            segments.next(),
        ) {
            (Some(system), Some(code), Some(value), None)
                if !system.is_empty() && !code.is_empty() && !value.is_empty() =>
            {
                Ok(IdentifierTypeValue {
                    system,
                    code,
                    value,
                })
            }
            _ => Err(QueryBuildError::InvalidParameterValue(
                segments.source().to_string(),
            )),
        }
    }
}

impl<'a> ReferenceValue<'a> {
    /// `id`, `Type/id` or a Haste Health canonical
    /// (`https://haste.health/fhir/Membership/<id>`), which is indexed as the
    /// `Type/id` it names. `type_name`, from `:[type]`, supplies the type an
    /// `id` lacks.
    ///
    /// # Errors
    ///
    /// Returns an error for anything else, and for a `Type/id` whose type
    /// isn't `type_name`.
    pub fn parse(value: Cow<'a, str>, type_name: Option<&'a str>) -> Result<Self, QueryBuildError> {
        let invalid = || QueryBuildError::InvalidParameterValue(value.to_string());
        // The parts borrow from `value`, so an owned value owns them too.
        let (resource_type, id) = match &value {
            Cow::Borrowed(value) => {
                let (resource_type, id) = reference_parts(value).ok_or_else(invalid)?;
                (resource_type.map(Cow::Borrowed), Cow::Borrowed(id))
            }
            Cow::Owned(value) => {
                let (resource_type, id) = reference_parts(value).ok_or_else(invalid)?;
                (
                    resource_type.map(|resource_type| Cow::Owned(resource_type.to_string())),
                    Cow::Owned(id.to_string()),
                )
            }
        };
        let resource_type = match (type_name, resource_type) {
            (Some(type_name), None) => Some(Cow::Borrowed(type_name)),
            (Some(type_name), Some(resource_type)) if resource_type != type_name => {
                return Err(invalid());
            }
            (_, resource_type) => resource_type,
        };
        Ok(ReferenceValue { resource_type, id })
    }
}

/// `(type, id)` of `id`, `Type/id` or a Haste Health canonical; `None` for
/// anything else.
fn reference_parts(value: &str) -> Option<(Option<&str>, &str)> {
    if let Some((resource_type, id)) = parse_haste_canonical(value) {
        return Some((Some(resource_type), id));
    }
    match value.split_once('/') {
        None => Some((None, value)),
        Some((resource_type, id)) if !id.contains('/') => Some((Some(resource_type), id)),
        Some(_) => None,
    }
}

impl DateValue {
    /// # Errors
    ///
    /// Returns [`QueryBuildError::InvalidDateFormat`] for a date that doesn't
    /// parse.
    pub fn parse(value: &str) -> Result<Self, QueryBuildError> {
        let (prefix, date) = parse_prefix(value);
        let invalid = || QueryBuildError::InvalidDateFormat(date.to_string());
        let date_time = parse_datetime(date).map_err(|_| invalid())?;
        let range = date_time_range(&date_time).map_err(|_| invalid())?;
        let range = match prefix {
            Prefix::Ap => {
                let (start, end) = approximate_date(range.start, range.end, now_ms());
                DateRange { start, end }
            }
            _ => range,
        };
        Ok(DateValue { prefix, range })
    }
}

impl NumberValue {
    /// # Errors
    ///
    /// Returns [`QueryBuildError::InvalidParameterValue`] for a number that
    /// doesn't parse.
    pub fn parse(value: &str) -> Result<Self, QueryBuildError> {
        let (prefix, number) = parse_prefix(value);
        let invalid = || QueryBuildError::InvalidParameterValue(number.to_string());
        let range = get_decimal_range(number).map_err(|_| invalid())?;
        let range = match prefix {
            Prefix::Ap => {
                let (start, end) = approximate_decimal(&range);
                DecimalRange { start, end }
            }
            _ => range,
        };
        Ok(NumberValue {
            prefix,
            value: number.parse().map_err(|_| invalid())?,
            range,
        })
    }
}

impl<'a> QuantityValue<'a> {
    /// # Errors
    ///
    /// Returns an error for a value with other than one or three segments, or
    /// an amount that doesn't parse.
    pub fn parse(mut segments: Segments<'a>) -> Result<Self, QueryBuildError> {
        let value = segments.source();
        let (amount, system, code) = match (
            segments.next(),
            segments.next(),
            segments.next(),
            segments.next(),
        ) {
            // A bare value matches any unit.
            (Some(amount), None, _, _) => (amount, None, None),
            (Some(amount), Some(system), Some(code), None) => (amount, Some(system), Some(code)),
            (Some(_), Some(_), Some(_), Some(_)) if segments.next().is_none() => {
                return Err(QueryBuildError::UnsupportedParameterValue(
                    value.to_string(),
                ));
            }
            _ => return Err(QueryBuildError::InvalidParameterValue(value.to_string())),
        };
        Ok(QuantityValue {
            amount: (!amount.is_empty())
                .then(|| NumberValue::parse(&amount))
                .transpose()?,
            system: system.filter(|system| !system.is_empty()),
            code: code.filter(|code| !code.is_empty()),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(
        type_: &BoundCode<SearchParamType>,
        modifier: &str,
        values: &[&str],
    ) -> Result<Values<'static>, QueryBuildError> {
        let parameter = Box::leak(Box::new(Parameter {
            name: "p".to_string(),
            modifier: (!modifier.is_empty()).then(|| modifier.to_string()),
            value: values.iter().map(|v| (*v).to_string()).collect(),
            chains: Vec::new(),
        }));
        Values::parse(parameter, type_)
    }

    fn parse_token(value: &str) -> Result<TokenValue<'_>, QueryBuildError> {
        TokenValue::parse(Segments::of(value, '|'))
    }

    #[test]
    fn escaped_separators_are_literal_in_tokens() {
        let token = parse_token(r"http://a\|b|123").unwrap();
        assert_eq!(token.system, TokenSystem::Is("http://a|b".into()));
        assert_eq!(token.code.as_deref(), Some("123"));

        let token = parse_token(r"x\\|y\|z").unwrap();
        assert_eq!(token.system, TokenSystem::Is(r"x\".into()));
        assert_eq!(token.code.as_deref(), Some("y|z"));

        // An escaped pipe is not a separator.
        let token = parse_token(r"a\|b").unwrap();
        assert_eq!(token.system, TokenSystem::Any);
        assert_eq!(token.code.as_deref(), Some("a|b"));

        assert!(parse_token("a|b|c").is_err());
    }

    #[test]
    fn token_shapes() {
        let token = parse_token("|").unwrap();
        assert_eq!(token.system, TokenSystem::Any);
        assert_eq!(token.code, None);
        let token = parse_token("|c").unwrap();
        assert_eq!(token.system, TokenSystem::Absent);
        assert_eq!(token.code.as_deref(), Some("c"));
        let token = parse_token("s|").unwrap();
        assert_eq!(token.system, TokenSystem::Is("s".into()));
        assert_eq!(token.code, None);
    }

    #[test]
    fn escaped_separators_are_literal_in_quantities() {
        let quantity = QuantityValue::parse(Segments::of(r"5|http://a\|b|m\|s", '|')).unwrap();
        assert_eq!(quantity.system.as_deref(), Some("http://a|b"));
        assert_eq!(quantity.code.as_deref(), Some("m|s"));
    }

    #[test]
    fn strings_resolve_their_escapes() {
        let Values::String(StringValues::Exact(resolved)) =
            parse(&SearchParamType::string(), "exact", &[r"a\|b", "plain"]).unwrap()
        else {
            panic!("expected :exact")
        };
        assert_eq!(&*resolved, ["a|b", "plain"]);
        assert!(matches!(
            parse(&SearchParamType::string(), "exact", &["plain"]).unwrap(),
            Values::String(StringValues::Exact(Cow::Borrowed(_)))
        ));
    }

    #[test]
    fn references_resolve_their_escapes() {
        let Values::Reference(ReferenceValues::References(references)) = parse(
            &SearchParamType::reference(),
            "",
            &[r"Patient/a\,b", "Patient/c"],
        )
        .unwrap() else {
            panic!("expected references")
        };
        assert_eq!(references[0].resource_type.as_deref(), Some("Patient"));
        assert_eq!(references[0].id, "a,b");
        assert!(matches!(references[1].id, Cow::Borrowed("c")));
    }

    #[test]
    fn missing_is_accepted_for_every_stored_type() {
        assert!(matches!(
            parse(&SearchParamType::reference(), "missing", &["true"]).unwrap(),
            Values::Missing(true)
        ));
        assert!(matches!(
            parse(&SearchParamType::date(), "missing", &["false"]).unwrap(),
            Values::Missing(false)
        ));
        assert!(parse(&SearchParamType::composite(), "missing", &["true"]).is_err());
        assert!(parse(&SearchParamType::date(), "missing", &["maybe"]).is_err());
        assert!(parse(&SearchParamType::date(), "missing", &["true", "false"]).is_err());
    }

    #[test]
    fn type_specific_modifiers_are_limited_to_their_type() {
        assert!(matches!(
            parse(&SearchParamType::token(), "not", &["a"]).unwrap(),
            Values::Token(TokenValues::Not(_))
        ));
        assert!(matches!(
            parse(&SearchParamType::string(), "exact", &["a"]).unwrap(),
            Values::String(StringValues::Exact(_))
        ));
        match parse(&SearchParamType::reference(), "Patient", &["a"]).unwrap() {
            Values::Reference(ReferenceValues::References(references)) => {
                assert_eq!(references[0].resource_type.as_deref(), Some("Patient"));
            }
            other => panic!("`:[type]` is applied to the values, got {other:?}"),
        }
        assert!(parse(&SearchParamType::string(), "not", &["a"]).is_err());
        assert!(parse(&SearchParamType::token(), "exact", &["a"]).is_err());
        assert!(parse(&SearchParamType::reference(), "foo", &["a"]).is_err());
        assert!(parse(&SearchParamType::date(), "exact", &["2020"]).is_err());
    }
}
