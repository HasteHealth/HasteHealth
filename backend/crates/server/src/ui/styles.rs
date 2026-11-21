//! Tailwind classes shared by the server-rendered pages, so that every page
//! draws its cards, forms, buttons and lists the same way. The colours are the
//! roles in `css/app.css`.

/// The panel under the banner that holds a page's content.
pub const CARD: &str =
    "w-full rounded-lg border border-primary-soft bg-surface text-ink-body shadow";
/// The padding and spacing inside a [`CARD`].
pub const CARD_BODY: &str = "p-6 space-y-4 md:space-y-6 sm:p-8";

pub const HEADING: &str = "text-xl font-bold leading-tight tracking-tight text-ink md:text-2xl";
/// A sentence or two under a [`HEADING`].
pub const TEXT: &str = "text-sm text-ink-muted";
/// An email, workspace or app name inside a [`TEXT`].
pub const EMPHASIS: &str = "font-semibold text-ink-body";

pub const LABEL: &str = "block mb-2 text-sm font-medium text-ink-body";
pub const INPUT: &str = "block w-full p-2.5 rounded-lg border border-line-strong bg-surface-muted text-ink sm:text-sm focus:border-primary focus:ring-primary";

pub const BUTTON: &str = "block w-full cursor-pointer rounded-lg bg-primary px-5 py-2.5 text-center text-sm font-medium text-on-primary hover:bg-primary-hover focus:outline-none focus:ring-4 focus:ring-primary-ring";
/// The alternative to a [`BUTTON`]: decline, deny, another way to sign in.
pub const BUTTON_SECONDARY: &str = "block w-full cursor-pointer rounded-lg border border-line-strong bg-surface px-5 py-2.5 text-center text-sm font-medium text-ink-body hover:bg-surface-muted focus:outline-none focus:ring-4 focus:ring-line";
/// Takes its size from the surrounding text.
pub const LINK: &str = "font-medium text-primary hover:underline";

/// One entry in a list to pick from: a workspace, a project, an MFA method.
pub const CHOICE: &str = "block w-full cursor-pointer rounded-lg border border-line bg-surface px-4 py-2.5 text-center text-sm font-medium text-ink transition-colors hover:border-primary-line hover:bg-primary-soft";
/// One entry in a list with its own actions: an invitation, an MFA method.
pub const ROW: &str =
    "flex items-center justify-between gap-3 rounded-lg border border-line bg-surface px-4 py-2.5";
pub const ROW_BUTTON: &str = "cursor-pointer rounded-lg bg-primary px-3 py-1.5 text-sm font-medium text-on-primary transition-colors hover:bg-primary-hover";
pub const ROW_BUTTON_SECONDARY: &str = "cursor-pointer rounded-lg border border-line-strong bg-surface px-3 py-1.5 text-sm font-medium text-ink-body transition-colors hover:bg-surface-muted";
pub const ROW_BUTTON_DANGER: &str = "cursor-pointer rounded-lg border border-error-line bg-surface px-3 py-1.5 text-sm font-medium text-error transition-colors hover:bg-error-soft";
