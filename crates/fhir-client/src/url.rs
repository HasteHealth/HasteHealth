use thiserror::Error;

#[derive(Debug)]
pub struct Parameter {
    pub name: String,
    pub value: Vec<String>,
    pub modifier: Option<String>,
    pub chains: Option<Vec<String>>,
}

/// Represnet both resource parameters IE Patient.name and
/// result parameters IE _count
#[derive(Debug)]
pub enum ParsedParameter {
    Result(Parameter),
    Resource(Parameter),
}

// // type SPECIAL_CHARACTER = "\\" | "|" | "$" | ",";
// // const SPECIAL_CHARACTERS: SPECIAL_CHARACTER[] = ["\\", "|", "$", ","];

// // /**
// //  * Returns string with split pieces and unescapes special characters from the split piece.
// //  * @param parameter Parameter to be split
// //  * @param specialCharacter One of special characters that get escaped on parameter.
// //  */
// // export function splitParameter(
// //   parameter: string,
// //   specialCharacter: SPECIAL_CHARACTER,
// // ): string[] {
// //   const specialCharEg = new RegExp(`\\${specialCharacter}`, "g");
// //   let prevIndex = -1;
// //   const pieces = [];
// //   let match;

// //   while ((match = specialCharEg.exec(parameter))) {
// //     if (match.index === 0 || parameter[match.index - 1] !== "\\") {
// //       pieces.push(parameter.substring(prevIndex + 1, match.index));
// //       prevIndex = match.index;
// //     }
// //   }
// //   pieces.push(parameter.substring(prevIndex + 1));

// //   return pieces.map(unescapeParameter);
// // }

// // /**
// //  * Escapes a parameter values special characters
// //  * Reference: https://hl7.org/fhir/R4/search.html#escaping
// //  * @param parameter Parameter value to escape
// //  * @returns Escaped Parameter
// //  */
// // export function escapeParameter(parameter: string): string {
// //   return SPECIAL_CHARACTERS.reduce(
// //     (parameter: string, character: string): string => {
// //       return parameter.replaceAll(character, `\\${character}`);
// //     },
// //     parameter,
// //   );
// // }

// // /**
// //  * Unescapes a parameter values special characters.
// //  * Reference: https://hl7.org/fhir/R4/search.html#escaping
// //  * @param parameter Escaped Parameter
// //  * @returns Unescaped Parameter
// //  */
// // export function unescapeParameter(parameter: string): string {
// //   return SPECIAL_CHARACTERS.reduce(
// //     (parameter: string, character: string): string => {
// //       return parameter.replaceAll(`\\${character}`, character);
// //     },
// //     parameter,
// //   );
// // }

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("Error parsing query parameters: '{0}'")]
    InvalidParameter(String),
}

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

pub fn parse_query(query_params: &str) -> Result<Vec<ParsedParameter>, ParseError> {
    query_params
        .split('&')
        .map(|param| {
            let [param_name, value] = param.split('=').collect::<Vec<&str>>()[..] else {
                return Err(ParseError::InvalidParameter(param.to_string()));
            };

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
        .collect()
}
