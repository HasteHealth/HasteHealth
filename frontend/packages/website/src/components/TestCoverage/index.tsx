import React, { useDeferredValue, useEffect, useMemo, useState } from "react";

import {
  BACKENDS,
  BACKEND_NAMES,
  INTERACTIONS,
  type Backend,
  type ParamType,
  type ResourceSupport,
  type Result,
  type SupportData,
  type Tally,
  combine,
  featureCoverage,
  groupsResult,
  interactionCoverage,
  knownGaps,
  parameterCoverage,
  summarize,
  useSupportData,
} from "./data";
import {
  INTERACTION_EXAMPLES,
  INTERACTION_NAMES,
  TYPE_DESCRIPTIONS,
  TYPE_NAMES,
  TYPE_ORDER,
  featureInfo,
  featureOrder,
} from "./labels";
import styles from "./styles.module.css";

// A parameter name per type, for example queries.
const EXAMPLE_PARAMETER: Record<ParamType, string> = {
  string: "name",
  token: "code",
  reference: "subject",
  date: "date",
  number: "probability",
  quantity: "value-quantity",
  uri: "url",
};

// ---------------------------------------------------------------------------
// Primitives
// ---------------------------------------------------------------------------

const RESULT_TEXT: Record<Result, string> = {
  pass: "Passes",
  fail: "Fails",
  "not-run": "Not tested",
};

const ICON_TONES: Record<Result, string> = {
  pass: "text-emerald-600 dark:text-emerald-400",
  fail: "text-rose-600 dark:text-rose-400",
  "not-run": "text-slate-400 dark:text-slate-500",
};

const ICON_PATHS: Record<Result, string> = {
  pass: "M10 18a8 8 0 1 0 0-16 8 8 0 0 0 0 16Zm3.857-9.809a.75.75 0 0 0-1.214-.882l-3.483 4.79-1.88-1.88a.75.75 0 1 0-1.06 1.061l2.5 2.5a.75.75 0 0 0 1.137-.089l4-5.5Z",
  fail: "M10 18a8 8 0 1 0 0-16 8 8 0 0 0 0 16ZM8.28 7.22a.75.75 0 0 0-1.06 1.06L8.94 10l-1.72 1.72a.75.75 0 1 0 1.06 1.06L10 11.06l1.72 1.72a.75.75 0 1 0 1.06-1.06L11.06 10l1.72-1.72a.75.75 0 0 0-1.06-1.06L10 8.94 8.28 7.22Z",
  "not-run": "M10 18a8 8 0 1 0 0-16 8 8 0 0 0 0 16ZM6.75 9.25a.75.75 0 0 0 0 1.5h6.5a.75.75 0 0 0 0-1.5h-6.5Z",
};

function StatusIcon({ result, className = "h-4 w-4" }: Readonly<{ result: Result; className?: string }>) {
  return (
    <svg
      viewBox="0 0 20 20"
      fill="currentColor"
      role="img"
      aria-label={RESULT_TEXT[result]}
      className={`inline-block shrink-0 ${ICON_TONES[result]} ${className}`}
    >
      <path fillRule="evenodd" d={ICON_PATHS[result]} clipRule="evenodd" />
    </svg>
  );
}

function tallyResult(tally: Tally): Result {
  if (tally.total === 0) return "not-run";
  return tally.pass === tally.total ? "pass" : "fail";
}

/** `icon 54 / 54`, red when anything failed. */
function TallyCell({ tally, unit }: Readonly<{ tally: Tally; unit?: string }>) {
  const result = tallyResult(tally);
  return (
    <span className="inline-flex items-center justify-end gap-1.5">
      <StatusIcon result={result} />
      <span className={result === "fail" ? "font-semibold text-rose-700 dark:text-rose-300" : ""}>
        {tally.pass} / {tally.total}
      </span>
      {unit ? <span className="text-slate-500 dark:text-slate-400">{unit}</span> : null}
    </span>
  );
}

