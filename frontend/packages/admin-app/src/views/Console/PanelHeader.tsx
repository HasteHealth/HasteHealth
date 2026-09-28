/**
 * The header every console panel wears.
 *
 * A type, an instance and a history listing are all "here is what you are
 * looking at, here is what you can do to it", so they share one shape: the
 * title on the left at one size, badges beside it, actions pushed right, and
 * an optional line of prose underneath. Panels differ in what they put in
 * those slots, never in how the slots are laid out.
 */
import classNames from "classnames";
import React from "react";

export interface PanelHeaderProps {
  /** The heading, e.g. `Patient` or `Patient/123`. */
  title: React.ReactNode;
  /** Badges beside the title: status, version, read only. */
  badges?: React.ReactNode;
  /** Buttons, pushed to the right. */
  actions?: React.ReactNode;
  /** One line under the title. The row is reserved even when empty, so the
   *  content below does not jump once it loads. */
  description?: React.ReactNode;
}

export function PanelHeader({
  title,
  badges,
  actions,
  description,
}: Readonly<PanelHeaderProps>) {
  return (
    <header className="mb-3 border-b border-slate-200 pb-3">
      <div className="flex flex-wrap items-center gap-2">
        <h1 className="text-lg font-semibold leading-7 text-slate-900">
          {title}
        </h1>
        {badges}
        {actions && (
          <div className="ml-auto flex shrink-0 gap-2">{actions}</div>
        )}
      </div>
      {description !== undefined && (
        <p className="mt-1 min-h-5 max-w-3xl text-sm text-slate-500">
          {description}
        </p>
      )}
    </header>
  );
}

/** A small label for a piece of metadata, beside a panel's title. */
export function Badge({
  children,
  tone = "slate",
}: Readonly<{ children: React.ReactNode; tone?: "slate" | "amber" }>) {
  return (
    <span
      className={classNames(
        "rounded px-1.5 py-0.5 text-xs",
        tone === "amber"
          ? "bg-amber-100 text-amber-800"
          : "bg-slate-100 text-slate-600",
      )}
    >
      {children}
    </span>
  );
}
