# Haste Health Worker

## Configuration

Worker configuration is loaded with [Figment](https://docs.rs/figment), merging two sources in order (`backend/src/commands/worker.rs`):

```rust
let config: Arc<search_indexing::WorkerEnvironment> = Arc::new(
    Figment::new()
        .merge(Toml::file("haste.toml"))
        .merge(Env::prefixed("HASTE_"))
        .extract()?,
);
```

1. **`haste.toml`** — read relative to the process's working directory (i.e. `backend/` when running `cargo run worker` from there). If the file doesn't exist, this step is silently skipped.
2. **`HASTE_`-prefixed environment variables** — merged _after_ the TOML file, so **environment variables win** on any key present in both.

Any field not set by either source falls back to the default documented below.

This is the same `HASTE_` prefix and TOML file used by the [server](./server_configuration.md), so a single `haste.toml`/environment can configure both processes, though `repo`/`search` are defined independently for the worker (`WorkerEnvironment` in `backend/crates/worker/src/search_indexing/mod.rs`) and only support the fields listed below.

### Environment variable naming

- Top-level fields: `HASTE_<FIELD>`, e.g. `HASTE_MAX_CONCURRENT_LIMIT`.
- Nested sections: `HASTE_<SECTION>.<field>` — the prefix plus the section name, then a **literal dot**, then the field name as it appears in TOML (lowercase), e.g. `HASTE_REPO.database_url`, `HASTE_SEARCH.username`.
- Sections that pick a backend via a tag (`repo`, `search`) need that tag set too, e.g. `HASTE_REPO.backend=postgres`.

### Reference

#### Top level

| Key                    | Env var                      | Default | Notes                                                                                 |
| ---------------------- | ---------------------------- | ------- | ------------------------------------------------------------------------------------- |
| `max_concurrent_limit` | `HASTE_MAX_CONCURRENT_LIMIT` | `1000`  | Max number of resources fetched/indexed per tenant on each pass of the indexing loop. |

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

### Example `haste.toml`

```toml
max_concurrent_limit = 1000

[repo]
backend = "postgres"
database_url = "postgresql://postgres:postgres@localhost:5432/haste_health"
max_connections = 10

[search]
backend = "elasticsearch"
url = "http://localhost:9200"
username = "elastic"
password = "elastic"
```

### Example environment variables (container deployment)

```bash
HASTE_MAX_CONCURRENT_LIMIT=1000

HASTE_REPO.backend=postgres
HASTE_REPO.database_url=postgresql://postgres:postgres@postgres:5432/haste_health
HASTE_REPO.max_connections=10

HASTE_SEARCH.backend=elasticsearch
HASTE_SEARCH.url=http://elasticsearch:9200
HASTE_SEARCH.username=elastic
HASTE_SEARCH.password=elastic
```
