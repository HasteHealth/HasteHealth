# HL7 FHIRPath test suite

The official FHIRPath test suite, run against this engine by `../hl7_suite.rs`.

## Source

Both are from [FHIR/fhir-test-cases](https://github.com/FHIR/fhir-test-cases)
(Apache-2.0) at commit `2c1b5e8b98dd9e2ed0c1181d61b95dd21e1c596c`:

- `tests-fhir-r4.json`: the tests in `r4/fhirpath/tests-fhir-r4.xml`, one entry per
  `<test>`. Attributes and the expression's `invalid` and `mode` keep their names
  (the expression's `mode` is `expressionMode`), outputs keep their `type` and text,
  and `inputfile` names the JSON file in `input/`. The suite reuses the name
  `testEquivalent23`, so every test also has a unique `id` (`testEquivalent23#2`).
- `input/`: the resources the tests evaluate against, from `r4/`. The engine reads
  JSON only, so the XML files were converted to FHIR JSON using the R4
  StructureDefinitions in `artifacts/r4/hl7-core`. Decimals are written as they
  appear in the XML (`9.5e2`, `1.0`). The four inputs the suite already ships as
  JSON are copied unchanged.

## Running

```sh
cargo test -p haste-fhirpath --test hl7_suite -- --nocapture
```

prints every failing test and a summary. `known_failures.txt` lists the tests that
fail today. The run fails when a test fails that the list does not name, or when a
listed test passes. After fixing or breaking tests, regenerate the list:

```sh
HL7_UPDATE_KNOWN_FAILURES=1 cargo test -p haste-fhirpath --test hl7_suite
```

## How results are compared

- Each result item is compared with the expected output's type and value. System
  types use the suite's names (`System.String` is `string`) and FHIR types keep
  theirs (`code`, `date`).
- Date, dateTime and time outputs are written as literals (`@2014-01-01`). The `@`
  is ignored.
- A value that differs only in written precision (`1.58650000` against `1.5865`)
  passes. The summary counts these, since the engine stores decimals as `f64`.
- `predicate="true"` turns the result into a boolean: empty is false, a single
  boolean is itself, anything else is true.
- `ordered="false"` compares the outputs in any order.
- A test whose expression is marked `invalid` passes when evaluation returns an
  error of any kind.
- `context` is evaluated first, and the expression runs against its result.
- No environment variables (`%resource`, `%ucum`, `%vs-...`) are supplied, as the
  engine defines none itself.
