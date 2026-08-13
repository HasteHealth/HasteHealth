
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

* `fhir-path` — Data gets pulled from stdin
* `generate` — 
* `server` — 
* `api` — 
* `config` — 
* `worker` — 
* `testscript` — 
* `admin` — 
* `hl7v2` — 
* `doc` — 



## `haste-health fhir-path`

Data gets pulled from stdin

**Usage:** `haste-health fhir-path <FHIRPATH>`

###### **Arguments:**

* `<FHIRPATH>` — FHIRPath expression to evaluate



## `haste-health generate`

**Usage:** `haste-health generate <COMMAND>`

###### **Subcommands:**

* `types` — 
* `operations` — 
* `test-scripts` — 



## `haste-health generate types`

**Usage:** `haste-health generate types [OPTIONS] --output <OUTPUT>`

###### **Options:**

* `-i`, `--input <INPUT>`
* `-o`, `--output <OUTPUT>` — Output Rust file path
* `-l`, `--level <LEVEL>`

  Possible values: `primitive`, `complex`, `resource`




## `haste-health generate operations`

**Usage:** `haste-health generate operations [OPTIONS]`

###### **Options:**

* `-i`, `--input <INPUT>`
* `-o`, `--output <OUTPUT>` — Output Rust file path



## `haste-health generate test-scripts`

**Usage:** `haste-health generate test-scripts [OPTIONS] --output <OUTPUT>`

###### **Options:**

* `-i`, `--input <INPUT>`
* `-o`, `--output <OUTPUT>` — Output Rust file path



## `haste-health server`

**Usage:** `haste-health server <COMMAND>`

###### **Subcommands:**

* `start` — 



## `haste-health server start`

**Usage:** `haste-health server start [OPTIONS]`

###### **Options:**

* `-p`, `--port <PORT>`



## `haste-health api`

**Usage:** `haste-health api <COMMAND>`

###### **Subcommands:**

* `create` — 
* `read` — 
* `version-read` — 
* `patch` — 
* `update` — 
* `transaction` — 
* `batch` — 
* `history-system` — 
* `history-type` — 
* `history-instance` — 
* `search-type` — 
* `search-system` — 
* `invoke-system` — 
* `invoke-type` — 
* `capabilities` — 
* `delete-instance` — 
* `delete-type` — 
* `delete-system` — 
* `invoke-instance` — 



## `haste-health api create`

**Usage:** `haste-health api create [OPTIONS] <RESOURCE_TYPE>`

###### **Arguments:**

* `<RESOURCE_TYPE>`

###### **Options:**

* `-d`, `--data <DATA>`
* `-f`, `--file <FILE>`



## `haste-health api read`

**Usage:** `haste-health api read <RESOURCE_TYPE> <ID>`

###### **Arguments:**

* `<RESOURCE_TYPE>`
* `<ID>`



## `haste-health api version-read`

**Usage:** `haste-health api version-read <RESOURCE_TYPE> <ID> <VERSION_ID>`

###### **Arguments:**

* `<RESOURCE_TYPE>`
* `<ID>`
* `<VERSION_ID>`



## `haste-health api patch`

**Usage:** `haste-health api patch [OPTIONS] <RESOURCE_TYPE> <ID>`

###### **Arguments:**

* `<RESOURCE_TYPE>`
* `<ID>`

###### **Options:**

* `-d`, `--data <DATA>`
* `-f`, `--file <FILE>`



## `haste-health api update`

**Usage:** `haste-health api update [OPTIONS] <RESOURCE_TYPE> <ID>`

###### **Arguments:**

* `<RESOURCE_TYPE>`
* `<ID>`

###### **Options:**

* `-d`, `--data <DATA>`
* `-f`, `--file <FILE>`



## `haste-health api transaction`

**Usage:** `haste-health api transaction [OPTIONS]`

###### **Options:**

* `-d`, `--data <DATA>`
* `-p`, `--parallel <PARALLEL>`
* `-f`, `--file <FILE>`
* `-o`, `--output <OUTPUT>`

  Possible values: `true`, `false`




## `haste-health api batch`

**Usage:** `haste-health api batch [OPTIONS]`

###### **Options:**

