import { type ReactNode } from "react";
import clsx from "clsx";
import Link from "@docusaurus/Link";
import Heading from "@theme/Heading";

import { Icon, type IconName } from "./icons";
import { riseDelay } from "./motion";
import styles from "./styles.module.css";

/*
 * The building blocks the marketing pages (home, pricing, contact) are made
 * of, so that a button or a section heading is the same thing on all three.
 * They assume the Tailwind reset, so they belong inside a page's
 * `<main id="tw-scope">`.
 */

/** Which surface a control sits on, so it can pick legible colours. */
export type Surface = "light" | "dark";

export function Container({
  children,
  className,
}: Readonly<{ children: ReactNode; className?: string }>) {
  return (
    <div
      className={clsx("mx-auto w-full max-w-[84rem] px-5 sm:px-8", className)}
    >
      {children}
    </div>
  );
}

const BUTTON =
  "inline-flex items-center justify-center gap-2 whitespace-nowrap rounded-full font-semibold no-underline transition-colors hover:no-underline focus-visible:outline-2 focus-visible:outline-offset-2";

const BUTTON_SIZES = {
  md: "px-6 py-3 text-[1.0625rem]",
  sm: "px-5 py-2.5 text-[0.9375rem]",
};

const BUTTON_STYLES: Record<
  "primary" | "secondary",
  Record<Surface, string>
> = {
  primary: {
    dark: "bg-white text-ink-950 hover:bg-brand-100 hover:text-ink-950 focus-visible:outline-white",
    light:
      "bg-brand-800 text-white hover:bg-brand-900 hover:text-white focus-visible:outline-brand-800",
  },
  secondary: {
    dark: "border border-white/25 text-white hover:border-white/50 hover:bg-white/10 hover:text-white focus-visible:outline-white",
    light:
      "border border-slate-300 bg-white text-ink-900 hover:border-slate-400 hover:bg-slate-50 hover:text-ink-900 focus-visible:outline-brand-800",
  },
};

export function Button({
  to,
  children,
  variant = "primary",
  on = "light",
  size = "md",
  className,
}: Readonly<{
  to: string;
  children: ReactNode;
  variant?: "primary" | "secondary";
  on?: Surface;
  size?: keyof typeof BUTTON_SIZES;
  className?: string;
}>) {
  return (
    <Link
      to={to}
      className={clsx(
        BUTTON,
        BUTTON_SIZES[size],
        BUTTON_STYLES[variant][on],
        className,
      )}
    >
      {children}
    </Link>
  );
}

/**
 * Docusaurus opens every off-site link in a new tab. A mail link opens the
 * mail client, so it should not open a blank tab as well.
 */
function mailTarget(to: string): { target?: "_self" } {
  return to.startsWith("mailto:") ? { target: "_self" } : {};
}

/** A quiet text link with an arrow that leans forward on hover. */
export function ArrowLink({
  to,
  children,
  on = "light",
}: Readonly<{ to: string; children: ReactNode; on?: Surface }>) {
  return (
    <Link
      to={to}
      {...mailTarget(to)}
      className={clsx(
        "inline-flex items-center gap-1.5 text-[1.0625rem] font-semibold no-underline hover:no-underline",
        on === "dark"
          ? "text-brand-300 hover:text-brand-200"
          : "text-brand-800 hover:text-brand-900",
      )}
    >
      {children}
      <Icon name="arrowRight" className={clsx("h-4 w-4", styles.arrow)} />
    </Link>
  );
}

/** The small capitals above a heading. */
export function Eyebrow({
  children,
  on = "light",
  className,
}: Readonly<{ children: ReactNode; on?: Surface; className?: string }>) {
  return (
    <p
      className={clsx(
        "text-[0.8125rem] font-semibold uppercase tracking-[0.16em]",
        on === "dark" ? "text-brand-300" : "text-brand-800",
        className,
      )}
    >
      {children}
    </p>
  );
}

export function SectionHeader({
  eyebrow,
  title,
  children,
  on = "light",
  centered = false,
}: Readonly<{
  eyebrow?: string;
  title: string;
  children?: ReactNode;
  on?: Surface;
  centered?: boolean;
}>) {
  return (
    <div
      className={clsx("max-w-[52rem]", centered && "mx-auto text-center")}
      data-reveal=""
    >
      {eyebrow ? <Eyebrow on={on}>{eyebrow}</Eyebrow> : null}
      <Heading
        as="h2"
        className={clsx(
          "text-[2rem] font-semibold leading-[1.12] tracking-[-0.025em] text-balance md:text-[2.75rem]",
          eyebrow && "mt-4",
          on === "dark" ? "text-white" : "text-ink-950",
        )}
      >
        {title}
      </Heading>
      {children ? (
        <p
          className={clsx(
            "mt-5 text-lg leading-relaxed md:text-xl",
            on === "dark" ? "text-slate-300" : "text-slate-600",
          )}
        >
          {children}
        </p>
      ) : null}
    </div>
  );
}

