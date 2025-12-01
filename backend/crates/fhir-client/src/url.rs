use haste_fhir_operation_error::derive::OperationOutcomeError;
use std::{collections::HashMap, fmt::Display};

#[derive(Debug, Clone)]
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
];

#[derive(Debug, Clone)]
pub struct ParsedParameters(Vec<ParsedParameter>);

impl ParsedParameters {
    pub fn new(params: Vec<ParsedParameter>) -> Self {
        Self(params)
    }
    pub fn parameters(&self) -> &Vec<ParsedParameter> {
        &self.0
    }
    pub fn get(&self, name: &str) -> Option<&ParsedParameter> {
        self.0.iter().find(|p| match p {
            ParsedParameter::Resource(param) | ParsedParameter::Result(param) => param.name == name,
        })
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

#[cfg(test)]
mod tests {
    use super::*;

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