function TypeBadge({ type }: Readonly<{ type?: ParamType }>) {
  if (!type) return null;
  return (
    <span className="rounded-sm bg-slate-100 px-1.5 py-0.5 text-[11px] font-medium text-slate-600 dark:bg-slate-800 dark:text-slate-300">
      {TYPE_NAMES[type]}
    </span>
  );
}

function Card({ children, className = "" }: Readonly<{ children: React.ReactNode; className?: string }>) {
  return (
    <div
      className={`overflow-hidden rounded-lg border border-slate-200 bg-white dark:border-slate-700 dark:bg-slate-900 ${className}`}
    >
      {children}
    </div>
  );
}

function Loaded({ children }: Readonly<{ children: (data: SupportData) => React.ReactNode }>) {
  const state = useSupportData();
  if (state.status === "error") {
    return (
      <p className="text-sm text-rose-700 dark:text-rose-300">
        Test results couldn't be loaded. Refresh the page to try again.
      </p>
    );
  }
  if (state.status === "loading") {
    return <div className="h-24 animate-pulse rounded-lg bg-slate-100 dark:bg-slate-800" aria-label="Loading test results" />;
  }
  return <div className={styles.root}>{children(state.data)}</div>;
}

/** A tally as a percentage: whole when complete, else one decimal. */
function percent(tally: Tally): string {
  if (tally.total === 0) return "0";
  const digits = tally.pass === tally.total ? 0 : 1;
  return ((tally.pass / tally.total) * 100).toFixed(digits);
}

// ---------------------------------------------------------------------------
// Summary
// ---------------------------------------------------------------------------

export function CoverageSummary() {
  return (
    <Loaded>
      {(data) => {
        const summary = summarize(data);
        const verified = new Date(data.generatedAt);
        return (
          <div className="grid gap-4 lg:grid-cols-3">
            <Card className="p-5">
              <div className="text-xs font-semibold uppercase tracking-wide text-slate-500 dark:text-slate-400">
                Tested scope
              </div>
              <div className="mt-2 text-3xl font-semibold text-slate-900 dark:text-white">
                {summary.resourceTypes} resource types
              </div>
              <div className="mt-1 text-sm text-slate-600 dark:text-slate-300">
                {summary.searchParameters.toLocaleString()} search parameters ·{" "}
                {summary.checks.toLocaleString()} checks
              </div>
              <div className="mt-4 border-t border-slate-100 pt-3 text-xs text-slate-500 dark:border-slate-800 dark:text-slate-400">
                Both backends agree on {percent({ pass: summary.agreeing, total: summary.checks })}% of
                checks. Last verified{" "}
                <time dateTime={data.generatedAt}>
                  {verified.toLocaleDateString(undefined, { year: "numeric", month: "long", day: "numeric" })}
                </time>
                .
              </div>
            </Card>
            {BACKENDS.map((backend) => {
              const result = summary.backends[backend];
              const allPass = result.resourcesPassing === summary.resourceTypes;
              return (
                <Card key={backend} className="p-5">
                  <div className="flex items-center justify-between">
                    <div className="text-xs font-semibold uppercase tracking-wide text-slate-500 dark:text-slate-400">
                      {BACKEND_NAMES[backend]}
                    </div>
                    <StatusIcon result={allPass ? "pass" : "fail"} className="h-5 w-5" />
                  </div>
                  <div className="mt-2 text-3xl font-semibold text-slate-900 dark:text-white">
                    {result.resourcesPassing}
                    <span className="text-lg font-normal text-slate-500 dark:text-slate-400">
                      {" "}
                      / {summary.resourceTypes}
                    </span>
                  </div>
                  <div className="mt-1 text-sm text-slate-600 dark:text-slate-300">
                    resource types pass every check
                  </div>
                  <div
                    className="mt-4 h-1.5 overflow-hidden rounded-full bg-slate-100 dark:bg-slate-800"
                    aria-hidden="true"
                  >
                    <div
                      className="h-full rounded-full bg-emerald-500"
                      style={{ width: `${(result.checks.pass / Math.max(result.checks.total, 1)) * 100}%` }}
                    />
                  </div>
                  <div className="mt-2 text-xs text-slate-500 dark:text-slate-400">
                    {result.checks.pass.toLocaleString()} of {result.checks.total.toLocaleString()} checks
                    pass ({percent(result.checks)}%)
                  </div>
                </Card>
              );
            })}
          </div>
        );
      }}
    </Loaded>
  );
}

