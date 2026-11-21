//! The error type of the Haste Health Rust stack: a FHIR `OperationOutcome`
//! for the client, plus what a log needs to trace the failure to its origin.

use std::{
    backtrace::Backtrace,
    borrow::Cow,
    error::Error,
    fmt::{self, Debug, Display},
    panic::Location,
    sync::atomic::{AtomicBool, Ordering},
};

use haste_fhir_model::r4::generated::{
    resources::{OperationOutcome, OperationOutcomeIssue},
    terminology::{BoundCode, IssueSeverity, IssueType},
    types::FHIRString,
};

#[cfg(feature = "derive")]
pub mod derive;

#[cfg(feature = "axum")]
pub mod axum;

/// The error an [`OperationOutcomeError`] wraps.
pub type Source = Box<dyn Error + Send + Sync + 'static>;

/// An HTTP status that takes precedence over the one derived from the first issue's code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StatusOverride {
    /// 422: the request is well formed but cannot be processed as it is.
    UnprocessableEntity,
}

/// One [`OperationOutcomeError::context`] call: what the caller was doing and
/// where it said so.
pub struct Hop {
    message: Cow<'static, str>,
    location: &'static Location<'static>,
}

impl Hop {
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }

    #[must_use]
    pub fn location(&self) -> &'static Location<'static> {
        self.location
    }
}

impl Debug for Hop {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?} at {}", self.message, self.location)
    }
}

struct Inner {
    outcome: OperationOutcome,
    source: Option<Source>,
    status_override: Option<StatusOverride>,
    /// Where the error was built, or where a `?` turned a foreign error into one.
    origin: &'static Location<'static>,
    /// `context` calls, innermost first. Empty until the first call, so it
    /// costs nothing to carry.
    trail: Vec<Hop>,
    /// Captured for server errors once [`OperationOutcomeError::capture_backtraces`]
    /// has been turned on.
    backtrace: Option<Backtrace>,
}

/// A failure as a FHIR `OperationOutcome`, with the HTTP status it answers
/// and a record of where it came from.
///
/// Boxed so that a `Result<T, OperationOutcomeError>` is no larger than `T`
/// needs: the Ok path pays nothing for the error's bookkeeping.
pub struct OperationOutcomeError(Box<Inner>);

static CAPTURE_BACKTRACES: AtomicBool = AtomicBool::new(false);

