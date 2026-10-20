//! The examples the `FHIRPath` N1 specification gives for each function and
//! operator, linked per test. <https://hl7.org/fhirpath/N1/>

use crate::FPEngine;
use haste_fhir_model::r4::{
    conversion::downcast_string,
    generated::{
        resources::Patient,
        types::{FHIRBoolean, FHIRDecimal, FHIRInteger, HumanName, Quantity},
    },
};
use haste_reflect::MetaValue;

/// Renders a result item the way the specification writes its examples:
/// `1` for an Integer, `1.0` for a Decimal, `true`, `'a'`.
fn render(value: &dyn MetaValue) -> String {
    let any = value.as_any();
    if let Some(integer) = any.downcast_ref::<FHIRInteger>() {
        integer.value.unwrap().to_string()
    } else if let Some(decimal) = any.downcast_ref::<FHIRDecimal>() {
        format!("{:?}", decimal.value.unwrap())
    } else if let Some(boolean) = any.downcast_ref::<FHIRBoolean>() {
        boolean.value.unwrap().to_string()
    } else {
        format!("'{}'", downcast_string(value).unwrap())
    }
}

async fn evaluate_on(expression: &str, input: Vec<&dyn MetaValue>) -> Vec<String> {
    let result = FPEngine::new()
        .evaluate(expression, input)
        .await
        .unwrap_or_else(|e| panic!("{expression} failed: {e}"));
    result.iter().map(render).collect()
}

async fn assert_evaluates(expression: &str, expected: &[&str]) {
    assert_eq!(
        evaluate_on(expression, vec![]).await,
        expected,
        "{expression}"
    );
}

async fn assert_errors(expression: &str) {
    let result = FPEngine::new().evaluate(expression, vec![]).await;
    assert!(result.is_err(), "{expression} should signal an error");
}

/// <https://hl7.org/fhirpath/N1/#allcriteria-expression-boolean>
#[tokio::test]
async fn all() {
    assert_evaluates("(1 | 2 | 3).all($this > 0)", &["true"]).await;
    assert_evaluates("(1 | 2 | 3).all($this > 1)", &["false"]).await;
    assert_evaluates("{}.all(false)", &["true"]).await;
}

/// <https://hl7.org/fhirpath/N1/#alltrue-boolean> and the three that follow it.
#[tokio::test]
async fn boolean_aggregates() {
    assert_evaluates("true.allTrue()", &["true"]).await;
    assert_evaluates("true.combine(false).allTrue()", &["false"]).await;
    assert_evaluates("{}.allTrue()", &["true"]).await;

    assert_evaluates("false.combine(true).anyTrue()", &["true"]).await;
    assert_evaluates("false.anyTrue()", &["false"]).await;
    assert_evaluates("{}.anyTrue()", &["false"]).await;

    assert_evaluates("false.allFalse()", &["true"]).await;
    assert_evaluates("true.combine(false).allFalse()", &["false"]).await;
    assert_evaluates("{}.allFalse()", &["true"]).await;

    assert_evaluates("true.combine(false).anyFalse()", &["true"]).await;
    assert_evaluates("true.anyFalse()", &["false"]).await;
    assert_evaluates("{}.anyFalse()", &["false"]).await;
}

/// <https://hl7.org/fhirpath/N1/#subsetofother-collection-boolean> and
/// <https://hl7.org/fhirpath/N1/#supersetofother-collection-boolean>
#[tokio::test]
async fn subset_and_superset() {
    assert_evaluates("(1 | 2).subsetOf(1 | 2 | 3)", &["true"]).await;
    assert_evaluates("(1 | 4).subsetOf(1 | 2 | 3)", &["false"]).await;
    assert_evaluates("{}.subsetOf(1 | 2)", &["true"]).await;
    assert_evaluates("(1 | 2).subsetOf({})", &["false"]).await;

    assert_evaluates("(1 | 2 | 3).supersetOf(1 | 2)", &["true"]).await;
    assert_evaluates("(1 | 2).supersetOf(1 | 4)", &["false"]).await;
    assert_evaluates("(1 | 2).supersetOf({})", &["true"]).await;
}

/// <https://hl7.org/fhirpath/N1/#distinct-collection> and
/// <https://hl7.org/fhirpath/N1/#isdistinct-boolean>
#[tokio::test]
async fn distinct() {
    assert_evaluates("1.combine(1).combine(2).distinct()", &["1", "2"]).await;
    assert_evaluates("{}.distinct()", &[]).await;

    assert_evaluates("1.combine(1).isDistinct()", &["false"]).await;
    assert_evaluates("(1 | 2).isDistinct()", &["true"]).await;
    assert_evaluates("{}.isDistinct()", &["true"]).await;
}

