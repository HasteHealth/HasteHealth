import Link from "@docusaurus/Link";
import React, { useDeferredValue, useMemo, useState } from "react";

import { useHomepageFacts } from "../home/facts";
import { StatusIcon } from "../TestCoverage";
import { BACKEND_NAMES } from "../TestCoverage/data";
import styles from "../TestCoverage/styles.module.css";
import {
  BACKEND_SHORT,
  ISSUES,
  REPOSITORY,
  type Backend,
  type Result,
  type SuiteGroup,
  type SuiteId,
  type SuiteReport,
  type SuiteScript,
  gapsByReason,
  passesEverywhere,
  totalsOf,
  useSuite,
  worst,
} from "./data";

// ---------------------------------------------------------------------------
// Primitives
// ---------------------------------------------------------------------------

function Card({
  children,
  className = "",
}: Readonly<{ children: React.ReactNode; className?: string }>) {
  return (
    <div
      className={`overflow-hidden rounded-lg border border-slate-200 bg-white dark:border-slate-700 dark:bg-slate-900 ${className}`}
    >
      {children}
    </div>
  );
}

function Loaded({
  suite,
  children,
}: Readonly<{
  suite: SuiteId;
  children: (data: SuiteReport) => React.ReactNode;
}>) {
  const state = useSuite(suite);
  if (state.status === "error") {
    return (
      <p className="text-sm text-rose-700 dark:text-rose-300">
        Test results couldn't be loaded. Refresh the page to try again.
      </p>
    );
  }
  if (state.status === "loading") {
    return (
      <div
        className="h-24 animate-pulse rounded-lg bg-slate-100 dark:bg-slate-800"
        aria-label="Loading test results"
      />
    );
  }
  return <div className={styles.root}>{children(state.data)}</div>;
}

const RESULT_WORDS: Record<Result, string> = {
  pass: "Passes",
  warn: "Known gap",
  fail: "Fails",
  "not-run": "Not run",
};

const RESULT_TONES: Record<Result, string> = {
  pass: "text-slate-600 dark:text-slate-300",
  warn: "font-semibold text-amber-700 dark:text-amber-300",
  fail: "font-semibold text-rose-700 dark:text-rose-300",
  "not-run": "text-slate-500 dark:text-slate-400",
};

function Chevron({ open }: Readonly<{ open: boolean }>) {
  return (
    <svg
      viewBox="0 0 20 20"
      fill="currentColor"
      aria-hidden="true"
      className={`h-4 w-4 shrink-0 text-slate-400 transition-transform ${open ? "rotate-90" : ""}`}
    >
      <path
        fillRule="evenodd"
        d="M7.21 14.77a.75.75 0 0 1 .02-1.06L11.168 10 7.23 6.29a.75.75 0 1 1 1.04-1.08l4.5 4.25a.75.75 0 0 1 0 1.08l-4.5 4.25a.75.75 0 0 1-1.06-.02Z"
        clipRule="evenodd"
      />
    </svg>
  );
}

function sourceUrl(script: SuiteScript): string {
  return `${REPOSITORY}/${script.path}`;
}

