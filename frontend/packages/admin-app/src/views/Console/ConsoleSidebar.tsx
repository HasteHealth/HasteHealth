import {
  ArrowUpTrayIcon,
  BanknotesIcon,
  BeakerIcon,
  ChevronDownIcon,
  ChevronRightIcon,
  ClockIcon,
  Cog6ToothIcon,
  CommandLineIcon,
  CubeIcon,
  ExclamationTriangleIcon,
  HeartIcon,
  LockClosedIcon,
  MagnifyingGlassIcon,
  PuzzlePieceIcon,
  QuestionMarkCircleIcon,
  ShieldCheckIcon,
  Squares2X2Icon,
  StarIcon,
  TableCellsIcon,
  WrenchScrewdriverIcon,
} from "@heroicons/react/24/outline";
import { StarIcon as StarSolidIcon } from "@heroicons/react/24/solid";
import classNames from "classnames";
import { useAtomValue, useSetAtom } from "jotai";
import React, { useEffect, useMemo, useState } from "react";

import { CategoryGroup, categorise } from "../../db/categories";
import { pinnedTypesAtom, togglePinnedTypeAtom } from "../../query/atoms";
import { modLabel } from "../../hooks/useShortcuts";

/**
 * An icon per module, so a group is recognisable before its label is read.
 * Falls back to a neutral mark for a module not listed here.
 */
const MODULE_ICONS: Record<string, typeof CubeIcon> = {
  Base: CubeIcon,
  Clinical: HeartIcon,
  Financial: BanknotesIcon,
  Specialized: BeakerIcon,
  Foundation: WrenchScrewdriverIcon,
  "SQL on FHIR": TableCellsIcon,
  Platform: LockClosedIcon,
};

