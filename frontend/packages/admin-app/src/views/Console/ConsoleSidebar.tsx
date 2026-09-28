import {
  ArrowUpTrayIcon,
  ChevronDownIcon,
  ChevronRightIcon,
  ClockIcon,
  Cog6ToothIcon,
  ExclamationTriangleIcon,
  MagnifyingGlassIcon,
  QuestionMarkCircleIcon,
  StarIcon,
  CommandLineIcon,
  TableCellsIcon,
} from "@heroicons/react/24/outline";
import { StarIcon as StarSolidIcon } from "@heroicons/react/24/solid";
import classNames from "classnames";
import { useAtomValue, useSetAtom } from "jotai";
import React, { useEffect, useMemo, useState } from "react";

import { CategoryGroup, categorise } from "../../db/categories";
import { pinnedTypesAtom, togglePinnedTypeAtom } from "../../query/atoms";
import { modLabel } from "../../hooks/useShortcuts";

const OPERATE_LINKS = [
  { path: "/r/_history", label: "Event history", Icon: ClockIcon },
  { path: "/import", label: "Import bundle", Icon: ArrowUpTrayIcon },
  {
    path: "/indexing-errors",
    label: "Indexing errors",
    Icon: ExclamationTriangleIcon,
  },
  { path: "/r/ViewDefinition", label: "Projections", Icon: TableCellsIcon },
  // Custom operations are authored as OperationDefinitions, so the listing is
  // the way into building one.
  {
    path: "/r/OperationDefinition",
    label: "Custom operations",
    Icon: CommandLineIcon,
  },
  { path: "/settings", label: "Settings", Icon: Cog6ToothIcon },
];

export interface ConsoleSidebarProps {
  /** Every type the server exposes, from its CapabilityStatement. */
  resourceTypes: string[];
  /** The resource type currently in view, if any. */
  activeType?: string;
  activePath: string;
  onNavigate: (path: string) => void;
  onShowShortcuts: () => void;
}

/**
 * The console's sidebar.
 *
 * A server exposes around 155 resource types, so they are grouped into
 * collapsible modules by FHIR's own taxonomy, and the few types a workspace
 * actually uses are pinned above them. Filtering flattens the groups,
 * because when you are searching the categories are in the way rather than
 * helping.
 */
/** Indent for a type nested `depth` levels inside its group. */
function indentClass(depth: number): string {
  if (depth === 0) return "pl-2";
  return depth === 1 ? "pl-5" : "pl-7";
}

/** One resource type, with its pin toggle. */
function TypeRow({
  type,
  depth = 0,
  activeType,
  pinned,
  onNavigate,
  onTogglePin,
}: Readonly<{
  type: string;
  depth?: number;
  activeType?: string;
  pinned: boolean;
  onNavigate: (path: string) => void;
  onTogglePin: (type: string) => void;
}>) {
  const active = type === activeType;
  return (
    <li className="group flex items-center">
      <button
        className={classNames(
          "min-w-0 flex-1 truncate rounded py-1 pr-1 text-left text-sm",
          indentClass(depth),
          active
            ? "bg-brand-50 font-medium text-brand-900"
            : "text-slate-700 hover:bg-slate-100",
        )}
        onClick={() => onNavigate(`/r/${type}`)}
        title={type}
        type="button"
      >
        {type}
      </button>
      <button
        className={classNames(
          "rounded p-1 text-slate-300 hover:text-amber-500",
          pinned ? "visible text-amber-500" : "invisible group-hover:visible",
        )}
        onClick={() => onTogglePin(type)}
        title={pinned ? `Unpin ${type}` : `Pin ${type}`}
        type="button"
      >
        {pinned ? (
          <StarSolidIcon className="h-3.5 w-3.5" />
        ) : (
          <StarIcon className="h-3.5 w-3.5" />
        )}
      </button>
    </li>
  );
}