* `-d`, `--data <DATA>`
* `-f`, `--file <FILE>`
* `-o`, `--output <OUTPUT>`

  Possible values: `true`, `false`




## `haste-health api history-system`

**Usage:** `haste-health api history-system [PARAMETERS]`

###### **Arguments:**

* `<PARAMETERS>`



## `haste-health api history-type`

**Usage:** `haste-health api history-type <RESOURCE_TYPE> [PARAMETERS]`

###### **Arguments:**

* `<RESOURCE_TYPE>`
* `<PARAMETERS>`



## `haste-health api history-instance`

**Usage:** `haste-health api history-instance <RESOURCE_TYPE> <ID> [PARAMETERS]`

###### **Arguments:**

* `<RESOURCE_TYPE>`
* `<ID>`
* `<PARAMETERS>`



## `haste-health api search-type`

**Usage:** `haste-health api search-type <RESOURCE_TYPE> [PARAMETERS]`

###### **Arguments:**

* `<RESOURCE_TYPE>`
* `<PARAMETERS>`



## `haste-health api search-system`

**Usage:** `haste-health api search-system [PARAMETERS]`

###### **Arguments:**

* `<PARAMETERS>`



## `haste-health api invoke-system`

**Usage:** `haste-health api invoke-system [OPTIONS] <OPERATION_NAME>`

###### **Arguments:**

* `<OPERATION_NAME>`

###### **Options:**

* `-d`, `--data <DATA>`
* `-f`, `--file <FILE>`



## `haste-health api invoke-type`

**Usage:** `haste-health api invoke-type [OPTIONS] <RESOURCE_TYPE> <OPERATION_NAME>`

###### **Arguments:**

* `<RESOURCE_TYPE>`
* `<OPERATION_NAME>`

###### **Options:**

* `-d`, `--data <DATA>`
* `-f`, `--file <FILE>`



## `haste-health api capabilities`

**Usage:** `haste-health api capabilities`



## `haste-health api delete-instance`

**Usage:** `haste-health api delete-instance <RESOURCE_TYPE> <ID>`

###### **Arguments:**

* `<RESOURCE_TYPE>`
* `<ID>`



## `haste-health api delete-type`

**Usage:** `haste-health api delete-type <RESOURCE_TYPE> [PARAMETERS]`

###### **Arguments:**

* `<RESOURCE_TYPE>`
* `<PARAMETERS>`



## `haste-health api delete-system`

**Usage:** `haste-health api delete-system [PARAMETERS]`

###### **Arguments:**

* `<PARAMETERS>`



## `haste-health api invoke-instance`

**Usage:** `haste-health api invoke-instance [OPTIONS] <RESOURCE_TYPE> <ID> <OPERATION_NAME>`

###### **Arguments:**

* `<RESOURCE_TYPE>`
* `<ID>`
* `<OPERATION_NAME>`

###### **Options:**

* `-d`, `--data <DATA>`
* `-f`, `--file <FILE>`



## `haste-health config`

**Usage:** `haste-health config <COMMAND>`

###### **Subcommands:**

* `show-profile` — 
* `create-profile` — 
* `delete-profile` — 
* `set-active-profile` — 



## `haste-health config show-profile`

**Usage:** `haste-health config show-profile`



## `haste-health config create-profile`

**Usage:** `haste-health config create-profile [OPTIONS]`

###### **Options:**

* `-n`, `--name <NAME>`
* `-r`, `--r4-url <R4_URL>`
* `-d`, `--discovery-uri <DISCOVERY_URI>`
* `-i`, `--id <ID>`
* `-s`, `--secret <SECRET>`



## `haste-health config delete-profile`

**Usage:** `haste-health config delete-profile [OPTIONS]`

###### **Options:**

* `-n`, `--name <NAME>`
* `-c`, `--confirm <CONFIRM>`

  Possible values: `true`, `false`




## `haste-health config set-active-profile`

**Usage:** `haste-health config set-active-profile [OPTIONS]`

###### **Options:**

* `-n`, `--name <NAME>`



## `haste-health worker`

**Usage:** `haste-health worker [COMMAND]`

###### **Subcommands:**

* `worker` — 
* `wal-worker` — 



## `haste-health worker worker`

**Usage:** `haste-health worker worker`



