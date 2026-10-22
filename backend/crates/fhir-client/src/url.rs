use derivative::Derivative;
use haste_fhir_operation_error::derive::OperationOutcomeError;
use haste_reflect::{MetaValue, derive::Reflect};
use std::{borrow::Cow, collections::HashMap, fmt::Display};

static RESULT_PARAMETERS: &[&str] = &[
    "_count",
    "_offset",
    "_total",
    "_sort",
    "_include",
    "_revinclude",
    "_summary",
    "_elements",
    "_contained",
    "_containedType",
    "_since",
    "_type",
];

/// One `name[:modifier]` segment of a chained parameter's name.
#[derive(Debug, Clone, PartialEq, Eq, Reflect)]
#[fhir_type = "BackboneElement"]
pub struct ChainLink {
    pub name: String,
    pub modifier: Option<String>,
}

impl ChainLink {
    /// Parses one segment, rejecting an empty name or modifier and a second
    /// modifier (`:b`, `a:`, `a:b:c`).
    fn parse(segment: &str) -> Result<Self, ParseError> {
        let (name, modifier) = match segment.split_once(':') {
            Some((name, modifier)) => (name, Some(modifier)),
            None => (segment, None),
        };

        if name.is_empty() || modifier.is_some_and(|m| m.is_empty() || m.contains(':')) {
            return Err(ParseError::InvalidParameter(segment.to_string()));
        }

        Ok(ChainLink {
            name: name.to_string(),
            modifier: modifier.map(str::to_string),
        })
    }
}

/// A search parameter as written in the URL.
///
/// `name` and `modifier` are the first segment and `chains` the rest, each
/// with its own modifier: `subject:Patient.name:exact` is `subject` with
/// modifier `Patient`, then `name` with modifier `exact`.
#[derive(Derivative, Clone, Reflect)]
#[fhir_type = "BackboneElement"]
#[derivative(Debug)]
pub struct Parameter {
    pub name: String,
    /// The values as written after `=`, one per unescaped comma, with their
    /// escapes (`\,` `\|` `\$` `\\`) still in place. Read them through
    /// [`Parameter::values`] or [`Parameter::segmented`], which resolve the
    /// escapes once the parameter's type says what a `|` or `$` means; build
    /// one from literal text with [`escape`].
    #[derivative(Debug(format_with = "crate::redact"))]
    pub value: Vec<String>,
    pub modifier: Option<String>,
    pub chains: Vec<ChainLink>,
}

/// Whether `value` holds an escape to resolve: one memchr, the common case
/// being none.
fn has_escape(value: &str) -> bool {
    value.contains('\\')
}

/// `value` with every escape (`\,` `\|` `\$` `\\`) resolved to its literal
/// character (<https://hl7.org/fhir/R4/search.html#escaping>); borrowed when
/// it has none. A backslash before any other character stays as written.
fn unescape(value: &str) -> Cow<'_, str> {
    if !has_escape(value) {
        return Cow::Borrowed(value);
    }
    let mut out = String::with_capacity(value.len());
    let mut chars = value.chars();
    while let Some(c) = chars.next() {
        if c != '\\' {
            out.push(c);
        } else if let Some(next) = chars.next() {
            if !matches!(next, ',' | '|' | '$' | '\\') {
                out.push('\\');
            }
            out.push(next);
        } else {
            out.push(c);
        }
    }
    Cow::Owned(out)
}

/// `literal` escaped so that it reads back as itself: the inverse of what
/// [`Parameter::values`] resolves. For building a parameter from text that
/// was never in a URL.
#[must_use]
pub fn escape(literal: &str) -> Cow<'_, str> {
    if !literal.contains([',', '|', '$', '\\']) {
        return Cow::Borrowed(literal);
    }
    let mut out = String::with_capacity(literal.len() + 2);
    for c in literal.chars() {
        if matches!(c, ',' | '|' | '$' | '\\') {
            out.push('\\');
        }
        out.push(c);
    }
    Cow::Owned(out)
}

