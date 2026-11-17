import { ChevronLeftIcon, ChevronRightIcon } from "@heroicons/react/24/outline";
import classNames from "classnames";
import React from "react";

const SidebarOpenContext = React.createContext(true);

export interface SideBarItemProps
  extends React.DetailedHTMLProps<
    React.LiHTMLAttributes<HTMLLIElement>,
    HTMLLIElement
  > {
  active?: boolean;
  logo?: React.ReactNode;
  children: React.ReactNode;
}

export function SideBarItem({
  active,
  logo,
  children,
  ...props
}: SideBarItemProps) {
  const isOpen = React.useContext(SidebarOpenContext);

  return (
    <li
      title={!isOpen && typeof children === "string" ? children : undefined}
      {...props}
    >
      <div
        className={classNames(
          "cursor-pointer flex items-center px-3 py-2 group transition-colors text-sm",
          isOpen ? undefined : "justify-center",
          {
            "text-[var(--ci-ink-muted)] hover:text-[var(--ci-ink)] hover:bg-[var(--ci-surface-sunken)] border-l-2 border-transparent":
              !active,
            "text-[var(--ci-signal-ink)] bg-[var(--ci-signal-soft)] border-l-2 border-[var(--ci-signal)] font-medium":
              active,
          },
        )}
      >
        {logo && (
          <div
            className={classNames("flex-none w-5 h-5 transition duration-75", {
              "mr-3": isOpen,
            })}
          >
            {logo}
          </div>
        )}
        {isOpen && <span className="flex-1 whitespace-nowrap">{children}</span>}
      </div>
    </li>
  );
}

export interface SideBarItemGroupProps
  extends React.DetailedHTMLProps<
    React.LiHTMLAttributes<HTMLLIElement>,
    HTMLLIElement
  > {
  label?: string;
}

export function SideBarItemGroup({
  label,
  children,
  ...props
}: Readonly<SideBarItemGroupProps>) {
  const isOpen = React.useContext(SidebarOpenContext);

  return (
    <li {...props}>
      {isOpen ? (
        <div className="px-3 mb-1 text-[10px] font-mono font-medium uppercase tracking-wider text-[var(--ci-ink-subtle)]">
          {label}
        </div>
      ) : (
        <div className="mx-2 border-t border-[var(--ci-line)]" />
      )}
      <div className={classNames("mt-1", { "ml-1": isOpen })}>
        <ul className="space-y-1">{children}</ul>
      </div>
    </li>
  );
}

export function SideBar({
  top,
  isOpen = true,
  onToggle,
  children,
}: {
  isOpen?: boolean;
  onToggle?: () => void;
  top?: React.ReactNode;
  children?: React.ReactNode;
}) {
  return (
    <SidebarOpenContext.Provider value={isOpen}>
      <aside
        id="sidebar-multi-level-sidebar"
        className={classNames(
          "relative flex h-full shrink-0 flex-col border-r border-[var(--ci-line)] bg-[var(--ci-surface)] transition-[width] duration-200",
          isOpen ? "w-[260px]" : "w-[76px]",
        )}
        aria-label="Sidebar"
      >
        <nav className="flex flex-1 flex-col overflow-y-auto overflow-x-hidden py-2">
          <div className="px-3 py-2">{top}</div>
          <ul className="px-3 flex flex-1 flex-col text-sm gap-y-6">
            {children}
          </ul>
        </nav>
        {onToggle && (
          <button
            type="button"
            aria-label={isOpen ? "Collapse sidebar" : "Expand sidebar"}
            onClick={onToggle}
            className="absolute top-1/2 -right-3 z-10 flex h-6 w-6 -translate-y-1/2 items-center justify-center rounded-none border border-[var(--ci-line)] bg-[var(--ci-surface-raised)] text-[var(--ci-ink-muted)] shadow-sm hover:text-[var(--ci-ink)] hover:border-[var(--ci-signal)]"
          >
            {isOpen ? (
              <ChevronLeftIcon className="h-4 w-4" />
            ) : (
              <ChevronRightIcon className="h-4 w-4" />
            )}
          </button>
        )}
      </aside>
    </SidebarOpenContext.Provider>
  );
}

export const SidebarLayout = ({
  children,
  sidebar,
  navbar,
}: {
  navbar: React.ReactNode;
  children: React.ReactNode;
  sidebar: React.ReactNode;
}) => {
  return (
    <div className="flex h-screen w-full bg-[var(--ci-bg)] text-[var(--ci-ink)]">
      {sidebar}
      <div className="flex min-w-0 flex-1 flex-col">
        {navbar}
        <div className="flex-1 overflow-x-auto overflow-y-hidden bg-[var(--ci-bg)]">
          {children}
        </div>
      </div>
    </div>
  );
};