/**
 * An icon in a tinted tile, the lead-in to a feature. Pass `name` for one of
 * the line icons, or a glyph of your own as the child.
 */
export function IconTile({
  name,
  children,
  on = "light",
}: Readonly<{ name?: IconName; children?: ReactNode; on?: Surface }>) {
  return (
    <span
      className={clsx(
        "flex h-12 w-12 shrink-0 items-center justify-center rounded-xl",
        on === "dark"
          ? "bg-white/[0.06] text-brand-300 ring-1 ring-white/10"
          : "bg-brand-50 text-brand-800 ring-1 ring-brand-200/70",
      )}
    >
      {name ? (
        <Icon name={name} className="h-[1.375rem] w-[1.375rem]" />
      ) : (
        children
      )}
    </span>
  );
}

/** A card that is one link: a tile, a claim, and where it leads. */
export function LinkCard({
  to,
  icon,
  title,
  children,
  label,
}: Readonly<{
  to: string;
  icon: ReactNode;
  title: string;
  children: ReactNode;
  label: string;
}>) {
  return (
    <Link
      to={to}
      {...mailTarget(to)}
      className={clsx(
        styles.card,
        "flex h-full flex-col rounded-2xl border border-slate-200 bg-white p-6 no-underline hover:no-underline md:p-7",
      )}
    >
      {icon}
      <h3 className="mt-5 text-xl font-semibold tracking-tight text-ink-950">
        {title}
      </h3>
      <p className="mt-2 flex-1 text-base leading-relaxed text-slate-600">
        {children}
      </p>
      <span className="mt-5 inline-flex items-center gap-1.5 text-base font-semibold text-brand-800">
        {label}
        <Icon name="arrowRight" className={clsx("h-4 w-4", styles.arrow)} />
      </span>
    </Link>
  );
}

/** The pill above a hero's headline. */
export function HeroBadge({
  children,
  className,
}: Readonly<{ children: ReactNode; className?: string }>) {
  return (
    <p
      className={clsx(
        "inline-flex items-center gap-2.5 rounded-full border border-white/15 bg-white/[0.06] px-4 py-1.5 text-[0.8125rem] font-medium tracking-wide text-brand-100",
        className,
      )}
    >
      <span
        className="h-1.5 w-1.5 rounded-full bg-brand-400"
        aria-hidden="true"
      />
      {children}
    </p>
  );
}

/**
 * The dark band that opens an interior page. The homepage's hero is its own
 * markup (src/pages/index.tsx), but it is drawn from the same backdrop, badge
 * and load-in, one size up.
 */
export function PageHero({
  badge,
  title,
  children,
  actions,
  overlap = false,
}: Readonly<{
  badge?: string;
  title: string;
  children?: ReactNode;
  actions?: ReactNode;
  /** Leave room at the bottom for the next section to ride up over the hero. */
  overlap?: boolean;
}>) {
  return (
    <section className="relative isolate text-white">
      <div className={clsx(styles.heroBackdrop, "-z-10")} aria-hidden="true">
        <div className={styles.grid} />
      </div>
      <Container
        className={clsx(
          "flex flex-col items-center pt-14 text-center md:pt-20",
          overlap ? "pb-40 md:pb-48" : "pb-16 md:pb-24",
        )}
      >
        {badge ? <HeroBadge className={styles.rise}>{badge}</HeroBadge> : null}
        <Heading
          as="h1"
          className={clsx(
            styles.rise,
            "text-[2.375rem] font-semibold leading-[1.05] tracking-[-0.035em] text-white text-balance md:text-5xl lg:text-6xl",
            badge && "mt-7",
          )}
          style={riseDelay(60)}
        >
          {title}
        </Heading>
        {children ? (
          <p
            className={clsx(
              styles.rise,
              "mt-6 max-w-[46rem] text-lg leading-relaxed text-slate-300 md:text-xl",
            )}
            style={riseDelay(140)}
          >
            {children}
          </p>
        ) : null}
        {actions ? (
          <div
            className={clsx(
              styles.rise,
              "mt-9 flex w-full flex-col gap-3 sm:w-auto sm:flex-row",
            )}
            style={riseDelay(220)}
          >
            {actions}
          </div>
        ) : null}
      </Container>
    </section>
  );
}
