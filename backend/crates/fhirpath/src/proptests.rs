//! Property tests for `FHIRPath`: the parser and the evaluator take any input
//! without panicking, and an expression evaluates the same every time and
//! the same in parentheses.
//!
//! Expressions are generated from the grammar over a fixed Patient, up to a
//! few levels deep, from every function the specification names except
//! `repeat`, whose unbounded recursion would hang on a cyclic projection.

use std::sync::LazyLock;

use haste_fhir_model::r4::generated::{resources::Patient, types::FHIRBoolean};
use haste_reflect::MetaValue;
use proptest::prelude::*;
use proptest::test_runner::TestCaseError;
use serde_json::json;

use crate::{FHIRPathError, FPEngine, parser};

static RUNTIME: LazyLock<tokio::runtime::Runtime> = LazyLock::new(|| {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("a runtime")
});

static PATIENT: LazyLock<Patient> = LazyLock::new(|| {
    serde_json::from_value(json!({
        "resourceType": "Patient",
        "id": "example",
        "active": true,
        "identifier": [
            {"system": "urn:oid:1.2.3", "value": "12345"},
            {"value": "67890"}
        ],
        "name": [
            {"use": "official", "family": "Chalmers", "given": ["Peter", "James"]},
            {"use": "nickname", "given": ["Jim"]}
        ],
        "telecom": [{"system": "phone", "value": "(03) 5555 6473", "use": "work", "rank": 1}],
        "gender": "male",
        "birthDate": "1974-12-25",
        "deceasedBoolean": false,
        "address": [{
            "use": "home",
            "line": ["534 Erewhon St"],
            "city": "PleasantVille",
            "state": "Vic",
            "postalCode": "3999",
            "period": {"start": "1974-12-25"}
        }],
        "contact": [{
            "relationship": [{"coding": [{
                "system": "http://terminology.hl7.org/CodeSystem/v2-0131",
                "code": "N"
            }]}],
            "name": {"family": "du Marché", "given": ["Bénédicte"]}
        }],
        "managingOrganization": {"reference": "Organization/1"}
    }))
    .expect("a valid Patient")
});

const LITERALS: &[&str] = &[
    "{}",
    "true",
    "false",
    "0",
    "1",
    "-1",
    "99999999999999999999",
    "0.5",
    "1.0",
    "'it\\'s'",
    "'\\u0041'",
    "''",
    "@2020",
    "@2020-01-01",
    "@2020-02-30",
    "@T12:00:00",
    "@2020-01-01T12:00:00Z",
    "@2020-01-01T12:00:00+05:00",
    "1 'mg'",
    "2 years",
];

const IDENTIFIERS: &[&str] = &[
    "Patient",
    "name",
    "given",
    "family",
    "use",
    "text",
    "birthDate",
    "active",
    "gender",
    "identifier",
    "system",
    "value",
    "telecom",
    "address",
    "line",
    "city",
    "period",
    "start",
    "end",
    "extension",
    "url",
    "id",
    "deceased",
    "contact",
    "relationship",
    "coding",
    "code",
    "managingOrganization",
    "reference",
    "resourceType",
    "nothing",
];

const FUNCTIONS: &[&str] = &[
    "empty",
    "exists",
    "all",
    "allTrue",
    "anyTrue",
    "allFalse",
    "anyFalse",
    "subsetOf",
    "supersetOf",
    "count",
    "distinct",
    "isDistinct",
    "where",
    "select",
    "ofType",
    "single",
    "first",
    "last",
    "tail",
    "skip",
    "take",
    "intersect",
    "exclude",
    "union",
    "combine",
    "iif",
    "toBoolean",
    "convertsToBoolean",
    "toInteger",
    "convertsToInteger",
    "toDate",
    "convertsToDate",
    "toDateTime",
    "convertsToDateTime",
    "toDecimal",
    "convertsToDecimal",
    "toQuantity",
    "convertsToQuantity",
    "toString",
    "convertsToString",
    "toTime",
    "convertsToTime",
    "indexOf",
    "substring",
    "startsWith",
    "endsWith",
    "contains",
    "upper",
    "lower",
    "replace",
    "matches",
    "replaceMatches",
    "length",
    "toChars",
    "split",
    "join",
    "trim",
    "encode",
    "decode",
    "escape",
    "unescape",
    "abs",
    "ceiling",
    "exp",
    "floor",
    "ln",
    "log",
    "power",
    "round",
    "sqrt",
    "truncate",
    "children",
    "descendants",
    "trace",
    "now",
    "timeOfDay",
    "today",
    "not",
    "hasValue",
    "getValue",
    "extension",
    "resolve",
    "memberOf",
    "subsumes",
    "subsumedBy",
    "htmlChecks",
    "aggregate",
    "is",
    "as",
    "type",
    "conformsTo",
    "lowBoundary",
    "highBoundary",
    "precision",
    "comparable",
    "unknownFunction",
];

