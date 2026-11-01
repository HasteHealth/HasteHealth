//! `profile` command: manage named server connection profiles.
//!
//! This module only owns the CLI surface (argument parsing, prompts, output). The persisted
//! schema lives in [`crate::cli::config`]; secrets in [`crate::cli::secrets`].
//!
//! Every value can be passed on the command line. Anything missing is prompted for in a
//! terminal; outside one it falls back to its default or fails naming the option to pass,
//! so scripts never hang on a prompt.

use crate::cli::{
    client::unix_now,
    config::{
        CliConfiguration, DEFAULT_CLIENT_CREDENTIALS_SCOPE, Profile, ProfileAuth, write_config,
    },
    secrets::{self, ProfileSecrets, StoredTokens},
    state::{CONFIG_LOCATION, CliState, SECRETS_LOCATION},
};
use clap::{Args, Subcommand, ValueEnum};
use dialoguer::{Confirm, Input, Password, Select, theme::ColorfulTheme};
use haste_fhir_model::r4::generated::terminology::IssueType;
use haste_fhir_operation_error::OperationOutcomeError;
use std::{io::IsTerminal, sync::Arc};
use tokio::sync::Mutex;

const DEFAULT_REDIRECT_URI: &str = "http://127.0.0.1:8976/callback";
const DEFAULT_AUTHORIZATION_CODE_SCOPE: &str = "openid profile fhirUser offline_access user/*.*";

const CREATE_HINT: &str = "Create one with `haste-health profile create`.";
const USE_HINT: &str = "Pick one with `haste-health profile use`.";

/// How the CLI authenticates for a profile.
#[derive(Clone, Copy, Debug, ValueEnum, PartialEq, Eq)]
pub(crate) enum AuthModeChoice {
    /// A human logs in through the browser (authorization_code + PKCE) with a public
    /// client. Run `haste-health login` afterwards to sign in.
    AuthorizationCode,
    /// A confidential (server-to-server) client authenticated with a client secret.
    ClientCredentials,
    /// HTTP basic authentication with a username and password.
    BasicAuth,
    /// No authentication: requests are sent anonymously.
    #[value(name = "none")]
    NoAuth,
}

impl AuthModeChoice {
    /// Order shown in the picker; the first is preselected.
    const ALL: [AuthModeChoice; 4] = [
        AuthModeChoice::AuthorizationCode,
        AuthModeChoice::ClientCredentials,
        AuthModeChoice::BasicAuth,
        AuthModeChoice::NoAuth,
    ];

    fn of(auth: &ProfileAuth) -> Self {
        match auth {
            ProfileAuth::AuthorizationCode { .. } => AuthModeChoice::AuthorizationCode,
            ProfileAuth::ClientCredentails { .. } => AuthModeChoice::ClientCredentials,
            ProfileAuth::Basic { .. } => AuthModeChoice::BasicAuth,
            ProfileAuth::Public {} => AuthModeChoice::NoAuth,
        }
    }

    /// The `--auth-mode` value, e.g. `client-credentials`.
    fn name(self) -> String {
        self.to_possible_value()
            .expect("no auth mode is skipped")
            .get_name()
            .to_string()
    }

    fn picker_label(self) -> &'static str {
        match self {
            AuthModeChoice::AuthorizationCode => "Authorization code (log in through the browser)",
            AuthModeChoice::ClientCredentials => "Client credentials (client ID and secret)",
            AuthModeChoice::BasicAuth => "Basic auth (username and password)",
            AuthModeChoice::NoAuth => "None (anonymous requests)",
        }
    }

    /// Whether the mode gets tokens from an OIDC provider, and so needs a discovery URI
    /// and client ID.
    fn uses_oidc(self) -> bool {
        matches!(
            self,
            AuthModeChoice::AuthorizationCode | AuthModeChoice::ClientCredentials
        )
    }
}

