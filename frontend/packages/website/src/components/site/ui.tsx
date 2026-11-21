import { type ReactNode } from "react";
import clsx from "clsx";
import Link from "@docusaurus/Link";
import Heading from "@theme/Heading";

import { Icon, type IconName } from "./icons";
import styles from "./styles.module.css";

/*
 * The building blocks the marketing pages (home, pricing, contact) are made
 * of, so that a button or a section heading is the same thing on all three.
 * They assume the Tailwind reset, so they belong inside a page's
 * `<main id="tw-scope">`.
 *
 * Every surface is white. The brand teal is a highlight: the primary button,
 * links, eyebrows and icons wear it, and nothing else does.
 */

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
  "inline-flex items-center justify-center gap-2 whitespace-nowrap rounded-full font-semibold no-underline transition-colors hover:no-underline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-brand-800";

const BUTTON_SIZES = {
  md: "px-6 py-3 text-[1.0625rem]",
  sm: "px-5 py-2.5 text-[0.9375rem]",
};

const BUTTON_STYLES = {
  primary: "bg-brand-800 text-white hover:bg-brand-900 hover:text-white",
  secondary:
    "border border-slate-300 bg-white text-ink-900 hover:border-slate-400 hover:bg-slate-50 hover:text-ink-900",
};

export function Button({
  to,
  children,
  variant = "primary",
  size = "md",
  className,
}: Readonly<{
  to: string;
  children: ReactNode;
  variant?: keyof typeof BUTTON_STYLES;
  size?: keyof typeof BUTTON_SIZES;
  className?: string;
}>) {
  return (
    <Link
      to={to}
      className={clsx(
        BUTTON,
        BUTTON_SIZES[size],
        BUTTON_STYLES[variant],
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
}: Readonly<{ to: string; children: ReactNode }>) {
  return (
    <Link
      to={to}
      {...mailTarget(to)}
      className="inline-flex items-center gap-1.5 text-[1.0625rem] font-semibold text-brand-800 no-underline hover:text-brand-900 hover:no-underline"
    >
      {children}
      <Icon name="arrowRight" className={clsx("h-4 w-4", styles.arrow)} />
    </Link>
  );
}

/** The small capitals above a heading. */
export function Eyebrow({
  children,
  className,
}: Readonly<{ children: ReactNode; className?: string }>) {
  return (
    <p
      className={clsx(
        "m-0 text-[0.8125rem] font-semibold uppercase tracking-[0.16em] text-brand-800",
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
  centered = false,
}: Readonly<{
  eyebrow?: string;
  title: string;
  children?: ReactNode;
  centered?: boolean;
}>) {
  return (
    <div
      className={clsx("max-w-[52rem]", centered && "mx-auto text-center")}
      data-reveal=""
    >
      {eyebrow ? <Eyebrow>{eyebrow}</Eyebrow> : null}
      <Heading
        as="h2"
        className={clsx(
          "text-[2rem] font-semibold leading-[1.12] tracking-[-0.025em] text-ink-950 text-balance md:text-[2.75rem]",
          eyebrow && "mt-4",
        )}
      >
        {title}
      </Heading>
      {children ? (
        <p className="mt-5 text-lg leading-relaxed text-slate-600 md:text-xl">
          {children}
        </p>
      ) : null}
    </div>
  );
}

const ICON_TILE_SIZES = {
  md: { tile: "h-12 w-12 rounded-xl", icon: "h-[1.375rem] w-[1.375rem]" },
  sm: { tile: "h-9 w-9 rounded-lg", icon: "h-[1.125rem] w-[1.125rem]" },
};

/**
 * An icon in a tinted tile, the lead-in to a feature. Pass `name` for one of
 * the line icons, or a glyph of your own as the child.
 */
export function IconTile({
  name,
  children,
  size = "md",
}: Readonly<{
  name?: IconName;
  children?: ReactNode;
  size?: keyof typeof ICON_TILE_SIZES;
}>) {
  return (
    <span
      className={clsx(
        "flex shrink-0 items-center justify-center bg-brand-50 text-brand-700 ring-1 ring-brand-200/70",
        ICON_TILE_SIZES[size].tile,
      )}
    >
      {name ? (
        <Icon name={name} className={ICON_TILE_SIZES[size].icon} />
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

/**
 * The block that opens an interior page: the title, the pitch under it and
 * any actions under that, centred, with the page's content following at
 * once. No band and no backdrop.
 */
export function PageHero({
  title,
  children,
  actions,
}: Readonly<{
  title: string;
  children?: ReactNode;
  actions?: ReactNode;
}>) {
  return (
    <section>
      <Container
        className={clsx(
          styles.rise,
          "flex flex-col items-center pb-8 pt-10 text-center md:pb-10 md:pt-16",
        )}
      >
        <Heading
          as="h1"
          className="m-0 text-[2.5rem] font-semibold leading-[1.02] tracking-[-0.035em] text-ink-950 text-balance md:text-5xl xl:text-[3.25rem]"
        >
          {title}
        </Heading>
        {children ? (
          <p className="m-0 mt-4 max-w-[40rem] text-lg leading-relaxed text-slate-600 text-balance">
            {children}
          </p>
        ) : null}
        {actions ? (
          <div className="mt-6 flex flex-wrap items-center justify-center gap-2.5">
            {actions}
          </div>
        ) : null}
      </Container>
    </section>
  );
}
