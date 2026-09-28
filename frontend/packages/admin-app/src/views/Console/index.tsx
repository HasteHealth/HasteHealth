import { ClockIcon, TrashIcon, XMarkIcon } from "@heroicons/react/24/outline";
import classNames from "classnames";
import { useAtom, useAtomValue, useSetAtom } from "jotai";
import React, { useCallback, useEffect, useMemo, useState } from "react";
import { useLocation, useNavigate, useSearchParams } from "react-router";

import { getCapabilities } from "../../db/capabilities";
import { CommandBar } from "../../components/CommandBar";
import {
  Command,
  CommandHistoryEntry,
  commandFromLocation,
  commandToPath,
  describeCommand,
  historyCommand,
  isMutation,
  sameCommand,
} from "../../query/model";
import {
  clearHistoryAtom,
  historyAtom,
  recentOpenAtom,
  recordCommandAtom,
  removeCommandAtom,
} from "../../query/atoms";
import { HistoryPanel } from "./HistoryPanel";
import { ResourceTypeHeader } from "./ResourceTypeHeader";
import { MutationPanel } from "./MutationPanel";
import { ResourcePanel } from "./ResourcePanel";
import { ResultsPanel } from "./ResultsPanel";

/** Renders a timestamp as a short relative age. */
function age(at: number): string {
  const seconds = Math.max(0, Math.round((Date.now() - at) / 1000));
  if (seconds < 60) return "just now";
  const minutes = Math.round(seconds / 60);
  if (minutes < 60) return `${minutes}m ago`;
  const hours = Math.round(minutes / 60);
  if (hours < 24) return `${hours}h ago`;
  return `${Math.round(hours / 24)}d ago`;
}

/** The commands run in this workspace, most recent first. */
function RecentPanel({
  entries,
  current,
  open,
  onOpenChange,
  onSelect,
  onRemove,
  onClear,
}: Readonly<{
  entries: CommandHistoryEntry[];
  current?: Command;
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onSelect: (command: Command) => void;
  onRemove: (command: Command) => void;
  onClear: () => void;
}>) {
  // Closed, it is a thin rail rather than nothing at all, so the history is
  // still one click away.
  if (!open) {
    const count = entries.length > 0 ? ` (${entries.length})` : "";

    return (
      <aside className="hidden shrink-0 xl:flex">
        <button
          className="flex flex-col items-center gap-2 rounded-lg border border-slate-200 bg-white px-2 py-3 text-slate-400 hover:bg-slate-50 hover:text-slate-600"
          onClick={() => onOpenChange(true)}
          title={`Recent commands${count}`}
          type="button"
        >
          <ClockIcon className="h-4 w-4" />
          <span className="text-[11px] [writing-mode:vertical-rl]">Recent</span>
        </button>
      </aside>
    );
  }

  return (
    <aside className="hidden w-64 shrink-0 flex-col rounded-lg border border-slate-200 bg-white xl:flex">
      <header className="flex items-center gap-2 border-b border-slate-200 px-3 py-2">
        <ClockIcon className="h-4 w-4 text-slate-400" />
        <span className="flex-1 text-sm font-medium text-slate-700">
          Recent
        </span>
        {entries.length > 0 && (
          <button
            className="rounded p-1 text-slate-400 hover:bg-slate-100 hover:text-slate-600"
            onClick={onClear}
            title="Clear"
            type="button"
          >
            <TrashIcon className="h-4 w-4" />
          </button>
        )}
        <button
          className="rounded p-1 text-slate-400 hover:bg-slate-100 hover:text-slate-600"
          onClick={() => onOpenChange(false)}
          title="Hide"
          type="button"
        >
          <XMarkIcon className="h-4 w-4" />
        </button>
      </header>
      {entries.length === 0 ? (
        <p className="px-3 py-4 text-xs text-slate-400">
          Commands you run appear here.
        </p>
      ) : (
        <ul className="flex-1 overflow-auto py-1">
          {entries.map((entry) => {
            const label = describeCommand(entry.command);
            const active =
              current !== undefined && sameCommand(entry.command, current);
            return (
              <li key={`${label}-${entry.at}`}>
                <div
                  className={classNames(
                    "group flex items-center gap-1 px-2 py-1.5 text-xs",
                    active ? "bg-brand-50" : "hover:bg-slate-50",
                  )}
                >
                  <button
                    className="min-w-0 flex-1 text-left"
                    onClick={() => onSelect(entry.command)}
                    title={label}
                    type="button"
                  >
                    <div
                      className={classNames(
                        "truncate font-mono",
                        active ? "text-brand-800" : "text-slate-700",
                      )}
                    >
                      {label}
                    </div>
                    <div className="text-[11px] text-slate-400">
                      {age(entry.at)}
                    </div>
                  </button>
                  <button
                    className="invisible rounded p-1 text-slate-400 hover:bg-slate-200 group-hover:visible"
                    onClick={() => onRemove(entry.command)}
                    title="Remove"
                    type="button"
                  >
                    <XMarkIcon className="h-3.5 w-3.5" />
                  </button>
                </div>
              </li>
            );
          })}
        </ul>
      )}
    </aside>
  );
}