#[derive(Args, Debug)]
pub(crate) struct CreateArgs {
    /// Profile name.
    name: Option<String>,
    /// Base URL of the FHIR R4 server.
    #[arg(short, long)]
    r4_url: Option<String>,
    /// How the CLI authenticates for this profile.
    #[arg(long, value_enum)]
    auth_mode: Option<AuthModeChoice>,
    /// OIDC discovery (`.well-known/openid-configuration`) URI. Used by authorization-code
    /// and client-credentials.
    #[arg(short, long)]
    discovery_uri: Option<String>,
    /// OAuth client ID. Used by authorization-code and client-credentials.
    #[arg(long)]
    client_id: Option<String>,
    /// OAuth client secret, for client-credentials. Stored in the secrets file, not the
    /// profile.
    #[arg(long)]
    client_secret: Option<String>,
    /// Loopback redirect URI for authorization-code (must be registered on the client).
    /// Defaults to `http://127.0.0.1:8976/callback`.
    #[arg(long)]
    redirect_uri: Option<String>,
    /// OAuth scope to request. Defaults to `openid profile fhirUser offline_access user/*.*`
    /// for authorization-code and `openid system/*.*` for client-credentials.
    #[arg(long)]
    scope: Option<String>,
    /// Username, for basic-auth.
    #[arg(long)]
    username: Option<String>,
    /// Password, for basic-auth. Stored in the secrets file, not the profile.
    #[arg(long)]
    password: Option<String>,
}

/// Manage named server connection profiles.
#[derive(Subcommand, Debug)]
pub(crate) enum ProfileCommands {
    /// List all profiles. `*` marks the active one.
    List,
    /// Show a profile's settings (never secrets or tokens). Defaults to the active profile.
    Show {
        /// Profile name.
        name: Option<String>,
    },
    /// Create a profile and make it active. Prompts for anything not passed as an option.
    Create(CreateArgs),
    /// Switch the active profile that other commands use. Without NAME, pick from a list.
    Use {
        /// Profile name.
        name: Option<String>,
    },
    /// Delete a profile and its stored secrets. Without NAME, pick from a list.
    Delete {
        /// Profile name.
        name: Option<String>,
        /// Delete without asking for confirmation.
        #[arg(short, long)]
        yes: bool,
    },
}

/// Fills in values that weren't passed on the command line.
struct Prompter {
    /// Whether there's a terminal to prompt in. Without one, a missing value falls back to
    /// its default, or is an error naming the option to pass.
    interactive: bool,
    theme: ColorfulTheme,
}

impl Prompter {
    fn new(interactive: bool) -> Self {
        Prompter {
            interactive,
            theme: ColorfulTheme::default(),
        }
    }

    fn text(
        &self,
        value: Option<&str>,
        option: &str,
        prompt: &str,
        default: Option<&str>,
    ) -> Result<String, OperationOutcomeError> {
        if let Some(value) = value {
            return Ok(value.to_string());
        }
        if !self.interactive {
            return default.map(str::to_string).ok_or_else(|| missing(option));
        }

        let mut input = Input::<String>::with_theme(&self.theme).with_prompt(prompt);
        if let Some(default) = default {
            input = input.default(default.to_string());
        }
        input.interact_text().map_err(prompt_failed)
    }

    fn secret(
        &self,
        value: Option<&str>,
        option: &str,
        prompt: &str,
    ) -> Result<String, OperationOutcomeError> {
        if let Some(value) = value {
            return Ok(value.to_string());
        }
        if !self.interactive {
            return Err(missing(option));
        }

        Password::with_theme(&self.theme)
            .with_prompt(prompt)
            .interact()
            .map_err(prompt_failed)
    }

    /// The index of the picked item; `default` is preselected.
    fn select(
        &self,
        option: &str,
        prompt: &str,
        items: &[&str],
        default: usize,
    ) -> Result<usize, OperationOutcomeError> {
        if !self.interactive {
            return Err(missing(option));
        }

        Select::with_theme(&self.theme)
            .with_prompt(prompt)
            .items(items)
            .default(default)
            .interact()
            .map_err(prompt_failed)
    }

