use derivative::Derivative;
use haste_fhir_search::config::SearchConfig;
use haste_repository::config::RepoConfig;
use haste_worker::async_jobs::AsyncJobWorkerConfig;
use serde::{Deserialize, Serialize};
use std::{path::PathBuf, time::Duration};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct ServerConfig {
    /// Port on which the server listens.
    pub port: u16,
    /// Whether artifact mutations are allowed. If `false`, any attempt to
    /// create, update, or delete artifacts will be rejected.
    pub allow_artifact_mutations: bool,
    /// Used for JWT signing/verification.
    pub certification_dir: PathBuf,
    /// Main root where the FHIR server is hosted.
    pub api_uri: String,
    /// Where to redirect for the hardcoded admin app.
    pub admin_app_redirect_uri: String,

    pub fhir: FHIRConfig,
    pub repo: RepoConfig,
    pub search: SearchConfig,
    pub email: Option<EmailConfig>,
    pub rate_limits: RateLimitsConfig,
    pub max_request_body_size: usize,
    pub monitoring: MonitoringConfig,
    pub security: SecurityConfig,
    pub operations: OperationsConfig,
    pub signup: SignupConfig,
    pub jobs: JobsConfig,
    pub packages: PackagesConfig,
}

/// Asynchronous jobs: an operation invoked with `Prefer: respond-async` today,
/// Bulk Data exports next. The first three apply to the server's kick-off and
/// status endpoints; the rest to the worker that runs the jobs.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct JobsConfig {
    /// How long a finished job and its result stay available to poll, in
    /// seconds, and how long a job that never ran waits before it is cleared.
    pub retention_seconds: u64,
    /// Queued or running jobs one project may have at once. A kick-off past
    /// this is refused with `429 Too Many Requests`, as the Bulk Data
    /// specification allows.
    pub max_live_per_project: i64,
    /// What a polling client is told to wait, in seconds (`Retry-After`),
    /// while its job is still queued or running.
    pub retry_after_seconds: u64,
    /// How long one claim or lease renewal holds a job for a worker, in
    /// seconds. A worker that stops renewing for this long loses the job.
    pub lease_seconds: u64,
    /// How often an idle worker looks for queued jobs, in milliseconds.
    pub poll_interval_ms: u64,
    /// Jobs one worker runs at once.
    pub max_concurrent: usize,
    /// How often a worker sweeps abandoned and expired jobs, in seconds.
    pub maintenance_interval_seconds: u64,
}

impl JobsConfig {
    #[must_use]
    pub fn retention(&self) -> Duration {
        Duration::from_secs(self.retention_seconds)
    }

    #[must_use]
    pub fn worker_config(&self) -> AsyncJobWorkerConfig {
        AsyncJobWorkerConfig {
            poll_interval: Duration::from_millis(self.poll_interval_ms),
            lease_duration: Duration::from_secs(self.lease_seconds),
            max_concurrent_jobs: self.max_concurrent,
            maintenance_interval: Duration::from_secs(self.maintenance_interval_seconds),
            retention: self.retention(),
        }
    }
}

impl Default for JobsConfig {
    fn default() -> Self {
        Self {
            retention_seconds: 24 * 60 * 60,
            max_live_per_project: 10,
            retry_after_seconds: 5,
            lease_seconds: 60,
            poll_interval_ms: 1000,
            max_concurrent: 4,
            maintenance_interval_seconds: 60,
        }
    }
}

/// Installing FHIR packages (implementation guides) into a project with
/// `ImplementationGuide/$install`.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct PackagesConfig {
    /// The package registry a package named by `name@version` is fetched
    /// from. It serves the npm-style API packages.fhir.org does and must be
    /// an https URL; a redirect is followed only within it. Empty turns off
    /// fetching, so packages can only be uploaded.
    pub registry_url: String,
    /// Packages not installed when another package depends on them: large
    /// terminology and cross-version packages, that would add thousands of
    /// resources to every project that installs an IG. Naming one in `$install` still
    /// installs it. The FHIR core packages are always skipped, as the server
    /// has them built in.
    pub skip_dependencies: Vec<String>,
    /// The largest package tarball fetched or uploaded, compressed, in bytes.
    pub max_package_bytes: u64,
    /// The most files one package may hold.
    pub max_files: usize,
    /// The most packages one install may fetch or upload, dependencies of
    /// dependencies included. A fetch that fails still counts.
    pub max_install_packages: usize,
    /// The most the packages of one install may unpack to together, uploads
    /// included, in bytes. It is enforced while unpacking. An install's peak
    /// memory is about four times what its packages unpack to.
    pub max_install_bytes: u64,
    /// Installs this server runs at once; one more is refused with 429. One
    /// install at the byte limit can peak near 800 MB, so the default is 1.
    pub max_concurrent_installs: usize,
    /// How often one Free tier tenant may start an install, in seconds;
    /// another within this time is refused with 429. 0 turns the wait off.
    pub free_tier_install_interval_seconds: u64,
}

