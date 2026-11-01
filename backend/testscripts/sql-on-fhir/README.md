# SQL on FHIR TestScripts

TestScripts for the [SQL on FHIR v2](https://build.fhir.org/ig/HL7/sql-on-fhir/) shared test suite:
the `tests/` directory of [FHIR/sql-on-fhir.js](https://github.com/FHIR/sql-on-fhir.js) at commit
`238d9b2be4ad0d66f9b5a9e640f6337f85b7904c`, the suite implementations run to be listed on the
[implementations page](https://sql-on-fhir.org/extra/impls.html). It is MIT licensed; `LICENSE.md`
here is its license.

The suite has 144 tests in 22 files. Each test a TestScript can express is one TestScript here
(106). The 38 it cannot are listed under [Not covered](#not-covered). Nothing in those TestScripts
comes from anywhere but its suite test. Two more are Haste Health's own:
`validate/06-invalid-column-path` (`sof-validate-column-path`), a column path that does not parse
is refused with a client error, and `repeat/20-repeat-this` (`sof-repeat-20`), a `repeat` of
`$this` ends and returns the node once. That makes 108.

## From a suite test to a TestScript

- `<file>/<NN>-<test title>.testscript.json`, where `<file>` is the suite file and `NN` the test's
  position in it. The id is `sof-<file>-<NN>`, with the file name's `_` as `-` (`sof-fn-boundary-03`).
  The title is the suite's test case and test titles, and the description holds the test's
  description, its tags (`shareable` or `experimental`) and its source.
- Each runs one `ViewDefinition/$viewdefinition-run` with the contained Parameters
  `run-parameters`: `viewResource` is the test's `view` (with `resourceType` added), one
  `resource` per resource of the test case's `resources`, in order, and `_format` `json`. Nothing is
  written to the server.
- A test with `expect` asserts a 200, a JSON `Binary`, and that `Binary.data` is the expected rows
  as compact JSON. The assert's description shows the rows.
  - The suite's own runner compares rows in any order and the keys of a row in any order. A
    TestScript compares the bytes, so each row's keys are written in the order the view defines its
    columns: a select's columns, then its nested selects' columns, then its `unionAll`'s, then the
    next select's. That is the order the suite's `column ordering` test (`basic.json`, test 11)
    expects, and it also covers the one test with `expectColumns`.
  - Rows are written in the suite's order, except in `sof-foreach-04` (`forEach: two on the same
level`). Its four rows are the suite's, written in the order Haste Health returns them.
- A test with `expectError` asserts that the response is an OperationOutcome holding an error and
  that it is not a server failure (5xx). Like the suite's runner, it accepts any such error.

## How they run

CI runs them in `.github/workflows/test_e2e_sql_on_fhir.yml` (called by `test_e2e.yml`) on the
Postgres search backend, as the admin client. Every TestScript runs. Each id in
`known-failures.txt` is passed as `--allow-failure`, so those TestReports stay `fail`, marked as
expected, and do not fail the job. A failure the list does not name fails it, and the run warns
when a listed TestScript passes.

To run them locally against a server that has a tenant and the `testing` admin profile, from
`backend/`:

```sh
allowed=()
while read -r id; do allowed+=(--allow-failure "$id"); done < <(
  sed -e 's/#.*//' -e 's/[[:space:]]//g' -e '/^$/d' testscripts/sql-on-fhir/known-failures.txt)
./target/debug/haste-health testscript run -i testscripts/sql-on-fhir -w 0 "${allowed[@]}" \
  -o sql-on-fhir-postgres.json
```

## Layout

| Directory            | Suite tests | TestScripts |
| -------------------- | ----------- | ----------- |
| `basic/`             | 11          | 9           |
| `collection/`        | 4           | 4           |
| `combinations/`      | 6           | 0           |
| `constant/`          | 8           | 7           |
| `constant_types/`    | 14          | 14          |
| `fhirpath/`          | 9           | 9           |
| `fhirpath_numbers/`  | 1           | 1           |
| `fn_boundary/`       | 8           | 8           |
| `fn_empty/`          | 1           | 1           |
| `fn_extension/`      | 2           | 0           |
| `fn_first/`          | 2           | 0           |
| `fn_join/`           | 3           | 0           |
| `fn_oftype/`         | 2           | 2           |
| `fn_reference_keys/` | 3           | 0           |
| `foreach/`           | 13          | 9           |
| `logic/`             | 3           | 0           |
| `repeat/`            | 19          | 20          |
| `row_index/`         | 9           | 9           |
| `union/`             | 10          | 10          |
| `validate/`          | 5           | 3           |
| `view_resource/`     | 3           | 2           |
| `where/`             | 8           | 0           |
| **Total**            | **144**     | **108**     |

## Known failures

On 2026-10-31, 89 of the 108 pass. The other 19 are in `known-failures.txt`, by cause. A
TestScript that fails for several causes is listed under the first that shows.

| TestScripts | Cause                                                                                                                                    |
| ----------- | ---------------------------------------------------------------------------------------------------------------------------------------- |
| 9           | `%rowIndex` is not defined, so a view that uses it is refused with 400 "Undefined constant: %rowIndex" (`sof-row-index-01` to `-09`).    |
| 8           | `lowBoundary()` and `highBoundary()` are not implemented, so the run answers 500 (`sof-fn-boundary-01` to `-08`).                        |
| 2           | `unionAll` branches whose columns differ, or come in another order, are not refused; the suite expects an error (`sof-union-09`, `-10`). |

## Not covered

A TestScript holds the view in a Parameters resource, so the view has to be a valid ViewDefinition.
These 38 suite tests' views are not:

- 33 tests that expect rows have a view with no `status`, which ViewDefinition requires. The server
  refuses such a view too, with 400 "missing field `status`", so all of them would fail. `basic.json`
  10 and 11; `combinations.json` 1 to 6; `fn_extension.json` 1 and 2; `fn_first.json` 1 and 2;
  `fn_join.json` 1 to 3; `fn_reference_keys.json` 1 to 3; `foreach.json` 10 to 13; `logic.json` 1 to
  3; `where.json` 1 to 8.
- 5 tests that expect an error have a view that is invalid on purpose. The server refuses each with
  400, which is what the suite expects:
  - `validate.json` 1 (`empty`) and 2 (`missing resource`): no `status`.
  - `validate.json` 4 (`wrong type in forEach`): `forEach` is the number `1`.
  - `constant.json` 8 (`incorrect constant definition`): a constant without a value.
  - `view_resource.json` 3 (`resource not specified`): no `resource`.
