import { Disclosure, Menu, Transition } from "@headlessui/react";
import {
  Bars3Icon,
  UserCircleIcon,
  XMarkIcon,
} from "@heroicons/react/24/outline";
import classNames from "classnames";
import React, { Fragment } from "react";

interface NavigationItem
  extends React.DetailedHTMLProps<
    React.AnchorHTMLAttributes<HTMLAnchorElement>,
    HTMLAnchorElement
  > {
  name: string;
}

export interface NavigationProps {
  userNavigation: NavigationItem[];
  navigation: NavigationItem[];
  active?: string;
  onNavigation?: (nav: NavigationItem) => void;
  brand?: {
    name: string;
    logo: { url: string };
  };
  user?: { imageUrl?: string; email?: string; name?: string };
}

interface ProfileDropdownProps {
  user?: { imageUrl?: string; email?: string; name?: string };
  children?: React.ReactNode;
}

export const ProfileDropdown = ({ user, children }: ProfileDropdownProps) => {
  return (
    <Menu as="div" className="relative ml-3">
      <div>
        <Menu.Button className="relative flex max-w-xs items-center rounded-none text-[var(--ci-muted)] hover:text-[var(--ci-signal)] transition-colors focus:outline-none">
          <span className="absolute -inset-1.5" />
          <span className="sr-only">Open user menu</span>
          {user?.imageUrl ? (
            <img className="h-8 w-8 rounded-none border border-[var(--ci-line)]" src={user?.imageUrl} alt="" />
          ) : (
            <UserCircleIcon
              className="h-8 w-8"
              aria-hidden="true"
            />
          )}
        </Menu.Button>
      </div>
      <Transition
        as={Fragment}
        enter="transition ease-out duration-100"
        enterFrom="transform opacity-0 scale-95"
        enterTo="transform opacity-100 scale-100"
        leave="transition ease-in duration-75"
        leaveFrom="transform opacity-100 scale-100"
        leaveTo="transform opacity-0 scale-95"
      >
        <div className="bg-[var(--ci-surface)] border border-[var(--ci-line)] text-[var(--ci-ink)] absolute right-0 z-10 mt-2 w-48 origin-top-right rounded-none py-1 shadow-2xl focus:outline-none">
          {children}
        </div>
      </Transition>
    </Menu>
  );
};

export const Navigation = ({
  onNavigation = () => {},
  navigation,
  active,
  userNavigation,
  user = {},
  brand = {
    name: "HasteHealth",
    logo: {
      url: "https://haste.health/img/logo.svg",
    },
  },
}: NavigationProps) => {
  return (
    <Disclosure as="nav" className=" border-b">
      {({ open }) => (
        <>
          <div className="mx-auto px-4 sm:px-6 lg:px-8">
            <div className="flex h-16 items-center justify-between">
              <div className="flex items-center">
                <div className="shrink-0">
                  <img
                    className="h-8 w-8"
                    src={brand.logo.url}
                    alt={brand.name}
                  />
                </div>
                <div className="hidden md:block">
                  <div className="ml-10 flex items-baseline space-x-4">
                    {navigation.map((item) => (
                      <a
                        key={item.name}
                        href={item.href}
                        onClick={() => onNavigation(item)}
                        className={classNames(
                          active === item.name
                            ? "bg-[var(--ci-signal)] text-white font-semibold"
                            : "text-[var(--ci-ink)] hover:bg-[var(--ci-surface-raised)]",
                          "cursor-pointer rounded-none px-3 py-1.5 font-mono text-xs uppercase tracking-wider transition-colors",
                        )}
                        aria-current={active === item.name ? "page" : undefined}
                      >
                        {item.name}
                      </a>
                    ))}
                  </div>
                </div>
              </div>
              <div className="hidden md:block">
                <div className="ml-4 flex items-center md:ml-6">
                  {/* Profile dropdown */}
                  <ProfileDropdown user={user} />
                </div>
              </div>
              <div className="-mr-2 md:hidden">
                {/* Mobile menu button */}
                <Disclosure.Button className="relative rounded-none p-1 text-[var(--ci-muted)] hover:text-[var(--ci-signal)] focus:outline-none transition-colors">
                  <span className="absolute -inset-0.5" />
                  <span className="sr-only">Open main menu</span>
                  {open ? (
                    <XMarkIcon className="block h-6 w-6" aria-hidden="true" />
                  ) : (
                    <Bars3Icon className="block h-6 w-6" aria-hidden="true" />
                  )}
                </Disclosure.Button>
              </div>
            </div>
          </div>

          <Disclosure.Panel className="md:hidden">
            <div className="space-y-1 px-2 pb-3 pt-2 sm:px-3">
              {navigation.map((item) => (
                <Disclosure.Button
                  key={item.name}
                  as="a"
                  href={item.href}
                  className={classNames(
                    active === item.name
                      ? "bg-[var(--ci-signal)] text-white font-semibold"
                      : "text-[var(--ci-ink)] hover:bg-[var(--ci-surface-raised)]",
                    "block rounded-none px-3 py-2 font-mono text-xs uppercase tracking-wider transition-colors",
                  )}
                  aria-current={item.name === active ? "page" : undefined}
                >
                  {item.name}
                </Disclosure.Button>
              ))}
            </div>
            <div className="border-t border-[var(--ci-line)] pb-3 pt-4">
              <div className="flex items-center px-5">
                <div className="shrink-0">
                  <img
                    className="h-10 w-10 rounded-none border border-[var(--ci-line)]"
                    src={user?.imageUrl}
                    alt=""
                  />
                </div>
                <div className="ml-3">
                  <div className="font-mono text-xs font-semibold leading-none text-[var(--ci-ink)]">
                    {user?.name}
                  </div>
                  <div className="font-mono text-[11px] leading-none text-[var(--ci-muted)]">
                    {user?.email}
                  </div>
                </div>
              </div>
              <div className="mt-3 space-y-1 px-2">
                {userNavigation.map((item) => (
                  <Disclosure.Button
                    key={item.name}
                    as="a"
                    href={item.href}
                    className={classNames(
                      "block rounded-none px-3 py-2 font-mono text-xs uppercase tracking-wider",
                      "text-[var(--ci-ink)] hover:bg-[var(--ci-surface-raised)] transition-colors",
                    )}
                  >
                    {item.name}
                  </Disclosure.Button>
                ))}
              </div>
            </div>
          </Disclosure.Panel>
        </>
      )}
    </Disclosure>
  );
};