impl Default for PackagesConfig {
    fn default() -> Self {
        Self {
            registry_url: "https://packages.fhir.org".to_string(),
            skip_dependencies: [
                "hl7.terminology",
                "hl7.terminology.r4",
                "hl7.terminology.r5",
                "us.nlm.vsac",
                "us.cdc.phinvads",
                "hl7.fhir.uv.xver-r5.r4",
                "hl7.fhir.uv.xver-r4b.r4",
                "hl7.fhir.uv.xver-r3.r4",
                "hl7.fhir.uv.xver-r2.r4",
            ]
            .map(str::to_string)
            .to_vec(),
            max_package_bytes: 20 * 1024 * 1024,
            max_files: 20_000,
            max_install_packages: 50,
            max_install_bytes: 200 * 1024 * 1024,
            max_concurrent_installs: 1,
            free_tier_install_interval_seconds: 120,
        }
    }
}

/// Sign-up and login at `/auth/signup` and `/auth/login`.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct SignupConfig {
    /// Whether a visitor may create a tenant for themselves. If `false`,
    /// `/auth/signup` is closed and tenants come only from
    /// `admin tenant create`. Login is unaffected.
    pub enabled: bool,
    /// Optional Cloudflare Turnstile bot check on the email form.
    pub turnstile: Option<TurnstileConfig>,
}

#[derive(Derivative, Clone, Deserialize, Serialize)]
#[derivative(Debug)]
pub struct TurnstileConfig {
    /// Public key rendered into the widget.
    pub site_key: String,
    /// Server-side key used to verify the widget's token.
    #[derivative(Debug = "ignore")]
    pub secret_key: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct SecurityConfig {
    pub publicize_fhir_metadata: bool,
    pub mfa: MFAConfig,
    pub encryption: SecretProviderConfig,
    pub aes_key: Option<String>,
    pub certification_key: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct MFAConfig {
    pub max_credentials_per_user: usize,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct OperationsConfig {
    /// Number of dedicated OS threads in the pool that executes custom
    /// (tenant-authored) FHIR operation scripts in isolated Deno/V8
    /// sandboxes. Each thread handles one script invocation at a time, so
    /// this is a hard ceiling on how many custom operations can run
    /// concurrently. Defaults to roughly half the host's available
    /// parallelism.
    pub deno_pool_threads: usize,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
#[derive(Default)]
pub struct MonitoringConfig {
    pub audit_enabled: bool,
    pub ip_source: IpSource,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SecretProviderConfig {
    Environment { prefix: Option<String> },
    GCP { project_id: String },
    AWS { region: String },
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct FHIRConfig {
    /// Max delete limit for type-delete and system-delete operations.
    pub delete_limit: u64,
}

#[derive(Derivative, Clone, Deserialize, Serialize)]
#[derivative(Debug)]
#[serde(tag = "backend", rename_all = "snake_case")]
pub enum EmailConfig {
    SendGrid {
        #[derivative(Debug = "ignore")]
        api_key: String,
        #[derivative(Debug = "ignore")]
        from_address: String,
    },
    /// Logs each message instead of sending it. For local development.
    Log,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct RateLimitsConfig {
    pub rate_limit_subscription_tiers: Option<[usize; 4]>,
    pub rate_limit_window_seconds: u64,
    pub rate_limit_operation_points: u32,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum IpSource {
    #[default]
    ConnectInfo,
    CfConnectingIp,
    XRealIp,
}

impl Default for FHIRConfig {
    fn default() -> Self {
        Self { delete_limit: 100 }
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            port: 3000,
            allow_artifact_mutations: false,
            certification_dir: PathBuf::from("certifications"),
            api_uri: "http://localhost:3000".into(),
            admin_app_redirect_uri: "http://*.localhost:3001".into(),
            fhir: FHIRConfig::default(),
            repo: RepoConfig::default(),
            search: SearchConfig::default(),
            email: None,
            max_request_body_size: 4 * 1024 * 1024,
            rate_limits: RateLimitsConfig::default(),
            monitoring: MonitoringConfig::default(),
            security: SecurityConfig::default(),
            operations: OperationsConfig::default(),
            signup: SignupConfig::default(),
            jobs: JobsConfig::default(),
            packages: PackagesConfig::default(),
        }
    }
}

impl Default for SignupConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            turnstile: None,
        }
    }
}

impl Default for RateLimitsConfig {
    fn default() -> Self {
        Self {
            rate_limit_subscription_tiers: None,
            rate_limit_window_seconds: 60 * 60 * 24, // 1 day in seconds
            rate_limit_operation_points: 100,
        }
    }
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            publicize_fhir_metadata: true,
            mfa: MFAConfig::default(),
            encryption: SecretProviderConfig::default(),
            aes_key: None,
            certification_key: None,
        }
    }
}

impl Default for MFAConfig {
    fn default() -> Self {
        Self {
            max_credentials_per_user: 1,
        }
    }
}

impl Default for OperationsConfig {
    fn default() -> Self {
        let available = std::thread::available_parallelism()
            .map(std::num::NonZeroUsize::get)
            .unwrap_or(4);

        Self {
            deno_pool_threads: (available / 2).max(2),
        }
    }
}

impl Default for SecretProviderConfig {
    fn default() -> Self {
        Self::Environment {
            prefix: Some("HASTE_SECRET_".to_string()),
        }
    }
}