const OPERATORS: &[&str] = &[
    "+", "-", "*", "/", "div", "mod", "&", "|", "=", "!=", "~", "!~", "<", ">", "<=", ">=", "in",
    "contains", "and", "or", "xor", "implies",
];

const TYPES: &[&str] = &[
    "Patient",
    "HumanName",
    "Identifier",
    "string",
    "integer",
    "decimal",
    "boolean",
    "date",
    "dateTime",
    "Quantity",
    "System.String",
    "System.Integer",
    "FHIR.Patient",
    "code",
    "Unknown",
];

/// Functions whose result depends on the clock.
const VOLATILE: &[&str] = &["now(", "timeOfDay(", "today("];

fn literal() -> impl Strategy<Value = String> {
    prop_oneof![
        3 => prop::sample::select(LITERALS).prop_map(String::from),
        1 => (0u64..10_000).prop_map(|n| n.to_string()),
        1 => "[0-9]{1,4}\\.[0-9]{1,4}",
        1 => "'[a-zA-Z0-9 ]{0,8}'",
    ]
}

fn leaf() -> impl Strategy<Value = String> {
    prop_oneof![
        2 => literal(),
        3 => prop::sample::select(IDENTIFIERS).prop_map(String::from),
        1 => prop::sample::select(vec![
            "$this", "$index", "$total", "%resource", "%context", "%ucum", "%undefined",
        ])
        .prop_map(String::from),
    ]
}

/// An expression from the grammar, up to four levels deep.
fn expression() -> impl Strategy<Value = String> {
    leaf().prop_recursive(4, 32, 3, |inner| {
        let arguments = prop::collection::vec(inner.clone(), 0..3);
        prop_oneof![
            (inner.clone(), prop::sample::select(IDENTIFIERS))
                .prop_map(|(target, member)| format!("{target}.{member}")),
            (
                inner.clone(),
                prop::sample::select(FUNCTIONS),
                arguments.clone()
            )
                .prop_map(|(target, function, arguments)| {
                    format!("{target}.{function}({})", arguments.join(", "))
                }),
            (prop::sample::select(FUNCTIONS), arguments)
                .prop_map(|(function, arguments)| format!("{function}({})", arguments.join(", "))),
            (inner.clone(), 0usize..4).prop_map(|(target, index)| format!("{target}[{index}]")),
            (
                inner.clone(),
                prop::sample::select(OPERATORS),
                inner.clone()
            )
                .prop_map(|(left, operator, right)| format!("{left} {operator} {right}")),
            (
                inner.clone(),
                prop::sample::select(vec!["is", "as"]),
                prop::sample::select(TYPES)
            )
                .prop_map(|(target, operator, type_name)| format!(
                    "{target} {operator} {type_name}"
                )),
            inner.clone().prop_map(|inner| format!("({inner})")),
            (prop::sample::select(vec!["-", "+"]), inner)
                .prop_map(|(sign, operand)| format!("{sign}{operand}")),
        ]
    })
}

/// The expression's values over the fixed Patient, rendered for comparison.
fn evaluate(expression: &str) -> Result<Vec<String>, FHIRPathError> {
    RUNTIME.block_on(async {
        let engine = FPEngine::new();
        let patient: &dyn MetaValue = &*PATIENT;
        let context = engine.evaluate(expression, vec![patient]).await?;
        Ok(context.iter().map(|value| format!("{value:?}")).collect())
    })
}

