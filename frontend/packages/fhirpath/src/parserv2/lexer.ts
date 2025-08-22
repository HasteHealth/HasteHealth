import * as chevrotain from "chevrotain";

export const IDENTIFIER = chevrotain.createToken({
  name: "Identifier",
  pattern: /[A-Za-z_][A-Za-z0-9_]*/,
});

export const DELIMINATED_IDENTIFIER = chevrotain.createToken({
  name: "DeliminatedIdentifier",
  pattern: /`[^`]+`/,
});

// Operators
// // Implies Operation
export const IMPLIES_OPERATION = chevrotain.createToken({
  name: "ImpliesOperation",
  pattern: /implies/,
  longer_alt: IDENTIFIER,
});
// // Or Operation
export const OR_OPERATION = chevrotain.createToken({
  name: "OrOperation",
  pattern: /or|xor/,
  longer_alt: IDENTIFIER,
});
// // And Operation
export const AND_OPERATION = chevrotain.createToken({
  name: "AndOperation",
  pattern: /and/,
  longer_alt: IDENTIFIER,
});
// // Membership operations
export const MEMBERSHIP_OPERATION = chevrotain.createToken({
  name: "MembershipOperation",
  pattern: /in|contains/,
  longer_alt: IDENTIFIER,
});

// // Equality operations
// $('<=' / '<' / '>' / '>=' / '=' / '~' / '!=' / '!~')
export const EQUALITY_OPERATION = chevrotain.createToken({
  name: "EqualityOperation",
  pattern: /<=|<|>|>=|=|~|!=|!~/,
});
// // Union operation
// $("|")
export const UNION_OPERATION = chevrotain.createToken({
  name: "UnionOperation",
  pattern: /\|/,
});
// // Type operations
// $("is" / "as")
export const TYPE_OPERATION = chevrotain.createToken({
  name: "TypeOperation",
  pattern: /is|as/,
  longer_alt: IDENTIFIER,
});
// // Additive operations
// ('+' / '-' / '&')
export const ADDITIVE_OPERATION = chevrotain.createToken({
  name: "AdditiveOperation",
  pattern: /\+|-|&/,
});
// // Multiplicative operations
// $('*' / '/' / 'div' / 'mod')
export const MULTIPLICATIVE_OPERATION = chevrotain.createToken({
  name: "MultiplicativeOperation",
  pattern: /\*|\/|div|mod/,
  longer_alt: IDENTIFIER,
});

export const COMMA = chevrotain.createToken({ name: "Comma", pattern: /,/ });

export const NUMBER = chevrotain.createToken({
  name: "Number",
  pattern: /[0-9]+(\.[0-9]+)?/,
});

export const STRING = chevrotain.createToken({
  name: "String",
  pattern: /'[^']*'/,
});

export const BOOLEAN = chevrotain.createToken({
  name: "Boolean",
  pattern: /true|false/,
  longer_alt: IDENTIFIER,
});

export const NULL = chevrotain.createToken({
  name: "Null",
  pattern: /{}/,
});

export const WHITESPACE = chevrotain.createToken({
  name: "WhiteSpace",
  pattern: /\s+/,
  group: chevrotain.Lexer.SKIPPED,
});

export const DATETIME_PRECISION = chevrotain.createToken({
  name: "DateTimePrecision",
  pattern: /year|month|week|day|hour|minute|second|millisecond/,
  longer_alt: IDENTIFIER,
});

export const PLURAL_DATETIME_PRECISION = chevrotain.createToken({
  name: "PluralDateTimePrecision",
  pattern: /years|months|weeks|days|hours|minutes|seconds|milliseconds/,
  longer_alt: IDENTIFIER,
});

const DATE_FORMAT = "[0-9][0-9][0-9][0-9](-[0-9][0-9](-[0-9][0-9])?)?";
const TIME_FORMAT = "[0-9][0-9](:[0-9][0-9](:[0-9][0-9](\\.[0-9]+)?)?)?";
const TIMEZONE_FORMAT = "(Z|(\\+|-)[0-9][0-9]:[0-9][0-9])";

export const TIME = chevrotain.createToken({
  name: "TIME",
  pattern: new RegExp(`@T${TIME_FORMAT}`),
});

export const DATE = chevrotain.createToken({
  name: "DATE",
  pattern: new RegExp(`@${DATE_FORMAT}`),
});

export const DATETIME = chevrotain.createToken({
  name: "DATETIME",
  pattern: new RegExp(`@${DATE_FORMAT}(T${TIME_FORMAT}(${TIMEZONE_FORMAT})?)?`),
});

export const THIS = chevrotain.createToken({
  name: "THIS",
  pattern: /\$this/,
});

export const INDEX = chevrotain.createToken({
  name: "INDEX",
  pattern: /\$index/,
});

export const TOTAL = chevrotain.createToken({
  name: "TOTAL",
  pattern: /\$total/,
});

export const PERCENTAGE = chevrotain.createToken({
  name: "TOTAL",
  pattern: /%/,
});

export const LEFT_PARAN = chevrotain.createToken({
  name: "LEFT_PARAN",
  pattern: /\(/,
});

export const RIGHT_PARAN = chevrotain.createToken({
  name: "RIGHT_PARAN",
  pattern: /\)/,
});

export const LEFT_BRACKET = chevrotain.createToken({
  name: "LEFT_PARAN",
  pattern: /\[/,
});

export const RIGHT_BRACKET = chevrotain.createToken({
  name: "RIGHT_PARAN",
  pattern: /\]/,
});

export const DOT = chevrotain.createToken({
  name: "DOT",
  pattern: /\./,
});

// note we are placing WhiteSpace first as it is very common thus it will speed up the lexer.
export const allTokens = [
  DELIMINATED_IDENTIFIER,
  WHITESPACE,
  COMMA,
  PERCENTAGE,
  LEFT_PARAN,
  RIGHT_PARAN,
  LEFT_BRACKET,
  RIGHT_BRACKET,
  DOT,

  // "keywords" appear before the Identifier
  IMPLIES_OPERATION,
  OR_OPERATION,
  AND_OPERATION,
  MEMBERSHIP_OPERATION,
  EQUALITY_OPERATION,
  UNION_OPERATION,
  TYPE_OPERATION,
  ADDITIVE_OPERATION,
  MULTIPLICATIVE_OPERATION,

  DATETIME,
  DATE,
  TIME,
  BOOLEAN,
  DATETIME_PRECISION,
  PLURAL_DATETIME_PRECISION,

  THIS,
  TOTAL,
  INDEX,

  // After Operators for keywords.
  IDENTIFIER,

  // Literals
  NULL,
  NUMBER,
  STRING,
];

const SelectLexer = new chevrotain.Lexer(allTokens);

export default SelectLexer;
