//! The examples the `FHIRPath` N1 specification gives for each function and
//! operator, linked per test. <https://hl7.org/fhirpath/N1/>

use crate::FPEngine;
use haste_fhir_model::r4::{
    conversion::{downcast_datetime, downcast_string},
    datetime::Date,
    generated::{
        resources::Patient,
        types::{
            FHIRBoolean, FHIRDate, FHIRDateTime, FHIRDecimal, FHIRInteger, FHIRTime, HumanName,
            Quantity,
        },
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
    } else if any.is::<FHIRDate>() || any.is::<FHIRDateTime>() || any.is::<FHIRTime>() {
        format!("'{}'", downcast_datetime(value).unwrap())
    } else if let Some(datetime) = any.downcast_ref::<crate::PartialDateTime>() {
        format!("'{datetime}'")
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

/// <https://hl7.org/fhirpath/N1/#and>
#[tokio::test]
async fn and() {
    assert_evaluates("true and true", &["true"]).await;
    assert_evaluates("true and false", &["false"]).await;
    assert_evaluates("true and {}", &[]).await;
    assert_evaluates("false and true", &["false"]).await;
    assert_evaluates("false and false", &["false"]).await;
    assert_evaluates("false and {}", &["false"]).await;
    assert_evaluates("{} and true", &[]).await;
    assert_evaluates("{} and false", &["false"]).await;
    assert_evaluates("{} and {}", &[]).await;
}

/// <https://hl7.org/fhirpath/N1/#or>
#[tokio::test]
async fn or() {
    assert_evaluates("true or true", &["true"]).await;
    assert_evaluates("true or false", &["true"]).await;
    assert_evaluates("true or {}", &["true"]).await;
    assert_evaluates("false or true", &["true"]).await;
    assert_evaluates("false or false", &["false"]).await;
    assert_evaluates("false or {}", &[]).await;
    assert_evaluates("{} or true", &["true"]).await;
    assert_evaluates("{} or false", &[]).await;
    assert_evaluates("{} or {}", &[]).await;
}

/// <https://hl7.org/fhirpath/N1/#xor>
#[tokio::test]
async fn xor() {
    assert_evaluates("true xor true", &["false"]).await;
    assert_evaluates("true xor false", &["true"]).await;
    assert_evaluates("true xor {}", &[]).await;
    assert_evaluates("false xor true", &["true"]).await;
    assert_evaluates("false xor false", &["false"]).await;
    assert_evaluates("false xor {}", &[]).await;
    assert_evaluates("{} xor true", &[]).await;
    assert_evaluates("{} xor false", &[]).await;
    assert_evaluates("{} xor {}", &[]).await;
}

/// A single non-Boolean operand counts as true; multiple items are an error.
#[tokio::test]
async fn boolean_operator_singleton_evaluation() {
    assert_evaluates("'a' and true", &["true"]).await;
    assert_evaluates("false or 1", &["true"]).await;
    assert_evaluates("'a' xor true", &["false"]).await;
    for expression in [
        "(true | false) and true",
        "false and (1 | 2)",
        "true or (1 | 2)",
    ] {
        assert_errors(expression).await;
    }
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

/// <https://hl7.org/fhirpath/N1/#indexofsubstring-string-integer> through
/// <https://hl7.org/fhirpath/N1/#tochars-collection>
#[tokio::test]
async fn string_functions() {
    assert_evaluates("'abcdefg'.indexOf('bc')", &["1"]).await;
    assert_evaluates("'abcdefg'.indexOf('x')", &["-1"]).await;
    assert_evaluates("'abcdefg'.indexOf('abcdefg')", &["0"]).await;
    assert_evaluates("'abcdefg'.indexOf('')", &["0"]).await;
    assert_evaluates("{}.indexOf('a')", &[]).await;
    assert_evaluates("'abc'.indexOf({})", &[]).await;

    assert_evaluates("'abcdefg'.substring(3)", &["'defg'"]).await;
    assert_evaluates("'abcdefg'.substring(1, 2)", &["'bc'"]).await;
    assert_evaluates("'abcdefg'.substring(6, 2)", &["'g'"]).await;
    assert_evaluates("'abcdefg'.substring(7, 1)", &[]).await;
    assert_evaluates("'abcdefg'.substring(-1)", &[]).await;
    assert_evaluates("'abcdefg'.substring(2, {})", &["'cdefg'"]).await;
    assert_errors("'abcdefg'.substring(1.5)").await;

    assert_evaluates("'abcdefg'.replace('cde', '123')", &["'ab123fg'"]).await;
    assert_evaluates("'abcdefg'.replace('cde', '')", &["'abfg'"]).await;
    assert_evaluates("'abc'.replace('', 'x')", &["'xaxbxcx'"]).await;
    assert_evaluates("'abc'.replace({}, 'x')", &[]).await;

    assert_evaluates("'abc123'.matches('^[a-z]+[0-9]+$')", &["true"]).await;
    assert_evaluates("'abc'.matches('[0-9]')", &["false"]).await;
    assert_evaluates("{}.matches('a')", &[]).await;
    assert_errors("'abc'.matches('(')").await;
    // An R4 invariant's pattern, whose \\. is a FHIRPath escape for the regex \.
    let identifier =
        r"'Patient.name'.matches('[A-Za-z][A-Za-z0-9]*(\\.[a-z][A-Za-z0-9]*(\\[x])?)*')";
    assert_evaluates(identifier, &["true"]).await;

    assert_evaluates(
        "'11/30/1972'.replaceMatches('(?<month>[0-9]{1,2})/(?<day>[0-9]{1,2})/(?<year>[0-9]{2,4})', '${day}-${month}-${year}')",
        &["'30-11-1972'"],
    )
    .await;

    assert_evaluates("'abc'.length()", &["3"]).await;
    assert_evaluates("''.length()", &["0"]).await;
    assert_evaluates("{}.length()", &[]).await;

    assert_evaluates("'abc'.toChars()", &["'a'", "'b'", "'c'"]).await;
    assert_evaluates("{}.toChars()", &[]).await;

    assert_evaluates(r"' \t\r\n123456\n\r\t '.trim()", &["'123456'"]).await;
    assert_evaluates("'  '.trim() = ''", &["true"]).await;
    assert_evaluates("{}.trim()", &[]).await;
    assert_evaluates("' 123456 '.trim()", &["' 123456 '"]).await;

    assert_errors("('a' | 'b').length()").await;
    assert_errors("('a' | 'b').trim()").await;
}

/// <https://hl7.org/fhirpath/N1/#string>
#[tokio::test]
async fn string_escapes() {
    assert_evaluates(r"'it\'s'", &["'it's'"]).await;
    assert_evaluates(r"'a\\b'.length()", &["3"]).await;
    assert_evaluates(r"'A\t'.toChars()", &["'A'", "'\t'"]).await;
    // A \ before any other character is ignored.
    assert_evaluates(r"'\p'", &["'p'"]).await;
}

/// <https://hl7.org/fhirpath/N1/#comments>
#[tokio::test]
async fn comments() {
    assert_evaluates("2 + 2 // This is a single-line comment", &["4.0"]).await;
    assert_evaluates("/*\nThis is a multi-line comment\n*/ 2 + 2", &["4.0"]).await;
    assert_evaluates("'a//b' /* not in strings */", &["'a//b'"]).await;
}

/// <https://hl7.org/fhirpath/N1/#boolean-conversion-functions>
#[tokio::test]
async fn boolean_conversion() {
    assert_evaluates("'yes'.toBoolean()", &["true"]).await;
    assert_evaluates("'F'.toBoolean()", &["false"]).await;
    assert_evaluates("1.toBoolean()", &["true"]).await;
    assert_evaluates("0.0.toBoolean()", &["false"]).await;
    assert_evaluates("2.toBoolean()", &[]).await;
    assert_evaluates("'maybe'.toBoolean()", &[]).await;
    assert_evaluates("{}.toBoolean()", &[]).await;

    assert_evaluates("'T'.convertsToBoolean()", &["true"]).await;
    assert_evaluates("'maybe'.convertsToBoolean()", &["false"]).await;
    assert_evaluates("{}.convertsToBoolean()", &[]).await;
    assert_errors("(true | false).toBoolean()").await;
}

/// <https://hl7.org/fhirpath/N1/#integer-conversion-functions>
#[tokio::test]
async fn integer_conversion() {
    assert_evaluates("'-12'.toInteger()", &["-12"]).await;
    assert_evaluates("'+12'.toInteger()", &["12"]).await;
    assert_evaluates("true.toInteger()", &["1"]).await;
    assert_evaluates("'1.5'.toInteger()", &[]).await;
    assert_evaluates("1.5.toInteger()", &[]).await;

    assert_evaluates("5.convertsToInteger()", &["true"]).await;
    assert_evaluates("'12a'.convertsToInteger()", &["false"]).await;
}

/// <https://hl7.org/fhirpath/N1/#decimal-conversion-functions>
#[tokio::test]
async fn decimal_conversion() {
    assert_evaluates("'1.5'.toDecimal()", &["1.5"]).await;
    assert_evaluates("'-1'.toDecimal()", &["-1.0"]).await;
    assert_evaluates("5.toDecimal()", &["5.0"]).await;
    assert_evaluates("false.toDecimal()", &["0.0"]).await;
    // Only (\+|-)?\d+(\.\d+)? converts.
    assert_evaluates("'1.'.toDecimal()", &[]).await;
    assert_evaluates("'1e5'.toDecimal()", &[]).await;

    assert_evaluates("'abc'.convertsToDecimal()", &["false"]).await;
    assert_evaluates("'+0.5'.convertsToDecimal()", &["true"]).await;
}

/// <https://hl7.org/fhirpath/N1/#string-conversion-functions>
#[tokio::test]
async fn string_conversion() {
    assert_evaluates("5.toString()", &["'5'"]).await;
    assert_evaluates("1.5.toString()", &["'1.5'"]).await;
    assert_evaluates("true.toString()", &["'true'"]).await;
    assert_evaluates("'a'.toString()", &["'a'"]).await;

    let patient = Patient {
        birthDate: Some(Box::new(FHIRDate {
            value: Some(Date::YearMonthDay(1974, 12, 25)),
            ..Default::default()
        })),
        name: Some(vec![HumanName::default()]),
        ..Default::default()
    };
    let birth_date = evaluate_on("Patient.birthDate.toString()", vec![&patient]).await;
    assert_eq!(birth_date, ["'1974-12-25'"]);
    let name = evaluate_on("Patient.name.convertsToString()", vec![&patient]).await;
    assert_eq!(name, ["false"]);

    let quantity = Quantity {
        value: Some(Box::new(FHIRDecimal {
            value: Some(5.5),
            ..Default::default()
        })),
        unit: Some(Box::new("mg".to_string().into())),
        ..Default::default()
    };
    let quantity = evaluate_on("$this.toString()", vec![&quantity]).await;
    assert_eq!(quantity, ["'5.5 'mg''"]);
}

/// <https://hl7.org/fhirpath/N1/#date-conversion-functions>
#[tokio::test]
async fn date_conversion() {
    assert_evaluates("'2014-01-25'.toDate()", &["'2014-01-25'"]).await;
    assert_evaluates("'2014-01'.toDate()", &["'2014-01'"]).await;
    assert_evaluates("'2014-01-25T14:30:00Z'.toDate()", &["'2014-01-25'"]).await;
    assert_evaluates("'abc'.toDate()", &[]).await;
    assert_evaluates("1.toDate()", &[]).await;
    assert_evaluates("{}.toDate()", &[]).await;

    assert_evaluates("'2014-01-25'.convertsToDate()", &["true"]).await;
    assert_evaluates("'2014-01-25T14:30:00Z'.convertsToDate()", &["true"]).await;
    assert_evaluates("'14:30'.convertsToDate()", &["false"]).await;
    assert_evaluates("true.convertsToDate()", &["false"]).await;
    assert_evaluates("{}.convertsToDate()", &[]).await;
    assert_errors("('2014' | '2015').toDate()").await;

    let patient = Patient {
        birthDate: Some(Box::new(FHIRDate {
            value: Some(Date::YearMonthDay(1974, 12, 25)),
            ..Default::default()
        })),
        ..Default::default()
    };
    let birth_date = evaluate_on("Patient.birthDate.toDate()", vec![&patient]).await;
    assert_eq!(birth_date, ["'1974-12-25'"]);
}

/// <https://hl7.org/fhirpath/N1/#datetime-conversion-functions>
#[tokio::test]
async fn date_time_conversion() {
    assert_evaluates(
        "'2014-01-25T14:30:00Z'.toDateTime()",
        &["'2014-01-25T14:30:00+00:00'"],
    )
    .await;
    assert_evaluates("'2014-01-25'.toDateTime()", &["'2014-01-25'"]).await;
    assert_evaluates("'2014-01'.toDateTime()", &["'2014-01'"]).await;
    assert_evaluates("'abc'.toDateTime()", &[]).await;
    assert_evaluates("1.toDateTime()", &[]).await;
    assert_evaluates("{}.toDateTime()", &[]).await;

    assert_evaluates("'2014-01-25'.convertsToDateTime()", &["true"]).await;
    assert_evaluates("'2014-01-25T14:30:00Z'.convertsToDateTime()", &["true"]).await;
    assert_evaluates("'14:30'.convertsToDateTime()", &["false"]).await;
    assert_evaluates("true.convertsToDateTime()", &["false"]).await;
    assert_evaluates("{}.convertsToDateTime()", &[]).await;
    assert_errors("('2014' | '2015').toDateTime()").await;
}

/// <https://hl7.org/fhirpath/N1/#time-conversion-functions>
#[tokio::test]
async fn time_conversion() {
    assert_evaluates("'14:30:15'.toTime()", &["'14:30:15'"]).await;
    assert_evaluates("'14:30:15.250'.toTime()", &["'14:30:15.250'"]).await;
    assert_evaluates("'14:30'.toTime()", &["'14:30'"]).await;
    assert_evaluates("'14'.toTime()", &["'14'"]).await;
    assert_evaluates("'25:00'.toTime()", &[]).await;
    assert_evaluates("'2014-01-25'.toTime()", &[]).await;
    assert_evaluates("1.toTime()", &[]).await;
    assert_evaluates("{}.toTime()", &[]).await;

    assert_evaluates("'14:30'.convertsToTime()", &["true"]).await;
    assert_evaluates("'abc'.convertsToTime()", &["false"]).await;
    assert_evaluates("true.convertsToTime()", &["false"]).await;
    assert_evaluates("{}.convertsToTime()", &[]).await;
    assert_errors("('14:30' | '15:30').toTime()").await;
}

/// <https://hl7.org/fhirpath/N1/#date>, <#datetime> and <#time> literals
#[tokio::test]
async fn temporal_literals() {
    assert_evaluates("@2020", &["'2020'"]).await;
    assert_evaluates("@2020-01", &["'2020-01'"]).await;
    assert_evaluates("@2020-01-01", &["'2020-01-01'"]).await;
    assert_evaluates("@2020-01-01T", &["'2020-01-01'"]).await;
    assert_evaluates("@2020-01-01T12:00:00Z", &["'2020-01-01T12:00:00+00:00'"]).await;
    assert_evaluates("@2020-01-01T12:00:00", &["'2020-01-01T12:00:00'"]).await;
    assert_evaluates(
        "@2020-01-01T12:00:00+05:00",
        &["'2020-01-01T07:00:00+00:00'"],
    )
    .await;
    assert_evaluates("@T12", &["'12'"]).await;
    assert_evaluates("@T12:00", &["'12:00'"]).await;
    assert_evaluates("@T12:00:00.250", &["'12:00:00.250'"]).await;
    assert_evaluates("@2020-01-01.toDate()", &["'2020-01-01'"]).await;
}

/// <https://hl7.org/fhirpath/N1/#tracename-string-projection-expression-collection>
#[tokio::test]
async fn trace() {
    assert_evaluates("(1 | 2).trace('numbers')", &["1", "2"]).await;
    assert_evaluates("(1 | 2).trace('numbers', $this + 1)", &["1", "2"]).await;
}

/// <https://hl7.org/fhirpath/N1/#istype-type-specifier> and
/// <https://hl7.org/fhirpath/N1/#is-type-specifier>
#[tokio::test]
async fn is_and_as() {
    let patient = Patient {
        name: Some(vec![HumanName::default(), HumanName::default()]),
        ..Default::default()
    };
    let is = async |expression| evaluate_on(expression, vec![&patient]).await;

    assert_eq!(is("$this.is(Patient)").await, ["true"]);
    assert_eq!(is("$this.is(Observation)").await, ["false"]);
    assert_eq!(is("$this is FHIR.Patient").await, ["true"]);
    assert_eq!(is("{}.is(Patient)").await, Vec::<String>::new());
    assert_eq!(is("{} is Patient").await, Vec::<String>::new());
    assert!(
        FPEngine::new()
            .evaluate("Patient.name.is(HumanName)", vec![&patient])
            .await
            .is_err()
    );

    assert_eq!(is("($this as Patient).exists()").await, ["true"]);
    assert_eq!(is("($this as Observation).exists()").await, ["false"]);
    // R4 search parameters apply as() to collections, so it filters them.
    assert_eq!(is("Patient.name.as(HumanName).count()").await, ["2"]);
}

/// <https://hl7.org/fhirpath/N1/#greater-than>
#[tokio::test]
async fn greater_than() {
    assert_evaluates("2 > 1", &["true"]).await;
    assert_evaluates("1 > 2", &["false"]).await;
    assert_evaluates("1 > 1", &["false"]).await;
    assert_evaluates("1.5 > 1", &["true"]).await;
    assert_evaluates("'b' > 'a'", &["true"]).await;
    assert_evaluates("'a' > 'b'", &["false"]).await;
    assert_evaluates("'abc' > 'ab'", &["true"]).await;
    assert_evaluates("{} > 1", &[]).await;
    assert_evaluates("1 > {}", &[]).await;
    assert_errors("'a' > 1").await;
    assert_errors("(1 | 2) > 1").await;
}

/// <https://hl7.org/fhirpath/N1/#less-than>
#[tokio::test]
async fn less_than() {
    assert_evaluates("1 < 2", &["true"]).await;
    assert_evaluates("2 < 1", &["false"]).await;
    assert_evaluates("1 < 1", &["false"]).await;
    assert_evaluates("1 < 1.5", &["true"]).await;
    assert_evaluates("'a' < 'b'", &["true"]).await;
    assert_evaluates("'b' < 'a'", &["false"]).await;
    assert_evaluates("'ab' < 'abc'", &["true"]).await;
    assert_evaluates("{} < 1", &[]).await;
    assert_evaluates("1 < {}", &[]).await;
    assert_errors("'a' < 1").await;
    assert_errors("(1 | 2) < 1").await;
}

/// <https://hl7.org/fhirpath/N1/#greater-or-equal>
#[tokio::test]
async fn greater_or_equal() {
    assert_evaluates("2 >= 1", &["true"]).await;
    assert_evaluates("1 >= 2", &["false"]).await;
    assert_evaluates("1 >= 1", &["true"]).await;
    assert_evaluates("1.5 >= 1", &["true"]).await;
    assert_evaluates("1.0 >= 1", &["true"]).await;
    assert_evaluates("'b' >= 'a'", &["true"]).await;
    assert_evaluates("'a' >= 'a'", &["true"]).await;
    assert_evaluates("'abc' >= 'ab'", &["true"]).await;
    assert_evaluates("{} >= 1", &[]).await;
    assert_evaluates("1 >= {}", &[]).await;
    assert_errors("'a' >= 1").await;
    assert_errors("(1 | 2) >= 1").await;
}

/// <https://hl7.org/fhirpath/N1/#less-or-equal>
#[tokio::test]
async fn less_or_equal() {
    assert_evaluates("1 <= 2", &["true"]).await;
    assert_evaluates("2 <= 1", &["false"]).await;
    assert_evaluates("1 <= 1", &["true"]).await;
    assert_evaluates("1 <= 1.5", &["true"]).await;
    assert_evaluates("'a' <= 'b'", &["true"]).await;
    assert_evaluates("'b' <= 'a'", &["false"]).await;
    assert_evaluates("'ab' <= 'abc'", &["true"]).await;
    assert_evaluates("{} <= 1", &[]).await;
    assert_evaluates("1 <= {}", &[]).await;
    assert_errors("'a' <= 1").await;
    assert_errors("(1 | 2) <= 1").await;
}

/// <https://hl7.org/fhirpath/N1/#less-than>, <https://hl7.org/fhirpath/N1/#less-or-equal>
/// and <https://hl7.org/fhirpath/N1/#greater-or-equal> on strings.
#[tokio::test]
async fn string_ordering() {
    assert_evaluates("'a' < 'b'", &["true"]).await;
    assert_evaluates("'a' <= 'a'", &["true"]).await;
    assert_evaluates("'a' >= 'b'", &["false"]).await;
}

/// <https://hl7.org/fhirpath/N1/#equivalent> and
/// <https://hl7.org/fhirpath/N1/#not-equivalent>
#[tokio::test]
async fn equivalent() {
    assert_evaluates("{} ~ {}", &["true"]).await;
    assert_evaluates("{} ~ 1", &["false"]).await;
    assert_evaluates("'Abc  d' ~ 'aBC D'", &["true"]).await;
    assert_evaluates("'a' ~ 'b'", &["false"]).await;
    assert_evaluates("0.1 ~ 0.10", &["true"]).await;
    assert_evaluates("(1 | 2 | 3) ~ (3 | 1 | 2)", &["true"]).await;
    assert_evaluates("(1 | 2) ~ (1 | 2 | 3)", &["false"]).await;
    assert_evaluates("{} !~ {}", &["false"]).await;
    assert_evaluates("'a' !~ 'A'", &["false"]).await;
    assert_evaluates("1 !~ 2", &["true"]).await;
}

/// <https://hl7.org/fhirpath/N1/#quantity>
#[tokio::test]
async fn quantity_literals() {
    assert_evaluates("4.5 'mg'.value", &["4.5"]).await;
    assert_evaluates("4.5 'mg'.code", &["'mg'"]).await;
    assert_evaluates("2 years.value", &["2.0"]).await;
    assert_evaluates("2 years.code", &["'years'"]).await;
    assert_evaluates(
        "2 years.system",
        &["'http://hl7.org/fhirpath/CodeSystem/calendar-units'"],
    )
    .await;
    assert_evaluates("(4.5 'mg').toString()", &["'4.5 'mg''"]).await;
}

/// <https://hl7.org/fhirpath/N1/#quantity-conversion-functions>
#[tokio::test]
async fn quantity_conversion() {
    let to_parts = |expression: &str| format!("({expression}).value | ({expression}).code");
    assert_evaluates(&to_parts("1.toQuantity()"), &["1.0", "'1'"]).await;
    assert_evaluates(&to_parts("2.5.toQuantity()"), &["2.5", "'1'"]).await;
    assert_evaluates(&to_parts("true.toQuantity()"), &["1.0", "'1'"]).await;
    assert_evaluates(&to_parts("'4 days'.toQuantity()"), &["4.0", "'days'"]).await;
    assert_evaluates(&to_parts("'-5.5 \\'mg\\''.toQuantity()"), &["-5.5", "'mg'"]).await;
    assert_evaluates(&to_parts("'6'.toQuantity()"), &["6.0", "'1'"]).await;
    assert_evaluates("'abc'.toQuantity()", &[]).await;
    assert_evaluates("'4 d ays'.toQuantity()", &[]).await;
    assert_evaluates("{}.toQuantity()", &[]).await;

    assert_evaluates("1.toQuantity('1').value", &["1.0"]).await;
    assert_evaluates("'4 days'.toQuantity('days').value", &["4.0"]).await;
    assert_evaluates("'4 days'.toQuantity('mg')", &[]).await;

    assert_evaluates("1.convertsToQuantity()", &["true"]).await;
    assert_evaluates("'4 days'.convertsToQuantity()", &["true"]).await;
    assert_evaluates("'abc'.convertsToQuantity()", &["false"]).await;
    assert_evaluates("'2014-01-25'.toDate().convertsToQuantity()", &["false"]).await;
    assert_evaluates("1.convertsToQuantity('mg')", &["false"]).await;
    assert_evaluates("{}.convertsToQuantity()", &[]).await;

    let quantity = Quantity {
        value: Some(Box::new(FHIRDecimal {
            value: Some(5.5),
            ..Default::default()
        })),
        unit: Some(Box::new("mg".to_string().into())),
        ..Default::default()
    };
    let converted = evaluate_on("$this.toQuantity('mg').value", vec![&quantity]).await;
    assert_eq!(converted, ["5.5"]);
    let converted = evaluate_on("$this.convertsToQuantity('g')", vec![&quantity]).await;
    assert_eq!(converted, ["false"]);
}

/// <https://hl7.org/fhirpath/N1/#datetime-arithmetic>
#[tokio::test]
async fn date_time_arithmetic() {
    assert_evaluates("@2014 + 24 months", &["'2016'"]).await;
    assert_evaluates("@2019-03-01 + 1 day", &["'2019-03-02'"]).await;
    assert_evaluates("@2020-01-31 + 1 month", &["'2020-02-29'"]).await;
    assert_evaluates("@2020-01-01 - 1 day", &["'2019-12-31'"]).await;
    assert_evaluates("@2020-01-01 + 2 weeks", &["'2020-01-15'"]).await;
    assert_evaluates("@2020-01-01 + 1.9 years", &["'2021-01-01'"]).await;
    assert_evaluates("@2020-03 - 3 months", &["'2019-12'"]).await;
    assert_evaluates("@2020-01-01 + 1 'd'", &["'2020-01-02'"]).await;
    assert_evaluates(
        "@2020-01-01T12:00:00Z + 90 minutes",
        &["'2020-01-01T13:30:00+00:00'"],
    )
    .await;
    assert_evaluates(
        "@2020-01-01T12:00:00Z - 1 year",
        &["'2019-01-01T12:00:00+00:00'"],
    )
    .await;
    assert_evaluates(
        "@2020-01-01T12:00:00Z + 1.5 seconds",
        &["'2020-01-01T12:00:01.500+00:00'"],
    )
    .await;
    assert_evaluates("@T10:00 + 1 hour", &["'11:00'"]).await;
    assert_evaluates("@T23:30:00 + 1 hour", &["'00:30:00'"]).await;
    assert_evaluates("@T10 + 90 minutes", &["'11'"]).await;
    // Units the value's precision cannot take, or not time units, are empty.
    assert_evaluates("@2020-01-01 + 1 hour", &[]).await;
    assert_evaluates("@T10:00 + 1 day", &[]).await;
    assert_evaluates("@2014 + 1 day", &[]).await;
    assert_evaluates("@2020-01-01 + 1 'mg'", &[]).await;
    assert_evaluates("@2020-01-01 + {}", &[]).await;
    assert_evaluates("1 + 2", &["3.0"]).await;
    assert_evaluates("5 - 2", &["3.0"]).await;
}

/// <https://hl7.org/fhirpath/N1/#functions>
#[tokio::test]
async fn index_in_iteration_functions() {
    assert_evaluates("('a' | 'b' | 'c').where($index > 0)", &["'b'", "'c'"]).await;
    assert_evaluates("('a' | 'b' | 'c').select($index)", &["0", "1", "2"]).await;
    assert_evaluates("('a' | 'b' | 'c').all($index < 3)", &["true"]).await;
    assert_evaluates("('a' | 'b' | 'c').all($index < 2)", &["false"]).await;
    // Nested iterations bind their own index.
    assert_evaluates(
        "('a' | 'b').select(('x' | 'y' | 'z').where($index = 2))",
        &["'z'", "'z'"],
    )
    .await;
    // Outside an iteration there is no index.
    assert_evaluates("$index", &[]).await;
}