fn values(result: &Result<Vec<String>, FHIRPathError>) -> Option<&Vec<String>> {
    result.as_ref().ok()
}

/// The expression's single Boolean result, `None` for an empty result.
fn boolean(expression: &str) -> Result<Option<bool>, TestCaseError> {
    RUNTIME.block_on(async {
        let context = FPEngine::new()
            .evaluate(expression, vec![])
            .await
            .map_err(|error| TestCaseError::fail(format!("{expression}: {error}")))?;
        match context.iter().collect::<Vec<_>>().as_slice() {
            [] => Ok(None),
            [value] => value
                .as_any()
                .downcast_ref::<FHIRBoolean>()
                .and_then(|boolean| boolean.value)
                .map(Some)
                .ok_or_else(|| TestCaseError::fail(format!("{expression} is not a Boolean"))),
            many => Err(TestCaseError::fail(format!(
                "{expression} gave {} values",
                many.len()
            ))),
        }
    })
}

/// A Decimal literal: fixed notation, since the grammar has no exponent.
fn decimal_literal(value: f64) -> String {
    format!("{value:.4}")
}

const COMPARISONS: [&str; 4] = ["<", ">", "<=", ">="];

fn compare<T: PartialOrd>(operator: &str, left: &T, right: &T) -> bool {
    match operator {
        "<" => left < right,
        ">" => left > right,
        "<=" => left <= right,
        _ => left >= right,
    }
}

proptest! {
    /// The parser never panics, whatever the input.
    #[test]
    fn parser_never_panics_on_any_input(input in ".*") {
        let _ = parser::parse(&input);
    }

    /// Nor on input shaped like an expression.
    #[test]
    fn parser_never_panics_on_expression_shaped_input(input in expression()) {
        let _ = parser::parse(&input);
    }

    /// Evaluation never panics: an expression yields values or an error.
    #[test]
    fn evaluation_never_panics(input in expression()) {
        let _ = evaluate(&input);
    }

    /// An expression evaluates the same twice, and the same in parentheses.
    #[test]
    fn evaluation_is_stable(input in expression()) {
        prop_assume!(!VOLATILE.iter().any(|function| input.contains(function)));
        let first = evaluate(&input);
        let second = evaluate(&input);
        let wrapped = evaluate(&format!("({input})"));
        prop_assert_eq!(values(&first), values(&second), "{} evaluates differently each time", input);
        prop_assert_eq!(values(&first), values(&wrapped), "({}) evaluates differently", input);
    }
}

proptest! {
    /// `<`, `>`, `<=` and `>=` agree with Rust on Integers, on Decimals, and
    /// across the two (<https://hl7.org/fhirpath/N1/#comparison>).
    #[test]
    fn numeric_comparisons_agree_with_rust(
        a in -1_000_000i64..=1_000_000, b in -1_000_000i64..=1_000_000,
        x in -1.0e6f64..=1.0e6, y in -1.0e6f64..=1.0e6,
    ) {
        let (xs, ys) = (decimal_literal(x), decimal_literal(y));
        let (xv, yv): (f64, f64) = (xs.parse().unwrap(), ys.parse().unwrap());
        #[allow(clippy::cast_precision_loss)]
        let af = a as f64;
        for operator in COMPARISONS {
            prop_assert_eq!(boolean(&format!("{a} {operator} {b}"))?, Some(compare(operator, &a, &b)));
            prop_assert_eq!(boolean(&format!("{xs} {operator} {ys}"))?, Some(compare(operator, &xv, &yv)));
            prop_assert_eq!(boolean(&format!("{a} {operator} {ys}"))?, Some(compare(operator, &af, &yv)));
        }
    }

    /// A comparison with an empty operand gives an empty result.
    #[test]
    fn comparison_with_an_empty_operand_is_empty(n in 0i64..100) {
        for operator in COMPARISONS {
            prop_assert_eq!(boolean(&format!("{{}} {operator} {n}"))?, None);
            prop_assert_eq!(boolean(&format!("{n} {operator} {{}}"))?, None);
        }
    }
}