/// One value's segments between unescaped `separator`s, each with its escapes
/// resolved: `system|code` of a token, `value|system|code` of a quantity.
///
/// A segment borrows from the value unless it held an escape. An empty value
/// is one empty segment, as with `str::split`.
pub struct Segments<'a> {
    source: &'a str,
    rest: Option<&'a str>,
    separator: char,
    escaped: bool,
}

impl<'a> Segments<'a> {
    /// The segments of `value` as written in a URL.
    #[must_use]
    pub fn of(value: &'a str, separator: char) -> Self {
        Segments {
            source: value,
            rest: Some(value),
            separator,
            escaped: has_escape(value),
        }
    }

    /// The value as written, for an error that names it.
    #[must_use]
    pub fn source(&self) -> &'a str {
        self.source
    }

    /// The pieces as written, their escapes still in place.
    fn raw(mut self) -> impl Iterator<Item = &'a str> {
        std::iter::from_fn(move || self.next_raw())
    }

    /// The next raw piece, its escapes still in place. A value without a
    /// backslash splits with `str::find`.
    fn next_raw(&mut self) -> Option<&'a str> {
        let current = self.rest?;
        let split = if self.escaped {
            let mut escaped = false;
            current.char_indices().find_map(|(i, c)| {
                if escaped {
                    escaped = false;
                    None
                } else if c == '\\' {
                    escaped = true;
                    None
                } else {
                    (c == self.separator).then_some(i)
                }
            })
        } else {
            current.find(self.separator)
        };
        if let Some(i) = split {
            self.rest = Some(&current[i + self.separator.len_utf8()..]);
            Some(&current[..i])
        } else {
            self.rest = None;
            Some(current)
        }
    }
}

impl<'a> Iterator for Segments<'a> {
    type Item = Cow<'a, str>;

    fn next(&mut self) -> Option<Self::Item> {
        let raw = self.next_raw()?;
        Some(if self.escaped {
            unescape(raw)
        } else {
            Cow::Borrowed(raw)
        })
    }
}

impl Parameter {
    /// Whether this parameter follows a reference (`subject.name`).
    #[must_use]
    pub fn is_chained(&self) -> bool {
        !self.chains.is_empty()
    }

    /// The parameter's name as written in a URL: `subject:Patient.name:exact`.
    #[must_use]
    pub fn key(&self) -> String {
        let segments = std::iter::once((&self.name, &self.modifier))
            .chain(self.chains.iter().map(|link| (&link.name, &link.modifier)));

        let mut key = String::new();
        for (i, (name, modifier)) in segments.enumerate() {
            if i > 0 {
                key.push('.');
            }
            key.push_str(name);
            if let Some(modifier) = modifier {
                key.push(':');
                key.push_str(modifier);
            }
        }
        key
    }

    /// The values as written after `=` in a URL: the pieces joined by commas,
    /// their escapes intact, so a parsed query serialises to what was read.
    #[must_use]
    pub fn value_string(&self) -> String {
        self.value.join(",")
    }

    /// Every value with its escapes resolved. Borrowed from the parameter
    /// when none has an escape, the common case.
    #[must_use]
    pub fn values(&self) -> Cow<'_, [String]> {
        if self.value.iter().any(|value| has_escape(value)) {
            Cow::Owned(
                self.value
                    .iter()
                    .map(|value| unescape(value).into_owned())
                    .collect(),
            )
        } else {
            Cow::Borrowed(&self.value)
        }
    }

    /// Each value with its escapes resolved, borrowed when it has none. For
    /// a parser that keeps a slice of the value.
    pub fn each_value(&self) -> impl Iterator<Item = Cow<'_, str>> {
        self.value.iter().map(|value| unescape(value))
    }

    /// Each value split on unescaped `separator`s, every segment with its
    /// escapes resolved: `'|'` for a token's `system|code`.
    pub fn segmented(&self, separator: char) -> impl Iterator<Item = Segments<'_>> {
        self.value
            .iter()
            .map(move |value| Segments::of(value, separator))
    }

    fn parse(key: &str, value: &str) -> Result<Self, ParseError> {
        let mut segments = key.split('.').map(ChainLink::parse);
        // `split` always yields at least one segment.
        let first = segments
            .next()
            .ok_or_else(|| ParseError::InvalidParameter(key.to_string()))??;

        Ok(Parameter {
            name: first.name,
            modifier: first.modifier,
            // The pieces keep their escapes: which of `|` and `$` separate is
            // only known with the parameter's type.
            value: Segments::of(value, ',').raw().map(str::to_string).collect(),
            chains: segments.collect::<Result<_, _>>()?,
        })
    }
}

