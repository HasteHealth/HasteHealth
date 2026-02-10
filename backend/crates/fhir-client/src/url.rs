use haste_fhir_operation_error::derive::OperationOutcomeError;
use haste_reflect::{MetaValue, derive::Reflect};
use std::{collections::HashMap, fmt::Display};

#[derive(Debug, Clone, Reflect)]
pub struct Parameter {
    pub name: String,
    pub value: Vec<String>,
    pub modifier: Option<String>,
    pub chains: Option<Vec<String>>,
}

/// Represnet both resource parameters IE Patient.name and
/// result parameters IE _count
#[derive(Debug, Clone)]
pub enum ParsedParameter {
    Result(Parameter),
    Resource(Parameter),
}

impl ParsedParameter {
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

    fn get_index<'a>(&'a self, index: usize) -> Option<&'a dyn MetaValue> {
        match self {
            ParsedParameter::Result(p) | ParsedParameter::Resource(p) => p.get_index(index),
        }
    }

    fn get_index_mut<'a>(&'a mut self, index: usize) -> Option<&'a mut dyn MetaValue> {
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

    fn typename(&self) -> &'static str {
        match self {
            ParsedParameter::Result(p) | ParsedParameter::Resource(p) => p.typename(),
        }
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
                write!(f, "Invalid query parameter: {}", param)
            }
        }
    }
}

impl std::error::Error for ParseError {}

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
];

#[derive(Debug, Clone)]
pub struct ParsedParameters(Vec<ParsedParameter>);

impl ParsedParameters {
    pub fn new(params: Vec<ParsedParameter>) -> Self {
        Self(params)
    }
    pub fn parameters<'a>(&'a self) -> &'a Vec<ParsedParameter> {
        &self.0
    }
    pub fn owned_parameters<'a>(self) -> Vec<ParsedParameter> {
        self.0
    }
    pub fn get<'a>(&'a self, name: &str) -> Option<&'a ParsedParameter> {
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

    fn get_index<'a>(&'a self, _index: usize) -> Option<&'a dyn MetaValue> {
        None
    }

    fn get_index_mut<'a>(&'a mut self, _index: usize) -> Option<&'a mut dyn MetaValue> {
        None
    }

    fn flatten(&self) -> Vec<&dyn MetaValue> {
        vec![]
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn typename(&self) -> &'static str {
        "ParsedParameters"
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

        let query_map = query_string.split('&').fold(
            Ok(HashMap::new()),
            |acc: Result<HashMap<String, String>, ParseError>, pair| {
                let mut map = acc?;
                let mut split = pair.splitn(2, '=');
                let key = split
                    .next()
                    .ok_or_else(|| ParseError::InvalidParameter(pair.to_string()))?;
                let value = split
                    .next()
                    .ok_or_else(|| ParseError::InvalidParameter(pair.to_string()))?;
                map.insert(key.to_string(), value.to_string());
                Ok(map)
            },
        )?;

        Self::try_from(&query_map)
    }
}

impl TryFrom<&HashMap<String, String>> for ParsedParameters {
    type Error = ParseError;
    fn try_from(query_params: &HashMap<String, String>) -> Result<Self, ParseError> {
        if query_params.is_empty() {
            return Ok(Self(vec![]));
        }

        let params = query_params
            .keys()
            .map(|param_name| {
                let value = query_params.get(param_name).unwrap();

                let chain = param_name
                    .split('.')
                    .map(|s| s.to_string())
                    .collect::<Vec<String>>();

                if chain.is_empty() {
                    return Err(ParseError::InvalidParameter(param_name.to_string()));
                }

                let name_and_modifier = chain[0].split(':').collect::<Vec<&str>>();

                if name_and_modifier.len() > 2 || name_and_modifier.is_empty() {
                    return Err(ParseError::InvalidParameter(param_name.to_string()));
                }

                let name = name_and_modifier[0].to_string();

                let param = Parameter {
                    name,
                    modifier: name_and_modifier.get(1).map(|s| s.to_string()),
                    value: value.split(',').map(|v| v.to_string()).collect(),
                    chains: if chain.len() > 1 {
                        Some(chain[1..].to_vec())
                    } else {
                        None
                    },
                };

                if RESULT_PARAMETERS.contains(&param.name.as_str()) {
                    Ok(ParsedParameter::Result(param))
                } else {
                    Ok(ParsedParameter::Resource(param))
                }
            })
            .collect::<Result<Vec<ParsedParameter>, ParseError>>()?;

        Ok(Self(params))
    }
}

pub fn parse_prefix<'a>(v: &'a str) -> (Option<&'a str>, &'a str) {
    if v.len() < 3 {
        return (None, v);
    }

    let sub_str = &v[..2];
    let remainder = &v[2..];

    match sub_str {
        "lt" => (Some(sub_str), remainder),
        "le" => (Some(sub_str), remainder),
        "gt" => (Some(sub_str), remainder),
        "ge" => (Some(sub_str), remainder),
        "eq" => (Some(sub_str), remainder),
        "ne" => (Some(sub_str), remainder),
        _ => (None, v),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_prefix() {
        let cases = vec![
            ("lt5.0", (Some("lt"), "5.0")),
            ("le10", (Some("le"), "10")),
            ("gt3.14", (Some("gt"), "3.14")),
            ("ge2.71", (Some("ge"), "2.71")),
            ("eq42", (Some("eq"), "42")),
            ("ne0", (Some("ne"), "0")),
            ("5.0", (None, "5.0")),
            ("10", (None, "10")),
        ];

        for (input, expected) in cases {
            let result = parse_prefix(input);
            assert_eq!(result, expected, "Failed for input: {}", input);
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
                assert!(param.chains.is_none());
            }
            _ => panic!("Expected Resource parameter"),
        }

        match parsed_params.get("_count") {
            Some(ParsedParameter::Result(param)) => {
                assert_eq!(param.name, "_count");
                assert_eq!(param.value, vec!["10"]);
                assert!(param.modifier.is_none());
                assert!(param.chains.is_none());
            }
            _ => panic!("Expected Result parameter"),
        }

        match parsed_params.get("address") {
            Some(ParsedParameter::Resource(param)) => {
                assert_eq!(param.name, "address");
                assert_eq!(param.value, vec!["NewYork"]);
                assert!(param.modifier.is_none());
                assert_eq!(param.chains, Some(vec!["city".to_string()]));
            }
            _ => panic!("Expected Resource parameter"),
        }

        match parsed_params.get("status") {
            Some(ParsedParameter::Resource(param)) => {
                assert_eq!(param.name, "status");
                assert_eq!(param.value, vec!["active"]);
                assert_eq!(param.modifier, Some("exact".to_string()));
                assert!(param.chains.is_none());
            }
            _ => panic!("Expected Resource parameter"),
        }
    }
}