/// <https://hl7.org/fhirpath/N1/#selectprojection-expression-collection>
#[tokio::test]
async fn select() {
    let patient = Patient {
        name: Some(vec![
            HumanName {
                given: Some(vec!["Bob".to_string().into(), "Jim".to_string().into()]),
                ..Default::default()
            },
            HumanName {
                given: Some(vec!["Al".to_string().into()]),
                ..Default::default()
            },
        ]),
        ..Default::default()
    };

    // Multiple items from one projection are all added, flattened.
    assert_eq!(
        evaluate_on("Patient.name.select(given)", vec![&patient]).await,
        ["'Bob'", "'Jim'", "'Al'"]
    );
    assert_evaluates("('a' | 'b').select($this & '!')", &["'a!'", "'b!'"]).await;
    assert_evaluates("{}.select($this)", &[]).await;
}

/// <https://hl7.org/fhirpath/N1/#single-collection> through
/// <https://hl7.org/fhirpath/N1/#excludeother-collection-collection>
#[tokio::test]
async fn subsetting() {
    assert_evaluates("5.single()", &["5"]).await;
    assert_evaluates("{}.single()", &[]).await;
    assert_errors("(1 | 2).single()").await;

    assert_evaluates("(1 | 2 | 3).last()", &["3"]).await;
    assert_evaluates("{}.last()", &[]).await;

    assert_evaluates("(1 | 2 | 3).tail()", &["2", "3"]).await;
    assert_evaluates("1.tail()", &[]).await;

    assert_evaluates("(1 | 2 | 3).skip(1)", &["2", "3"]).await;
    assert_evaluates("(1 | 2 | 3).skip(0)", &["1", "2", "3"]).await;
    assert_evaluates("(1 | 2 | 3).skip(-1)", &["1", "2", "3"]).await;
    assert_evaluates("(1 | 2 | 3).skip(5)", &[]).await;

    assert_evaluates("(1 | 2 | 3).take(2)", &["1", "2"]).await;
    assert_evaluates("(1 | 2 | 3).take(0)", &[]).await;
    assert_evaluates("(1 | 2 | 3).take(5)", &["1", "2", "3"]).await;

    // intersect removes duplicates, exclude keeps them and the order.
    assert_evaluates("(1 | 2 | 3).intersect(2 | 3 | 4)", &["2", "3"]).await;
    assert_evaluates("1.combine(1).intersect(1)", &["1"]).await;
    assert_evaluates("(1 | 2 | 3).exclude(2)", &["1", "3"]).await;
    assert_evaluates("1.combine(1).combine(2).exclude(2)", &["1", "1"]).await;
}

/// <https://hl7.org/fhirpath/N1/#unionother-collection>,
/// <https://hl7.org/fhirpath/N1/#combineother-collection-collection> and
/// <https://hl7.org/fhirpath/N1/#union-collections>
#[tokio::test]
async fn combining() {
    // A: 1, 1, 2, 3 and B: 2, 3, from the spec.
    assert_evaluates(
        "1.combine(1).combine(2).combine(3).union(2 | 3)",
        &["1", "2", "3"],
    )
    .await;
    assert_evaluates("(1 | 2 | 3).union({})", &["1", "2", "3"]).await;

    // The | operator is a synonym for union, so it also removes duplicates.
    assert_evaluates("1 | 1 | 2", &["1", "2"]).await;
    assert_evaluates("1.combine(1) | 2", &["1", "2"]).await;

    assert_evaluates("(1 | 2).combine(2)", &["1", "2", "2"]).await;
    assert_evaluates("{}.combine(1)", &["1"]).await;
}

/// <https://hl7.org/fhirpath/N1/#iifcriterion-expression-true-result-collection-otherwise-result-collection-collection>
#[tokio::test]
async fn iif() {
    assert_evaluates("iif(true, 'a', 'b')", &["'a'"]).await;
    assert_evaluates("iif(false, 'a', 'b')", &["'b'"]).await;
    assert_evaluates("iif({}, 'a', 'b')", &["'b'"]).await;
    assert_evaluates("iif(false, 'a')", &[]).await;
    // Short circuit: the branch not taken is not evaluated.
    assert_evaluates("iif(true, 'a', (1 | 2).single())", &["'a'"]).await;
    assert_evaluates("iif(false, (1 | 2).single(), 'b')", &["'b'"]).await;
}

