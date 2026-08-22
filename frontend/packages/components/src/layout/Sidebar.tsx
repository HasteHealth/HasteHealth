import {
  ChevronLeftIcon,
  ChevronRightIcon,
} from "@heroicons/react/24/outline";
import classNames from "classnames";
import React from "react";

const SidebarOpenContext = React.createContext(true);

export interface SideBarItemProps extends React.DetailedHTMLProps<
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
          "cursor-pointer flex items-center p-1 px-2 group rounded-lg group",
          isOpen ? undefined : "justify-center",
          {
            "text-slate-800 hover:bg-slate-200": !active,
            "text-brand-900 bg-brand-200": active,
          },
        )}
      >
        {logo && (
          <div
            className={classNames(
              "flex-none w-5 h-5 transition duration-75",
              { "mr-3": isOpen },
            )}
          >
            {logo}
          </div>
        )}
        {isOpen && (
          <span className="flex-1 whitespace-nowrap">{children}</span>
        )}
      </div>
    </li>
  );
}

export interface SideBarItemGroupProps extends React.DetailedHTMLProps<
  React.LiHTMLAttributes<HTMLLIElement>,
  HTMLLIElement
> {
  label?: string;
}

export function SideBarItemGroup({
  label,
  children,
  ...props
}: SideBarItemGroupProps) {
  const isOpen = React.useContext(SidebarOpenContext);

  return (
    <li {...props}>
      {isOpen ? (
        <div className="px-2 text-brand-900 text-xs underline">{label}</div>
      ) : (
        <div className="mx-2 border-t border-slate-200" />
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
          "relative flex h-full shrink-0 flex-col border-r bg-white transition-[width] duration-200",
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
            className="absolute top-1/2 -right-3 z-10 flex h-6 w-6 -translate-y-1/2 items-center justify-center rounded-full border border-slate-200 bg-white text-slate-500 shadow-sm hover:text-slate-700"
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
    <div className="flex h-screen w-full">
      {sidebar}
      <div className="flex min-w-0 flex-1 flex-col">
        {navbar}
        <div className="flex-1 overflow-x-auto overflow-y-hidden">
          {children}
        </div>
      </div>
    </div>
  );
};
