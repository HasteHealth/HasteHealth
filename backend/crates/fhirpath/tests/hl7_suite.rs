//! The HL7 FHIRPath test suite, run against this engine.
//!
//! `hl7/tests-fhir-r4.json` holds the tests extracted from the suite's XML and
//! `hl7/input/` the resources they evaluate against, converted to JSON (see
//! `hl7/README.md`). The run fails when a test fails that `hl7/known_failures.txt`
//! does not list, or when a listed test passes. Regenerate the list with
//! `HL7_UPDATE_KNOWN_FAILURES=1 cargo test -p haste-fhirpath --test hl7_suite`
//! and print the full report with `-- --nocapture`.

use haste_fhir_model::r4::{
    conversion::{
        BOOLEAN_TYPES, DATE_TIME_TYPES, NUMBER_TYPES, STRING_TYPES, downcast_bool,
        downcast_datetime, downcast_number, downcast_string,
    },
    generated::resources::Resource,
};
use haste_fhirpath::FPEngine;
use haste_reflect::MetaValue;
use serde_json::Value;
use std::{
    collections::{BTreeSet, HashMap},
    fmt::Write as _,
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};

const SYSTEM_PREFIX: &str = "http://hl7.org/fhirpath/System.";
const UPDATE_ENV: &str = "HL7_UPDATE_KNOWN_FAILURES";
const TIMEOUT: Duration = Duration::from_secs(10);

fn suite_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/hl7")
}

/// A result item the way the suite writes an output: a type name and a value.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Item {
    type_name: String,
    value: String,
}

impl std::fmt::Display for Item {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {:?}", self.type_name, self.value)
    }
}

/// `System.String` becomes `string` and `System.Quantity` stays `Quantity`, matching
/// the suite's output types. FHIR types keep their names.
fn type_name(value: &dyn MetaValue) -> String {
    let fhir_type = value.fhir_type();
    match fhir_type.strip_prefix(SYSTEM_PREFIX) {
        Some("Quantity") => "Quantity".to_string(),
        Some(system) => {
            let mut chars = system.chars();
            chars
                .next()
                .map(|first| first.to_ascii_lowercase().to_string() + chars.as_str())
                .unwrap_or_default()
        }
        None => fhir_type.to_string(),
    }
}

fn render(value: &dyn MetaValue) -> String {
    let fhir_type = value.fhir_type();
    if let Some(integer) = value.as_any().downcast_ref::<i64>() {
        return integer.to_string();
    }
    if BOOLEAN_TYPES.contains(fhir_type)
        && let Ok(boolean) = downcast_bool(value)
    {
        return boolean.to_string();
    }
    if NUMBER_TYPES.contains(fhir_type)
        && let Ok(number) = downcast_number(value)
    {
        return number.to_string();
    }
    if STRING_TYPES.contains(fhir_type)
        && let Ok(string) = downcast_string(value)
    {
        return string;
    }
    if DATE_TIME_TYPES.contains(fhir_type)
        && let Ok(temporal) = downcast_datetime(value)
    {
        return temporal;
    }
    if type_name(value) == "Quantity" {
        // flatten() unwraps the Box that get_field returns for the value.
        let field = |name: &str| {
            value
                .get_field(name)
                .and_then(|v| v.flatten().first().copied())
        };
        let number = field("value")
            .and_then(|v| downcast_number(v).ok())
            .map(|n| n.to_string())
            .unwrap_or_default();
        let unit = ["code", "unit"]
            .iter()
            .find_map(|name| field(name).and_then(|v| downcast_string(v).ok()))
            .unwrap_or_default();
        return format!("{number} '{unit}'");
    }
    format!("<{fhir_type}>")
}

#[derive(PartialEq)]
enum ValueMatch {
    Exact,
    /// Equal as numbers, but the suite writes a precision the engine does not keep.
    Numeric,
    No,
}

fn match_value(expected: &str, actual: &str) -> ValueMatch {
    // The suite writes date, dateTime and time outputs as literals (`@2014-01-01`).
    if expected == actual || expected.strip_prefix('@') == Some(actual) {
        return ValueMatch::Exact;
    }
    match (expected.parse::<f64>(), actual.parse::<f64>()) {
        (Ok(e), Ok(a)) if e == a => ValueMatch::Numeric,
        _ => ValueMatch::No,
    }
}

enum Outcome {
    Pass {
        numeric_only: bool,
    },
    Fail {
        category: &'static str,
        detail: String,
    },
}

fn fail(category: &'static str, detail: String) -> Outcome {
    Outcome::Fail { category, detail }
}

fn str_field<'a>(test: &'a Value, field: &str) -> Option<&'a str> {
    test.get(field).and_then(Value::as_str)
}

fn bool_field(test: &Value, field: &str) -> bool {
    test.get(field).and_then(Value::as_bool).unwrap_or(false)
}