/** A known-failure reason: `code` spans and #123 issue links. */
function Reason({ text }: Readonly<{ text: string }>) {
  return (
    <>
      {text.split(/(`[^`]+`|#\d+\b)/).map((part, index) => {
        if (part.startsWith("`") && part.endsWith("`") && part.length > 2) {
          return <code key={index}>{part.slice(1, -1)}</code>;
        }
        if (/^#\d+$/.test(part)) {
          return (
            <a key={index} href={`${ISSUES}/${part.slice(1)}`}>
              {part}
            </a>
          );
        }
        return part;
      })}
    </>
  );
}

/** How CI treats a suite's known gaps, for the gap notes. */
function gapRun(report: SuiteReport): string {
  return report.knownFailuresRun
    ? "run in CI and expected to fail"
    : "not run in CI";
}

// ---------------------------------------------------------------------------
// Last verified and summary
// ---------------------------------------------------------------------------

export function SuiteLastVerified({ suite }: Readonly<{ suite: SuiteId }>) {
  return (
    <div className="mb-6">
      <Loaded suite={suite}>
        {(data) => (
          <div className="flex items-center gap-2 rounded-lg border border-slate-200 bg-slate-50 px-4 py-3 text-sm text-slate-700 dark:border-slate-700 dark:bg-slate-900 dark:text-slate-300">
            <StatusIcon result="pass" className="h-5 w-5" />
            <span>
              Results below are from the latest automated run against{" "}
              <code>main</code> on{" "}
              {data.backends
                .map((backend) => BACKEND_NAMES[backend])
                .join(" and ")}
              , completed{" "}
              <time
                dateTime={data.generatedAt}
                className="font-semibold text-slate-900 dark:text-white"
              >
                {new Date(data.generatedAt).toLocaleString(undefined, {
                  year: "numeric",
                  month: "long",
                  day: "numeric",
                  hour: "numeric",
                  minute: "2-digit",
                  timeZoneName: "short",
                })}
              </time>
              .
            </span>
          </div>
        )}
      </Loaded>
    </div>
  );
}

function Stat({
  label,
  value,
  total,
  note,
  result,
}: Readonly<{
  label: string;
  value: number;
  total?: number;
  note?: React.ReactNode;
  result?: Result;
}>) {
  return (
    <Card className="p-5">
      <div className="flex items-center justify-between">
        <div className="text-xs font-semibold uppercase tracking-wide text-slate-500 dark:text-slate-400">
          {label}
        </div>
        {result ? <StatusIcon result={result} className="h-5 w-5" /> : null}
      </div>
      <div className="mt-2 text-3xl font-semibold text-slate-900 dark:text-white">
        {value.toLocaleString()}
        {total === undefined ? null : (
          <span className="text-lg font-normal text-slate-500 dark:text-slate-400">
            {" "}
            / {total.toLocaleString()}
          </span>
        )}
      </div>
      {note ? (
        <div className="mt-1 text-sm text-slate-600 dark:text-slate-300">
          {note}
        </div>
      ) : null}
    </Card>
  );
}

export function SuiteSummary({ suite }: Readonly<{ suite: SuiteId }>) {
  return (
    <Loaded suite={suite}>
      {(data) => {
        const totals = totalsOf(data);
        const scriptsResult: Result =
          totals.failing > 0 ? "fail" : totals.knownGaps > 0 ? "warn" : "pass";
        return (
          <div className="grid gap-4 lg:grid-cols-3">
            <Stat
              label="TestScripts passing"
              value={totals.scriptsPassing}
              total={totals.scripts}
              result={scriptsResult}
              note={
                totals.knownGaps > 0 ? (
                  <span className="text-amber-700 dark:text-amber-300">
                    {totals.knownGaps} known gap
                    {totals.knownGaps === 1 ? "" : "s"}, {gapRun(data)}
                  </span>
                ) : (
                  "every TestScript passes"
                )
              }
            />
            <Stat
              label="Checks passing"
              value={totals.checksPassing}
              total={totals.checks}
              note="assertions on what the server returned"
            />
            <Stat
              label="Search backends"
              value={data.backends.length}
              note={data.backends
                .map((backend) => BACKEND_NAMES[backend])
                .join(" and ")}
            />
          </div>
        );
      }}
    </Loaded>
  );
}

// ---------------------------------------------------------------------------
// Known gaps
// ---------------------------------------------------------------------------

export function SuiteKnownGaps({ suite }: Readonly<{ suite: SuiteId }>) {
  return (
    <Loaded suite={suite}>
      {(data) => {
        const gaps = gapsByReason(data);
        const failing = data.groups
          .flatMap((group) => group.scripts)
          .filter(
            (script) =>
              !script.knownFailure &&
              worst(script.result, data.backends) === "fail",
          );
        if (gaps.length === 0 && failing.length === 0) {
          return (
            <Card className="flex items-center gap-3 p-4 text-sm">
              <StatusIcon result="pass" className="h-5 w-5" />
              Every TestScript in this suite passes.
            </Card>
          );
        }
        return (
          <Card>
            {failing.length > 0 ? (
              <GapRow
                result="fail"
                reason="Failed in the latest run."
                scripts={failing}
              />
            ) : null}
            {gaps.map((gap) => (
              <GapRow
                key={gap.reason}
                result="warn"
                reason={gap.reason}
                scripts={gap.scripts}
              />
            ))}
          </Card>
        );
      }}
    </Loaded>
  );
}

function GapRow({
  result,
  reason,
  scripts,
}: Readonly<{ result: Result; reason: string; scripts: SuiteScript[] }>) {
  const [open, setOpen] = useState(false);
  return (
    <div className="border-b border-slate-100 p-4 last:border-b-0 dark:border-slate-800">
      <div className="flex items-start gap-3">
        <StatusIcon result={result} className="mt-0.5 h-5 w-5" />
        <div className="min-w-0 flex-1">
          <p className="m-0 text-sm text-slate-800 dark:text-slate-100">
            <Reason text={reason} />
          </p>
          <button
            type="button"
            aria-expanded={open}
            onClick={() => setOpen(!open)}
            className="mt-1.5 inline-flex cursor-pointer items-center gap-1 border-0 bg-transparent p-0 text-xs font-medium text-slate-500 hover:text-slate-900 dark:text-slate-400 dark:hover:text-white"
          >
            <Chevron open={open} />
            {scripts.length} TestScript{scripts.length === 1 ? "" : "s"}
          </button>
          {open ? (
            <ul className="m-0 mt-2 grid list-none gap-1 p-0 text-xs">
              {scripts.map((script) => (
                <li key={script.id} className="min-w-0">
                  <a href={sourceUrl(script)} className="font-medium">
                    {script.title}
                  </a>
                </li>
              ))}
            </ul>
          ) : null}
        </div>
      </div>
    </div>
  );
}

// ---------------------------------------------------------------------------
// The full report
// ---------------------------------------------------------------------------

function matches(script: SuiteScript, query: string): boolean {
  if (!query) return true;
  return [
    script.title,
    script.id,
    script.description ?? "",
    script.subgroup ?? "",
  ]
    .join(" ")
    .toLowerCase()
    .includes(query);
}

function ScriptDetail({
  script,
  report,
}: Readonly<{ script: SuiteScript; report: SuiteReport }>) {
  const { backends } = report;
  // A skipped known gap has no results of its own; one that ran does.
  const skipped = Boolean(script.knownFailure) && !report.knownFailuresRun;
  return (
    <div className="grid gap-3 px-4 py-3 text-xs">
      {script.description ? (
        <p className="m-0 max-w-3xl text-sm leading-relaxed text-slate-700 dark:text-slate-200">
          {script.description}
        </p>
      ) : null}
      {script.knownFailure ? (
        <p className="m-0 rounded-md bg-amber-50 px-3 py-2 text-sm text-amber-900 dark:bg-amber-950/40 dark:text-amber-200">
          <strong>Known gap, {gapRun(report)}.</strong>{" "}
          <Reason text={script.knownFailure} />
        </p>
      ) : null}
      {script.tests.map((test, testIndex) => (
        <div key={`${test.name}-${testIndex}`} className="min-w-0">
          <div className="font-semibold text-slate-900 dark:text-white">
            {test.name}
          </div>
          <div className="mt-1 grid gap-1">
            {test.assertions.map((assertion, index) => (
              <div
                key={`${assertion.label}-${index}`}
                className="flex min-w-0 items-start justify-between gap-3"
              >
                <span className="min-w-0 text-slate-700 dark:text-slate-200">
                  {assertion.label}
                  {assertion.expression &&
                  assertion.expression !== assertion.label ? (
                    <code className="ml-2 break-all text-[11px] text-slate-500 dark:text-slate-400">
                      {assertion.expression}
                    </code>
                  ) : null}
                </span>
                <span className="inline-flex shrink-0 gap-3">
                  {backends.map((backend) => (
                    <span
                      key={backend}
                      className="inline-flex items-center gap-1 text-slate-500"
                    >
                      {backends.length > 1 ? BACKEND_SHORT[backend] : null}
                      <StatusIcon
                        result={
                          skipped
                            ? "warn"
                            : (assertion.results[backend] ?? "not-run")
                        }
                        className="h-3.5 w-3.5"
                      />
                    </span>
                  ))}
                </span>
              </div>
            ))}
          </div>
        </div>
      ))}
      <a href={sourceUrl(script)} className="text-xs font-medium">
        View the TestScript source →
      </a>
    </div>
  );
}

function GroupTable({
  group,
  report,
  query,
  issuesOnly,
}: Readonly<{
  group: SuiteGroup;
  report: SuiteReport;
  query: string;
  issuesOnly: boolean;
}>) {
  const { backends } = report;
  const [open, setOpen] = useState<string>();
  const scripts = group.scripts.filter(
    (script) =>
      matches(script, query) &&
      (!issuesOnly || !passesEverywhere(script.result, backends)),
  );
  if (scripts.length === 0) return null;
  const passing = group.scripts.filter((script) =>
    passesEverywhere(script.result, backends),
  ).length;
  const columns = backends.length + 1;
  // Once a group has subgroups (Setup, a profile), scripts outside them get a
  // heading too, so they don't read as part of the subgroup above.
  const hasSubgroups = group.scripts.some((script) => script.subgroup);
  const headingOf = (script?: SuiteScript) =>
    script
      ? (script.subgroup ?? (hasSubgroups ? "Tests" : undefined))
      : undefined;

  return (
    <Card className="mb-4">
      <div className="flex flex-wrap items-baseline justify-between gap-2 border-b border-slate-200 px-4 py-3 dark:border-slate-700">
        <h3 className="m-0 text-base font-semibold text-slate-900 dark:text-white">
          {group.title}
        </h3>
        <span className="text-xs text-slate-500 dark:text-slate-400">
          {passing} of {group.scripts.length} TestScripts pass
        </span>
      </div>
      <div className={styles.scroll}>
        <table className={styles.table}>
          <thead>
            <tr>
              <th>TestScript</th>
              {backends.map((backend) => (
                <th key={backend} className={styles.numeric}>
                  {BACKEND_NAMES[backend]}
                </th>
              ))}
            </tr>
          </thead>
          <tbody>
            {scripts.map((script, index) => {
              const expanded = open === script.id;
              const heading = headingOf(script);
              const newSubgroup =
                heading !== undefined &&
                heading !== headingOf(scripts[index - 1]);
              return (
                <React.Fragment key={script.id}>
                  {newSubgroup ? (
                    <tr>
                      <td colSpan={columns} className={styles.group}>
                        <span className="text-sm font-semibold text-slate-900 dark:text-white">
                          {heading}
                        </span>
                      </td>
                    </tr>
                  ) : null}
                  <tr>
                    <td>
                      <button
                        type="button"
                        aria-expanded={expanded}
                        onClick={() =>
                          setOpen(expanded ? undefined : script.id)
                        }
                        className="inline-flex cursor-pointer items-start gap-2 border-0 bg-transparent p-0 text-left text-sm text-slate-900 hover:text-brand-800 dark:text-white dark:hover:text-brand-300"
                      >
                        <Chevron open={expanded} />
                        <span>{script.title}</span>
                      </button>
                    </td>
                    {backends.map((backend) => {
                      const result = script.result[backend] ?? "not-run";
                      return (
                        <td key={backend} className={styles.numeric}>
                          <span className="inline-flex items-center justify-end gap-1.5">
                            <StatusIcon result={result} />
                            <span
                              className={`${RESULT_TONES[result]} ${styles.optional}`}
                            >
                              {RESULT_WORDS[result]}
                            </span>
                          </span>
                        </td>
                      );
                    })}
                  </tr>
                  {expanded ? (
                    <tr>
                      <td colSpan={columns} className={styles.detail}>
                        <ScriptDetail script={script} report={report} />
                      </td>
                    </tr>
                  ) : null}
                </React.Fragment>
              );
            })}
          </tbody>
        </table>
      </div>
    </Card>
  );
}

function Report({ data }: Readonly<{ data: SuiteReport }>) {
  const [query, setQuery] = useState("");
  const [issuesOnly, setIssuesOnly] = useState(false);
  const deferredQuery = useDeferredValue(query.trim().toLowerCase());
  const shown = useMemo(
    () =>
      data.groups
        .flatMap((group) => group.scripts)
        .filter(
          (script) =>
            matches(script, deferredQuery) &&
            (!issuesOnly || !passesEverywhere(script.result, data.backends)),
        ).length,
    [data, deferredQuery, issuesOnly],
  );
  const total = data.groups.reduce(
    (sum, group) => sum + group.scripts.length,
    0,
  );

  return (
    <>
      <Card className="mb-4">
        <div className="flex flex-wrap items-center gap-3 p-3">
          <input
            type="search"
            value={query}
            onChange={(event) => setQuery(event.target.value)}
            placeholder="Filter TestScripts…"
            aria-label="Filter TestScripts"
            className="min-w-0 flex-1 rounded-md border border-slate-300 bg-white px-3 py-1.5 text-sm text-slate-900 outline-none focus:border-slate-500 dark:border-slate-600 dark:bg-slate-950 dark:text-white"
          />
          <label className="inline-flex cursor-pointer items-center gap-2 text-sm text-slate-700 dark:text-slate-300">
            <input
              type="checkbox"
              checked={issuesOnly}
              onChange={(event) => setIssuesOnly(event.target.checked)}
            />
            <span>Only show issues</span>
          </label>
          <span className="text-xs text-slate-500 dark:text-slate-400">
            {shown} of {total}
          </span>
        </div>
      </Card>
      {data.groups.map((group) => (
        <GroupTable
          key={group.id}
          group={group}
          report={data}
          query={deferredQuery}
          issuesOnly={issuesOnly}
        />
      ))}
      {shown === 0 ? (
        <p className="text-sm text-slate-500 dark:text-slate-400">
          No TestScripts match.
        </p>
      ) : null}
    </>
  );
}

export function SuiteReportTable({ suite }: Readonly<{ suite: SuiteId }>) {
  return <Loaded suite={suite}>{(data) => <Report data={data} />}</Loaded>;
}

// ---------------------------------------------------------------------------
// Overview of every suite
// ---------------------------------------------------------------------------

const SUITE_BLURBS: Record<string, string> = {
  core: "Create, read, update, patch, delete and search for every FHIR R4 resource type, and the search features each parameter type supports.",
  "us-core":
    "Every US Core 9.0.0 example resource written with its profile claimed, so the server validates it on the way in, and a conformance TestScript per profile.",
  "us-core-7":
    "The same checks for US Core 7.0.0, the version the ONC (g)(10) test kit certifies against: its examples written with their profiles claimed, and a conformance TestScript per profile.",
  g10: "The checks of the ONC (g)(10) Standardized API test kit that can run unattended: SMART launch scenarios, the US Core 7 single patient API, Bulk Data and fine-grained scopes.",
  "access-control":
    "What patients, users and backend clients can reach under SMART patient, user and system scopes, and EHR launches.",
  "sql-on-fhir":
    "The SQL on FHIR v2 shared test suite: ViewDefinitions run with $viewdefinition-run, checking the rows that come back and the views that must be refused.",
  "carin-bb":
    "Every CARIN Blue Button 2.2.0 example resource written with its profile claimed, and a conformance TestScript per claims data profile with the search parameters the guide adds.",
};

export function ConformanceOverview() {
  const { suites } = useHomepageFacts();
  const checks = suites.reduce((sum, suite) => sum + suite.assertions, 0);
  const passing = suites.reduce(
    (sum, suite) => sum + suite.assertionsPassing,
    0,
  );

  return (
    <div className={styles.root}>
      <p className="mb-6 text-base text-slate-700 dark:text-slate-200">
        <strong className="text-slate-900 dark:text-white">
          {passing.toLocaleString()} of {checks.toLocaleString()}
        </strong>{" "}
        automated checks pass across {suites.length} test suites.
      </p>
      <div className="grid gap-4 md:grid-cols-2">
        {suites.map((suite) => (
          <Card key={suite.id} className="flex flex-col p-5">
            <div className="flex items-center justify-between gap-3">
              <h3 className="m-0 text-lg font-semibold text-slate-900 dark:text-white">
                {suite.title}
              </h3>
              <StatusIcon
                result={
                  suite.testScriptsPassing === suite.testScripts
                    ? "pass"
                    : "warn"
                }
                className="h-5 w-5"
              />
            </div>
            <p className="m-0 mt-2 text-sm leading-relaxed text-slate-600 dark:text-slate-300">
              {SUITE_BLURBS[suite.id]}
            </p>
            <dl className="m-0 mt-4 grid grid-cols-2 gap-3 border-t border-slate-100 pt-4 dark:border-slate-800">
              <div className="flex flex-col-reverse">
                <dt className="text-xs text-slate-500 dark:text-slate-400">
                  checks passing
                </dt>
                <dd className="m-0 text-xl font-semibold text-slate-900 dark:text-white">
                  {suite.assertionsPassing.toLocaleString()}
                  <span className="text-sm font-normal text-slate-500">
                    {" "}
                    / {suite.assertions.toLocaleString()}
                  </span>
                </dd>
              </div>
              <div className="flex flex-col-reverse">
                <dt className="text-xs text-slate-500 dark:text-slate-400">
                  TestScripts passing
                  {suite.knownGaps > 0
                    ? `, ${suite.knownGaps} known gap${suite.knownGaps === 1 ? "" : "s"}`
                    : ""}
                </dt>
                <dd className="m-0 text-xl font-semibold text-slate-900 dark:text-white">
                  {suite.testScriptsPassing}
                  <span className="text-sm font-normal text-slate-500">
                    {" "}
                    / {suite.testScripts}
                  </span>
                </dd>
              </div>
            </dl>
            <div className="mt-auto pt-4 text-sm">
              <Link to={suite.page} className="font-medium">
                See the {suite.title} results →
              </Link>
            </div>
          </Card>
        ))}
      </div>
    </div>
  );
}
