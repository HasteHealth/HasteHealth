
| Context | Invocation                     |
| ------- | ------------------------------ |
| Source  | `cargo run <command>`          |
| Binary  | `./haste-health <command>`     |
| Docker  | `docker run <image> <command>` |


# Command-Line Help for `haste-health`

This document contains the help content for the `haste-health` command-line program.

**Command Overview:**

* [`haste-health`↴](#haste-health)
* [`haste-health fhir-path`↴](#haste-health-fhir-path)
* [`haste-health generate`↴](#haste-health-generate)
* [`haste-health generate types`↴](#haste-health-generate-types)
* [`haste-health generate operations`↴](#haste-health-generate-operations)
* [`haste-health generate test-scripts`↴](#haste-health-generate-test-scripts)
* [`haste-health server`↴](#haste-health-server)
* [`haste-health server start`↴](#haste-health-server-start)
* [`haste-health api`↴](#haste-health-api)
* [`haste-health api create`↴](#haste-health-api-create)
* [`haste-health api read`↴](#haste-health-api-read)
* [`haste-health api version-read`↴](#haste-health-api-version-read)
* [`haste-health api patch`↴](#haste-health-api-patch)
* [`haste-health api update`↴](#haste-health-api-update)
* [`haste-health api transaction`↴](#haste-health-api-transaction)
* [`haste-health api batch`↴](#haste-health-api-batch)
* [`haste-health api history-system`↴](#haste-health-api-history-system)
* [`haste-health api history-type`↴](#haste-health-api-history-type)
* [`haste-health api history-instance`↴](#haste-health-api-history-instance)
* [`haste-health api search-type`↴](#haste-health-api-search-type)
* [`haste-health api search-system`↴](#haste-health-api-search-system)
* [`haste-health api invoke-system`↴](#haste-health-api-invoke-system)
* [`haste-health api invoke-type`↴](#haste-health-api-invoke-type)
* [`haste-health api capabilities`↴](#haste-health-api-capabilities)
* [`haste-health api delete-instance`↴](#haste-health-api-delete-instance)
* [`haste-health api delete-type`↴](#haste-health-api-delete-type)
* [`haste-health api delete-system`↴](#haste-health-api-delete-system)
* [`haste-health api invoke-instance`↴](#haste-health-api-invoke-instance)
* [`haste-health config`↴](#haste-health-config)
* [`haste-health config show-profile`↴](#haste-health-config-show-profile)
* [`haste-health config create-profile`↴](#haste-health-config-create-profile)
* [`haste-health config delete-profile`↴](#haste-health-config-delete-profile)
* [`haste-health config set-active-profile`↴](#haste-health-config-set-active-profile)
* [`haste-health login`↴](#haste-health-login)
* [`haste-health worker`↴](#haste-health-worker)
* [`haste-health worker worker`↴](#haste-health-worker-worker)
* [`haste-health worker wal-worker`↴](#haste-health-worker-wal-worker)
* [`haste-health testscript`↴](#haste-health-testscript)
* [`haste-health testscript run`↴](#haste-health-testscript-run)
* [`haste-health admin`↴](#haste-health-admin)
* [`haste-health admin tenant`↴](#haste-health-admin-tenant)
* [`haste-health admin tenant create`↴](#haste-health-admin-tenant-create)
* [`haste-health admin user`↴](#haste-health-admin-user)
* [`haste-health admin user create`↴](#haste-health-admin-user-create)
* [`haste-health admin client`↴](#haste-health-admin-client)
* [`haste-health admin client create`↴](#haste-health-admin-client-create)
* [`haste-health admin migrate`↴](#haste-health-admin-migrate)
* [`haste-health admin migrate artifacts`↴](#haste-health-admin-migrate-artifacts)
* [`haste-health admin migrate reset-artifacts`↴](#haste-health-admin-migrate-reset-artifacts)
* [`haste-health admin migrate repo`↴](#haste-health-admin-migrate-repo)
* [`haste-health admin migrate search`↴](#haste-health-admin-migrate-search)
* [`haste-health admin migrate all`↴](#haste-health-admin-migrate-all)
* [`haste-health hl7v2`↴](#haste-health-hl7v2)
* [`haste-health hl7v2 receiver`↴](#haste-health-hl7v2-receiver)
* [`haste-health hl7v2 sender`↴](#haste-health-hl7v2-sender)
* [`haste-health doc`↴](#haste-health-doc)

## `haste-health`

Haste Health binary.

**Usage:** `haste-health <COMMAND>`

###### **Subcommands:**

* `fhir-path` — Evaluate a FHIRPath expression against a FHIR resource read from stdin
* `generate` — Code generators (Rust FHIR types, operations, TestScripts) used to build this crate
* `server` — Run the FHIR server
* `api` — Make FHIR REST API calls against the active profile's server
* `config` — Manage named server connection profiles used by other commands
* `login` — Log in as a human user via the browser (authorization_code + PKCE flow)
* `worker` — Run background workers (search indexing, WAL processing)
* `testscript` — Run FHIR TestScript resources against the active profile's server
* `admin` — Server-side administrative operations (tenants, users, clients, migrations)
* `hl7v2` — Bridge HL7v2 messages to and from the FHIR server
* `doc` — Generate Markdown documentation for this CLI's commands



## `haste-health fhir-path`

Evaluate a FHIRPath expression against a FHIR resource read from stdin

**Usage:** `haste-health fhir-path <FHIRPATH>`

###### **Arguments:**

* `<FHIRPATH>` — FHIRPath expression to evaluate



## `haste-health generate`

Code generators (Rust FHIR types, operations, TestScripts) used to build this crate

**Usage:** `haste-health generate <COMMAND>`

###### **Subcommands:**

* `types` — Generate Rust structs for FHIR resources/types/terminology from StructureDefinitions
* `operations` — Generate Rust bindings for FHIR OperationDefinitions
* `test-scripts` — Generate FHIR TestScript resources



## `haste-health generate types`

Generate Rust structs for FHIR resources/types/terminology from StructureDefinitions

**Usage:** `haste-health generate types [OPTIONS] --output <OUTPUT>`

###### **Options:**

* `-i`, `--input <INPUT>` — Input FHIR StructureDefinition file(s) or directories (JSON). Repeatable
* `-o`, `--output <OUTPUT>` — Output directory for the generated `resources.rs`, `types.rs`, `terminology.rs`, `mod.rs`
* `-l`, `--level <LEVEL>` — Restrict generation to one tier of types. Defaults to generating all tiers

  Possible values: `primitive`, `complex`, `resource`




## `haste-health generate operations`

Generate Rust bindings for FHIR OperationDefinitions

**Usage:** `haste-health generate operations [OPTIONS]`

###### **Options:**

* `-i`, `--input <INPUT>` — Input FHIR OperationDefinition file(s) or directories (JSON). Repeatable
* `-o`, `--output <OUTPUT>` — Output Rust file path. Prints to stdout if omitted



## `haste-health generate test-scripts`

Generate FHIR TestScript resources

**Usage:** `haste-health generate test-scripts [OPTIONS] --output <OUTPUT>`

###### **Options:**

* `-i`, `--input <INPUT>` — Input file(s) or directories describing the TestScripts to generate. Repeatable
* `-o`, `--output <OUTPUT>` — Output directory for the generated TestScript JSON files



## `haste-health server`

Run the FHIR server

**Usage:** `haste-health server <COMMAND>`

###### **Subcommands:**

* `start` — Start the HTTP server. Configuration is read from `haste.toml` and `HASTE_*` env vars



## `haste-health server start`

Start the HTTP server. Configuration is read from `haste.toml` and `HASTE_*` env vars

**Usage:** `haste-health server start [OPTIONS]`

###### **Options:**

* `-p`, `--port <PORT>` — Port to listen on. Defaults to 3000



## `haste-health api`

Make FHIR REST API calls against the active profile's server

**Usage:** `haste-health api <COMMAND>`

###### **Subcommands:**

* `create` — Create a resource (`POST [base]/[type]`)
* `read` — Read the current version of a resource (`GET [base]/[type]/[id]`)
* `version-read` — Read a specific historical version of a resource (`GET [base]/[type]/[id]/_history/[vid]`)
* `patch` — Apply a JSON Patch to a resource (`PATCH [base]/[type]/[id]`)
* `update` — Create or replace a resource at a known ID (`PUT [base]/[type]/[id]`)
* `transaction` — Submit a transaction Bundle (`POST [base]`, type `transaction`)
* `batch` — Submit a batch Bundle (`POST [base]`, type `batch`)
* `history-system` — Fetch the system-wide history (`GET [base]/_history`)
* `history-type` — Fetch the history of a resource type (`GET [base]/[type]/_history`)
* `history-instance` — Fetch the history of a single resource instance (`GET [base]/[type]/[id]/_history`)
* `search-type` — Search a resource type (`GET [base]/[type]?...`)
* `search-system` — Search across all resource types (`GET [base]?...`)
* `invoke-system` — Invoke a system-level operation (`POST [base]/$[operation_name]`)
* `invoke-type` — Invoke a type-level operation (`POST [base]/[type]/$[operation_name]`)
* `capabilities` — Fetch the server's CapabilityStatement (`GET [base]/metadata`)
* `delete-instance` — Delete a single resource instance (`DELETE [base]/[type]/[id]`)
* `delete-type` — Delete all resources of a type matching search parameters (`DELETE [base]/[type]?...`)
* `delete-system` — Delete all resources matching system-level search parameters (`DELETE [base]?...`)
* `invoke-instance` — Invoke an instance-level operation (`POST [base]/[type]/[id]/$[operation_name]`)



## `haste-health api create`

Create a resource (`POST [base]/[type]`)

**Usage:** `haste-health api create [OPTIONS] <RESOURCE_TYPE>`

###### **Arguments:**

* `<RESOURCE_TYPE>` — FHIR resource type to create

###### **Options:**

* `-d`, `--data <DATA>` — Resource JSON, inline
* `-f`, `--file <FILE>` — Path to a file containing the resource JSON



## `haste-health api read`

Read the current version of a resource (`GET [base]/[type]/[id]`)

**Usage:** `haste-health api read <RESOURCE_TYPE> <ID>`

###### **Arguments:**

* `<RESOURCE_TYPE>`
* `<ID>`



## `haste-health api version-read`

Read a specific historical version of a resource (`GET [base]/[type]/[id]/_history/[vid]`)

**Usage:** `haste-health api version-read <RESOURCE_TYPE> <ID> <VERSION_ID>`

###### **Arguments:**

* `<RESOURCE_TYPE>`
* `<ID>`
* `<VERSION_ID>`



## `haste-health api patch`

Apply a JSON Patch to a resource (`PATCH [base]/[type]/[id]`)

**Usage:** `haste-health api patch [OPTIONS] <RESOURCE_TYPE> <ID>`

###### **Arguments:**

* `<RESOURCE_TYPE>`
* `<ID>`

###### **Options:**

* `-d`, `--data <DATA>` — JSON Patch document, inline
* `-f`, `--file <FILE>` — Path to a file containing the JSON Patch document



## `haste-health api update`

Create or replace a resource at a known ID (`PUT [base]/[type]/[id]`)

**Usage:** `haste-health api update [OPTIONS] <RESOURCE_TYPE> <ID>`

###### **Arguments:**

* `<RESOURCE_TYPE>`
* `<ID>`

###### **Options:**

* `-d`, `--data <DATA>` — Resource JSON, inline
* `-f`, `--file <FILE>` — Path to a file containing the resource JSON



## `haste-health api transaction`

Submit a transaction Bundle (`POST [base]`, type `transaction`)

**Usage:** `haste-health api transaction [OPTIONS]`

###### **Options:**

* `-d`, `--data <DATA>` — Bundle JSON, inline
* `-p`, `--parallel <PARALLEL>` — Submit the same bundle this many times concurrently. Defaults to 1
* `-f`, `--file <FILE>` — Path to a file containing the Bundle JSON
* `-o`, `--output <OUTPUT>` — Print each response bundle to stdout

  Possible values: `true`, `false`




## `haste-health api batch`

Submit a batch Bundle (`POST [base]`, type `batch`)

**Usage:** `haste-health api batch [OPTIONS]`

###### **Options:**

* `-d`, `--data <DATA>` — Bundle JSON, inline
* `-f`, `--file <FILE>` — Path to a file containing the Bundle JSON
* `-o`, `--output <OUTPUT>` — Print the response bundle to stdout

  Possible values: `true`, `false`




## `haste-health api history-system`

Fetch the system-wide history (`GET [base]/_history`)

**Usage:** `haste-health api history-system [PARAMETERS]`

###### **Arguments:**

* `<PARAMETERS>` — FHIR search-style parameters, e.g. `_since=2024-01-01`



## `haste-health api history-type`

Fetch the history of a resource type (`GET [base]/[type]/_history`)

**Usage:** `haste-health api history-type <RESOURCE_TYPE> [PARAMETERS]`

###### **Arguments:**

* `<RESOURCE_TYPE>`
* `<PARAMETERS>` — FHIR search-style parameters, e.g. `_since=2024-01-01`



## `haste-health api history-instance`

Fetch the history of a single resource instance (`GET [base]/[type]/[id]/_history`)

**Usage:** `haste-health api history-instance <RESOURCE_TYPE> <ID> [PARAMETERS]`

###### **Arguments:**

* `<RESOURCE_TYPE>`
* `<ID>`
* `<PARAMETERS>` — FHIR search-style parameters, e.g. `_since=2024-01-01`



## `haste-health api search-type`

Search a resource type (`GET [base]/[type]?...`)

**Usage:** `haste-health api search-type <RESOURCE_TYPE> [PARAMETERS]`

###### **Arguments:**

* `<RESOURCE_TYPE>`
* `<PARAMETERS>` — FHIR search parameters, e.g. `name=eve&_count=20`



## `haste-health api search-system`

Search across all resource types (`GET [base]?...`)

**Usage:** `haste-health api search-system [PARAMETERS]`

###### **Arguments:**

* `<PARAMETERS>` — FHIR search parameters, e.g. `_lastUpdated=gt2024-01-01`



## `haste-health api invoke-system`

Invoke a system-level operation (`POST [base]/$[operation_name]`)

**Usage:** `haste-health api invoke-system [OPTIONS] <OPERATION_NAME>`

###### **Arguments:**

* `<OPERATION_NAME>`

###### **Options:**

* `-d`, `--data <DATA>` — Parameters resource JSON, inline
* `-f`, `--file <FILE>` — Path to a file containing the Parameters resource JSON



## `haste-health api invoke-type`

Invoke a type-level operation (`POST [base]/[type]/$[operation_name]`)

**Usage:** `haste-health api invoke-type [OPTIONS] <RESOURCE_TYPE> <OPERATION_NAME>`

###### **Arguments:**

* `<RESOURCE_TYPE>`
* `<OPERATION_NAME>`

###### **Options:**

* `-d`, `--data <DATA>` — Parameters resource JSON, inline
* `-f`, `--file <FILE>` — Path to a file containing the Parameters resource JSON



## `haste-health api capabilities`

Fetch the server's CapabilityStatement (`GET [base]/metadata`)

**Usage:** `haste-health api capabilities`



## `haste-health api delete-instance`

Delete a single resource instance (`DELETE [base]/[type]/[id]`)

**Usage:** `haste-health api delete-instance <RESOURCE_TYPE> <ID>`

###### **Arguments:**

* `<RESOURCE_TYPE>`
* `<ID>`



## `haste-health api delete-type`

Delete all resources of a type matching search parameters (`DELETE [base]/[type]?...`)

**Usage:** `haste-health api delete-type <RESOURCE_TYPE> [PARAMETERS]`

###### **Arguments:**

* `<RESOURCE_TYPE>`
* `<PARAMETERS>` — FHIR search parameters selecting which resources to delete



## `haste-health api delete-system`

Delete all resources matching system-level search parameters (`DELETE [base]?...`)

**Usage:** `haste-health api delete-system [PARAMETERS]`

###### **Arguments:**

* `<PARAMETERS>` — FHIR search parameters selecting which resources to delete



## `haste-health api invoke-instance`

Invoke an instance-level operation (`POST [base]/[type]/[id]/$[operation_name]`)

**Usage:** `haste-health api invoke-instance [OPTIONS] <RESOURCE_TYPE> <ID> <OPERATION_NAME>`

###### **Arguments:**

* `<RESOURCE_TYPE>`
* `<ID>`
* `<OPERATION_NAME>`

###### **Options:**

* `-d`, `--data <DATA>` — Parameters resource JSON, inline
* `-f`, `--file <FILE>` — Path to a file containing the Parameters resource JSON



## `haste-health config`

Manage named server connection profiles used by other commands

**Usage:** `haste-health config <COMMAND>`

###### **Subcommands:**

* `show-profile` — Print the currently active profile (never includes secrets or tokens)
* `create-profile` — Create a new profile and set it as active. Prompts interactively for any option not passed on the command line
* `delete-profile` — Delete a profile and its stored secrets
* `set-active-profile` — Change which profile is used by default



## `haste-health config show-profile`

Print the currently active profile (never includes secrets or tokens)

**Usage:** `haste-health config show-profile`



## `haste-health config create-profile`

Create a new profile and set it as active. Prompts interactively for any option not passed on the command line

**Usage:** `haste-health config create-profile [OPTIONS]`

###### **Options:**

* `-n`, `--name <NAME>` — Name to identify this profile by
* `-r`, `--r4-url <R4_URL>` — Base URL of the FHIR R4 server
* `-d`, `--discovery-uri <DISCOVERY_URI>` — OIDC discovery (`.well-known/openid-configuration`) URI
* `--auth-mode <AUTH_MODE>` — How the CLI should authenticate as this profile

  Possible values:
  - `client-credentials`:
    A confidential (server-to-server) client authenticated with a client secret
  - `authorization-code`:
    A public client a human logs into via the browser (authorization_code + PKCE). Use `haste-health login` afterwards to obtain tokens

* `-i`, `--id <ID>` — OIDC client ID
* `-s`, `--secret <SECRET>` — Client secret. Required for --auth-mode client-credentials, ignored otherwise. Stored in the secrets file, not the profile itself
* `--redirect-uri <REDIRECT_URI>` — Loopback redirect URI for --auth-mode authorization-code (must be registered on the server client)
* `--scope <SCOPE>` — OAuth scope to request for --auth-mode authorization-code



## `haste-health config delete-profile`

Delete a profile and its stored secrets

**Usage:** `haste-health config delete-profile [OPTIONS]`

###### **Options:**

* `-n`, `--name <NAME>` — Name of the profile to delete
* `-c`, `--confirm <CONFIRM>` — Skip the interactive confirmation prompt

  Possible values: `true`, `false`




## `haste-health config set-active-profile`

Change which profile is used by default

**Usage:** `haste-health config set-active-profile [OPTIONS]`

###### **Options:**

* `-n`, `--name <NAME>` — Name of the profile to activate



## `haste-health login`

Log in as a human user via the browser (authorization_code + PKCE flow)

**Usage:** `haste-health login`



## `haste-health worker`

Run background workers (search indexing, WAL processing)

**Usage:** `haste-health worker [COMMAND]`

###### **Subcommands:**

* `worker` — Run the search-indexing worker. Default when no subcommand is given
* `wal-worker` — Run the Postgres WAL (write-ahead log) worker. Not yet implemented



## `haste-health worker worker`

Run the search-indexing worker. Default when no subcommand is given

**Usage:** `haste-health worker worker`



## `haste-health worker wal-worker`

Run the Postgres WAL (write-ahead log) worker. Not yet implemented

**Usage:** `haste-health worker wal-worker`



## `haste-health testscript`

Run FHIR TestScript resources against the active profile's server

**Usage:** `haste-health testscript <COMMAND>`

###### **Subcommands:**

* `run` — Run every TestScript resource found under the given input path(s), in parallel, and write a transaction Bundle of the resulting TestReports



## `haste-health testscript run`

Run every TestScript resource found under the given input path(s), in parallel, and write a transaction Bundle of the resulting TestReports

**Usage:** `haste-health testscript run [OPTIONS]`

###### **Options:**

* `-i`, `--input <INPUT>` — File or directory to search for TestScript resources (JSON). Repeatable
* `-o`, `--output <OUTPUT>` — Write the resulting TestReport bundle to this file instead of stdout
* `-w`, `--wait-between-operations-ms <WAIT_BETWEEN_OPERATIONS_MS>` — Delay between operations within a TestScript, in milliseconds



## `haste-health admin`

Server-side administrative operations (tenants, users, clients, migrations)

**Usage:** `haste-health admin <COMMAND>`

###### **Subcommands:**

* `tenant` — Manage tenants
* `user` — Manage users
* `client` — Manage OIDC ClientApplication resources
* `migrate` — Run database/search/artifact migrations



## `haste-health admin tenant`

Manage tenants

**Usage:** `haste-health admin tenant <COMMAND>`

###### **Subcommands:**

* `create` — Create a tenant and its owner user



## `haste-health admin tenant create`

Create a tenant and its owner user

**Usage:** `haste-health admin tenant create [OPTIONS] --id <ID> --owner-email <OWNER_EMAIL> --owner-password <OWNER_PASSWORD>`

###### **Options:**

* `-i`, `--id <ID>` — Tenant ID to create
* `-s`, `--subscription-tier <SUBSCRIPTION_TIER>` — Subscription tier to assign. Defaults to Free

  Possible values: `free`, `professional`, `team`, `unlimited`

* `--owner-email <OWNER_EMAIL>` — Email address for the tenant's owner user
* `--owner-password <OWNER_PASSWORD>` — Password for the tenant's owner user



## `haste-health admin user`

Manage users

**Usage:** `haste-health admin user <COMMAND>`

###### **Subcommands:**

* `create` — Create an admin user within a tenant



## `haste-health admin user create`

Create an admin user within a tenant

**Usage:** `haste-health admin user create --email <EMAIL> --password <PASSWORD> --tenant <TENANT>`

###### **Options:**

* `-e`, `--email <EMAIL>` — Email address for the new user
* `-p`, `--password <PASSWORD>` — Password for the new user
* `-t`, `--tenant <TENANT>` — Tenant to create the user in



## `haste-health admin client`

Manage OIDC ClientApplication resources

**Usage:** `haste-health admin client <COMMAND>`

###### **Subcommands:**

* `create` — Create a ClientApplication and, for client-credentials clients, an AccessPolicyV2 granting it full access



## `haste-health admin client create`

Create a ClientApplication and, for client-credentials clients, an AccessPolicyV2 granting it full access

**Usage:** `haste-health admin client create [OPTIONS] --id <ID> --tenant <TENANT> --project <PROJECT>`

###### **Options:**

* `-i`, `--id <ID>` — OIDC client ID to create
* `-s`, `--secret <SECRET>` — Required for --grant-type client-credentials. Ignored (and unset, making the client public) for --grant-type authorization-code
* `-t`, `--tenant <TENANT>` — Tenant to create the client in
* `-p`, `--project <PROJECT>` — Project to create the client in
* `--grant-type <GRANT_TYPE>` — OAuth grant type the client uses to authenticate

  Default value: `client-credentials`

  Possible values:
  - `client-credentials`:
    A confidential (server-to-server) client authenticated with a client secret
  - `authorization-code`:
    A public client (no secret) a human logs into via the browser (authorization_code + PKCE)

* `--redirect-uri <REDIRECT_URI>` — Loopback redirect URI(s) to allow, e.g. http://127.0.0.1:8976/callback. Required for --grant-type authorization-code
* `--scope <SCOPE>` — OAuth scope to grant the client. Defaults depend on --grant-type



## `haste-health admin migrate`

Run database/search/artifact migrations

**Usage:** `haste-health admin migrate <COMMAND>`

###### **Subcommands:**

* `artifacts` — Load the built-in FHIR artifacts (StructureDefinitions, ValueSets, etc)
* `reset-artifacts` — Reload the built-in FHIR artifacts from scratch, discarding local edits to them
* `repo` — Run pending repository (Postgres) migrations
* `search` — Run pending search index (ElasticSearch) migrations
* `all` — Run all of the above: repo, then search, then artifacts



## `haste-health admin migrate artifacts`

Load the built-in FHIR artifacts (StructureDefinitions, ValueSets, etc)

**Usage:** `haste-health admin migrate artifacts`



## `haste-health admin migrate reset-artifacts`

Reload the built-in FHIR artifacts from scratch, discarding local edits to them

**Usage:** `haste-health admin migrate reset-artifacts`



## `haste-health admin migrate repo`

Run pending repository (Postgres) migrations

**Usage:** `haste-health admin migrate repo`



## `haste-health admin migrate search`

Run pending search index (ElasticSearch) migrations

**Usage:** `haste-health admin migrate search`



## `haste-health admin migrate all`

Run all of the above: repo, then search, then artifacts

**Usage:** `haste-health admin migrate all`



## `haste-health hl7v2`

Bridge HL7v2 messages to and from the FHIR server

**Usage:** `haste-health hl7v2 <COMMAND>`

###### **Subcommands:**

* `receiver` — Listen for MLLP-framed HL7v2 messages, convert them to FHIR, and submit them
* `sender` — Send HL7v2 messages over MLLP. Not yet implemented



## `haste-health hl7v2 receiver`

Listen for MLLP-framed HL7v2 messages, convert them to FHIR, and submit them

**Usage:** `haste-health hl7v2 receiver --address <ADDRESS> --port <PORT> --main <MAIN> --template-dir <TEMPLATE_DIR>`

###### **Options:**

* `-a`, `--address <ADDRESS>` — Address to bind the MLLP listener to
* `-p`, `--port <PORT>` — Port to bind the MLLP listener to
* `-m`, `--main <MAIN>` — Entry template file name (resolved within --template-dir) used to convert incoming HL7v2 messages to FHIR
* `-t`, `--template-dir <TEMPLATE_DIR>` — Directory containing the conversion templates



## `haste-health hl7v2 sender`

Send HL7v2 messages over MLLP. Not yet implemented

**Usage:** `haste-health hl7v2 sender --address <ADDRESS> --port <PORT>`

###### **Options:**

* `-a`, `--address <ADDRESS>` — Address of the MLLP receiver to send to
* `-p`, `--port <PORT>` — Port of the MLLP receiver to send to



## `haste-health doc`

Generate Markdown documentation for this CLI's commands

**Usage:** `haste-health doc --output <OUTPUT>`

###### **Options:**

* `-o`, `--output <OUTPUT>` — Output markdown file path



<hr/>

<small><i>
    This document was generated automatically by
    <a href="https://crates.io/crates/clap-markdown"><code>clap-markdown</code></a>.
</i></small>
