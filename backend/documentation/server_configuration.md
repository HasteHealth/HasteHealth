# Haste Health Server

## Configuration

Server configuration is loaded with [Figment](https://docs.rs/figment), merging two sources in order (`backend/src/commands/server.rs`):

```rust
let config: ServerConfig = Figment::new()
    .merge(Toml::file("haste.toml"))
    .merge(Env::prefixed("HASTE_"))
    .extract()?;
```

1. **`haste.toml`** — read relative to the process's working directory (i.e. `backend/` when running `cargo run server start` from there). If the file doesn't exist, this step is silently skipped.
2. **`HASTE_`-prefixed environment variables** — merged _after_ the TOML file, so **environment variables win** on any key present in both.

Any field not set by either source falls back to the default documented below.

### Environment variable naming

- Top-level fields: `HASTE_<FIELD>`, e.g. `HASTE_API_URI`, `HASTE_CERTIFICATION_DIR`, `HASTE_ADMIN_APP_REDIRECT_URI`, `HASTE_MAX_REQUEST_BODY_SIZE`.
- Nested sections: `HASTE_<SECTION>.<field>` — the prefix plus the section name, then a **literal dot**, then the field name as it appears in TOML (lowercase), e.g. `HASTE_REPO.database_url`, `HASTE_SEARCH.username`, `HASTE_SECURITY.publicize_fhir_metadata`.
- Sections that pick a backend/provider via a tag (`repo`, `search`, `document_storage`, `email`, `security.encryption`) need that tag set too, e.g. `HASTE_REPO.backend=postgres`.

### Reference

#### Top level

| Key                        | Env var                          | Default                   | Notes                                                                                                                  |
| -------------------------- | -------------------------------- | ------------------------- | ---------------------------------------------------------------------------------------------------------------------- |
| `api_uri`                  | `HASTE_API_URI`                  | `http://localhost:3000`   | Root URL the FHIR server is hosted at; used to build absolute URLs in responses (OIDC discovery, FHIR base URLs, etc). |
| `certification_dir`        | `HASTE_CERTIFICATION_DIR`        | `certifications`          | Directory used for JWT signing/verification key material.                                                              |
| `admin_app_redirect_uri`   | `HASTE_ADMIN_APP_REDIRECT_URI`   | `http://*.localhost:3001` | Redirect target for the hardcoded admin app OIDC client.                                                               |
| `allow_artifact_mutations` | `HASTE_ALLOW_ARTIFACT_MUTATIONS` | `false`                   | Allows mutating built-in/embedded artifact resources.                                                                  |
| `max_request_body_size`    | `HASTE_MAX_REQUEST_BODY_SIZE`    | `4194304` (4MB)           | Max accepted request body size, in bytes.                                                                              |

#### `[fhir]`

| Key                 | Env var                   | Default                                                             |
| ------------------- | ------------------------- | ------------------------------------------------------------------- |
| `fhir.delete_limit` | `HASTE_FHIR.delete_limit` | `100` — max resources removed by a single type/system-level delete. |

#### `[repo]` — resource storage backend

Tagged by `backend`; only `postgres` exists today.

| Key                    | Env var                      | Default                                                      |
| ---------------------- | ---------------------------- | ------------------------------------------------------------ |
| `repo.backend`         | `HASTE_REPO.backend`         | `postgres`                                                   |
| `repo.database_url`    | `HASTE_REPO.database_url`    | `postgresql://postgres:postgres@localhost:5432/haste_health` |
| `repo.max_connections` | `HASTE_REPO.max_connections` | `10`                                                         |

#### `[search]` — search index backend

Tagged by `backend`: either `postgres` or `elasticsearch`. Defaults to
`postgres`, which indexes into its own database and so needs no extra service;
the shipped `haste.toml` and the Docker Compose files set it explicitly. Pick
`elasticsearch` to index into an Elasticsearch cluster instead.

With `backend = "postgres"`:

| Key                      | Env var                        | Default                                                        |
| ------------------------ | ------------------------------ | -------------------------------------------------------------- |
| `search.backend`         | `HASTE_SEARCH.backend`         | `postgres`                                                     |
| `search.database_url`    | `HASTE_SEARCH.database_url`    | `postgresql://postgres:postgres@127.0.0.1/haste_health_search` |
| `search.max_connections` | `HASTE_SEARCH.max_connections` | `20`                                                           |

The search index is kept in its own database, separate from `repo.database_url`,
so the two never share tables. Create it before running `admin migrate search`:

```bash
psql -h localhost -U postgres -c "CREATE DATABASE haste_health_search;"
```

`admin migrate search` (also part of `admin migrate all`) runs the sqlx migrations
in `backend/crates/fhir-search/pg-migrations` that have not run yet, recorded in
the `_sqlx_search_migrations` table, and then fails if the database still lacks a
table, column or index the search parameters need.

With `backend = "elasticsearch"`:

| Key                                      | Env var                                        | Default                 |
| ---------------------------------------- | ---------------------------------------------- | ----------------------- |
| `search.backend`                         | `HASTE_SEARCH.backend`                         | `elasticsearch`         |
| `search.url`                             | `HASTE_SEARCH.url`                             | `http://localhost:9200` |
| `search.username`                        | `HASTE_SEARCH.username`                        | `elastic`               |
| `search.password`                        | `HASTE_SEARCH.password`                        | `elastic`               |
| `search.prune_removed_search_parameters` | `HASTE_SEARCH.prune_removed_search_parameters` | `false`                 |

#### `[document_storage]`

On by default. `Binary.data` of at least `binary_min_bytes` bytes (after base64
decoding) is written to document storage instead of the resource row and put
back on every read (read, vread, history and search), so clients see no
difference. Smaller content stays in the row. Every version keeps its own
document, and a delete leaves earlier versions' documents in place.

A document belongs to the tenant and project of its resource and is named
`Binary/<document id>` within them. Every storage call names that tenant and
project, so a request can only ever reach its own project's documents. They are
columns in Postgres, a directory per tenant and project on the filesystem, and
the leading segments of the object name in a bucket.

Tagged by `backend`:

- `postgres` (the default): one table in the repository database
  (`document_storage`, with the content in a `bytea` column), so a deployment
  needs nothing beyond the database it already has. `haste-health admin
  migrate repo` (or `migrate all`) creates it; until it has run, writing a
  Binary over the threshold fails with an error naming that command.
  `database_url` puts it in another database; in the repository database,
  deleting a project deletes its documents with it.
- `filesystem`: a directory, for development, tests and single-node
  deployments.
- `gcs`: a Google Cloud Storage bucket.
- `none`: all content stays in the resource rows.

Setting any key of the section, such as `binary_min_bytes`, means setting
`backend` as well.

| Key                                       | Env var                                                                       | Default                                            |
| ----------------------------------------- | ----------------------------------------------------------------------------- | -------------------------------------------------- |
| `document_storage.backend`                | `HASTE_DOCUMENT_STORAGE.backend` (`postgres` / `filesystem` / `gcs` / `none`) | `postgres`                                         |
| `document_storage.binary_min_bytes`       | `HASTE_DOCUMENT_STORAGE.binary_min_bytes`                                     | `65536` (64 KiB)                                   |
| `document_storage.sweep_interval_seconds` | `HASTE_DOCUMENT_STORAGE.sweep_interval_seconds`                               | `86400` (a day); `0` turns the automatic sweep off |
| `document_storage.sweep_grace_seconds`    | `HASTE_DOCUMENT_STORAGE.sweep_grace_seconds`                                  | `86400`; must exceed the longest transaction       |

With `backend = "postgres"`:

| Key                                | Env var                                  | Default             |
| ---------------------------------- | ---------------------------------------- | ------------------- |
| `document_storage.database_url`    | `HASTE_DOCUMENT_STORAGE.database_url`    | the `repo` database |
| `document_storage.max_connections` | `HASTE_DOCUMENT_STORAGE.max_connections` | `10`, its own pool  |

With `backend = "filesystem"`:

| Key                     | Env var                       | Default                        |
| ----------------------- | ----------------------------- | ------------------------------ |
| `document_storage.root` | `HASTE_DOCUMENT_STORAGE.root` | required; created when missing |

With `backend = "gcs"`:

| Key                                 | Env var                                   | Default                                                                        |
| ----------------------------------- | ----------------------------------------- | ------------------------------------------------------------------------------ |
| `document_storage.bucket`           | `HASTE_DOCUMENT_STORAGE.bucket`           | —                                                                              |
| `document_storage.prefix`           | `HASTE_DOCUMENT_STORAGE.prefix`           | none; a folder-like prefix before every object, so deployments can share a bucket |
| `document_storage.endpoint`         | `HASTE_DOCUMENT_STORAGE.endpoint`         | `https://storage.googleapis.com`; point it at an emulator for development      |
| `document_storage.credentials.type` | `HASTE_DOCUMENT_STORAGE.credentials.type` | `application_default`                                                          |

Credential types, with their extra field:

- `application_default`: the file named by `GOOGLE_APPLICATION_CREDENTIALS`, then the file `gcloud auth application-default login` writes, then the metadata server.
- `service_account_key_file`, with `path`: a service account key file.
- `service_account_key_json`, with `json`: the contents of a service account key file.
- `metadata_server`: the metadata server of GCE, GKE, Cloud Run and similar runtimes.
- `anonymous`: no credentials, for emulators such as fake-gcs-server.
- `access_token`, with `token`: a fixed OAuth2 access token, for tests.

The server and the `worker` (which runs asynchronous jobs through the same
FHIR client) need the same settings. A process with storage off answers 500
for a Binary whose content is in document storage, rather than returning it
without its content.

Give every deployment storage of its own (a database, directory, bucket or
prefix): the sweep below removes any document the deployment's own database
has no version for.

A Binary still arrives as base64 inside a FHIR JSON body, so
`max_request_body_size` limits its size as before.

A write uploads its document before the database row commits, so a rolled
back transaction or a failed write can leave a document that no version refers
to. Each async-job worker removes those every `sweep_interval_seconds`, once
they are older than `sweep_grace_seconds`. `haste-health admin
document-storage sweep` does the same on demand, for documents older than
`--older-than-hours` (default 24); `--dry-run` only counts them.

#### `[email]` — optional

`None`/absent by default (email sending disabled, so sign-up and password reset fail). Tagged by `backend`: `sendgrid` sends through SendGrid; `log` writes each message to the server log instead of sending it, for local development, where the sign-up code can be read off the console.

| Key                  | Env var                                    |
| -------------------- | ------------------------------------------ |
| `email.backend`      | `HASTE_EMAIL.backend` (`sendgrid` / `log`) |
| `email.api_key`      | `HASTE_EMAIL.api_key` (sendgrid only)      |
| `email.from_address` | `HASTE_EMAIL.from_address` (sendgrid only) |

#### `[signup]`

Self-service sign-up and login at `/auth/signup` and `/auth/login`. Optional: a [Cloudflare Turnstile](https://developers.cloudflare.com/turnstile/) check on the form, enabled by setting both keys. The Content-Security-Policy opens to `challenges.cloudflare.com` only when it is configured.

Set `signup.enabled = false` on a deployment where visitors must not create tenants (workspaces) for themselves. `/auth/signup` then answers `403` with a "sign-up is not available" page, the links to it are hidden, and an address with no account that logs in at `/auth/login` is told there is no account instead of being offered a new tenant. Login for existing users is unaffected, and tenants are created with `admin tenant create` (see `cli_commands.md`).

| Key                           | Env var                             | Default |
| ----------------------------- | ----------------------------------- | ------- |
| `signup.enabled`              | `HASTE_SIGNUP.enabled`              | `true`  |
| `signup.turnstile.site_key`   | `HASTE_SIGNUP.turnstile.site_key`   | unset   |
| `signup.turnstile.secret_key` | `HASTE_SIGNUP.turnstile.secret_key` | unset   |

#### `[rate_limits]`

| Key                                         | Env var                                           | Default                                                                                               |
| ------------------------------------------- | ------------------------------------------------- | ----------------------------------------------------------------------------------------------------- |
| `rate_limits.rate_limit_subscription_tiers` | `HASTE_RATE_LIMITS.rate_limit_subscription_tiers` | unset (`None`) — a 4-element array, one limit per subscription tier free,profressional,team,unlimited |
| `rate_limits.rate_limit_window_seconds`     | `HASTE_RATE_LIMITS.rate_limit_window_seconds`     | `86400` (1 day)                                                                                       |
| `rate_limits.rate_limit_operation_points`   | `HASTE_RATE_LIMITS.rate_limit_operation_points`   | `100`                                                                                                 |

#### `[monitoring]`

| Key                        | Env var                          | Default                                                                                                                                                     |
| -------------------------- | -------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `monitoring.audit_enabled` | `HASTE_MONITORING.audit_enabled` | `false` — gates the [auditing middleware](#).                                                                                                               |
| `monitoring.ip_source`     | `HASTE_MONITORING.ip_source`     | `connect_info` — one of `connect_info`, `cf_connecting_ip`, `x_real_ip`; how client IP is derived (direct connection vs. Cloudflare/reverse-proxy headers). |

#### `[security]`

| Key                                | Env var                                  |
| ---------------------------------- | ---------------------------------------- |
| `security.publicize_fhir_metadata` | `HASTE_SECURITY.publicize_fhir_metadata` |
| `security.aes_key`                 | `HASTE_SECURITY.aes_key`                 |
| `security.certification_key`       | `HASTE_SECURITY.certification_key`       |

##### `[security.mfa]`

| Key                                     | Env var                                       | Default |
| --------------------------------------- | --------------------------------------------- | ------- |
| `security.mfa.max_credentials_per_user` | `HASTE_SECURITY.mfa.max_credentials_per_user` | `1`     |

##### `[security.encryption]` — secret provider for `aes_key`/`certification_key`

Tagged by `type`: `environment` (default), `gcp`, or `aws`.

| Key                                             | Env var                                | Default         |
| ----------------------------------------------- | -------------------------------------- | --------------- |
| `security.encryption.type`                      | `HASTE_SECURITY.encryption.type`       | `environment`   |
| `security.encryption.prefix` (environment only) | `HASTE_SECURITY.encryption.prefix`     | `HASTE_SECRET_` |
| `security.encryption.project_id` (gcp only)     | `HASTE_SECURITY.encryption.project_id` | —               |
| `security.encryption.region` (aws only)         | `HASTE_SECURITY.encryption.region`     | —               |

With the default `environment` provider, a secret named `aes_key = "AES_KEY"` is fetched from the environment variable `<prefix><name>` — e.g. `HASTE_SECRET_AES_KEY` — and must be **base64-encoded**. This is a separate env var namespace from the `HASTE_` config prefix above; the `HASTE_SECRET_` prefix is itself just the default value of `security.encryption.prefix`, not hardcoded.

### Example `haste.toml`

```toml
api_uri = "http://localhost:3000"
certification_dir = "certifications"
admin_app_redirect_uri = "http://*.localhost:3001"
max_request_body_size = 20971520 # 20MB

[repo]
backend = "postgres"
database_url = "postgresql://postgres:postgres@localhost:5432/haste_health"
max_connections = 20

[search]
backend = "postgres"
database_url = "postgresql://postgres:postgres@localhost:5432/haste_health_search"
max_connections = 20

# Binary content of 64 KiB or more goes to a table in the repo database by
# default. To keep it in a directory instead:
# [document_storage]
# backend = "filesystem"
# root = "/var/lib/haste-health/documents"

[monitoring]
audit_enabled = false

# Sign-up codes are logged rather than emailed. Use `backend = "sendgrid"`
# with `api_key` and `from_address` to send them.
[email]
backend = "log"

# Set to false to stop visitors creating tenants for themselves at
# /auth/signup. Tenants then come only from `admin tenant create`.
[signup]
enabled = true

# Optional bot check on the sign-up and login forms.
# [signup.turnstile]
# site_key = "..."
# secret_key = "..."

[security]
publicize_fhir_metadata = true
aes_key = "AES_KEY"
certification_key = "CERTIFICATE_KEY"

[security.mfa]
max_credentials_per_user = 1

[security.encryption]
type = "environment"
prefix = "HASTE_SECRET_"
```

### Example environment variables (container deployment)

```bash
HASTE_API_URI=http://localhost:3000
HASTE_CERTIFICATION_DIR=certifications
HASTE_ADMIN_APP_REDIRECT_URI=http://*.localhost:3001

HASTE_REPO.backend=postgres
HASTE_REPO.database_url=postgresql://postgres:postgres@postgres:5432/haste_health
HASTE_REPO.max_connections=20

HASTE_SEARCH.backend=postgres
HASTE_SEARCH.database_url=postgresql://postgres:postgres@postgres:5432/haste_health_search
HASTE_SEARCH.max_connections=20

# Optional: Binary content in a Google Cloud Storage bucket instead of the
# repository database.
# HASTE_DOCUMENT_STORAGE.backend=gcs
# HASTE_DOCUMENT_STORAGE.bucket=my-bucket
# HASTE_DOCUMENT_STORAGE.credentials.type=metadata_server
```