/** Shown at `/`, before anything has been run. */
function Welcome({
  resourceTypeCount,
}: Readonly<{ resourceTypeCount: number }>) {
  return (
    <div className="rounded-lg border border-dashed border-slate-300 bg-white p-6 text-sm text-slate-600">
      <p className="font-medium text-slate-800">
        Type a FHIR path above and press Enter.
      </p>
      <ul className="mt-3 space-y-1 font-mono text-xs text-slate-500">
        <li>Patient?name=Smith</li>
        <li>Patient/123</li>
        <li>Observation?subject:Patient.name=Smith</li>
        <li>Patient/_history</li>
        <li>DELETE Patient/123</li>
      </ul>
      <p className="mt-3 text-xs text-slate-400">
        {resourceTypeCount} resource types available. Pick a verb on the left of
        the bar to write instead of read.
      </p>
    </div>
  );
}

/** `pathname` with `params` appended, omitting the `?` when it is empty. */
function withQuery(pathname: string, params: URLSearchParams): string {
  const query = params.toString();
  return query ? `${pathname}?${query}` : pathname;
}

/**
 * The console.
 *
 * One command bar, and whatever the command asks for beneath it. Reads live
 * at a URL so they are shareable and the browser's back button steps through
 * them; writes are staged rather than navigated to, because a refresh should
 * never re-send a delete.
 */