## `haste-health worker wal-worker`

**Usage:** `haste-health worker wal-worker`



## `haste-health testscript`

**Usage:** `haste-health testscript <COMMAND>`

###### **Subcommands:**

* `run` — 



## `haste-health testscript run`

**Usage:** `haste-health testscript run [OPTIONS]`

###### **Options:**

* `-i`, `--input <INPUT>`
* `-o`, `--output <OUTPUT>`
* `-w`, `--wait-between-operations-ms <WAIT_BETWEEN_OPERATIONS_MS>`



## `haste-health admin`

**Usage:** `haste-health admin <COMMAND>`

###### **Subcommands:**

* `tenant` — 
* `user` — 
* `client` — 
* `migrate` — 



## `haste-health admin tenant`

**Usage:** `haste-health admin tenant <COMMAND>`

###### **Subcommands:**

* `create` — 



## `haste-health admin tenant create`

**Usage:** `haste-health admin tenant create [OPTIONS] --id <ID> --owner-email <OWNER_EMAIL> --owner-password <OWNER_PASSWORD>`

###### **Options:**

* `-i`, `--id <ID>`
* `-s`, `--subscription-tier <SUBSCRIPTION_TIER>`

  Possible values: `free`, `professional`, `team`, `unlimited`

* `--owner-email <OWNER_EMAIL>`
* `--owner-password <OWNER_PASSWORD>`



## `haste-health admin user`

**Usage:** `haste-health admin user <COMMAND>`

###### **Subcommands:**

* `create` — 



## `haste-health admin user create`

**Usage:** `haste-health admin user create --email <EMAIL> --password <PASSWORD> --tenant <TENANT>`

###### **Options:**

* `-e`, `--email <EMAIL>`
* `-p`, `--password <PASSWORD>`
* `-t`, `--tenant <TENANT>`



## `haste-health admin client`

**Usage:** `haste-health admin client <COMMAND>`

###### **Subcommands:**

* `create` — 



## `haste-health admin client create`

**Usage:** `haste-health admin client create --id <ID> --secret <SECRET> --tenant <TENANT> --project <PROJECT>`

###### **Options:**

* `-i`, `--id <ID>`
* `-s`, `--secret <SECRET>`
* `-t`, `--tenant <TENANT>`
* `-p`, `--project <PROJECT>`



## `haste-health admin migrate`

**Usage:** `haste-health admin migrate <COMMAND>`

###### **Subcommands:**

* `artifacts` — 
* `reset-artifacts` — 
* `repo` — 
* `search` — 
* `all` — 



## `haste-health admin migrate artifacts`

**Usage:** `haste-health admin migrate artifacts`



## `haste-health admin migrate reset-artifacts`

**Usage:** `haste-health admin migrate reset-artifacts`



## `haste-health admin migrate repo`

**Usage:** `haste-health admin migrate repo`



## `haste-health admin migrate search`

**Usage:** `haste-health admin migrate search`



## `haste-health admin migrate all`

**Usage:** `haste-health admin migrate all`



## `haste-health hl7v2`

**Usage:** `haste-health hl7v2 <COMMAND>`

###### **Subcommands:**

* `receiver` — 
* `sender` — 



## `haste-health hl7v2 receiver`

**Usage:** `haste-health hl7v2 receiver --address <ADDRESS> --port <PORT> --main <MAIN> --template-dir <TEMPLATE_DIR>`

###### **Options:**

* `-a`, `--address <ADDRESS>`
* `-p`, `--port <PORT>`
* `-m`, `--main <MAIN>`
* `-t`, `--template-dir <TEMPLATE_DIR>`



## `haste-health hl7v2 sender`

**Usage:** `haste-health hl7v2 sender --address <ADDRESS> --port <PORT>`

###### **Options:**

* `-a`, `--address <ADDRESS>`
* `-p`, `--port <PORT>`



## `haste-health doc`

**Usage:** `haste-health doc --output <OUTPUT>`

###### **Options:**

* `-o`, `--output <OUTPUT>` — Output markdown file path



<hr/>

<small><i>
    This document was generated automatically by
    <a href="https://crates.io/crates/clap-markdown"><code>clap-markdown</code></a>.
</i></small>
