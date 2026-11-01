# ONC (g)(10) TestScripts

TestScripts for the checks of the ONC Certification (g)(10) Standardized API test kit
([onc-certification-g10-test-kit](https://github.com/onc-healthit/onc-certification-g10-test-kit),
version 8.0.7, which runs on Inferno) that a TestScript can express. Each TestScript makes the
requests one Inferno test makes and asserts what it asserts, against Haste Health.

- Short ids are the kit's, for the suite options SMART App Launch 2.2.0, US Core 7.0.0 and Bulk
  Data 2.0.0. A TestScript's title starts with its short id. Its description names the requirement
  ids (from the kit's (g)(10) test procedure mapping) and the equivalent ids under SMART 2.0.0,
  US Core 6.1.0 or Bulk Data 1.0.1.
- Where a check cannot be expressed, it is listed under [Not covered](#not-covered) with the reason.
- Where Haste Health fails a check today, the TestScript is listed in `known-failures.txt`, and CI
  does not run it. See [Known failures](#known-failures).

## How they run

Most g10 checks depend on who is asking and with which SMART scopes: a patient, a practitioner or a
backend client. So the directory follows the access control layout. It runs with
`.github/scripts/access_control.mjs`, with `SCENARIOS=testscripts/g10`:

- `setup/` runs first, as the admin client. It writes the shared US Core record that every
  scenario reads (see [The record](#the-record)).
- Each other directory is a scenario:
  - `scenario.json` says who signs in and with which scopes. With `grantedScope`, the sign-in
    approves only those on the consent screen and unchecks the rest.
  - `setup/` writes that principal as the admin: a User linked to a Patient or Practitioner, a
    full-access policy so only the scopes limit it, and the client.
  - `tests/` runs as the signed-in principal.
- `known-failures.txt` lists the TestScripts the driver skips. A line ending in `/` skips a whole
  scenario.

CI runs this in `.github/workflows/test_e2e_g10.yml` (called by `test_e2e.yml`), on the Postgres
search backend. The steps are:

1. Load the US Core 7.0.0 profiles (`scripts/load_us_core.mjs hl7.fhir.us.core-7.0.0`).
2. Wait until the CapabilityStatement lists them. The server caches it for up to 30 seconds, and
   12.1.05 checks its `supportedProfile`.
3. Run the setup phase.
4. Wait for indexing.
5. Sign in.
6. Run the tests.

To run it locally against a server that has a tenant and the `testing` admin profile, run from the
repository root (and give the CapabilityStatement 30 seconds after loading US Core):

```sh
node backend/scripts/load_us_core.mjs hl7.fhir.us.core-7.0.0
SCENARIOS=testscripts/g10 node .github/scripts/access_control.mjs setup
bash .github/scripts/wait_for_indexing.sh 600
SCENARIOS=testscripts/g10 node .github/scripts/access_control.mjs sign-in
SCENARIOS=testscripts/g10 node .github/scripts/access_control.mjs test
```

TestReports are written to `backend/g10-reports/`.

## Layout

| Directory                                          | g10 scenario                                                                               | Signs in as                                                                                | TestScripts |
| -------------------------------------------------- | ------------------------------------------------------------------------------------------ | ------------------------------------------------------------------------------------------ | ----------- |
| `setup/`                                           | (shared)                                                                                   | admin client                                                                               | the record  |
| `standalone-patient-app/`                          | 1 Standalone Patient App, full access (discovery 1.8, launch 1.9, OpenID 1.11, access 1.7) | patient linked to `Patient/g10-example`, `patient/[type].rs` for the 26 types g10 requests | 26          |
| `limited-access-app/`                              | 2 Standalone Patient App, limited access (2.4, 2.3)                                        | the same patient, granting only Patient, Condition and Observation at consent              | 18          |
| `ehr-practitioner-app/`                            | 3 EHR Practitioner App (3.8, 3.10), and 9.19.01                                            | practitioner linked to `Practitioner/g10-practitioner-1`, `user/[type].rs`                 | 6           |
| `single-patient-api/`                              | 12 Single Patient API (US Core 7.0.0), one directory per group                             | practitioner linked to `Practitioner/g10-practitioner-2`, `user/[type].rs` (plus `Media`)  | 294         |
| `profile-validation/`                              | 12 profile validation tests (12.x validation)                                              | backend client, `system/*.cruds`                                                           | 47          |
| `multi-patient-api/`                               | 8 Multi-Patient Authorization and API (Bulk Data 2.0.0)                                    | backend client, `system/*.rs`                                                              | 6           |
| `smart-v1-scopes/`                                 | 9.22 App Launch with SMART v1 scopes                                                       | patient, `patient/[type].read`                                                             | 19          |
| `fine-grained-scopes-1/`, `fine-grained-scopes-2/` | 9.25 Fine-grained scopes, groups 1 and 2                                                   | patient linked to `Patient/g10-granular-patient`, granular scope groups 1 and 2            | 7 + 7       |

The SMART scenarios sign in through a public PKCE client. That is not the confidential client g10
registers, but client authentication is not what these TestScripts check.

## The record

`setup/g10-shared-record.testscript.json` writes one transaction of PUTs with fixed ids. It holds
the US Core examples from `artifacts/r4/test-data/testscripts` (`us_core` and `disabled`), each
with its id prefixed `g10-` and tagged `g10`:

- Four patients. `Patient/g10-example` holds data for every US Core profile, and the pediatric
  profiles belong to `g10-child-example` and `g10-infant-example`.
- The practitioners, organizations, locations, specimens and medications those patients'
  resources reference.

The examples are US Core 9.0.0; g10 tests US Core 6.1.0 or 7.0.0, and the suite loads 7.0.0. So the
record claims the US Core 7.0.0 profiles instead of the 9.0.0 ones. The implanted devices (`udi-2`,
`udi-3`) claim `us-core-implantable-device`, which 9.0.0 folds into `us-core-device`, and claims to
profiles 7.0.0 does not have are dropped: the devices that are not implants, the advance directive
DocumentReferences and Observations, and FamilyMemberHistory. The record also adds what US Core 7
requires or marks must support and the 9.0.0 examples dropped:

- `Patient.gender`, the `us-core-sex` extension, the CarePlan `assess-plan` category and narrative,
  and MedicationRequest `category`.
- Encounter `identifier`, `participant` and `reasonCode`.
- An abated encounter diagnosis, a not-done Immunization with `statusReason`, and a Device
  `distinctIdentifier`.
- A lab Observation with `valueString`, and coded preference Observations.
- A pulse oximetry concentration component, and blood pressure readings with a data-absent component.
- A Specimen `collection.bodySite`.

It also adds what the g10 guidance groups look for:

- 12.49: the eight clinical note types, a Pathology report, and report attachments shared with
  notes.
- 12.50: the screening categories.
- 12.51: a data-absent-reason extension.
- 12.x revinclude tests: one US Core Provenance per profile with a `_revinclude` test.
- 9.25: `Patient/g10-granular-patient` with one Condition or Observation per category.

Narratives are dropped (except CarePlan's, which is must support) and large attachments shortened.
R5 cross-version extensions are removed, since the server cannot resolve them. The four problem and
health concern Conditions are stored without `meta.profile`, because the server refuses them with it
(see [Known failures](#known-failures)).

The server validates every write against its `meta.profile`, so writing the record also checks
that the data conforms to US Core 7.0.0.

## Coverage

### 1 Standalone Patient App, full patient access (`standalone-patient-app/`)

- 1.8.01, 1.8.03, 1.8.04: the SMART configuration is served as JSON without a token and allows
  CORS; the CapabilityStatement allows CORS. Also stands for 1.2.01, 3.7.01, 3.7.03, 3.7.04,
  3.2.01 and the discovery steps of 9.x.
- 1.9.10: an invalid token is refused with 401. 1.9.11: the patient in context can be read.
- 1.11.02: the OpenID configuration is served at the id_token issuer. 1.11.04: the JWKS is served.
  1.11.07 and 1.11.08: the fhirUser can be read, with CORS.
- 1.7.02 to 1.7.14 and 1.7.21 to 1.7.24: the first US Core search of each type answers 200 and
  returns only the patient's data. 1.7.01 checks the granted scope string, so the TestScript checks
  the behaviour instead: the remaining types can be searched.

### 2 Standalone Patient App, limited access (`limited-access-app/`)

The app requests the same scopes as scenario 1. At consent the sign-in unchecks every resource type
but Patient, Condition and Observation (`grantedScope` in `scenario.json`), as the g10 tester does.

- 2.4.09: the patient in context can be read.
- 2.3.01 to 2.3.13 and 2.3.20 to 2.3.23: Patient, Condition and Observation searches answer 200;
  the others are refused with 401 or 403.

### 3 EHR Practitioner App (`ehr-practitioner-app/`)

- 3.8.01 and 3.8.02 (server side): `ClientApplication/[id]/$launch` returns launch, iss and the
  launch URL.
- 3.8.12: an invalid token is refused. 3.8.13 and 3.8.16: the patient and encounter in context can
  be read.
- 3.10.07 and 3.10.08: the practitioner fhirUser can be read, with CORS.
- 9.19.01 (server side of an EHR launch of a patient-scoped app) also lives here.

### 12 Single Patient API, US Core 7.0.0 (`single-patient-api/`, `profile-validation/`)

One directory per group, one TestScript per Inferno test:

- **Searches (12.x.01 to .05).** Every SHALL search g10 runs, with values taken from a record
  resource of the profile (as Inferno takes them from what the first search returned). Every result
  must match every parameter, and that record resource must be returned. Each search also runs its
  variants:
  - one request per fixed value the record holds;
  - `patient=Patient/[id]`, which must return the same count;
  - token `system|code`;
  - the gt, ge, lt and le date comparators, where every result must satisfy the comparator (by day,
    and, as in Inferno, an empty result passes);
  - comma-separated multiple-or values;
  - `_include=[type]:medication`;
  - POST `_search` for a group's first search.
- **Reads.** Read tests, including every referenced instance for the reference-only groups
  (Organization, Practitioner, Provenance, RelatedPerson, Specimen).
- **Provenance.** `_revinclude=Provenance:target` returns the Provenance targeting the record's
  resource.
- **Must support.** Every must-support element of the profile, USCDI ones included, appears in what
  the server returns for the record's instances of the profile. Slices are matched by their
  discriminator and extensions by URL.
- **Reference resolution.** Each must-support reference of the record's instances resolves.
- **Validation (12.x validation, `profile-validation/`).** Inferno validates returned resources with
  the HL7 validator, which a TestScript cannot run. The closest server-side check: the server has
  the profile, and a copy of a record resource claiming it is accepted, which the server allows only
  after validating it against US Core 7.0.0. The copies point at their own patient and are deleted
  afterwards.
- **12.1.02 to 12.1.05.** metadata is served; FHIR R4; JSON; the 46 US Core profiles g10 requires
  are listed in `supportedProfile`.
- **Guidance groups.** 12.49.01 and 12.49.02 (clinical notes), 12.50.01 (screening categories),
  12.51.01 and 12.51.02 (data absent reason).

### 8 Multi-Patient Authorization and API, Bulk Data 2.0.0 (`multi-patient-api/`)

These run as a client-credentials client with `system/*.rs`, standing in for SMART Backend Services.

- 8.2.02: the CapabilityStatement declares Group `$export` and instantiates the Bulk Data
  CapabilityStatement.
- 8.2.03: a kick-off with an invalid token is refused with 400 or 401.
- 8.2.04 and 8.2.05: the kick-off answers 202 with an absolute Content-Location, and polling it
  reaches 200 with `application/json`.
- 8.4.01 and 8.4.02: cancel answers 202, then polling answers 404 with an OperationOutcome.
- 8.5.01 and 8.5.02: `_outputFormat` (all three spellings) and `_since` are accepted.

The same checks cover 7.2.02 to 7.2.05 and 7.4.01 for Bulk Data 1.0.1.

### 9 Additional Authorization Tests

- 9.22.4.10 and 9.22.4.11: with SMART v1 scopes (`smart-v1-scopes/`), an invalid token is refused
  and the patient can be read.
- 9.22.5.02 to 9.22.5.14 and 9.22.5.17 to 9.22.5.20: every resource access search answers 200.
  These also stand for 9.13.
- 9.25.1 and 9.25.2 (`fine-grained-scopes-1/`, `-2/`): with granular scope group 1 or 2, Condition
  and Observation searches (by category, by patient, by code, by category and date) return exactly
  the granted categories, and reads outside them are refused. These also stand for 9.14, 9.23 and
  9.24.
- 9.19.01 (in `ehr-practitioner-app/`): the `$launch` part of an EHR launch with patient scopes.
- 9.16.08 (patient context with a public client) is what `standalone-patient-app/` already checks,
  since that scenario signs in through a public client.

## Known failures

`known-failures.txt` lists the TestScripts that fail against Haste Health today, and CI skips them.
Each group below is one server gap.

- **Granular scopes (`fine-grained-scopes-1/`, `-2/`, 14 TestScripts).** The authorize endpoint
  answers 501 "Invalid smart resource scope" for `patient/[type].rs?category=...`, so these
  scenarios cannot sign in.
- **Device under a patient scope (1.7.07, 9.22.5.07).** The server answers 403. It treats Device as
  outside the Patient compartment and not shared reference data: R4's Patient CompartmentDefinition
  has no parameter for Device. US Core implantable devices name the patient in `Device.patient`,
  and g10 expects `patient/Device.rs` to reach them.
- **OpenID configuration at the issuer (1.11.02).** Inferno fetches
  `[id_token iss]/.well-known/openid-configuration`, and that path answers 404. The server publishes
  the document only at `/.well-known/openid-configuration/w/[tenant]/[project]`.
- **Bulk Data (8.2.02, 8.2.04 and 8.2.05, 8.4, 8.5).** There is no `$export` operation, and GET
  operation invocations answer 501. The kick-off TestScripts end with an execution error, because
  no Content-Location comes back. 8.2.03 passes (an invalid token is refused before routing).
- **Problem list Condition validation (12.7.05).** The server refuses a conformant US Core problem
  list Condition that claims its profile, with "Condition.category:us-core ... found '0'". The
  slice is a value discriminator on a required binding, which the validator does not match. It
  answers 500 rather than a 4xx.

## Not covered

These g10 tests have no TestScript, because a TestScript cannot express them. The short ids are for
SMART 2.2.0 / US Core 7 / Bulk Data 2.0.0; the SMART 2.0.0, US Core 6 and Bulk Data 1.0.1 variants
are not covered for the same reasons.

- **TLS version probing.**
  - 1.9.01, 1.9.04, 2.4.01, 2.4.04, 3.8.03, 3.8.06, 12.1.01, 8.1.01, 8.2.01, 8.3.01, and the same
    tests in 9.x.
  - The runner cannot choose the TLS version of a connection.
  - 11.15 is the matching attestation.
- **The browser part of SMART App Launch.** The runner cannot follow a login and consent in a
  browser session.
  - The authorize redirect and the code it returns: 1.9.02, 1.9.03, 2.4.02, 2.4.03, 3.8.04,
    3.8.05, and every redirect and code test in 9.x.
  - The invalid `aud` launch (9.4).
  - The EHR launch navigation (3.8.01 and 3.8.02 beyond `$launch`, 9.19 beyond 9.19.01). An EHR
    launch sign-in would need the harness to pass a `launch` value.
  - Granular scope selection on the consent screen (9.28, 9.15, 9.26, 9.27). The tester swaps
    the requested resource-level scopes for granular ones the app did not request. The consent
    screen lets the user uncheck requested scopes, not add narrower ones.
- **Requests to the token endpoint.** A plain HTTP operation in a TestScript cannot send a body.
  - Token exchange: 1.9.05, 2.4.05, 3.8.07.
  - Token response headers and token CORS: 1.9.07, 1.9.08, 2.4.07, 2.4.08, 3.8.09, 3.8.10.
  - Refresh: 1.6.01 to 1.6.04, 1.6.06, 3.6.x.
  - Invalid code and invalid client: 9.17.03, 9.17.04.
  - Invalid PKCE verifiers: 9.18.01 to 9.18.12.
  - Refresh after revocation: 9.3.03.
  - Asymmetric client authentication: 9.21.
- **Token, id_token and discovery document contents.** These are JSON documents, not FHIR
  resources, and the runner asserts only on FHIR bodies.
  - The token response body: 1.9.06, 2.4.06, 3.8.08.
  - The granted scope checks: 1.9.09, 1.7.01 as a string check, 2.4.10, 3.8.11, 9.22.4.09,
    9.25.x.1.2.09.
  - `smart_style_url` and `need_patient_banner`: 3.8.17, 3.8.18.
  - The id_token checks: 1.11.01, 1.11.03, 1.11.05, 1.11.06.
  - The SMART configuration fields and capabilities: 1.8.02, 1.8.05, 3.7.02, 3.7.05.
- **SMART Backend Services authorization.** 8.1.02 to 8.1.06 POST a JWT client assertion, signed
  with Inferno's key, to the token endpoint.
- **Token introspection (9.20) and revocation (9.3.01, 9.3.02).** Introspection is a POST with a
  body. Revocation needs a step between sign-in and the tests that revokes the scenario's tokens,
  which the harness does not have.
- **Bulk Data output.**
  - The manifest fields: 8.2.05 beyond status and content type, and 8.2.06. The manifest is JSON,
    not a FHIR resource.
  - The NDJSON files: 8.3.01 to 8.3.45, which cover file access without a token, per-line profile
    validation, must support, patient count and patient ids. The runner cannot read NDJSON.
- **Validation with the HL7 validator (12.x validation).** Modelled only by write-time validation;
  see `profile-validation/`.
- **Attestations and visual inspection (11.01 to 11.21).** These are tester attestations with no
  request to check.
- **Inferno bookkeeping.** These make no request: the credential export and TLS warning steps
  (1.01 to 1.03, 3.01 to 3.03, 8.01 to 8.03, 12.02, 9.22.01, 9.22.02) and the patient id list
  (12.01).

## How these differ from Inferno

- **No Authorization header.** The runner cannot drop the header, so these TestScripts send
  `Authorization: Bearer g10-invalid-token` instead. The server answers both the same way: 401 on
  FHIR routes, ignored on public ones.
- **POST `_search`.** The runner sends the parameters in the query string of a POST with no body,
  where Inferno sends a form body.
- **Values come from the record, not from what the server returned.** This lets each search also
  require that the record's resource comes back, which is stricter than Inferno (Inferno skips
  when a search returns nothing).
- **Date comparators are checked by day.** That way the check holds whatever precision a value is
  stored at.
- **Must support.** It is checked on the record's instances of a profile, fetched together by
  `_id`, rather than on everything earlier searches returned.
- **Scopes beyond g10's list.** The Single Patient API principal also holds `user/Media.rs`, since
  12.10.09 resolves `DiagnosticReport.media.link`.
- **FHIRPath shape.** Asserts are written so that each side of an `and` or `or` is always a
  boolean (`x.where(...).exists()`). Haste Health's FHIRPath engine answers empty for `{} or true`.
