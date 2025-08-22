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
            "text-slate-800 hover:bg-gray-200": !active,
            "text-teal-800 bg-teal-200 ": active,
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
      <div className="px-2 text-teal-800 text-xs">{props.label}</div>
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
        "flex fixed top-0 left-0 z-40 w-64 h-screen transition-transform",
        { "translate-x-0": isOpen, "-translate-x-full": !isOpen }
      )}
      aria-label="Sidebar"
    >
      <nav className="flex flex-1 flex-col py-2 overflow-y-auto bg-gray-100">
        <div className="px-3 py-2">{top}</div>
        <ul
          role="list"
          className="px-3 flex flex-1 flex-col text-base overflow-y-auto gap-y-6"
        >
          {children}
        </ul>
      </nav>
    </aside>
  );
}

export const SidebarLayout = ({
  children,
  sidebar,
}: {
  children: React.ReactNode;
  sidebar: React.ReactNode;
}) => {
  // const [sidebarOpen, setSidebarOpen] = useState(true);
  return (
    <>
      {sidebar}
      <div
        style={{ width: "calc(100% - 16rem)" }}
        className="sm:ml-64 flex flex-col flex-grow"
      >
        {children}
      </div>
    </>
  );
};