const OPERATE_LINKS = [
  { path: "/r/_history", label: "Event history", Icon: ClockIcon },
  { path: "/import", label: "Import bundle", Icon: ArrowUpTrayIcon },
  { path: "/packages", label: "Packages", Icon: PuzzlePieceIcon },
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
  { path: "/roles", label: "Roles & permissions", Icon: ShieldCheckIcon },
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
          "min-w-0 flex-1 truncate rounded-none py-1 pr-1 text-left text-sm transition-colors",
          indentClass(depth),
          active
            ? "bg-[var(--ci-signal-soft)] font-medium text-[var(--ci-signal-ink)] border-l-2 border-[var(--ci-signal)]"
            : "text-[var(--ci-ink-muted)] hover:text-[var(--ci-ink)] hover:bg-[var(--ci-surface-sunken)] border-l-2 border-transparent",
        )}
        onClick={() => onNavigate(`/r/${type}`)}
        title={type}
        type="button"
      >
        {type}
      </button>
      <button
        className={classNames(
          "p-1 text-[var(--ci-muted)] hover:text-[var(--ci-signal)] transition-colors",
          pinned ? "visible text-[var(--ci-signal)]" : "invisible group-hover:visible",
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
  const ModuleIcon = MODULE_ICONS[group.module] ?? Squares2X2Icon;
  return (
    <li>
      <button
        className="flex w-full items-center gap-1.5 rounded-none px-2 py-1 text-left text-sm font-medium text-[var(--ci-ink)] hover:bg-[var(--ci-surface-sunken)] transition-colors"
        onClick={onToggle}
        aria-expanded={isOpen}
        type="button"
      >
        {isOpen ? (
          <ChevronDownIcon className="h-3.5 w-3.5 shrink-0 text-[var(--ci-ink-subtle)]" />
        ) : (
          <ChevronRightIcon className="h-3.5 w-3.5 shrink-0 text-[var(--ci-ink-subtle)]" />
        )}
        <ModuleIcon className="h-4 w-4 shrink-0 text-[var(--ci-ink-subtle)]" />
        <span className="min-w-0 flex-1 truncate">{group.module}</span>
        <span className="shrink-0 text-xs font-mono text-[var(--ci-ink-subtle)]">{count}</span>
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
                    <span className="text-[10px] font-mono font-medium uppercase tracking-wider text-[var(--ci-ink-subtle)]">
                      {section.label}
                    </span>
                    <span className="h-px flex-1 bg-[var(--ci-line)]" />
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

  /** The list body: every group, the filter's matches, or a nothing-found row. */
  let typeList: React.ReactNode;
  if (needle === "") {
    typeList = groups.map((group) => (
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
    ));
  } else if (matches.length > 0) {
    typeList = matches.map((type) => (
      <TypeRow
        key={type}
        type={type}
        activeType={activeType}
        pinned={pinnedTypes.includes(type)}
        onNavigate={onNavigate}
        onTogglePin={onTogglePin}
      />
    ));
  } else {
    typeList = (
      <li className="px-2 py-2 font-mono text-xs text-[var(--ci-muted)]">
        No type matches “{filter}”.
      </li>
    );
  }

  return (
    <nav
      data-sidebar
      className="flex h-full w-56 shrink-0 flex-col gap-3 overflow-hidden border-r border-[var(--ci-line)] bg-[var(--ci-surface)] px-2 py-3 xl:w-64"
    >
      {pinnedTypes.length > 0 && (
        <section>
          <h2 className="px-2 pb-1 text-[10px] font-mono font-medium uppercase tracking-wider text-[var(--ci-ink-subtle)]">
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
        <h2 className="px-2 pb-1 text-[10px] font-mono font-medium uppercase tracking-wider text-[var(--ci-ink-subtle)]">
          Resource types
        </h2>
        <div className="relative mb-1 px-1">
          <MagnifyingGlassIcon className="pointer-events-none absolute left-3 top-1.5 h-3.5 w-3.5 text-[var(--ci-ink-subtle)]" />
          <input
            className="w-full rounded-none border border-[var(--ci-line)] bg-[var(--ci-surface-raised)] py-1 pl-7 pr-2 text-xs text-[var(--ci-ink)] placeholder:text-[var(--ci-ink-subtle)] focus:border-[var(--ci-signal)] focus:outline-none focus:ring-1 focus:ring-[var(--ci-signal)]"
            placeholder={`Filter ${resourceTypes.length} types`}
            value={filter}
            onChange={(event) => setFilter(event.target.value)}
          />
        </div>

        <ul className="min-h-0 flex-1 overflow-auto">{typeList}</ul>
      </section>

      <section className="border-t border-[var(--ci-line)] pt-2">
        <h2 className="px-2 pb-1 text-[10px] font-mono font-medium uppercase tracking-wider text-[var(--ci-ink-subtle)]">
          Operate
        </h2>
        <ul>
          {OPERATE_LINKS.map(({ path, label, Icon }) => (
            <li key={path}>
              <button
                className={classNames(
                  "flex w-full items-center gap-2 rounded-none px-2 py-1 text-left text-sm transition-colors",
                  activePath === path
                    ? "bg-[var(--ci-signal-soft)] font-medium text-[var(--ci-signal-ink)] border-l-2 border-[var(--ci-signal)]"
                    : "text-[var(--ci-ink-muted)] hover:text-[var(--ci-ink)] hover:bg-[var(--ci-surface-sunken)] border-l-2 border-transparent",
                )}
                onClick={() => onNavigate(path)}
                type="button"
              >
                <Icon className="h-4 w-4 shrink-0 text-[var(--ci-ink-subtle)]" />
                {label}
              </button>
            </li>
          ))}
          <li>
            {/* The shortcuts are only discoverable if something points at
                them, so they get a permanent place rather than only a chord. */}
            <button
              className="flex w-full items-center gap-2 rounded-none px-2 py-1 text-left text-sm text-[var(--ci-ink-muted)] hover:text-[var(--ci-ink)] hover:bg-[var(--ci-surface-sunken)] transition-colors border-l-2 border-transparent"
              onClick={onShowShortcuts}
              type="button"
            >
              <QuestionMarkCircleIcon className="h-4 w-4 shrink-0 text-[var(--ci-ink-subtle)]" />
              <span className="flex-1">Keyboard shortcuts</span>
              <kbd className="shrink-0 rounded-none border border-[var(--ci-line)] bg-[var(--ci-surface-raised)] px-1 font-mono text-[10px] text-[var(--ci-ink-muted)]">
                {modLabel()} /
              </kbd>
            </button>
          </li>
        </ul>
      </section>
    </nav>
  );
}
