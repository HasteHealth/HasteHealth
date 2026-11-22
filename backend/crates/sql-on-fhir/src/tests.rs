//! A run builds no more rows than it can use: up to `_limit`, else one past `max_rows`. And it
//! takes no more than `max_steps` steps, whether or not they give rows.

use super::*;
use serde_json::{Value, json};

/// A Patient with two names, no contact and `identifiers` identifiers, valued `v0`, `v1`, ...
fn patient(id: &str, identifiers: usize) -> Resource {
    let identifier = (0..identifiers)
        .map(|i| json!({"value": format!("v{i}")}))
        .collect::<Vec<_>>();

    serde_json::from_value(json!({
        "resourceType": "Patient",
        "id": id,
        "name": [
            {"family": "F1", "given": ["G1", "G2"]},
            {"family": "F2", "given": ["G3"]},
        ],
        "identifier": identifier,
    }))
    .unwrap()
}

fn view(selects: &Value) -> ViewDefinition {
    serde_json::from_value(json!({
        "resourceType": "ViewDefinition",
        "resource": "Patient",
        "status": "active",
        "select": selects,
    }))
    .unwrap()
}

/// A select with a row per identifier: its value, in column `name`.
fn per_identifier(name: &str) -> Value {
    json!({"forEach": "identifier", "column": [{"name": name, "path": "value"}]})
}

/// `count` selects side by side, each with a row per identifier, in columns `c0`, `c1`, ...
fn per_identifier_selects(count: usize) -> Vec<Value> {
    (0..count)
        .map(|i| per_identifier(&format!("c{i}")))
        .collect()
}

/// A select for under a name, with a row per given name: its value, in column `name`.
fn per_given(name: &str) -> Value {
    json!({"forEach": "given", "column": [{"name": name, "path": "$this"}]})
}

/// A select that gives a [`patient`] no rows.
fn no_rows() -> Value {
    json!({"forEach": "contact", "column": [{"name": "contact", "path": "id"}]})
}

/// A select with a row per name and per value of its `unionAll`: the name's given names, then
/// its family name.
fn names_with_union() -> Value {
    json!({
        "forEach": "name",
        "column": [{"name": "family", "path": "family"}],
        "unionAll": [
            per_given("u"),
            {"column": [{"name": "u", "path": "family"}]},
            {"forEach": "contact", "column": [{"name": "u", "path": "id"}]},
        ],
    })
}

/// A select nested in `depth` selects, each read on six nodes. It gives no rows, so neither do
/// the selects around it, on any of their 6^`depth` nodes.
fn nested_without_rows(depth: usize) -> Value {
    (0..depth).fold(
        no_rows(),
        |select, _| json!({"forEach": "(1 | 2 | 3 | 4 | 5 | 6)", "select": [select]}),
    )
}

/// The view's rows over `resources`, each as the JSON output writes it, and the steps the run
/// took.
async fn run(
    view: &ViewDefinition,
    resources: &[Resource],
    limit: Option<usize>,
    limits: RunLimits,
) -> (Result<Vec<String>, OperationOutcomeError>, usize) {
    let fp_engine = FPEngine::new();
    let run = Run {
        fp_engine: &fp_engine,
        variables: Arc::new(build_hashmap_fp_variables(view)),
        view_definition: view,
        where_clauses: Vec::new(),
        resource_type: view_resource_type(view).unwrap(),
        limit,
        limits,
        steps: Steps::new(limits.max_steps),
    };

    let mut rows = Vec::new();
    let result = add_rows(&run, resources, &mut rows).await.map(|_| {
        rows.iter()
            .map(|row| serde_json::to_string(row).unwrap())
            .collect()
    });

    (result, run.steps.taken.load(Ordering::Relaxed))
}

/// The view's rows over `resources`, each as the JSON output writes it.
async fn rows(
    view: &ViewDefinition,
    resources: &[Resource],
    limit: Option<usize>,
    max_rows: usize,
) -> Result<Vec<String>, OperationOutcomeError> {
    let limits = RunLimits {
        max_rows,
        ..RunLimits::default()
    };

    run(view, resources, limit, limits).await.0
}

/// The run of the view over `resources`, with at most `max_steps` steps.
async fn run_within(
    view: &ViewDefinition,
    resources: &[Resource],
    max_steps: usize,
) -> (Result<Vec<String>, OperationOutcomeError>, usize) {
    let limits = RunLimits {
        max_steps,
        ..RunLimits::default()
    };

    run(view, resources, None, limits).await
}

/// Whether the run was refused for giving more than `max_rows` rows.
fn refused(result: &Result<Vec<String>, OperationOutcomeError>) -> bool {
    result
        .as_ref()
        .is_err_and(|error| error.outcome().issue[0].code.as_str() == Some("too-costly"))
}

// The products below have 10^12 rows or more. A run that built them would not end.

