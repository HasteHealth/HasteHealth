import { Menu } from "@headlessui/react";
import classNames from "classnames";
import React, { DetailedHTMLProps, HTMLAttributes } from "react";

interface Link
  extends DetailedHTMLProps<
    HTMLAttributes<HTMLAnchorElement>,
    HTMLAnchorElement
  > {
  label: React.ReactNode;
}

export interface DropdownMenuProps {
  links: Link[];
  children: React.ReactNode;
}

function DropDownMenu({ links, children }: DropdownMenuProps) {
  return (
    <Menu>
      <div>
        <Menu.Button as="div">{children}</Menu.Button>
        <div className="relative">
          <Menu.Items className="absolute right-0 z-10 mt-2 w-48 origin-top-right rounded-none border border-[var(--ci-line)] bg-[var(--ci-surface)] py-1 font-mono text-xs shadow-2xl focus:outline-none">
            {links.map((link) => (
              <Menu.Item key={link.key}>
                {({ active }) => (
                  <a
                    {...link}
                    className={classNames(
                      "cursor-pointer block px-4 py-2 text-xs font-mono transition-colors",
                      active
                        ? "bg-[var(--ci-signal)] text-white"
                        : "text-[var(--ci-ink)] hover:bg-[var(--ci-surface-raised)]",
                      link.className,
                    )}
                  >
                    {link.label}
                  </a>
                )}
              </Menu.Item>
            ))}
          </Menu.Items>
        </div>
      </div>
    </Menu>
  );
}

export { Menu, DropDownMenu };
