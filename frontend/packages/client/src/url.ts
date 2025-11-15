import { FHIR_VERSION, Resource } from "@haste-health/fhir-types/versions";
import { OperationError, outcomeError } from "@haste-health/operation-outcomes";

type SPECIAL_CHARACTER = "\\" | "|" | "$" | ",";
const SPECIAL_CHARACTERS: SPECIAL_CHARACTER[] = ["\\", "|", "$", ","];

/**
 * Returns string with split pieces and unescapes special characters from the split piece.
 * @param parameter Parameter to be split
 * @param specialCharacter One of special characters that get escaped on parameter.
 */
export function splitParameter(
  parameter: string,
  specialCharacter: SPECIAL_CHARACTER
): string[] {
  const specialCharEg = new RegExp(`\\${specialCharacter}`, "g");
  let prevIndex = -1;
  const pieces = [];
  let match;

  while ((match = specialCharEg.exec(parameter))) {
    if (match.index === 0 || parameter[match.index - 1] !== "\\") {
      pieces.push(parameter.substring(prevIndex + 1, match.index));
      prevIndex = match.index;
    }
  }
  pieces.push(parameter.substring(prevIndex + 1));

  return pieces.map(unescapeParameter);
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
    parameter
  );
}

/**
 * Unescapes a parameter values special characters.
 * Reference: https://hl7.org/fhir/R4/search.html#escaping
 * @param parameter Escaped Parameter
 * @returns Unescaped Parameter
 */
export function unescapeParameter(parameter: string): string {
  return SPECIAL_CHARACTERS.reduce(
    (parameter: string, character: string): string => {
      return parameter.replaceAll(`\\${character}`, character);
    },
    parameter
  );
}

export interface ParsedParameter<T> {
  name: string;
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
  queryParams: string | undefined
): ParsedParameter<string>[] {
  const parameters = !queryParams
    ? []
    : queryParams
        .split("&")
        .map((param) => param.split("="))
        .reduce(
          (
            parameters,
            [key, value]
          ): Record<string, ParsedParameter<string>> => {
            const chains = key.split(".");

            const [name, modifier] = chains[0].split(":");

            const searchParam: ParsedParameter<string> = {
              name,
              modifier,
              value: value.split(",").map((v) => decodeURIComponent(v)),
            };

            if (chains.length > 1) searchParam.chains = chains.slice(1);
            if (modifier) searchParam.modifier = modifier;

            return { ...parameters, [searchParam.name]: searchParam };
          },
          {}
        );

  return Object.values(parameters);
}

/**
 * Given a url string parsequery parameters.
 * @param url Any url to parse out query parameters.
 * @returns Record of parsed parameters with name modifier and value.
 */
export default function parseUrl(
  url: string
): ParsedParameter<string | number>[] {
  const chunks = url.split("?");
  if (chunks.length > 2)
    throw new OperationError(outcomeError("invalid", "Invalid query string"));
  const [_, queryParams] = chunks;
  return parseQuery(queryParams);
}
