import { FHIR_VERSION, Resource } from "@haste-health/fhir-types/versions";
import { OperationError, outcomeError } from "@haste-health/operation-outcomes";

type SPECIAL_CHARACTER = "\\" | "|" | "$" | ",";
const SPECIAL_CHARACTERS: SPECIAL_CHARACTER[] = ["\\", "|", "$", ","];

/**
 * Splits on each separator not escaped by a backslash, leaving the pieces' escapes in place.
 * A value without a backslash takes the plain split.
 * @param parameter Parameter to be split
 * @param separator One of special characters that get escaped on parameter.
 */
export function splitUnescaped(
  parameter: string,
  separator: SPECIAL_CHARACTER,
): string[] {
  if (!parameter.includes("\\")) return parameter.split(separator);
  const pieces: string[] = [];
  let start = 0;
  for (let i = 0; i < parameter.length; i++) {
    if (parameter[i] === "\\") i++;
    else if (parameter[i] === separator) {
      pieces.push(parameter.substring(start, i));
      start = i + 1;
    }
  }
  pieces.push(parameter.substring(start));
  return pieces;
}

/**
 * A value's segments between unescaped separators, each with its escapes resolved:
 * `system|code` of a token. The counterpart of `Parameter::segmented` in the Rust client.
 * @param parameter Parameter value as written in the URL
 * @param specialCharacter One of special characters that get escaped on parameter.
 */
export function splitParameter(
  parameter: string,
  specialCharacter: SPECIAL_CHARACTER,
): string[] {
  return splitUnescaped(parameter, specialCharacter).map((piece) =>
    unescapeParameter(piece),
  );
}

/**
 * A parameter's values with their escapes resolved. The counterpart of
 * `Parameter::values` in the Rust client.
 * @param parameter A parsed parameter, whose values are as written in the URL.
 */
export function parameterValues(parameter: ParsedParameter<string>): string[] {
  return parameter.value.map((value) => unescapeParameter(value));
}

/**
 * Escapes a parameter values special characters
 * Reference: https://hl7.org/fhir/R4/search.html#escaping
 * @param parameter Parameter value to escape
 * @returns Escaped Parameter
 */
export function escapeParameter(parameter: string): string {
  return SPECIAL_CHARACTERS.reduce(
    (parameter: string, character: string): string => {
      return parameter.replaceAll(character, `\\${character}`);
    },
    parameter,
  );
}

/**
 * Unescapes a parameter values special characters.
 * Reference: https://hl7.org/fhir/R4/search.html#escaping
 * @param parameter Escaped Parameter
 * @returns Unescaped Parameter
 */
export function unescapeParameter(parameter: string): string {
  if (!parameter.includes("\\")) return parameter;
  // Escapes are read left to right so `\\,` is a backslash then a separator.
  return parameter.replace(/\\([\s\S])/g, (escape, character) =>
    (SPECIAL_CHARACTERS as readonly string[]).includes(character)
      ? character
      : escape,
  );
}

export interface ParsedParameter<T> {
  name: string;
  /**
   * The values as written after `=`, one per unescaped comma, with their escapes
   * (`\,` `\|` `\$` `\\`) still in place: which of `|` and `$` separate is only known
   * with the parameter's type. Read them through `parameterValues` or `splitParameter`.
   */
  value: T[];
  modifier?: string;
  chains?: string[];
}

export interface SearchParameterResource<Version extends FHIR_VERSION>
  extends ParsedParameter<string | number> {
  type: "resource";
  searchParameter: Resource<Version, "SearchParameter">;
  chainedParameters?: Resource<Version, "SearchParameter">[][];
}

export interface SearchParameterResult
  extends ParsedParameter<string | number> {
  type: "result";
}

export type MetaParameter<Version extends FHIR_VERSION> =
  | SearchParameterResource<Version>
  | SearchParameterResult;

export type Parameters<Version extends FHIR_VERSION> =
  | ParsedParameter<string | number>[]
  | MetaParameter<Version>[];

/**
 * Given a query string create complex FHIR Query object.
 * @param queryParams Raw query parameters pulled off url
 * @returns Record of parsed parameters with name modifier and value.
 */
export function parseQuery(
  queryParams: string | undefined,
): ParsedParameter<string>[] {
  const parameters = !queryParams
    ? []
    : queryParams
        .split("&")
        .map((param) => param.split("="))
        .reduce(
          (
            parameters,
            [key, value],
          ): Record<string, ParsedParameter<string>> => {
            const chains = key.split(".");

            const [name, modifier] = chains[0].split(":");

            const searchParam: ParsedParameter<string> = {
              name,
              modifier,
              // The pieces keep their escapes; see ParsedParameter.value.
              value: splitUnescaped(value, ",").map((v) =>
                decodeURIComponent(v),
              ),
            };

            if (chains.length > 1) searchParam.chains = chains.slice(1);
            if (modifier) searchParam.modifier = modifier;

            return { ...parameters, [searchParam.name]: searchParam };
          },
          {},
        );

  return Object.values(parameters);
}

/**
 * Given a url string parsequery parameters.
 * @param url Any url to parse out query parameters.
 * @returns Record of parsed parameters with name modifier and value.
 */
export default function parseUrl(
  url: string,
): ParsedParameter<string | number>[] {
  const chunks = url.split("?");
  if (chunks.length > 2)
    throw new OperationError(outcomeError("invalid", "Invalid query string"));
  const [_, queryParams] = chunks;
  return parseQuery(queryParams);
}