export default function Console() {
  const navigate = useNavigate();
  const location = useLocation();
  const [searchParams] = useSearchParams();
  const capabilities = useAtomValue(getCapabilities);

  const resourceTypes = useMemo(
    () =>
      (capabilities?.rest?.[0]?.resource ?? [])
        .map((entry) => entry.type as string)
        .filter(Boolean)
        .sort((a, b) => a.localeCompare(b)),
    [capabilities],
  );

  // A write is held here rather than in the URL.
  const [staged, setStaged] = useState<Command>();

  const urlCommand = useMemo(
    () => commandFromLocation(location.pathname, location.search),
    [location.pathname, location.search],
  );
  const command = staged ?? urlCommand;

  // History is workspace state that persists itself.
  const history = useAtomValue(historyAtom);
  const [recentOpen, setRecentOpen] = useAtom(recentOpenAtom);
  const recordCommand = useSetAtom(recordCommandAtom);
  const removeCommand = useSetAtom(removeCommandAtom);
  const clearHistory = useSetAtom(clearHistoryAtom);

  // Arriving at a command is what records it; a bare type listing is not
  // worth a history entry since that is what the sidebar does.
  useEffect(() => {
    if (!urlCommand) return;
    const isBareType =
      urlCommand.target.level === "type" &&
      !urlCommand.target.search &&
      urlCommand.interaction.kind === "resource";
    if (isBareType) return;
    recordCommand(urlCommand);
  }, [urlCommand]);

  const run = useCallback(
    (next: Command) => {
      if (isMutation(next)) {
        // A write opens its editor rather than being sent: the URL must stay
        // safe to refresh, and nothing is sent until the editor's own button
        // is pressed.
        setStaged(next);
        return;
      }
      setStaged(undefined);
      // Re-running the path already in the address bar does not change the
      // location, so leaving compose mode has to happen here rather than
      // only in the effect below.
      navigate(commandToPath(next));
    },
    [navigate],
  );

  /** Leaves a staged write without running it. */
  const cancelStaged = useCallback(() => setStaged(undefined), []);

  // Navigating away - a sidebar click, the back button, a row - abandons a
  // staged write, so the console never strands you in an editor you cannot
  // leave.
  useEffect(() => {
    setStaged(undefined);
  }, [location.pathname, location.search]);

  const offset = Number(searchParams.get("_offset") ?? 0);

  const onOffsetChange = useCallback(
    (next: number) => {
      const params = new URLSearchParams(location.search);
      if (next > 0) params.set("_offset", String(next));
      else params.delete("_offset");
      // Paging is not a new command, so it should not fill up back/forward.
      navigate(withQuery(location.pathname, params), { replace: true });
    },
    [navigate, location.pathname, location.search],
  );

  const onSortChange = useCallback(
    (next: string | undefined) => {
      const params = new URLSearchParams(location.search);
      if (next) params.set("_sort", next);
      else params.delete("_sort");
      navigate(withQuery(location.pathname, params));
    },
    [navigate, location.pathname, location.search],
  );

  /** The search text for a results table, with paging stripped out. */
  const searchText = useMemo(() => {
    const params = new URLSearchParams(location.search);
    params.delete("_offset");
    return params.toString();
  }, [location.search]);

  const body = (() => {
    if (!command) return <Welcome resourceTypeCount={resourceTypes.length} />;

    if (staged && isMutation(staged)) {
      return (
        <MutationPanel
          command={staged}
          onCancel={cancelStaged}
          onDone={() => {
            setStaged(undefined);
            // A delete leaves nothing to show, so fall back to the listing.
            if (staged.target.level !== "system") {
              navigate(`/r/${staged.target.resourceType}`);
            }
          }}
        />
      );
    }

    if (command.interaction.kind === "history") {
      return <HistoryPanel target={command.target} heading />;
    }

    if (command.target.level === "instance") {
      return <ResourcePanel target={command.target} />;
    }

    if (command.target.level === "type") {
      const { resourceType } = command.target;
      return (
        <div className="flex min-h-0 flex-1 flex-col">
          <ResourceTypeHeader
            resourceType={resourceType}
            onHistory={() =>
              run(historyCommand({ level: "type", resourceType }))
            }
            onNew={() =>
              run({
                verb: "POST",
                target: { level: "type", resourceType },
                interaction: { kind: "resource" },
              })
            }
          />
          <ResultsPanel
            command={command}
            resourceType={resourceType}
            search={searchText}
            offset={offset}
            onOffsetChange={onOffsetChange}
            sort={searchParams.get("_sort") ?? undefined}
            onSortChange={onSortChange}
          />
        </div>
      );
    }

    return (
      <div className="rounded-lg border border-slate-200 bg-white p-4 text-sm text-slate-600">
        System level operations are not runnable from here yet.
      </div>
    );
  })();

  return (
    // `w-full` so the console fills the main column even when its content
    // is narrow, such as the empty state before anything has been run.
    <div className="flex min-h-0 w-full flex-1 flex-col gap-3">
      <CommandBar
        command={command}
        onRun={run}
        onVerbChange={(verb, next) => {
          // Choosing a write is the intent to write, so the editor opens
          // straight away; choosing GET again returns to reading.
          if (verb === "GET") setStaged(undefined);
          else if (next) setStaged(next);
        }}
      />

      <div className="flex min-h-0 w-full flex-1 gap-3">
        <section className="flex min-w-0 flex-1 flex-col overflow-auto rounded-lg border border-slate-200 bg-white p-3">
          {body}
        </section>

        <RecentPanel
          entries={history}
          current={command}
          open={recentOpen}
          onOpenChange={setRecentOpen}
          onSelect={run}
          onRemove={removeCommand}
          onClear={clearHistory}
        />
      </div>
    </div>
  );
}