impl From<(String, Vec<String>)> for Parameter {
    fn from(tuple: (String, Vec<String>)) -> Self {
        Parameter {
            name: tuple.0,
            value: tuple.1,
            modifier: None,
            chains: Vec::new(),
        }
    }
}

impl From<(String, Vec<String>, String)> for Parameter {
    fn from(tuple: (String, Vec<String>, String)) -> Self {
        Parameter {
            name: tuple.0,
            value: tuple.1,
            modifier: Some(tuple.2),
            chains: Vec::new(),
        }
    }
}

/// Represent both resource parameters IE Patient.name and
/// result parameters IE _count
#[derive(Derivative, Clone)]
#[derivative(Debug = "transparent")]
pub enum ParsedParameter {
    #[derivative(Debug = "transparent")]
    Result(Parameter),
    #[derivative(Debug = "transparent")]
    Resource(Parameter),
}

impl<T: Into<Parameter>> From<T> for ParsedParameter {
    fn from(param: T) -> Self {
        let param = param.into();
        if RESULT_PARAMETERS.contains(&param.name.as_str()) {
            ParsedParameter::Result(param)
        } else {
            ParsedParameter::Resource(param)
        }
    }
}

impl ParsedParameter {
    #[must_use]
    pub fn name(&self) -> &str {
        match self {
            ParsedParameter::Result(p) | ParsedParameter::Resource(p) => &p.name,
        }
    }
}

impl MetaValue for ParsedParameter {
    fn fields(&self) -> Vec<&'static str> {
        match self {
            ParsedParameter::Result(p) | ParsedParameter::Resource(p) => p.fields(),
        }
    }

    fn get_field<'a>(&'a self, field: &str) -> Option<&'a dyn MetaValue> {
        match self {
            ParsedParameter::Result(p) | ParsedParameter::Resource(p) => p.get_field(field),
        }
    }

    fn get_field_mut<'a>(&'a mut self, field: &str) -> Option<&'a mut dyn MetaValue> {
        match self {
            ParsedParameter::Result(p) | ParsedParameter::Resource(p) => p.get_field_mut(field),
        }
    }

    fn get_index(&self, index: usize) -> Option<&dyn MetaValue> {
        match self {
            ParsedParameter::Result(p) | ParsedParameter::Resource(p) => p.get_index(index),
        }
    }

    fn get_index_mut(&mut self, index: usize) -> Option<&mut dyn MetaValue> {
        match self {
            ParsedParameter::Result(p) | ParsedParameter::Resource(p) => p.get_index_mut(index),
        }
    }

    fn flatten(&self) -> Vec<&dyn MetaValue> {
        match self {
            ParsedParameter::Result(p) | ParsedParameter::Resource(p) => p.flatten(),
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        match self {
            ParsedParameter::Result(p) | ParsedParameter::Resource(p) => p.as_any(),
        }
    }

    fn fhir_type(&self) -> &'static str {
        match self {
            ParsedParameter::Result(p) | ParsedParameter::Resource(p) => p.fhir_type(),
        }
    }

    fn is_many(&self) -> bool {
        false
    }
}

