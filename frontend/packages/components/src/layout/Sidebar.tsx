import classNames from "classnames";
import React from "react";

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
  return (
    <li {...props}>
      <div
        className={classNames(
          "cursor-pointer flex items-center p-1 px-2 group rounded-lg group",
          {
            "text-slate-800 hover:bg-slate-200": !active,
            "text-orange-900 bg-orange-200 ": active,
          }
        )}
      >
        {logo && (
          <div className="flex-none w-5 h-5 mr-3 transition duration-75">
            {logo}
          </div>
        )}
        <span className="flex-1 whitespace-nowrap">{children}</span>
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

export function SideBarItemGroup(props: SideBarItemGroupProps) {
  return (
    <li {...props}>
      <div className="px-2 text-orange-900 text-xs underline">
        {props.label}
      </div>
      <div className="mt-1 ml-1">
        <ul className="space-y-1">{props.children}</ul>
      </div>
    </li>
  );
}

export function SideBar({
  top,
  isOpen = true,
  children,
}: {
  isOpen?: boolean;
  top?: React.ReactNode;
  children: React.ReactNode;
}) {
  return (
    <aside
      id="sidebar-multi-level-sidebar"
      className={classNames(
        "flex relative top-0 left-0 z-1 transition-transform ",
        { "translate-x-0": isOpen, "-translate-x-full": !isOpen }
      )}
      style={{ width: "250px" }}
      aria-label="Sidebar"
    >
      <nav className="flex flex-1 flex-col py-2 ">
        <div className="px-3 py-2">{top}</div>
        <ul className="px-3 flex flex-1 flex-col text-sm gap-y-6">
          {children}
        </ul>
      </nav>
    </aside>
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
  // const [sidebarOpen, setSidebarOpen] = useState(true);
  return (
    <div className="w-full">
      {navbar}
      <div className="flex">
        <div
          style={{ height: "calc(100vh - 65px)" }}
          className="overflow-y-auto overflow-x-hidden border-r"
        >
          {sidebar}
        </div>
        <div
          className="flex flex-col overflow-x-auto overflow-y-hidden"
          style={{ height: "calc(100vh - 65px)", width: "calc(100% - 250px)" }}
        >
          {children}
        </div>
      </div>
    </div>
  );
};