    fn confirm(&self, option: &str, prompt: &str) -> Result<bool, OperationOutcomeError> {
        if !self.interactive {
            return Err(missing(option));
        }

        Confirm::with_theme(&self.theme)
            .with_prompt(prompt)
            .interact()
            .map_err(prompt_failed)
    }
}

fn error(message: impl Into<String>) -> OperationOutcomeError {
    OperationOutcomeError::error(IssueType::invalid(), message.into())
}

fn missing(option: &str) -> OperationOutcomeError {
    error(format!(
        "Missing {option}. Pass it on the command line, or run in a terminal to be prompted for it."
    ))
}

fn prompt_failed(e: dialoguer::Error) -> OperationOutcomeError {
    OperationOutcomeError::error(IssueType::exception(), format!("Prompt failed: {e}"))
}

fn no_active_hint(config: &CliConfiguration) -> &'static str {
    if config.profiles.is_empty() {
        CREATE_HINT
    } else {
        USE_HINT
    }
}

fn find<'a>(
    config: &'a CliConfiguration,
    name: &str,
) -> Result<&'a Profile, OperationOutcomeError> {
    config
        .profiles
        .iter()
        .find(|profile| profile.name == name)
        .ok_or_else(|| {
            error(format!(
                "No profile named '{name}'. Run `haste-health profile list` to see them."
            ))
        })
}

/// The named profile, or one picked from a list when no name was given.
fn pick<'a>(
    config: &'a CliConfiguration,
    name: Option<&str>,
    prompter: &Prompter,
    prompt: &str,
) -> Result<&'a Profile, OperationOutcomeError> {
    if let Some(name) = name {
        return find(config, name);
    }
    if config.profiles.is_empty() {
        return Err(error(format!("No profiles yet. {CREATE_HINT}")));
    }

    let names: Vec<&str> = config.profiles.iter().map(|p| p.name.as_str()).collect();
    let active = config
        .active_profile
        .as_deref()
        .and_then(|active| names.iter().position(|name| *name == active))
        .unwrap_or(0);
    let index = prompter.select("the profile name", prompt, &names, active)?;

    Ok(&config.profiles[index])
}

/// Builds the profile `create` saves, plus the secret (client secret or password) that
/// goes in the secrets file rather than the profile.
fn build_profile(
    name: String,
    args: &CreateArgs,
    prompter: &Prompter,
) -> Result<(Profile, Option<String>), OperationOutcomeError> {
    let r4_url = prompter.text(
        args.r4_url.as_deref(),
        "--r4-url",
        "FHIR R4 server URL",
        None,
    )?;

    let auth_mode = match args.auth_mode {
        Some(mode) => mode,
        None => {
            let labels = AuthModeChoice::ALL.map(AuthModeChoice::picker_label);
            AuthModeChoice::ALL[prompter.select("--auth-mode", "Auth mode", &labels, 0)?]
        }
    };

    // Basic auth and anonymous profiles never fetch the discovery document, so it's only
    // kept if passed.
    let oidc_discovery_uri = if auth_mode.uses_oidc() {
        prompter.text(
            args.discovery_uri.as_deref(),
            "--discovery-uri",
            "OIDC discovery URI",
            None,
        )?
    } else {
        args.discovery_uri.clone().unwrap_or_default()
    };

    let client_id = || {
        prompter.text(
            args.client_id.as_deref(),
            "--client-id",
            "OAuth client ID",
            None,
        )
    };

    let (auth, secret) = match auth_mode {
        AuthModeChoice::AuthorizationCode => (
            ProfileAuth::AuthorizationCode {
                client_id: client_id()?,
                redirect_uri: prompter.text(
                    args.redirect_uri.as_deref(),
                    "--redirect-uri",
                    "Loopback redirect URI",
                    Some(DEFAULT_REDIRECT_URI),
                )?,
                scope: prompter.text(
                    args.scope.as_deref(),
                    "--scope",
                    "OAuth scope",
                    Some(DEFAULT_AUTHORIZATION_CODE_SCOPE),
                )?,
            },
            None,
        ),
        AuthModeChoice::ClientCredentials => {
            let client_id = client_id()?;
            let client_secret = prompter.secret(
                args.client_secret.as_deref(),
                "--client-secret",
                "OAuth client secret",
            )?;
            let scope = prompter.text(
                args.scope.as_deref(),
                "--scope",
                "OAuth scope",
                Some(DEFAULT_CLIENT_CREDENTIALS_SCOPE),
            )?;

            (
                ProfileAuth::ClientCredentails {
                    client_id,
                    // Unset means "the default", which is what the client falls back to.
                    scope: (scope != DEFAULT_CLIENT_CREDENTIALS_SCOPE).then_some(scope),
                },
                Some(client_secret),
            )
        }
        AuthModeChoice::BasicAuth => (
            ProfileAuth::Basic {
                username: prompter.text(
                    args.username.as_deref(),
                    "--username",
                    "Username",
                    None,
                )?,
            },
            Some(prompter.secret(args.password.as_deref(), "--password", "Password")?),
        ),
        AuthModeChoice::NoAuth => (ProfileAuth::Public {}, None),
    };

    Ok((
        Profile {
            name,
            r4_url,
            oidc_discovery_uri,
            auth,
        },
        secret,
    ))
}