#[derive(Debug, OperationOutcomeError)]
pub enum ParseError {
    #[fatal(
        code = "invalid",
        diagnostic = "Error parsing query parameters '{arg0}'"
    )]
    InvalidParameter(String),
}

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::InvalidParameter(param) => {
                write!(f, "Invalid query parameter: {param}")
            }
        }
    }
}

impl std::error::Error for ParseError {}

#[derive(Derivative, Clone)]
#[derivative(Debug = "transparent")]
pub struct ParsedParameters(Vec<ParsedParameter>);

impl<T: Into<ParsedParameter>> From<Vec<T>> for ParsedParameters {
    fn from(params: Vec<T>) -> Self {
        let parsed_params = params.into_iter().map(std::convert::Into::into).collect();
        ParsedParameters(parsed_params)
    }
}

impl ParsedParameters {
    #[must_use]
    pub fn new(params: Vec<ParsedParameter>) -> Self {
        Self(params)
    }
    #[must_use]
    pub fn parameters(&self) -> &Vec<ParsedParameter> {
        &self.0
    }
    #[must_use]
    pub fn owned_parameters(self) -> Vec<ParsedParameter> {
        self.0
    }
    #[must_use]
    pub fn get(&self, name: &str) -> Option<&ParsedParameter> {
        self.0.iter().find(|p| match p {
            ParsedParameter::Resource(param) | ParsedParameter::Result(param) => param.name == name,
        })
    }
}

impl MetaValue for ParsedParameters {
    fn fields(&self) -> Vec<&'static str> {
        todo!()
    }

    fn get_field<'a>(&'a self, field: &str) -> Option<&'a dyn MetaValue> {
        if let Some(p) = self.get(field) {
            Some(p)
        } else {
            None
        }
    }

    fn get_field_mut<'a>(&'a mut self, _field: &str) -> Option<&'a mut dyn MetaValue> {
        None
    }

    fn get_index(&self, _index: usize) -> Option<&dyn MetaValue> {
        None
    }

    fn get_index_mut(&mut self, _index: usize) -> Option<&mut dyn MetaValue> {
        None
    }

    fn flatten(&self) -> Vec<&dyn MetaValue> {
        vec![]
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn fhir_type(&self) -> &'static str {
        "BackboneElement"
    }

    fn is_many(&self) -> bool {
        true
    }
}

impl TryFrom<&str> for ParsedParameters {
    type Error = ParseError;
    fn try_from(query_string: &str) -> Result<Self, ParseError> {
        let mut query_string = query_string;
        if query_string.is_empty() {
            return Ok(Self(vec![]));
        }

        if query_string.starts_with('?') {
            query_string = &query_string[1..];
        }

        let pairs = query_string
            .split('&')
            .map(|pair| {
                pair.split_once('=')
                    .ok_or_else(|| ParseError::InvalidParameter(pair.to_string()))
            })
            .collect::<Result<Vec<(&str, &str)>, ParseError>>()?;

        Self::try_from(pairs.as_slice())
    }
}

impl TryFrom<&HashMap<String, String>> for ParsedParameters {
    type Error = ParseError;
    fn try_from(query_params: &HashMap<String, String>) -> Result<Self, ParseError> {
        let pairs: Vec<(&String, &String)> = query_params.iter().collect();

        Self::try_from(pairs.as_slice())
    }
}

/// Query pairs in order. A name that repeats stays one parameter per pair,
/// which a search reads as every one having to match; a map cannot carry a
/// repeat, a slice can.
impl<Name: AsRef<str>, Value: AsRef<str>> TryFrom<&[(Name, Value)]> for ParsedParameters {
    type Error = ParseError;
    fn try_from(pairs: &[(Name, Value)]) -> Result<Self, ParseError> {
        let params = pairs
            .iter()
            .map(|(name, value)| {
                Parameter::parse(name.as_ref(), value.as_ref()).map(ParsedParameter::from)
            })
            .collect::<Result<Vec<ParsedParameter>, ParseError>>()?;

        Ok(Self(params))
    }
}