fn create_operation_outcome(
    severity: BoundCode<IssueSeverity>,
    code: BoundCode<IssueType>,
    diagnostic: String,
) -> OperationOutcome {
    OperationOutcome {
        issue: vec![OperationOutcomeIssue {
            severity,
            code,
            diagnostics: Some(Box::new(FHIRString {
                value: Some(diagnostic),
                ..Default::default()
            })),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// The HTTP status an issue code answers with. 5xx marks a failure of the
/// server rather than of the request.
fn status_for_code(code: &BoundCode<IssueType>) -> u16 {
    match code.as_str() {
        // `invalid` and its children in the IssueType hierarchy.
        Some("invalid" | "structure" | "required" | "value" | "invariant") => 400,
        // The caller is not (or no longer) authenticated.
        Some("login" | "unknown" | "expired") => 401,
        Some("forbidden" | "security" | "suppressed") => 403,
        Some("not-found") => 404,
        // An operation that affects state invoked with GET.
        Some("business-rule") => 405,
        Some("conflict") => 409,
        Some("throttled") => 429,
        Some("not-supported") => 501,
        _ => 500,
    }
}

impl OperationOutcomeError {
    /// Capture a backtrace for every error that answers 5xx from now on.
    /// Off by default: a backtrace costs microseconds to capture and a
    /// rejected request does not need one.
    pub fn capture_backtraces(enabled: bool) {
        CAPTURE_BACKTRACES.store(enabled, Ordering::Relaxed);
    }

    fn build(
        source: Option<Source>,
        outcome: OperationOutcome,
        status_override: Option<StatusOverride>,
        origin: &'static Location<'static>,
    ) -> Self {
        let mut inner = Box::new(Inner {
            outcome,
            source,
            status_override,
            origin,
            trail: Vec::new(),
            backtrace: None,
        });
        if CAPTURE_BACKTRACES.load(Ordering::Relaxed) && inner.http_status() >= 500 {
            inner.backtrace = Some(Backtrace::force_capture());
        }
        OperationOutcomeError(inner)
    }

    #[must_use]
    #[track_caller]
    pub fn new(source: Option<Source>, outcome: OperationOutcome) -> Self {
        Self::build(source, outcome, None, Location::caller())
    }

    /// An error that answers 422 Unprocessable Entity whatever its issue codes:
    /// the request is well formed but cannot be processed as it is, such as a
    /// resource that violates an applicable profile.
    #[must_use]
    #[track_caller]
    pub fn unprocessable(outcome: OperationOutcome) -> Self {
        Self::build(
            None,
            outcome,
            Some(StatusOverride::UnprocessableEntity),
            Location::caller(),
        )
    }

    #[must_use]
    #[track_caller]
    pub fn fatal(code: BoundCode<IssueType>, diagnostic: String) -> Self {
        Self::build(
            None,
            create_operation_outcome(IssueSeverity::fatal(), code, diagnostic),
            None,
            Location::caller(),
        )
    }

    #[must_use]
    #[track_caller]
    pub fn error(code: BoundCode<IssueType>, diagnostic: String) -> Self {
        Self::build(
            None,
            create_operation_outcome(IssueSeverity::error(), code, diagnostic),
            None,
            Location::caller(),
        )
    }

    #[must_use]
    #[track_caller]
    pub fn warning(code: BoundCode<IssueType>, diagnostic: String) -> Self {
        Self::build(
            None,
            create_operation_outcome(IssueSeverity::warning(), code, diagnostic),
            None,
            Location::caller(),
        )
    }

    #[must_use]
    #[track_caller]
    pub fn information(code: BoundCode<IssueType>, diagnostic: String) -> Self {
        Self::build(
            None,
            create_operation_outcome(IssueSeverity::information(), code, diagnostic),
            None,
            Location::caller(),
        )
    }

    /// Attach the error this one wraps. Prefer this over formatting the cause
    /// into the diagnostic: the client keeps the message you wrote and the
    /// log gets the whole chain.
    #[must_use]
    pub fn with_source(mut self, source: impl Error + Send + Sync + 'static) -> Self {
        self.0.source = Some(Box::new(source));
        self
    }

    /// Record what the caller was doing when the error passed through. Shows
    /// up in the log next to the origin; the client never sees it.
    #[must_use]
    #[track_caller]
    pub fn context(mut self, message: impl Into<Cow<'static, str>>) -> Self {
        self.0.trail.push(Hop {
            message: message.into(),
            location: Location::caller(),
        });
        self
    }

    #[must_use]
    pub fn outcome(&self) -> &OperationOutcome {
        &self.0.outcome
    }

    pub fn push_issue(&mut self, issue: OperationOutcomeIssue) {
        self.0.outcome.issue.push(issue);
    }

    /// Where the error was built, or where a `?` turned a foreign error into one.
    #[must_use]
    pub fn origin(&self) -> &'static Location<'static> {
        self.0.origin
    }

    /// The [`context`](Self::context) calls the error passed through, innermost first.
    #[must_use]
    pub fn trail(&self) -> &[Hop] {
        &self.0.trail
    }

    #[must_use]
    pub fn backtrace(&self) -> Option<&Backtrace> {
        self.0.backtrace.as_ref()
    }

    /// The attached source error and everything it in turn was caused by.
    pub fn causes(&self) -> impl Iterator<Item = &(dyn Error + 'static)> {
        std::iter::successors(self.source(), |&error| error.source())
    }

    /// The HTTP status the error answers with: an override if set, otherwise
    /// the status the first issue's code maps to.
    #[must_use]
    pub fn http_status(&self) -> u16 {
        self.0.http_status()
    }

    /// Whether the status is 5xx: the server failed, not the request.
    #[must_use]
    pub fn is_server_error(&self) -> bool {
        self.http_status() >= 500
    }
}

impl Inner {
    fn http_status(&self) -> u16 {
        match self.status_override {
            Some(StatusOverride::UnprocessableEntity) => 422,
            None => self
                .outcome
                .issue
                .first()
                .map_or(500, |issue| status_for_code(&issue.code)),
        }
    }
}

/// [`OperationOutcomeError::context`] on a `Result`, applied only on the error path.
pub trait ResultExt {
    #[must_use]
    fn context(self, message: impl Into<Cow<'static, str>>) -> Self;
}

impl<T> ResultExt for Result<T, OperationOutcomeError> {
    #[track_caller]
    fn context(self, message: impl Into<Cow<'static, str>>) -> Self {
        match self {
            Ok(value) => Ok(value),
            Err(error) => Err(error.context(message)),
        }
    }
}

fn get_issue_diagnostics(issue: &OperationOutcomeIssue) -> Option<&str> {
    issue.diagnostics.as_ref().and_then(|d| d.value.as_deref())
}

impl Display for OperationOutcomeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Operation Error: '{}'",
            self.0
                .outcome
                .issue
                .iter()
                .filter_map(get_issue_diagnostics)
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

/// `severity code: diagnostics`, instead of the issue's many empty fields.
struct IssueSummary<'a>(&'a OperationOutcomeIssue);

impl Debug for IssueSummary<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {}: {:?}",
            self.0.severity.as_str().unwrap_or("?"),
            self.0.code.as_str().unwrap_or("?"),
            get_issue_diagnostics(self.0).unwrap_or("")
        )
    }
}

struct Issues<'a>(&'a [OperationOutcomeIssue]);

impl Debug for Issues<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list()
            .entries(self.0.iter().map(IssueSummary))
            .finish()
    }
}

struct Cause<'a>(&'a (dyn Error + 'static));

impl Debug for Cause<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt(self.0, f)
    }
}

/// The source chain, outermost first.
struct Causes<'a>(&'a OperationOutcomeError);

impl Debug for Causes<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.0.causes().map(Cause)).finish()
    }
}

impl Debug for OperationOutcomeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let inner = &self.0;
        let mut s = f.debug_struct("OperationOutcomeError");
        s.field("status", &inner.http_status())
            .field("issues", &Issues(&inner.outcome.issue))
            .field("origin", &format_args!("{}", inner.origin));
        if !inner.trail.is_empty() {
            s.field("trail", &inner.trail);
        }
        if inner.source.is_some() {
            s.field("causes", &Causes(self));
        }
        if let Some(backtrace) = &inner.backtrace {
            s.field("backtrace", &format_args!("{backtrace}"));
        }
        s.finish()
    }
}

impl Error for OperationOutcomeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.0
            .source
            .as_deref()
            .map(|source| source as &(dyn Error + 'static))
    }
}