fn expected_items(test: &Value) -> Vec<(Option<String>, String)> {
    test["outputs"]
        .as_array()
        .into_iter()
        .flatten()
        .map(|output| {
            (
                str_field(output, "type").map(str::to_string),
                str_field(output, "value").unwrap_or_default().to_string(),
            )
        })
        .collect()
}

fn format_expected(expected: &[(Option<String>, String)]) -> String {
    let items: Vec<String> = expected
        .iter()
        .map(|(t, v)| format!("{} {v:?}", t.as_deref().unwrap_or("(untyped)")))
        .collect();
    format!("[{}]", items.join(", "))
}

fn format_actual(actual: &[Item]) -> String {
    let items: Vec<String> = actual.iter().map(Item::to_string).collect();
    format!("[{}]", items.join(", "))
}

/// Evaluates the test's expression (after its `context` expression, when it has one)
/// on its own thread, so a panic or an evaluation that never finishes is reported as
/// that test's failure. A thread that times out is left running until the process exits.
fn evaluate(
    resource: Option<Arc<Resource>>,
    context: Option<String>,
    expression: String,
) -> Result<Result<Vec<Item>, String>, Outcome> {
    let (sender, receiver) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            tokio::runtime::Builder::new_current_thread()
                .build()
                .unwrap()
                .block_on(evaluate_async(resource, context, expression))
        }));
        let _ = sender.send(outcome.map_err(|panic| {
            panic
                .downcast_ref::<String>()
                .cloned()
                .or_else(|| panic.downcast_ref::<&str>().map(|s| (*s).to_string()))
                .unwrap_or_else(|| "panic".to_string())
        }));
    });
    match receiver.recv_timeout(TIMEOUT) {
        Ok(Ok(result)) => Ok(result),
        Ok(Err(panic)) => Err(fail("panic", format!("panicked: {panic}"))),
        Err(_) => Err(fail(
            "timeout",
            format!("did not finish within {} s", TIMEOUT.as_secs()),
        )),
    }
}

async fn evaluate_async(
    resource: Option<Arc<Resource>>,
    context: Option<String>,
    expression: String,
) -> Result<Vec<Item>, String> {
    let engine = FPEngine::new();
    let input: Vec<&dyn MetaValue> = match resource.as_deref() {
        Some(resource) => vec![resource],
        None => vec![],
    };
    let context_result = match &context {
        Some(context) => Some(
            engine
                .evaluate(context, input.clone())
                .await
                .map_err(|e| format!("context expression failed: {e}"))?,
        ),
        None => None,
    };
    let values = match &context_result {
        Some(result) => result.iter().collect(),
        None => input,
    };
    let result = engine
        .evaluate(&expression, values)
        .await
        .map_err(|e| e.to_string())?;
    Ok(result
        .iter()
        .map(|value| Item {
            type_name: type_name(value),
            value: render(value),
        })
        .collect())
}

fn run_test(test: &Value, resource: Option<Arc<Resource>>) -> Outcome {
    let expression = str_field(test, "expression")
        .unwrap_or_default()
        .to_string();
    let context = str_field(test, "context").map(str::to_string);
    let invalid = str_field(test, "invalid");
    let expected = expected_items(test);

    let result = match evaluate(resource, context, expression) {
        Ok(result) => result,
        Err(outcome) => return outcome,
    };

    let mut actual = match (result, invalid) {
        (Err(_), Some(_)) => {
            return Outcome::Pass {
                numeric_only: false,
            };
        }
        (Ok(actual), Some(kind)) => {
            return fail(
                "no error raised",
                format!("expected a {kind} error, got {}", format_actual(&actual)),
            );
        }
        (Err(error), None) => {
            return fail(
                "unexpected error",
                format!(
                    "expected {}, got error: {error}",
                    format_expected(&expected)
                ),
            );
        }
        (Ok(actual), None) => actual,
    };

    if bool_field(test, "predicate") {
        // Evaluated as a predicate: empty is false, a single boolean is itself and
        // anything else is true.
        let value = match actual.as_slice() {
            [] => false,
            [item] if item.type_name == "boolean" => item.value == "true",
            _ => true,
        };
        actual = vec![Item {
            type_name: "boolean".to_string(),
            value: value.to_string(),
        }];
    }

    let mut expected_sorted = expected.clone();
    if test.get("ordered").and_then(Value::as_bool) == Some(false) {
        expected_sorted.sort_by(|a, b| a.1.cmp(&b.1));
        actual.sort_by(|a, b| a.value.cmp(&b.value));
    }

    let mismatch = || {
        format!(
            "expected {}, got {}",
            format_expected(&expected),
            format_actual(&actual)
        )
    };
    if expected_sorted.len() != actual.len() {
        return fail("wrong result", mismatch());
    }
    let pairs = || expected_sorted.iter().zip(&actual);
    let matches: Vec<ValueMatch> = pairs()
        .map(|((_, expected_value), item)| match_value(expected_value, &item.value))
        .collect();
    if matches.contains(&ValueMatch::No) {
        return fail("wrong result", mismatch());
    }
    let types_match = pairs().all(|((expected_type, _), item)| {
        expected_type
            .as_ref()
            .is_none_or(|expected_type| expected_type == &item.type_name)
    });
    if !types_match {
        return fail("wrong type", mismatch());
    }
    Outcome::Pass {
        numeric_only: matches.contains(&ValueMatch::Numeric),
    }
}