/// The comparison prefix of a number, date or quantity value. A value
/// without one compares as `eq`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Prefix {
    Eq,
    Ne,
    Gt,
    Lt,
    Ge,
    Le,
    Sa,
    Eb,
    Ap,
}

impl Prefix {
    /// The prefix `code` names, if any.
    #[must_use]
    pub fn parse(code: &str) -> Option<Self> {
        Some(match code {
            "eq" => Prefix::Eq,
            "ne" => Prefix::Ne,
            "gt" => Prefix::Gt,
            "lt" => Prefix::Lt,
            "ge" => Prefix::Ge,
            "le" => Prefix::Le,
            "sa" => Prefix::Sa,
            "eb" => Prefix::Eb,
            "ap" => Prefix::Ap,
            _ => return None,
        })
    }
}

/// Splits the comparison prefix off a number, date or quantity value. A
/// value without one compares as `eq`; a bare prefix is a value.
#[must_use]
pub fn parse_prefix(value: &str) -> (Prefix, &str) {
    match (value.get(..2).and_then(Prefix::parse), value.get(2..)) {
        (Some(prefix), Some(rest)) if !rest.is_empty() => (prefix, rest),
        _ => (Prefix::Eq, value),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_prefix() {
        let cases = vec![
            ("lt5.0", (Prefix::Lt, "5.0")),
            ("le10", (Prefix::Le, "10")),
            ("gt3.14", (Prefix::Gt, "3.14")),
            ("ge2.71", (Prefix::Ge, "2.71")),
            ("eq42", (Prefix::Eq, "42")),
            ("ne0", (Prefix::Ne, "0")),
            ("sa2020", (Prefix::Sa, "2020")),
            ("eb2020-01", (Prefix::Eb, "2020-01")),
            ("ap70", (Prefix::Ap, "70")),
            ("ne", (Prefix::Eq, "ne")),
            ("é5", (Prefix::Eq, "é5")),
            ("5.0", (Prefix::Eq, "5.0")),
            ("10", (Prefix::Eq, "10")),
        ];

        for (input, expected) in cases {
            let result = parse_prefix(input);
            assert_eq!(result, expected, "Failed for input: {input}");
        }
    }

    #[test]
    fn test_split_unescaped() {
        let split = |v| Segments::of(v, ',').raw().collect::<Vec<_>>();
        assert_eq!(split("a,b"), ["a", "b"]);
        assert_eq!(split(r"a\,b"), [r"a\,b"]);
        assert_eq!(split(r"a\\,b"), [r"a\\", "b"]);
        assert_eq!(split(""), [""]);
        assert_eq!(split("é,ü"), ["é", "ü"]);
    }

    #[test]
    fn test_unescape() {
        assert_eq!(unescape(r"a\,b\|c\$d\\e"), r"a,b|c$d\e");
        assert_eq!(unescape(r"\x"), r"\x");
        assert_eq!(unescape("plain"), "plain");
        assert_eq!(unescape("trailing\\"), "trailing\\");
        assert!(matches!(unescape("plain"), Cow::Borrowed(_)));
    }

    #[test]
    fn test_escape_round_trips() {
        for literal in ["plain", "a,b|c$d\\e", r"\x", "é|ü"] {
            assert_eq!(unescape(&escape(literal)), literal);
        }
        assert!(matches!(escape("plain"), Cow::Borrowed(_)));
        assert_eq!(escape(r"a\,b"), r"a\\\,b");
    }

    #[test]
    fn test_segments() {
        let segments = |v| Segments::of(v, '|').collect::<Vec<_>>();
        assert_eq!(segments("a|b"), ["a", "b"]);
        assert_eq!(segments(r"a\|b|c"), ["a|b", "c"]);
        assert_eq!(segments(r"x\\|y\|z"), [r"x\", "y|z"]);
        assert_eq!(segments("|"), ["", ""]);
        assert_eq!(segments(""), [""]);
        assert_eq!(segments("é|ü"), ["é", "ü"]);
        // Without an escape every segment borrows.
        assert!(Segments::of("a|b", '|').all(|segment| matches!(segment, Cow::Borrowed(_))));
        assert_eq!(Segments::of("a|b", '|').source(), "a|b");
    }

    #[test]
    fn test_parse_escaped_values() {
        let parameter = |q| {
            let params = ParsedParameters::try_from(q).unwrap();
            let ParsedParameter::Resource(p) = params.get("p").unwrap() else {
                panic!("expected a resource parameter")
            };
            p.clone()
        };
        // The pieces keep their escapes.
        assert_eq!(parameter(r"p=Smith\,Jones").value, [r"Smith\,Jones"]);
        assert_eq!(parameter("p=Smith,Jones").value, ["Smith", "Jones"]);
        assert_eq!(
            parameter(r"p=http://a\|b|1,x").value,
            [r"http://a\|b|1", "x"]
        );
        assert_eq!(parameter(r"p=a\\,b").value, [r"a\\", "b"]);

        // The accessors resolve them.
        assert_eq!(&*parameter(r"p=Smith\,Jones").values(), ["Smith,Jones"]);
        assert_eq!(&*parameter(r"p=a\\,b").values(), [r"a\", "b"]);
        assert!(matches!(
            parameter("p=Smith,Jones").values(),
            Cow::Borrowed(_)
        ));
        assert_eq!(
            parameter(r"p=a\,b,c").each_value().collect::<Vec<_>>(),
            ["a,b", "c"]
        );
        assert_eq!(
            parameter(r"p=http://a\|b|1,x")
                .segmented('|')
                .map(Iterator::collect::<Vec<_>>)
                .collect::<Vec<_>>(),
            [vec!["http://a|b", "1"], vec!["x"]]
        );
    }

    #[test]
    fn test_value_string_round_trips() {
        for query in [
            r"p=Smith\,Jones",
            "p=Smith,Jones",
            r"p=a\|b|1,x\,y",
            r"p=a\\,b",
        ] {
            let params = ParsedParameters::try_from(query).unwrap();
            let ParsedParameter::Resource(p) = params.get("p").unwrap() else {
                panic!("expected a resource parameter")
            };
            assert_eq!(format!("p={}", p.value_string()), query);
        }
    }

    #[test]
    fn test_parse_parameters() {
        let query_string = "?name=John,Doe&_count=10&address.city=NewYork&status:exact=active";
        let parsed_params = ParsedParameters::try_from(query_string).unwrap();

        assert_eq!(parsed_params.parameters().len(), 4);

        match parsed_params.get("name") {
            Some(ParsedParameter::Resource(param)) => {
                assert_eq!(param.name, "name");
                assert_eq!(param.value, vec!["John", "Doe"]);
                assert!(param.modifier.is_none());
                assert!(param.chains.is_empty());
            }
            _ => panic!("Expected Resource parameter"),
        }

        match parsed_params.get("_count") {
            Some(ParsedParameter::Result(param)) => {
                assert_eq!(param.name, "_count");
                assert_eq!(param.value, vec!["10"]);
                assert!(param.modifier.is_none());
                assert!(param.chains.is_empty());
            }
            _ => panic!("Expected Result parameter"),
        }

        match parsed_params.get("address") {
            Some(ParsedParameter::Resource(param)) => {
                assert_eq!(param.name, "address");
                assert_eq!(param.value, vec!["NewYork"]);
                assert!(param.modifier.is_none());
                assert_eq!(
                    param.chains,
                    vec![ChainLink {
                        name: "city".to_string(),
                        modifier: None
                    }]
                );
            }
            _ => panic!("Expected Resource parameter"),
        }

        match parsed_params.get("status") {
            Some(ParsedParameter::Resource(param)) => {
                assert_eq!(param.name, "status");
                assert_eq!(param.value, vec!["active"]);
                assert_eq!(param.modifier, Some("exact".to_string()));
                assert!(param.chains.is_empty());
            }
            _ => panic!("Expected Resource parameter"),
        }
    }
}