/// <https://hl7.org/fhirpath/N1/#math>
#[tokio::test]
async fn math() {
    assert_evaluates("(-5).abs()", &["5"]).await;
    assert_evaluates("(-5.5).abs()", &["5.5"]).await;
    // The spec's (-5.5 'mg').abs(), on a FHIR Quantity: the unit is unchanged.
    let quantity = Quantity {
        value: Some(Box::new(FHIRDecimal {
            value: Some(-5.5),
            ..Default::default()
        })),
        unit: Some(Box::new("mg".to_string().into())),
        ..Default::default()
    };
    let absolute = evaluate_on("$this.abs().value | $this.abs().unit", vec![&quantity]).await;
    assert_eq!(absolute, ["5.5", "'mg'"]);

    assert_evaluates("1.ceiling()", &["1"]).await;
    assert_evaluates("1.1.ceiling()", &["2"]).await;
    assert_evaluates("(-1.1).ceiling()", &["-1"]).await;

    assert_evaluates("0.exp()", &["1.0"]).await;
    assert_evaluates("(-0.0).exp()", &["1.0"]).await;

    assert_evaluates("1.floor()", &["1"]).await;
    assert_evaluates("2.1.floor()", &["2"]).await;
    assert_evaluates("(-2.1).floor()", &["-3"]).await;

    assert_evaluates("1.ln()", &["0.0"]).await;
    assert_evaluates("1.0.ln()", &["0.0"]).await;

    assert_evaluates("16.log(2)", &["4.0"]).await;
    assert_evaluates("100.0.log(10.0)", &["2.0"]).await;
    assert_evaluates("16.log({})", &[]).await;

    assert_evaluates("2.power(3)", &["8"]).await;
    assert_evaluates("2.5.power(2)", &["6.25"]).await;
    assert_evaluates("(-1).power(0.5)", &[]).await;

    // round returns a Decimal, so the spec's `1` is `1.0` here.
    assert_evaluates("1.round()", &["1.0"]).await;
    assert_evaluates("3.14159.round(3)", &["3.142"]).await;
    assert_errors("3.14159.round(-1)").await;

    assert_evaluates("81.sqrt()", &["9.0"]).await;
    assert_evaluates("(-1).sqrt()", &[]).await;

    assert_evaluates("101.truncate()", &["101"]).await;
    assert_evaluates("1.00000001.truncate()", &["1"]).await;
    assert_evaluates("(-1.56).truncate()", &["-1"]).await;

    assert_evaluates("{}.abs()", &[]).await;
    assert_errors("(1 | 2).abs()").await;
}

/// <https://hl7.org/fhirpath/N1/#not-boolean>
#[tokio::test]
async fn not() {
    assert_evaluates("true.not()", &["false"]).await;
    assert_evaluates("false.not()", &["true"]).await;
    assert_evaluates("{}.not()", &[]).await;
}

/// <https://hl7.org/fhirpath/N1/#implies>
#[tokio::test]
async fn implies() {
    assert_evaluates("true implies true", &["true"]).await;
    assert_evaluates("true implies false", &["false"]).await;
    assert_evaluates("true implies {}", &[]).await;
    assert_evaluates("false implies true", &["true"]).await;
    assert_evaluates("false implies false", &["true"]).await;
    assert_evaluates("false implies {}", &["true"]).await;
    assert_evaluates("{} implies true", &["true"]).await;
    assert_evaluates("{} implies false", &[]).await;
    assert_evaluates("{} implies {}", &[]).await;
}

/// <https://hl7.org/fhirpath/N1/#div> and <https://hl7.org/fhirpath/N1/#mod>
#[tokio::test]
async fn div_and_mod() {
    assert_evaluates("5 div 2", &["2"]).await;
    assert_evaluates("5.5 div 0.7", &["7.0"]).await;
    assert_evaluates("5 div 0", &[]).await;

    assert_evaluates("5 mod 2", &["1"]).await;
    // Decimals are f64, so 5.5 mod 0.7 is 0.6 only up to rounding.
    assert_evaluates("((5.5 mod 0.7) - 0.6).abs() < 0.0000001", &["true"]).await;
    assert_evaluates("5 mod 0", &[]).await;
}

/// <https://hl7.org/fhirpath/N1/#string-concatenation>
#[tokio::test]
async fn string_concatenation() {
    assert_evaluates("'ABC' + 'DEF'", &["'ABCDEF'"]).await;
    assert_evaluates("'ABC' + {} + 'DEF'", &[]).await;
    assert_evaluates("'ABC' & 'DEF'", &["'ABCDEF'"]).await;
    assert_evaluates("'ABC' & {} & 'DEF'", &["'ABCDEF'"]).await;
}

/// <https://hl7.org/fhirpath/N1/#in-membership> and
/// <https://hl7.org/fhirpath/N1/#contains-containership>
#[tokio::test]
async fn membership() {
    assert_evaluates("2 in (1 | 2 | 3)", &["true"]).await;
    assert_evaluates("5 in (1 | 2 | 3)", &["false"]).await;
    assert_evaluates("{} in (1 | 2)", &[]).await;
    assert_evaluates("1 in {}", &["false"]).await;
    assert_errors("(1 | 2) in (1 | 2)").await;

    assert_evaluates("(1 | 2 | 3) contains 2", &["true"]).await;
    assert_evaluates("(1 | 2 | 3) contains 5", &["false"]).await;
    assert_evaluates("(1 | 2) contains {}", &[]).await;
    assert_evaluates("{} contains 1", &["false"]).await;
}

/// <https://hl7.org/fhirpath/N1/#operator-precedence> (unary + and -)
#[tokio::test]
async fn polarity() {
    assert_evaluates("-5", &["-5"]).await;
    assert_evaluates("-5.5", &["-5.5"]).await;
    assert_evaluates("+5", &["5"]).await;
    assert_evaluates("-{}", &[]).await;
}