/// Options that were passed but that `mode` doesn't use.
fn ignored_options(args: &CreateArgs, mode: AuthModeChoice) -> Vec<&'static str> {
    let given = |value: &Option<String>| value.as_deref().is_some_and(|v| !v.is_empty());

    [
        ("--client-id", given(&args.client_id), mode.uses_oidc()),
        (
            "--client-secret",
            given(&args.client_secret),
            mode == AuthModeChoice::ClientCredentials,
        ),
        (
            "--redirect-uri",
            given(&args.redirect_uri),
            mode == AuthModeChoice::AuthorizationCode,
        ),
        ("--scope", given(&args.scope), mode.uses_oidc()),
        (
            "--username",
            given(&args.username),
            mode == AuthModeChoice::BasicAuth,
        ),
        (
            "--password",
            given(&args.password),
            mode == AuthModeChoice::BasicAuth,
        ),
    ]
    .into_iter()
    .filter(|(_, given, used)| *given && !used)
    .map(|(option, ..)| option)
    .collect()
}

fn create(
    state: &mut CliState,
    args: &CreateArgs,
    prompter: &Prompter,
) -> Result<(), OperationOutcomeError> {
    let name = prompter.text(
        args.name.as_deref(),
        "the profile name",
        "Profile name",
        None,
    )?;
    if name.trim().is_empty() {
        return Err(error("The profile name can't be empty."));
    }
    // Checked before the other prompts so nobody fills them all in for nothing.
    if state.config.profiles.iter().any(|p| p.name == name) {
        return Err(error(format!(
            "A profile named '{name}' already exists. Delete it with `haste-health profile delete {name}` or pick another name."
        )));
    }

    let (profile, secret) = build_profile(name.clone(), args, prompter)?;
    let mode = AuthModeChoice::of(&profile.auth);
    let ignored = ignored_options(args, mode);
    if !ignored.is_empty() {
        eprintln!(
            "Ignored {}: not used by {} profiles.",
            ignored.join(", "),
            mode.name()
        );
    }

    state.config.profiles.push(profile);
    state.config.active_profile = Some(name.clone());

    // The client secret or password goes in the secrets file, never the profile.
    if let Some(secret) = secret {
        state.secrets.profile_mut(&name).client_secret = Some(secret);
        secrets::write_secrets(&SECRETS_LOCATION, &state.secrets)?;
    }
    write_config(&CONFIG_LOCATION, &state.config)?;

    eprintln!("Created profile '{name}' and made it active.");
    if mode == AuthModeChoice::AuthorizationCode {
        eprintln!("Run `haste-health login` to sign in.");
    }

    Ok(())
}

