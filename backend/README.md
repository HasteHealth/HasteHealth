# Haste Health Backend

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
- Sections that pick a backend/provider via a tag (`repo`, `search`, `email`, `security.encryption`) need that tag set too, e.g. `HASTE_REPO.backend=postgres`.

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

Tagged by `backend`; only `elasticsearch` exists today.

| Key               | Env var                 | Default                 |
| ----------------- | ----------------------- | ----------------------- |
| `search.backend`  | `HASTE_SEARCH.backend`  | `elasticsearch`         |
| `search.url`      | `HASTE_SEARCH.url`      | `http://localhost:9200` |
| `search.username` | `HASTE_SEARCH.username` | `elastic`               |
| `search.password` | `HASTE_SEARCH.password` | `elastic`               |

#### `[email]` — optional

`None`/absent by default (email sending disabled). Tagged by `backend`; only `sendgrid` exists today.

| Key                  | Env var                            |
| -------------------- | ---------------------------------- |
| `email.backend`      | `HASTE_EMAIL.backend` (`sendgrid`) |
| `email.api_key`      | `HASTE_EMAIL.api_key`              |
| `email.from_address` | `HASTE_EMAIL.from_address`         |

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
backend = "elasticsearch"
url = "http://localhost:9200"
username = "elastic"
password = "elastic"

[monitoring]
audit_enabled = false

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

HASTE_SEARCH.backend=elasticsearch
HASTE_SEARCH.url=http://elasticsearch:9200
HASTE_SEARCH.username=elastic
HASTE_SEARCH.password=elastic
```