/** One module, collapsible, with its sections and types. */
function Group({
  group,
  isOpen,
  onToggle,
  activeType,
  pinnedTypes,
  onNavigate,
  onTogglePin,
}: Readonly<{
  group: CategoryGroup;
  isOpen: boolean;
  onToggle: () => void;
  activeType?: string;
  pinnedTypes: string[];
  onNavigate: (path: string) => void;
  onTogglePin: (type: string) => void;
}>) {
  const count = group.sections.reduce((n, s) => n + s.types.length, 0);
  return (
    <li>
      <button
        className="flex w-full items-center gap-1 rounded px-2 py-1 text-left text-sm text-slate-700 hover:bg-slate-100"
        onClick={onToggle}
        aria-expanded={isOpen}
        type="button"
      >
        {isOpen ? (
          <ChevronDownIcon className="h-3.5 w-3.5 shrink-0 text-slate-400" />
        ) : (
          <ChevronRightIcon className="h-3.5 w-3.5 shrink-0 text-slate-400" />
        )}
        <span className="min-w-0 flex-1 truncate">{group.module}</span>
        <span className="shrink-0 text-xs text-slate-400">{count}</span>
      </button>

      {isOpen && (
        <div className="mb-1">
          <ul>
            {group.sections.map((section) => (
              <li key={section.label}>
                {/* A module with one section is not worth a subheading. The
                    heading is indented past its module and trails a rule, so
                    it reads as a divider inside the group rather than as a
                    new group of its own. */}
                {group.sections.length > 1 && (
                  <div className="flex items-center gap-1.5 pb-0.5 pl-5 pr-2 pt-2">
                    <span className="text-[10px] font-medium uppercase tracking-wide text-slate-400">
                      {section.label}
                    </span>
                    <span className="h-px flex-1 bg-slate-200" />
                  </div>
                )}
                <ul>
                  {section.types.map((type) => (
                    <TypeRow
                      key={type}
                      type={type}
                      depth={group.sections.length > 1 ? 2 : 1}
                      activeType={activeType}
                      pinned={pinnedTypes.includes(type)}
                      onNavigate={onNavigate}
                      onTogglePin={onTogglePin}
                    />
                  ))}
                </ul>
              </li>
            ))}
          </ul>
        </div>
      )}
    </li>
  );
}