#[tokio::test]
async fn a_product_past_max_rows_is_refused() {
    let view = view(&json!(per_identifier_selects(4)));

    let result = rows(&view, &[patient("p", 1000)], None, 10_000).await;

    assert!(refused(&result));
}

#[tokio::test]
async fn limit_gives_the_first_rows_of_a_product() {
    let view = view(&json!(per_identifier_selects(4)));

    let rows = rows(&view, &[patient("p", 1000)], Some(3), 10_000)
        .await
        .unwrap();

    // The last select's rows change fastest.
    assert_eq!(
        rows,
        [
            r#"{"c0":"v0","c1":"v0","c2":"v0","c3":"v0"}"#,
            r#"{"c0":"v0","c1":"v0","c2":"v0","c3":"v1"}"#,
            r#"{"c0":"v0","c1":"v0","c2":"v0","c3":"v2"}"#,
        ]
    );
}

#[tokio::test]
async fn nested_selects_multiply_no_further_than_the_limits() {
    // The first name has two given names, so its 40 nested selects multiply to 2^40 rows.
    let nested = (0..40)
        .map(|i| per_given(&format!("g{i}")))
        .collect::<Vec<_>>();
    let view = view(&json!([{"forEach": "name", "select": nested}]));
    let patients = [patient("p", 1)];

    assert!(refused(&rows(&view, &patients, None, 10_000).await));

    let rows = rows(&view, &patients, Some(2), 10_000).await.unwrap();
    assert_eq!(rows.len(), 2);
    assert!(rows[0].ends_with(r#""g38":"G1","g39":"G1"}"#));
    assert!(rows[1].ends_with(r#""g38":"G1","g39":"G2"}"#));
}

#[tokio::test]
async fn a_select_without_rows_leaves_the_product_empty() {
    // Wherever it is among the selects, and however many rows the others multiply to.
    for position in 0..=4 {
        let mut selects = per_identifier_selects(4);
        selects.insert(position, no_rows());

        let rows = rows(&view(&json!(selects)), &[patient("p", 1000)], None, 10_000)
            .await
            .unwrap();

        assert!(rows.is_empty());
    }
}

#[tokio::test]
async fn rows_keep_their_order() {
    let view = view(&json!([
        {"forEachOrNull": "contact", "column": [{"name": "contact", "path": "id"}]},
        names_with_union(),
        per_identifier("a"),
    ]));

    let rows = rows(&view, &[patient("p", 2)], None, 10_000).await.unwrap();

    assert_eq!(
        rows,
        [
            r#"{"contact":null,"family":"F1","u":"G1","a":"v0"}"#,
            r#"{"contact":null,"family":"F1","u":"G1","a":"v1"}"#,
            r#"{"contact":null,"family":"F1","u":"G2","a":"v0"}"#,
            r#"{"contact":null,"family":"F1","u":"G2","a":"v1"}"#,
            r#"{"contact":null,"family":"F1","u":"F1","a":"v0"}"#,
            r#"{"contact":null,"family":"F1","u":"F1","a":"v1"}"#,
            r#"{"contact":null,"family":"F2","u":"G3","a":"v0"}"#,
            r#"{"contact":null,"family":"F2","u":"G3","a":"v1"}"#,
            r#"{"contact":null,"family":"F2","u":"F2","a":"v0"}"#,
            r#"{"contact":null,"family":"F2","u":"F2","a":"v1"}"#,
        ]
    );
}

#[tokio::test]
async fn limits_cut_the_rows_a_whole_run_gives() {
    let views = [
        // Side by side.
        json!([
            per_identifier("a"),
            {"forEach": "name", "column": [{"name": "family", "path": "family"}]},
            per_identifier("b"),
        ]),
        // Nested, between selects of one row and of several.
        json!([
            {"column": [{"name": "id", "path": "id"}]},
            {
                "forEach": "name",
                "column": [{"name": "family", "path": "family"}],
                "select": [
                    per_given("given"),
                    {"forEachOrNull": "period", "column": [{"name": "start", "path": "start"}]},
                ],
            },
            per_identifier("a"),
        ]),
        // A `unionAll` under nested selects.
        json!([
            {
                "column": [{"name": "id", "path": "id"}],
                "select": [per_identifier("a"), names_with_union()],
            },
            {"forEachOrNull": "contact", "column": [{"name": "contact", "path": "id"}]},
        ]),
    ];
    let patients = [patient("p1", 3), patient("p2", 1), patient("p3", 4)];

    for selects in &views {
        let view = view(selects);
        let all = rows(&view, &patients, None, 10_000).await.unwrap();
        assert!(all.len() > 20, "{} rows", all.len());

        for count in 0..=all.len() + 1 {
            // `_limit` gives the first rows.
            let limited = rows(&view, &patients, Some(count), 10_000).await.unwrap();
            assert_eq!(limited, all[..count.min(all.len())]);

            // Without it, the run is refused unless `max_rows` covers every row.
            let result = rows(&view, &patients, None, count).await;
            if count < all.len() {
                assert!(refused(&result), "max_rows {count} of {}", all.len());
            } else {
                assert_eq!(result.unwrap(), all);
            }
        }
    }
}

#[tokio::test]
async fn a_mistake_is_reported_where_no_row_is_used() {
    // The first name has two given names: a collection, in a column not marked as one.
    let mistake = json!({"column": [{"name": "given", "path": "name.given"}]});
    let patients = [patient("p", 1)];

    // With a `_limit` of 0.
    let result = rows(&view(&json!([mistake])), &patients, Some(0), 10_000).await;
    assert_eq!(result.unwrap_err().http_status(), 400);

    // Before a select without rows.
    let result = rows(&view(&json!([mistake, no_rows()])), &patients, None, 10_000).await;
    assert_eq!(result.unwrap_err().http_status(), 400);
}

#[tokio::test]
async fn a_column_too_costly_to_evaluate_is_refused() {
    // A string ten times as long with every call: 100 MB.
    let path = format!("'a'{}.length()", ".replace('a', 'aaaaaaaaaa')".repeat(8));
    let column = json!({"column": [{"name": "length", "path": path}]});

    let result = rows(&view(&json!([column])), &[patient("p", 1)], None, 10_000).await;

    let error = result.unwrap_err();
    assert_eq!(error.outcome().issue[0].code.as_str(), Some("too-costly"));
    assert_eq!(error.http_status(), 422);
}

#[tokio::test]
async fn a_run_takes_a_step_per_evaluation_node_and_value() {
    let patients = [patient("p", 3)];
    let columns = json!({"column": [
        {"name": "id", "path": "id"},
        {"name": "active", "path": "active"},
    ]});
    let contact =
        json!({"forEachOrNull": "contact", "column": [{"name": "contact", "path": "id"}]});
    let steps = async |selects: Value| {
        let (rows, steps) = run_within(&view(&selects), &patients, usize::MAX).await;
        (rows.unwrap().len(), steps)
    };

    // The select and its node, and two columns.
    assert_eq!(steps(json!([columns])).await, (1, 4));
    // The select and its `forEach`, then three nodes with a column each.
    assert_eq!(steps(json!([per_identifier("a")])).await, (3, 8));
    // The select and its `forEachOrNull`, then the select and the column of the null row.
    assert_eq!(steps(json!([contact])).await, (1, 4));
    // Both of the first two, and their product: three rows of three values.
    assert_eq!(steps(json!([columns, per_identifier("a")])).await, (3, 21));
    // Nested, the inner select is read on each of the six nodes, where it evaluates its `forEach`.
    assert_eq!(steps(json!([nested_without_rows(1)])).await, (0, 20));
}

#[tokio::test]
async fn a_run_is_refused_past_max_steps() {
    let view = view(&json!([per_identifier("a"), per_identifier("b")]));
    let patients = [patient("p1", 3), patient("p2", 3)];

    let (rows, steps) = run_within(&view, &patients, usize::MAX).await;
    assert_eq!(rows.unwrap().len(), 18);

    let (rows, _) = run_within(&view, &patients, steps).await;
    assert_eq!(rows.unwrap().len(), 18);

    let (result, _) = run_within(&view, &patients, steps - 1).await;
    assert!(refused(&result));
    assert_eq!(result.unwrap_err().http_status(), 422);
}

#[tokio::test]
async fn selects_without_rows_are_refused_past_max_steps() {
    let patients = [patient("p", 1)];

    // 6^3 nodes.
    let (rows, steps) = run_within(&view(&json!([nested_without_rows(3)])), &patients, 1000).await;
    assert!(rows.unwrap().is_empty());
    assert_eq!(steps, 776);

    // 6^20 nodes: a run that read the innermost select on each would not end.
    let (result, steps) =
        run_within(&view(&json!([nested_without_rows(20)])), &patients, 100_000).await;
    assert!(refused(&result));
    assert_eq!(steps, 100_001);
}

#[tokio::test]
async fn a_long_run_lets_other_tasks_run() {
    // The test's runtime has one thread, so the other task gets a turn only when the run pauses.
    let turns = Arc::new(AtomicUsize::new(0));
    let other = tokio::spawn({
        let turns = turns.clone();
        async move {
            loop {
                turns.fetch_add(1, Ordering::Relaxed);
                tokio::task::yield_now().await;
            }
        }
    });

    let view = view(&json!([nested_without_rows(5)]));
    let (rows, steps) = run_within(&view, &[patient("p", 1)], usize::MAX).await;
    other.abort();

    assert!(rows.unwrap().is_empty());
    assert!(steps > 10 * STEPS_PER_PAUSE);
    assert!(turns.load(Ordering::Relaxed) >= steps / STEPS_PER_PAUSE);
}