fn delete(
    state: &mut CliState,
    name: Option<&str>,
    yes: bool,
    prompter: &Prompter,
) -> Result<(), OperationOutcomeError> {
    let name = pick(&state.config, name, prompter, "Profile to delete")?
        .name
        .clone();

    let confirmed = yes
        || prompter.confirm(
            "--yes",
            &format!("Delete profile '{name}' and its stored secrets?"),
        )?;
    if !confirmed {
        eprintln!("Cancelled. Profile '{name}' was not deleted.");
        return Ok(());
    }

    state.config.profiles.retain(|profile| profile.name != name);
    let was_active = state.config.active_profile.as_deref() == Some(name.as_str());
    if was_active {
        state.config.active_profile = None;
    }
    state.secrets.remove_profile(&name);

    secrets::write_secrets(&SECRETS_LOCATION, &state.secrets)?;
    write_config(&CONFIG_LOCATION, &state.config)?;

    eprintln!("Deleted profile '{name}'.");
    if was_active {
        eprintln!(
            "No profile is active now. {}",
            no_active_hint(&state.config)
        );
    }

    Ok(())
}

fn print_list(config: &CliConfiguration) {
    if config.profiles.is_empty() {
        eprintln!("No profiles yet. {CREATE_HINT}");
        return;
    }

    let rows: Vec<(&str, String, &str)> = config
        .profiles
        .iter()
        .map(|p| {
            (
                p.name.as_str(),
                AuthModeChoice::of(&p.auth).name(),
                p.r4_url.as_str(),
            )
        })
        .collect();
    let name_width = rows
        .iter()
        .map(|r| r.0.len())
        .chain(["NAME".len()])
        .max()
        .unwrap_or(0);
    let auth_width = rows
        .iter()
        .map(|r| r.1.len())
        .chain(["AUTH".len()])
        .max()
        .unwrap_or(0);

    println!(
        "  {:<name_width$}  {:<auth_width$}  FHIR R4 URL",
        "NAME", "AUTH"
    );
    for (name, auth, url) in &rows {
        let marker = if config.active_profile.as_deref() == Some(*name) {
            '*'
        } else {
            ' '
        };
        println!("{marker} {name:<name_width$}  {auth:<auth_width$}  {url}");
    }
}

fn print_profile(profile: &Profile, active: bool, secrets: Option<&ProfileSecrets>) {
    let mode = AuthModeChoice::of(&profile.auth);
    let has_secret = secrets.and_then(|s| s.client_secret.as_ref()).is_some();

    let mut rows = vec![
        (
            "Name",
            if active {
                format!("{} (active)", profile.name)
            } else {
                profile.name.clone()
            },
        ),
        ("FHIR R4 URL", profile.r4_url.clone()),
        ("Auth mode", mode.name()),
    ];
    if mode.uses_oidc() || !profile.oidc_discovery_uri.is_empty() {
        rows.push(("Discovery URI", profile.oidc_discovery_uri.clone()));
    }

    match &profile.auth {
        ProfileAuth::AuthorizationCode {
            client_id,
            redirect_uri,
            scope,
        } => {
            rows.push(("Client ID", client_id.clone()));
            rows.push(("Redirect URI", redirect_uri.clone()));
            rows.push(("Scope", scope.clone()));
            rows.push((
                "Login",
                login_status(secrets.and_then(|s| s.tokens.as_ref())),
            ));
        }
        ProfileAuth::ClientCredentails { client_id, scope } => {
            rows.push(("Client ID", client_id.clone()));
            rows.push((
                "Scope",
                scope
                    .clone()
                    .unwrap_or_else(|| DEFAULT_CLIENT_CREDENTIALS_SCOPE.to_string()),
            ));
            rows.push(("Client secret", secret_status(has_secret)));
        }
        ProfileAuth::Basic { username } => {
            rows.push(("Username", username.clone()));
            rows.push(("Password", secret_status(has_secret)));
        }
        ProfileAuth::Public {} => {}
    }

    let width = rows.iter().map(|(label, _)| label.len()).max().unwrap_or(0);
    for (label, value) in rows {
        println!("{label:<width$}  {value}");
    }
}