export function ConsoleSidebar({
  resourceTypes,
  activeType,
  activePath,
  onNavigate,
  onShowShortcuts,
}: Readonly<ConsoleSidebarProps>) {
  const [filter, setFilter] = useState("");
  const [open, setOpen] = useState<Record<string, boolean>>({});

  // Pinning is workspace state, so the sidebar reads and writes it directly
  // rather than having it threaded down from the shell.
  const pinned = useAtomValue(pinnedTypesAtom);
  const onTogglePin = useSetAtom(togglePinnedTypeAtom);

  const pinnedTypes = useMemo(
    () => pinned.filter((type) => resourceTypes.includes(type)),
    [pinned, resourceTypes],
  );

  const groups = useMemo(() => categorise(resourceTypes), [resourceTypes]);

  const needle = filter.trim().toLowerCase();

  /** While filtering, the groups collapse to one flat list of matches. */
  const matches = useMemo(
    () =>
      needle === ""
        ? []
        : resourceTypes
            .filter((type) => type.toLowerCase().includes(needle))
            .sort((a, b) => a.localeCompare(b)),
    [needle, resourceTypes],
  );

  // The group holding the type in view opens itself, so arriving from a link
  // or the command bar shows where you are.
  useEffect(() => {
    if (!activeType) return;
    const group = groups.find((g) =>
      g.sections.some((s) => s.types.includes(activeType)),
    );
    if (group) setOpen((current) => ({ ...current, [group.module]: true }));
  }, [activeType, groups]);

  // The group holding the type in view opens itself, so arriving from a link
  // or the command bar shows where you are.
  useEffect(() => {
    if (!activeType) return;
    const group = groups.find((g) =>
      g.sections.some((s) => s.types.includes(activeType)),
    );
    if (group) setOpen((current) => ({ ...current, [group.module]: true }));
  }, [activeType, groups]);

  return (
    <nav
      data-sidebar
      className="flex h-full w-56 shrink-0 flex-col gap-3 overflow-hidden border-r border-slate-200 bg-slate-50 px-2 py-3 xl:w-64"
    >
      {pinnedTypes.length > 0 && (
        <section>
          <h2 className="px-2 pb-1 text-[11px] font-semibold uppercase tracking-wide text-slate-400">
            Pinned
          </h2>
          <ul>
            {pinnedTypes.map((type) => (
              <TypeRow
                key={type}
                type={type}
                activeType={activeType}
                pinned={pinnedTypes.includes(type)}
                onNavigate={onNavigate}
                onTogglePin={onTogglePin}
              />
            ))}
          </ul>
        </section>
      )}

      <section className="flex min-h-0 flex-1 flex-col">
        <h2 className="px-2 pb-1 text-[11px] font-semibold uppercase tracking-wide text-slate-400">
          Resource types
        </h2>
        <div className="relative mb-1 px-1">
          <MagnifyingGlassIcon className="pointer-events-none absolute left-3 top-1.5 h-3.5 w-3.5 text-slate-400" />
          <input
            className="w-full rounded border border-slate-200 bg-white py-1 pl-7 pr-2 text-xs placeholder:text-slate-400 focus:border-brand-500 focus:ring-0"
            placeholder={`Filter ${resourceTypes.length} types`}
            value={filter}
            onChange={(event) => setFilter(event.target.value)}
          />
        </div>

        <ul className="min-h-0 flex-1 overflow-auto">
          {needle === "" ? (
            groups.map((group) => (
              <Group
                key={group.module}
                group={group}
                isOpen={open[group.module] ?? false}
                onToggle={() =>
                  setOpen((current) => ({
                    ...current,
                    [group.module]: !(current[group.module] ?? false),
                  }))
                }
                activeType={activeType}
                pinnedTypes={pinnedTypes}
                onNavigate={onNavigate}
                onTogglePin={onTogglePin}
              />
            ))
          ) : matches.length > 0 ? (
            matches.map((type) => (
              <TypeRow
                key={type}
                type={type}
                activeType={activeType}
                pinned={pinnedTypes.includes(type)}
                onNavigate={onNavigate}
                onTogglePin={onTogglePin}
              />
            ))
          ) : (
            <li className="px-2 py-2 text-xs text-slate-400">
              No type matches “{filter}”.
            </li>
          )}
        </ul>
      </section>

      <section className="border-t border-slate-200 pt-2">
        <h2 className="px-2 pb-1 text-[11px] font-semibold uppercase tracking-wide text-slate-400">
          Operate
        </h2>
        <ul>
          {OPERATE_LINKS.map(({ path, label, Icon }) => (
            <li key={path}>
              <button
                className={classNames(
                  "flex w-full items-center gap-2 rounded px-2 py-1 text-left text-sm",
                  activePath === path
                    ? "bg-brand-50 font-medium text-brand-900"
                    : "text-slate-700 hover:bg-slate-100",
                )}
                onClick={() => onNavigate(path)}
                type="button"
              >
                <Icon className="h-4 w-4 shrink-0 text-slate-400" />
                {label}
              </button>
            </li>
          ))}
          <li>
            {/* The shortcuts are only discoverable if something points at
                them, so they get a permanent place rather than only a chord. */}
            <button
              className="flex w-full items-center gap-2 rounded px-2 py-1 text-left text-sm text-slate-700 hover:bg-slate-100"
              onClick={onShowShortcuts}
              type="button"
            >
              <QuestionMarkCircleIcon className="h-4 w-4 shrink-0 text-slate-400" />
              <span className="flex-1">Keyboard shortcuts</span>
              <kbd className="shrink-0 rounded border border-slate-300 bg-white px-1 font-mono text-[10px] text-slate-500">
                {modLabel()} /
              </kbd>
            </button>
          </li>
        </ul>
      </section>
    </nav>
  );
}
