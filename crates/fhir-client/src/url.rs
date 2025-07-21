// use http::Uri;
// // import { FHIR_VERSION, Resource } from "@iguhealth/fhir-types/versions";
// // import { OperationError, outcomeError } from "@iguhealth/operation-outcomes";

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

// pub struct ParsedParameter {
//     name: String,
//     value: Vec<String>,
//     modifier: Option<String>,
//     chains: Option<Vec<String>>,
// }

// // impl ParsedParameter {
// //     /// Attempts to construct a [`ParsedParameter`] from a reference to a [`Uri`].
// //     ///
// //     /// # Example
// //     /// ```
// //     /// use axum::extract::Query;
// //     /// use http::Uri;
// //     /// use serde::Deserialize;
// //     ///
// //     /// #[derive(Deserialize)]
// //     /// struct ExampleParams {
// //     ///     foo: String,
// //     ///     bar: u32,
// //     /// }
// //     ///
// //     /// let uri: Uri = "http://example.com/path?foo=hello&bar=42".parse().unwrap();
// //     /// let result: Query<ExampleParams> = Query::try_from_uri(&uri).unwrap();
// //     /// assert_eq!(result.foo, String::from("hello"));
// //     /// assert_eq!(result.bar, 42);
// //     /// ```
// //     pub fn try_from_uri(value: &Uri) -> Result<Self, QueryRejection> {
// //         let query = value.query().unwrap_or_default();
// //         let k = query.split('&').into_iter();
// //         Ok(Query(params))
// //     }
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

// // export interface SearchParameterResource<Version extends FHIR_VERSION>
// //   extends ParsedParameter<string | number> {
// //   type: "resource";
// //   searchParameter: Resource<Version, "SearchParameter">;
// //   chainedParameters?: Resource<Version, "SearchParameter">[][];
// // }

// // export interface SearchParameterResult
// //   extends ParsedParameter<string | number> {
// //   type: "result";
// // }

// // export type MetaParameter<Version extends FHIR_VERSION> =
// //   | SearchParameterResource<Version>
// //   | SearchParameterResult;

// // export type Parameters<Version extends FHIR_VERSION> =
// //   | ParsedParameter<string | number>[]
// //   | MetaParameter<Version>[];

// // /**
// //  * Given a query string create complex FHIR Query object.
// //  * @param queryParams Raw query parameters pulled off url
// //  * @returns Record of parsed parameters with name modifier and value.
// //  */
// // export function parseQuery(
// //   queryParams: string | undefined,
// // ): ParsedParameter<string>[] {
// //   const parameters = !queryParams
// //     ? []
// //     : queryParams
// //         .split("&")
// //         .map((param) => param.split("="))
// //         .reduce(
// //           (
// //             parameters,
// //             [key, value],
// //           ): Record<string, ParsedParameter<string>> => {
// //             const chains = key.split(".");

// //             const [name, modifier] = chains[0].split(":");

// //             const searchParam: ParsedParameter<string> = {
// //               name,
// //               modifier,
// //               value: value.split(",").map((v) => decodeURIComponent(v)),
// //             };

// //             if (chains.length > 1) searchParam.chains = chains.slice(1);
// //             if (modifier) searchParam.modifier = modifier;

// //             return { ...parameters, [searchParam.name]: searchParam };
// //           },
// //           {},
// //         );

// //   return Object.values(parameters);
// // }

// // /**
// //  * Given a url string parsequery parameters.
// //  * @param url Any url to parse out query parameters.
// //  * @returns Record of parsed parameters with name modifier and value.
// //  */
// // export default function parseUrl(
// //   url: string,
// // ): ParsedParameter<string | number>[] {
// //   const chunks = url.split("?");
// //   if (chunks.length > 2)
// //     throw new OperationError(outcomeError("invalid", "Invalid query string"));
// //   const [_, queryParams] = chunks;
// //   return parseQuery(queryParams);
// // }