fn load_input(
    cache: &mut HashMap<String, Arc<Resource>>,
    file: &str,
) -> Result<Arc<Resource>, String> {
    if let Some(resource) = cache.get(file) {
        return Ok(resource.clone());
    }
    let path = suite_dir().join("input").join(file);
    let text = std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    let resource: Resource =
        serde_json::from_str(&text).map_err(|e| format!("{file} does not deserialize: {e}"))?;
    let resource = Arc::new(resource);
    cache.insert(file.to_string(), resource.clone());
    Ok(resource)
}

fn read_known_failures() -> BTreeSet<String> {
    std::fs::read_to_string(suite_dir().join("known_failures.txt"))
        .unwrap_or_default()
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(str::to_string)
        .collect()
}

fn write_known_failures(failures: &[(String, String)]) {
    let mut text = String::from(
        "# HL7 FHIRPath suite tests that fail today, by group. Regenerate with\n\
         # HL7_UPDATE_KNOWN_FAILURES=1 cargo test -p haste-fhirpath --test hl7_suite\n",
    );
    let mut group = "";
    for (id, test_group) in failures {
        if test_group != group {
            group = test_group;
            let _ = write!(text, "\n# {group}\n");
        }
        let _ = writeln!(text, "{id}");
    }
    std::fs::write(suite_dir().join("known_failures.txt"), text).unwrap();
}

#[test]
fn hl7_fhirpath_suite() {
    let suite: Value = serde_json::from_str(
        &std::fs::read_to_string(suite_dir().join("tests-fhir-r4.json")).unwrap(),
    )
    .unwrap();
    let tests = suite["tests"].as_array().unwrap();

    let mut inputs = HashMap::new();
    let mut report = String::new();
    let mut failed: Vec<(String, String)> = Vec::new();
    let mut categories: HashMap<&'static str, usize> = HashMap::new();
    let mut numeric_only = 0;

    // Panics are caught per test and reported below.
    std::panic::set_hook(Box::new(|_| {}));
    for test in tests {
        let id = str_field(test, "id").unwrap().to_string();
        let group = str_field(test, "group").unwrap_or_default().to_string();
        let outcome = match str_field(test, "inputfile") {
            Some(file) => match load_input(&mut inputs, file) {
                Ok(resource) => run_test(test, Some(resource)),
                Err(error) => fail("input", error),
            },
            None => run_test(test, None),
        };
        match outcome {
            Outcome::Pass { numeric_only: true } => numeric_only += 1,
            Outcome::Pass {
                numeric_only: false,
            } => {}
            Outcome::Fail { category, detail } => {
                *categories.entry(category).or_default() += 1;
                let tags: Vec<&str> = [
                    str_field(test, "mode"),
                    str_field(test, "expressionMode"),
                    str_field(test, "version").map(|_| "FHIRPath 2.1"),
                    bool_field(test, "checkOrderedFunctions").then_some("checkOrderedFunctions"),
                ]
                .into_iter()
                .flatten()
                .collect();
                let _ = writeln!(
                    report,
                    "FAIL {id} [{group}] {category}{}\n  expression: {}\n  {detail}",
                    if tags.is_empty() {
                        String::new()
                    } else {
                        format!(" ({})", tags.join(", "))
                    },
                    str_field(test, "expression")
                        .unwrap_or_default()
                        .replace('\n', "\n              "),
                );
                failed.push((id, group));
            }
        }
    }
    let _ = std::panic::take_hook();

    let mut categories: Vec<_> = categories.into_iter().collect();
    categories.sort_by_key(|(_, count)| std::cmp::Reverse(*count));
    println!("{report}");
    println!(
        "HL7 FHIRPath suite: {} tests, {} passed, {} failed",
        tests.len(),
        tests.len() - failed.len(),
        failed.len()
    );
    for (category, count) in &categories {
        println!("  {category}: {count}");
    }
    println!("  passed only as numbers (precision the suite writes is not kept): {numeric_only}");

    if std::env::var_os(UPDATE_ENV).is_some() {
        write_known_failures(&failed);
        return;
    }

    let known = read_known_failures();
    let failed_ids: BTreeSet<String> = failed.into_iter().map(|(id, _)| id).collect();
    let new_failures: Vec<_> = failed_ids.difference(&known).collect();
    let now_passing: Vec<_> = known.difference(&failed_ids).collect();
    assert!(
        new_failures.is_empty() && now_passing.is_empty(),
        "failing but not in known_failures.txt: {new_failures:?}\n\
         listed in known_failures.txt but passing: {now_passing:?}\n\
         rerun with {UPDATE_ENV}=1 to update the list"
    );
}
