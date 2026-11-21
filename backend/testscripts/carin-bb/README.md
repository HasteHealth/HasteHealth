# CARIN Blue Button 2.2.0 TestScripts

TestScripts for [CARIN Consumer Directed Payer Data Exchange (CARIN Blue Button)](https://hl7.org/fhir/us/carin-bb/STU2.2/) 2.2.0, run against the
`hl7.fhir.us.carin-bb` package in `artifacts/r4/hl7.fhir.us.carin-bb` (the package exactly as HL7
publishes it, never edited). The guide depends on US Core 7.0.0, which is installed first from
`artifacts/r4/hl7.fhir.us.core-7.0.0`.

## Layout

- `2.2.0/examples/generated/`: one TestScript per example the package publishes
  (42), written by `scripts/testscript_build.sh` from `example/`. Each creates the example
  with its profile claimed and asserts the server accepted it, so a pass means the resource
  validated against its profile.
- `2.2.0/profiles/`: one conformance TestScript per profile the guide's server
  CapabilityStatement (`CapabilityStatement-c4bb.json`) lists (11). Each writes a
  resource that fills the profile's required and must-support elements, fixed values and required
  extensions, reads it back and checks those elements are present, then runs every search
  parameter the guide adds for that resource type (equality, modifiers, prefixes and `:missing`
  as the parameter type allows) against it. `TestScript.metadata` links the profile and names the
  CapabilityStatement, and the `testscript-category` extension groups the actions the way the
  website's conformance page shows them.

## How they run

CI runs them in `.github/workflows/test_e2e_carin_bb.yml` (called by `test_e2e.yml`) on both search
backends, as the admin client, after installing US Core 7.0.0 and then the package with
`haste-health package install <folder> --no-dependencies`. The website's
[CARIN Blue Button 2.2.0](https://haste.health/docs/reference/conformance/carin-bb) page is built from the
TestReports.

To run them locally against a server that has a tenant and the `testing` admin profile, from
`backend/`:

```sh
./target/debug/haste-health package install ../artifacts/r4/hl7.fhir.us.core-7.0.0 --no-dependencies
./target/debug/haste-health package install ../artifacts/r4/hl7.fhir.us.carin-bb --no-dependencies
./target/debug/haste-health testscript run -i testscripts/carin-bb/2.2.0 -w 0 \
  --index-wait-ms 30000 -o carin-bb-postgres.json
```

## Known failures

None. Every TestScript passes on both search backends.