fn secret_status(stored: bool) -> String {
    if stored {
        format!("stored in {}", SECRETS_LOCATION.display())
    } else {
        "missing; delete and recreate the profile to set it".to_string()
    }
}

fn login_status(tokens: Option<&StoredTokens>) -> String {
    let Some(tokens) = tokens else {
        return "not logged in; run `haste-health login`".to_string();
    };

    let remaining = tokens.expires_at - unix_now();
    if remaining > 0 {
        format!(
            "logged in, access token valid for {}",
            format_duration(remaining)
        )
    } else if tokens.refresh_token.is_some() {
        "logged in, access token refreshes on next use".to_string()
    } else {
        "session expired; run `haste-health login`".to_string()
    }
}

fn format_duration(seconds: i64) -> String {
    match seconds / 60 {
        0 => "under a minute".to_string(),
        minutes @ 1..60 => format!("{minutes} min"),
        minutes => format!("{} h {} min", minutes / 60, minutes % 60),
    }
}

/// Runs the `profile` command group.
pub(crate) async fn run(
    state: Arc<Mutex<CliState>>,
    command: &ProfileCommands,
) -> Result<(), OperationOutcomeError> {
    let prompter = Prompter::new(std::io::stdin().is_terminal() && std::io::stderr().is_terminal());
    let mut state = state.lock().await;

    match command {
        ProfileCommands::List => {
            print_list(&state.config);
            Ok(())
        }
        ProfileCommands::Show { name } => {
            let profile = match name.as_deref() {
                Some(name) => find(&state.config, name)?,
                None => match state.config.current_profile() {
                    Some(profile) => profile,
                    None => {
                        eprintln!("No active profile. {}", no_active_hint(&state.config));
                        return Ok(());
                    }
                },
            };
            let active = state.config.active_profile.as_deref() == Some(profile.name.as_str());
            print_profile(profile, active, state.secrets.profile(&profile.name));
            Ok(())
        }
        ProfileCommands::Create(args) => create(&mut state, args, &prompter),
        ProfileCommands::Use { name } => {
            let name = pick(&state.config, name.as_deref(), &prompter, "Profile to use")?
                .name
                .clone();
            state.config.active_profile = Some(name.clone());
            write_config(&CONFIG_LOCATION, &state.config)?;
            eprintln!("Now using profile '{name}'.");
            Ok(())
        }
        ProfileCommands::Delete { name, yes } => {
            delete(&mut state, name.as_deref(), *yes, &prompter)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Cli, CliCommand};
    use clap::{CommandFactory, Parser};

    fn parse(args: &[&str]) -> Result<ProfileCommands, clap::Error> {
        let cli = Cli::try_parse_from(std::iter::once("haste-health").chain(args.iter().copied()))?;
        match cli.command {
            CliCommand::Profile { command } => Ok(command),
            _ => panic!("not a profile command: {args:?}"),
        }
    }

    fn create_args(args: &[&str]) -> CreateArgs {
        match parse(args).unwrap() {
            ProfileCommands::Create(args) => args,
            other => panic!("expected create, got {other:?}"),
        }
    }

    /// Runs `create`'s resolution as it would outside a terminal.
    fn build(args: &[&str]) -> Result<(Profile, Option<String>), OperationOutcomeError> {
        let args = create_args(args);
        let name = args.name.clone().unwrap();
        build_profile(name, &args, &Prompter::new(false))
    }

    #[test]
    fn cli_definition_is_valid() {
        Cli::command().debug_assert();
    }

    #[test]
    fn create_builds_each_auth_mode() {
        let (profile, secret) = build(&[
            "profile",
            "create",
            "browser",
            "--r4-url",
            "http://localhost:3000/fhir",
            "--auth-mode",
            "authorization-code",
            "--discovery-uri",
            "http://localhost:3000/.well-known/openid-configuration",
            "--client-id",
            "cli",
        ])
        .unwrap();
        // Outside a terminal, options with defaults fall back to them.
        assert!(matches!(
            profile.auth,
            ProfileAuth::AuthorizationCode { ref client_id, ref redirect_uri, ref scope }
                if client_id == "cli"
                    && redirect_uri == DEFAULT_REDIRECT_URI
                    && scope == DEFAULT_AUTHORIZATION_CODE_SCOPE
        ));
        assert_eq!(secret, None);

        let (profile, secret) = build(&[
            "profile",
            "create",
            "service",
            "--r4-url",
            "http://localhost:3000/fhir",
            "--auth-mode",
            "client-credentials",
            "--discovery-uri",
            "http://localhost:3000/.well-known/openid-configuration",
            "--client-id",
            "worker",
            "--client-secret",
            "s3cret",
        ])
        .unwrap();
        // The default scope is stored as unset, which the client reads as the default.
        assert!(matches!(
            profile.auth,
            ProfileAuth::ClientCredentails { ref client_id, scope: None } if client_id == "worker"
        ));
        assert_eq!(secret.as_deref(), Some("s3cret"));

        let (profile, secret) = build(&[
            "profile",
            "create",
            "basic",
            "--r4-url",
            "http://localhost:3000/fhir",
            "--auth-mode",
            "basic-auth",
            "--username",
            "alice",
            "--password",
            "hunter2",
        ])
        .unwrap();
        assert!(matches!(profile.auth, ProfileAuth::Basic { ref username } if username == "alice"));
        assert_eq!(secret.as_deref(), Some("hunter2"));

        let (profile, secret) = build(&[
            "profile",
            "create",
            "anonymous",
            "--r4-url",
            "http://localhost:3000/fhir",
            "--auth-mode",
            "none",
        ])
        .unwrap();
        assert!(matches!(profile.auth, ProfileAuth::Public {}));
        assert_eq!(secret, None);
    }

    #[test]
    fn missing_values_name_the_option_outside_a_terminal() {
        let error = build(&["profile", "create", "x", "--r4-url", "http://localhost"])
            .unwrap_err()
            .to_string();
        assert!(error.contains("--auth-mode"), "{error}");

        let error = build(&[
            "profile",
            "create",
            "x",
            "--r4-url",
            "http://localhost",
            "--auth-mode",
            "client-credentials",
            "--discovery-uri",
            "http://localhost/.well-known/openid-configuration",
            "--client-id",
            "worker",
        ])
        .unwrap_err()
        .to_string();
        assert!(error.contains("--client-secret"), "{error}");
    }

    #[test]
    fn options_the_auth_mode_does_not_use_are_reported() {
        let args = create_args(&[
            "profile",
            "create",
            "x",
            "--auth-mode",
            "authorization-code",
            "--client-id",
            "cli",
            "--client-secret",
            "unused",
            "--password",
            "unused",
        ]);
        assert_eq!(
            ignored_options(&args, AuthModeChoice::AuthorizationCode),
            ["--client-secret", "--password"]
        );
    }

    #[test]
    fn name_is_positional() {
        let ProfileCommands::Use { name } = parse(&["profile", "use", "staging"]).unwrap() else {
            panic!("expected use");
        };
        assert_eq!(name.as_deref(), Some("staging"));

        let ProfileCommands::Delete { name, yes } =
            parse(&["profile", "delete", "staging", "-y"]).unwrap()
        else {
            panic!("expected delete");
        };
        assert_eq!(name.as_deref(), Some("staging"));
        assert!(yes);
    }

    #[test]
    fn durations_read_naturally() {
        assert_eq!(format_duration(30), "under a minute");
        assert_eq!(format_duration(42 * 60), "42 min");
        assert_eq!(format_duration(2 * 3600 + 5 * 60), "2 h 5 min");
    }
}