// ---------------------------------------------------------------------------
// Interactions and search features
// ---------------------------------------------------------------------------

function BackendHeaders() {
  return (
    <>
      {BACKENDS.map((backend) => (
        <th key={backend} className={styles.numeric}>
          {BACKEND_NAMES[backend]}
        </th>
      ))}
    </>
  );
}

export function InteractionMatrix() {
  return (
    <Loaded>
      {(data) => (
        <Card>
          <div className={styles.scroll}>
            <table className={styles.table}>
              <thead>
                <tr>
                  <th>Interaction</th>
                  <th>Request</th>
                  <BackendHeaders />
                </tr>
              </thead>
              <tbody>
                {interactionCoverage(data).map(({ interaction, results }) => (
                  <tr key={interaction}>
                    <td className="font-medium text-slate-900 dark:text-white">{INTERACTION_NAMES[interaction]}</td>
                    <td>
                      <code className="text-xs">{INTERACTION_EXAMPLES[interaction]}</code>
                    </td>
                    {BACKENDS.map((backend) => (
                      <td key={backend} className={styles.numeric}>
                        <TallyCell tally={results[backend]} unit="types" />
                      </td>
                    ))}
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </Card>
      )}
    </Loaded>
  );
}

export function SearchFeatureMatrix() {
  return (
    <Loaded>
      {(data) => {
        const features = featureCoverage(data);
        return (
          <Card>
            <div className={styles.scroll}>
              <table className={styles.table}>
                <thead>
                  <tr>
                    <th>Feature</th>
                    <th className={styles.optional}>Example</th>
                    <th className={styles.numeric}>Parameters tested</th>
                    <BackendHeaders />
                  </tr>
                </thead>
                <tbody>
                  {TYPE_ORDER.map((type) => {
                    const rows = features
                      .filter((feature) => feature.type === type)
                      .sort((a, b) => featureOrder(type, a.feature) - featureOrder(type, b.feature));
                    if (rows.length === 0) return null;
                    return (
                      <React.Fragment key={type}>
                        <tr>
                          <td colSpan={5} className={styles.group}>
                            <span className="text-sm font-semibold text-slate-900 dark:text-white">
                              {TYPE_NAMES[type]}
                            </span>
                            <span className="ml-2 text-xs text-slate-500 dark:text-slate-400">
                              {TYPE_DESCRIPTIONS[type]}
                            </span>
                          </td>
                        </tr>
                        {rows.map((row) => {
                          const info = featureInfo(type, row.feature);
                          return (
                            <tr key={row.feature}>
                              <td className="whitespace-nowrap font-medium text-slate-800 dark:text-slate-100">
                                {info.label}
                              </td>
                              <td className={styles.optional}>
                                {info.example ? (
                                  <code className="whitespace-nowrap text-xs">
                                    ?{EXAMPLE_PARAMETER[type]}
                                    {info.example}
                                  </code>
                                ) : null}
                              </td>
                              <td className={styles.numeric}>{row.parameters}</td>
                              {BACKENDS.map((backend) => (
                                <td key={backend} className={styles.numeric}>
                                  <TallyCell tally={row.results[backend]} />
                                </td>
                              ))}
                            </tr>
                          );
                        })}
                      </React.Fragment>
                    );
                  })}
                </tbody>
              </table>
            </div>
          </Card>
        );
      }}
    </Loaded>
  );
}

// ---------------------------------------------------------------------------
// Per resource type
// ---------------------------------------------------------------------------

const rowId = (resourceType: string) => `resource-${resourceType}`;

function failingCount(resource: ResourceSupport, backend: Backend) {
  return resource.groups.filter((group) => group.results[backend] === "fail").length;
}

function ResourceStatus({ resource, backend }: Readonly<{ resource: ResourceSupport; backend: Backend }>) {
  const result = groupsResult(resource.groups, backend);
  const failing = failingCount(resource, backend);
  const total = resource.groups.length;
  return (
    <span
      className="inline-flex items-center justify-end gap-1.5 whitespace-nowrap"
      title={result === "fail" ? `${failing} of ${total} tests fail` : undefined}
    >
      <StatusIcon result={result} />
      <span className={result === "fail" ? "font-semibold text-rose-700 dark:text-rose-300" : ""}>
        {
          {
            pass: "All pass",
            fail: `${failing} of ${total} fail`,
            "not-run": "Not tested",
          }[result]
        }
      </span>
    </span>
  );
}

const CHIP_TONES: Record<Result, string> = {
  pass: "bg-emerald-50 text-emerald-700 ring-emerald-200 dark:bg-emerald-950 dark:text-emerald-300 dark:ring-emerald-800",
  fail: "bg-rose-50 text-rose-700 ring-rose-300 dark:bg-rose-950 dark:text-rose-300 dark:ring-rose-800",
  "not-run": "bg-slate-50 text-slate-400 ring-slate-200 dark:bg-slate-800 dark:text-slate-500 dark:ring-slate-700",
};

/** C R U P D, each coloured by its worst result across backends. */
function InteractionChips({ resource }: Readonly<{ resource: ResourceSupport }>) {
  return (
    <span className="inline-flex gap-1">
      {INTERACTIONS.map((interaction) => {
        const groups = resource.groups.filter((group) => group.group === interaction);
        const perBackend = BACKENDS.map((backend) => groupsResult(groups, backend));
        const result = combine(perBackend);
        const tone = CHIP_TONES[result];
        const title = `${INTERACTION_NAMES[interaction]}: ${BACKENDS.map(
          (backend, index) => `${BACKEND_NAMES[backend]} ${RESULT_TEXT[perBackend[index]].toLowerCase()}`,
        ).join(", ")}`;
        return (
          <span
            key={interaction}
            title={title}
            aria-label={title}
            className={`inline-flex h-6 w-6 items-center justify-center rounded-sm text-[11px] font-semibold ring-1 ring-inset ${tone}`}
          >
            {INTERACTION_NAMES[interaction][0]}
          </span>
        );
      })}
    </span>
  );
}

function FeatureChip({
  type,
  feature,
  results,
}: Readonly<{ type?: ParamType; feature: string; results: Record<Backend, Result> }>) {
  const failing = BACKENDS.filter((backend) => results[backend] === "fail");
  const label = featureInfo(type, feature).label;
  if (failing.length === 0) {
    return (
      <span className="inline-flex items-center rounded-sm bg-slate-100 px-1.5 py-0.5 font-mono text-[11px] text-slate-700 dark:bg-slate-800 dark:text-slate-200">
        {label}
      </span>
    );
  }
  const title = `Fails on ${failing.map((backend) => BACKEND_NAMES[backend]).join(" and ")}`;
  return (
    <span
      title={title}
      className="inline-flex items-center gap-1 rounded-sm bg-rose-50 px-1.5 py-0.5 font-mono text-[11px] text-rose-700 ring-1 ring-inset ring-rose-300 dark:bg-rose-950 dark:text-rose-300 dark:ring-rose-800"
    >
      <StatusIcon result="fail" className="h-3 w-3" />
      {label}
      <span className="sr-only">({title})</span>
    </span>
  );
}

function ResourceDetail({ resource }: Readonly<{ resource: ResourceSupport }>) {
  const parameters = parameterCoverage(resource);
  const [openParameter, setOpenParameter] = useState<string>();

  return (
    <div className="grid gap-4 p-4 lg:grid-cols-[16rem_minmax(0,1fr)]">
      <Card>
        <table className={styles.table}>
          <thead>
            <tr>
              <th>Interaction</th>
              {BACKENDS.map((backend) => (
                <th key={backend} className={styles.numeric}>
                  {backend === "postgres" ? "PG" : "ES"}
                </th>
              ))}
            </tr>
          </thead>
          <tbody>
            {INTERACTIONS.map((interaction) => {
              const groups = resource.groups.filter((group) => group.group === interaction);
              return (
                <tr key={interaction}>
                  <td>{INTERACTION_NAMES[interaction]}</td>
                  {BACKENDS.map((backend) => (
                    <td key={backend} className={styles.numeric}>
                      <StatusIcon result={groupsResult(groups, backend)} />
                    </td>
                  ))}
                </tr>
              );
            })}
          </tbody>
        </table>
      </Card>

      <Card>
        {parameters.length === 0 ? (
          <p className="m-0 p-4 text-sm text-slate-500 dark:text-slate-400">
            {resource.resourceType} defines no search parameters beyond the common ones.
          </p>
        ) : (
          <div className={styles.scroll}>
            <table className={styles.table}>
              <thead>
                <tr>
                  <th>Search parameter</th>
                  <th>Tested features</th>
                  {BACKENDS.map((backend) => (
                    <th key={backend} className={styles.numeric}>
                      {backend === "postgres" ? "PG" : "ES"}
                    </th>
                  ))}
                  <th>
                    <span className="sr-only">Checks</span>
                  </th>
                </tr>
              </thead>
              <tbody>
                {parameters.map((parameter) => {
                  const open = openParameter === parameter.code;
                  return (
                    <React.Fragment key={parameter.code}>
                      <tr>
                        <td className="whitespace-nowrap">
                          <a
                            href={parameter.url}
                            className="font-mono font-semibold"
                            title={parameter.url}
                          >
                            {parameter.code}
                          </a>{" "}
                          <TypeBadge type={parameter.type} />
                        </td>
                        <td>
                          <span className="flex flex-wrap gap-1">
                            {[...parameter.features]
                              .sort(
                                (a, b) =>
                                  featureOrder(parameter.type, a.feature) -
                                  featureOrder(parameter.type, b.feature),
                              )
                              .map(({ feature, groups }) => (
                                <FeatureChip
                                  key={feature}
                                  type={parameter.type}
                                  feature={feature}
                                  results={
                                    Object.fromEntries(
                                      BACKENDS.map((backend) => [backend, groupsResult(groups, backend)]),
                                    ) as Record<Backend, Result>
                                  }
                                />
                              ))}
                          </span>
                        </td>
                        {BACKENDS.map((backend) => (
                          <td key={backend} className={styles.numeric}>
                            <StatusIcon result={parameter.results[backend]} />
                          </td>
                        ))}
                        <td className={styles.numeric}>
                          <button
                            type="button"
                            aria-expanded={open}
                            onClick={() => setOpenParameter(open ? undefined : parameter.code)}
                            className="cursor-pointer rounded-sm border-0 bg-transparent px-1 text-xs text-slate-500 underline-offset-2 hover:text-slate-900 hover:underline dark:text-slate-400 dark:hover:text-white"
                          >
                            {open ? "Hide" : `${parameter.assertions.length} checks`}
                          </button>
                        </td>
                      </tr>
                      {open ? (
                        <tr>
                          <td colSpan={5} className={styles.detail}>
                            <div className="grid gap-1 px-4 py-3 text-xs">
                              {parameter.assertions.map((assertion, index) => (
                                <div
                                  key={`${assertion.label}-${index}`}
                                  className="flex flex-wrap items-center justify-between gap-2"
                                >
                                  <span className="font-mono text-slate-700 dark:text-slate-200">
                                    {assertion.label}
                                    <span className="ml-2 text-slate-500 dark:text-slate-400">
                                      {assertion.expression} {assertion.operator} {assertion.expected}
                                    </span>
                                  </span>
                                  <span className="inline-flex gap-3">
                                    {BACKENDS.map((backend) => (
                                      <span key={backend} className="inline-flex items-center gap-1 text-slate-500">
                                        {backend === "postgres" ? "PG" : "ES"}
                                        <StatusIcon result={assertion.results[backend]} className="h-3.5 w-3.5" />
                                      </span>
                                    ))}
                                  </span>
                                </div>
                              ))}
                            </div>
                          </td>
                        </tr>
                      ) : null}
                    </React.Fragment>
                  );
                })}
              </tbody>
            </table>
          </div>
        )}
      </Card>
    </div>
  );
}

/** Opens the resource named in the URL hash (`#resource-Patient`). */
function useHashResource(): [string | undefined, (resourceType: string | undefined) => void] {
  const [open, setOpen] = useState<string>();

  useEffect(() => {
    const sync = () => {
      const match = /^#resource-(.+)$/.exec(window.location.hash);
      if (match) {
        setOpen(decodeURIComponent(match[1]));
        requestAnimationFrame(() =>
          document.getElementById(window.location.hash.slice(1))?.scrollIntoView({ block: "start" }),
        );
      }
    };
    sync();
    window.addEventListener("hashchange", sync);
    return () => window.removeEventListener("hashchange", sync);
  }, []);

  const update = (resourceType: string | undefined) => {
    setOpen(resourceType);
    const hash = resourceType ? `#${rowId(resourceType)}` : " ";
    window.history.replaceState(null, "", hash === " " ? window.location.pathname : hash);
  };
  return [open, update];
}

function ResourceTable({ data }: Readonly<{ data: SupportData }>) {
  const [query, setQuery] = useState("");
  const [issuesOnly, setIssuesOnly] = useState(false);
  const [open, setOpen] = useHashResource();
  const deferredQuery = useDeferredValue(query.trim().toLowerCase());

  const resources = useMemo(
    () =>
      data.resources.filter(
        (resource) =>
          resource.resourceType.toLowerCase().includes(deferredQuery) &&
          (!issuesOnly || BACKENDS.some((backend) => groupsResult(resource.groups, backend) !== "pass")),
      ),
    [data, deferredQuery, issuesOnly],
  );

  return (
    <Card>
      <div className="flex flex-wrap items-center gap-3 border-b border-slate-200 p-3 dark:border-slate-700">
        <input
          type="search"
          value={query}
          onChange={(event) => setQuery(event.target.value)}
          placeholder="Filter resource types…"
          aria-label="Filter resource types"
          className="min-w-0 flex-1 rounded-md border border-slate-300 bg-white px-3 py-1.5 text-sm text-slate-900 outline-none focus:border-slate-500 dark:border-slate-600 dark:bg-slate-950 dark:text-white"
        />
        <label className="inline-flex cursor-pointer items-center gap-2 text-sm text-slate-700 dark:text-slate-300">
          <input type="checkbox" checked={issuesOnly} onChange={(event) => setIssuesOnly(event.target.checked)} />
          <span>Only show issues</span>
        </label>
        <span className="text-xs text-slate-500 dark:text-slate-400">
          {resources.length} of {data.resources.length}
        </span>
      </div>
      <div className={styles.scroll}>
        <table className={styles.table}>
          <thead>
            <tr>
              <th>Resource type</th>
              <th className={styles.optional}>Interactions</th>
              <th className={`${styles.numeric} ${styles.optional}`}>Search parameters</th>
              <BackendHeaders />
            </tr>
          </thead>
          <tbody>
            {resources.map((resource) => {
              const expanded = open === resource.resourceType;
              const parameterCount = new Set(
                resource.groups
                  .filter((group) => group.group === "search")
                  .map((group) => group.searchParameterCode),
              ).size;
              return (
                <React.Fragment key={resource.resourceType}>
                  <tr id={rowId(resource.resourceType)} className="scroll-mt-20">
                    <td>
                      <button
                        type="button"
                        aria-expanded={expanded}
                        onClick={() => setOpen(expanded ? undefined : resource.resourceType)}
                        className="inline-flex cursor-pointer items-center gap-2 border-0 bg-transparent p-0 text-left text-sm font-semibold text-slate-900 hover:text-brand-800 dark:text-white dark:hover:text-brand-300"
                      >
                        <svg
                          viewBox="0 0 20 20"
                          fill="currentColor"
                          aria-hidden="true"
                          className={`h-4 w-4 text-slate-400 transition-transform ${expanded ? "rotate-90" : ""}`}
                        >
                          <path
                            fillRule="evenodd"
                            d="M7.21 14.77a.75.75 0 0 1 .02-1.06L11.168 10 7.23 6.29a.75.75 0 1 1 1.04-1.08l4.5 4.25a.75.75 0 0 1 0 1.08l-4.5 4.25a.75.75 0 0 1-1.06-.02Z"
                            clipRule="evenodd"
                          />
                        </svg>
                        {resource.resourceType}
                      </button>
                    </td>
                    <td className={styles.optional}>
                      <InteractionChips resource={resource} />
                    </td>
                    <td className={`${styles.numeric} ${styles.optional}`}>
                      {parameterCount > 0 ? (
                        parameterCount
                      ) : (
                        <span className="text-slate-400" title="No search parameters">
                          —
                        </span>
                      )}
                    </td>
                    {BACKENDS.map((backend) => (
                      <td key={backend} className={styles.numeric}>
                        <ResourceStatus resource={resource} backend={backend} />
                      </td>
                    ))}
                  </tr>
                  {expanded ? (
                    <tr>
                      <td colSpan={5} className={styles.detail}>
                        <ResourceDetail resource={resource} />
                      </td>
                    </tr>
                  ) : null}
                </React.Fragment>
              );
            })}
            {resources.length === 0 ? (
              <tr>
                <td colSpan={5} className="text-center text-sm text-slate-500">
                  No resource types match.
                </td>
              </tr>
            ) : null}
          </tbody>
        </table>
      </div>
    </Card>
  );
}

export function ResourceCoverage() {
  return <Loaded>{(data) => <ResourceTable data={data} />}</Loaded>;
}

// ---------------------------------------------------------------------------
// Known gaps
// ---------------------------------------------------------------------------

export function KnownGaps() {
  return (
    <Loaded>
      {(data) => {
        const gaps = knownGaps(data);
        if (gaps.length === 0) {
          return (
            <Card className="flex items-center gap-3 p-4 text-sm">
              <StatusIcon result="pass" className="h-5 w-5" />
              Every tested behaviour passes on both backends.
            </Card>
          );
        }
        return (
          <Card>
            {gaps.map((gap, index) => (
              <div
                key={`${gap.resourceType}-${gap.subject}-${index}`}
                className="flex flex-wrap items-start justify-between gap-3 border-b border-slate-100 p-4 last:border-b-0 dark:border-slate-800"
              >
                <div className="flex items-start gap-3">
                  <StatusIcon result="fail" className="mt-0.5 h-5 w-5" />
                  <div>
                    <div className="text-sm font-semibold text-slate-900 dark:text-white">
                      {gap.resourceType}
                      {gap.kind === "search" ? (
                        <>
                          <span className="font-normal text-slate-500 dark:text-slate-400">
                            {" "}
                            · search parameter{" "}
                          </span>
                          <code className="text-xs">{gap.subject}</code>
                          {gap.type ? (
                            <span className="ml-2">
                              <TypeBadge type={gap.type} />
                            </span>
                          ) : null}
                        </>
                      ) : (
                        <span className="font-normal text-slate-500 dark:text-slate-400">
                          {" "}
                          · {INTERACTION_NAMES[gap.subject as keyof typeof INTERACTION_NAMES] ?? gap.subject}{" "}
                          interaction
                        </span>
                      )}
                    </div>
                    {gap.features.length > 0 ? (
                      <div className="mt-1.5 flex flex-wrap gap-1">
                        {gap.features.map((feature) => (
                          <span
                            key={feature}
                            className="rounded-sm bg-rose-50 px-1.5 py-0.5 font-mono text-[11px] text-rose-700 dark:bg-rose-950 dark:text-rose-300"
                          >
                            {featureInfo(gap.type, feature).label}
                          </span>
                        ))}
                      </div>
                    ) : null}
                    <div className="mt-1.5 text-xs text-slate-500 dark:text-slate-400">
                      Fails on {gap.backends.map((backend) => BACKEND_NAMES[backend]).join(" and ")}
                    </div>
                  </div>
                </div>
                <a href={`#${rowId(gap.resourceType)}`} className="text-xs font-medium">
                  View {gap.resourceType} →
                </a>
              </div>
            ))}
          </Card>
        );
      }}
    </Loaded>
  );
}
