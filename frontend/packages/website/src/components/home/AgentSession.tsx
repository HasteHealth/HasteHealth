import React, { type CSSProperties } from "react";
import clsx from "clsx";

import { Icon, LogoMark } from "../site/icons";
import styles from "./styles.module.css";

/**
 * The AI section's product panel: one agent question, followed from the MCP
 * call through every check the server applies before it answers.
 *
 * The steps are the server's real request path for an MCP tool call, in order
 * (token, SMART scope, access policy, the search itself, the AuditEvent), so
 * the picture is a claim the docs back up, not decoration.
 */
const TRACE = [
  { title: "Token verified", detail: "OAuth 2.0 client care-copilot" },
  { title: "Scope checked", detail: "user/Observation.read" },
  {
    title: "Access policy evaluated",
    detail: "AccessPolicyV2/care-team-read permits",
  },
  {
    title: "Search executed",
    detail: "Observation?code=85354-9&date=ge2026-09-04",
  },
  { title: "AuditEvent recorded", detail: "search-type, success" },
];

const READINGS = [
  { date: "Sep 8", value: "134/88" },
  { date: "Sep 17", value: "129/85" },
  { date: "Sep 26", value: "126/82" },
  { date: "Oct 2", value: "122/80" },
];

// When each piece arrives once the panel scrolls into view, in milliseconds.
const QUESTION_AT = 0;
const TOOL_AT = 350;
const TRACE_AT = 600;
const TRACE_STEP = 220;
const ANSWER_AT = TRACE_AT + TRACE.length * TRACE_STEP + 150;

function stepDelay(ms: number): CSSProperties {
  return { "--step-delay": `${ms}ms` } as CSSProperties;
}

function Label({ children }: Readonly<{ children: React.ReactNode }>) {
  return (
    <div className="text-[0.75rem] font-semibold uppercase tracking-[0.14em] text-slate-500">
      {children}
    </div>
  );
}

export default function AgentSession() {
  return (
    <>
      {/* A screen reader gets the picture described once, instead of a mock
          interface read out label by label. */}
      <p className="sr-only">
        Example agent session. A clinician asks how a patient&apos;s blood
        pressure has trended over the last 30 days. The agent calls the
        fhir_r4_search MCP tool, and the server verifies its token, checks its
        SMART scope, evaluates its access policy, runs the search and records an
        AuditEvent. The agent answers with four readings, trending down from 134
        over 88 to 122 over 80.
      </p>
      <div
        aria-hidden="true"
        className={clsx(
          styles.session,
          "overflow-hidden rounded-2xl border border-slate-200 bg-white text-left shadow-[0_1px_2px_rgb(15_23_42/0.04),0_24px_48px_-28px_rgb(15_23_42/0.18)]",
        )}
        data-reveal=""
      >
        <div className="flex items-center gap-3 border-b border-slate-200 bg-slate-50 px-5 py-3.5 text-sm">
          <LogoMark className="h-4 w-4 text-brand-700" />
          <span className="font-semibold text-ink-950">care-copilot</span>
          <span className="hidden font-mono text-[0.8125rem] text-slate-500 sm:inline">
            acme-health / production
          </span>
          <span className="ml-auto inline-flex items-center gap-2.5 text-[0.8125rem] font-medium text-slate-600">
            <span className={styles.live} aria-hidden="true" />
            MCP connected
          </span>
        </div>

        <div className="grid xl:grid-cols-[minmax(0,5fr)_minmax(0,6fr)]">
          <div className="flex min-w-0 flex-col gap-4 border-b border-slate-200 p-5 md:p-6 xl:border-b-0 xl:border-r">
            <div
              className={clsx(styles.step, "space-y-2")}
              style={stepDelay(QUESTION_AT)}
            >
              <Label>Clinician</Label>
              <p className="m-0 rounded-xl rounded-tl-sm bg-slate-100 px-4 py-3 text-[1rem] leading-relaxed text-ink-950">
                How has David Williams&apos; blood pressure trended over the
                last 30 days?
              </p>
            </div>

            <div
              className={clsx(
                styles.step,
                "flex flex-wrap items-center gap-x-2.5 gap-y-1 rounded-lg border border-slate-200 px-3 py-2 text-[0.8125rem] text-slate-600",
              )}
              style={stepDelay(TOOL_AT)}
            >
              <Icon name="tools" className="h-4 w-4 text-brand-700" />
              <span className="font-mono text-brand-800">fhir_r4_search</span>
              <span>Observation</span>
              <span className="ml-auto text-slate-500">4 results</span>
            </div>

            <div
              className={clsx(styles.step, "space-y-2")}
              style={stepDelay(ANSWER_AT)}
            >
              <Label>Agent</Label>
              <div className="rounded-xl rounded-tl-sm border border-brand-200 bg-brand-50 px-4 py-3">
                <p className="m-0 text-[1rem] leading-relaxed text-ink-950">
                  Four readings since September 4, trending down from 134/88 to
                  122/80 mmHg.
                </p>
                <dl className="mt-3 grid grid-cols-[repeat(auto-fit,minmax(3.25rem,1fr))] gap-1.5">
                  {READINGS.map((reading) => (
                    <div
                      key={reading.date}
                      className="min-w-0 rounded-md border border-brand-200/70 bg-white px-0.5 py-1.5 text-center"
                    >
                      <dt className="text-[0.6875rem] text-slate-500">
                        {reading.date}
                      </dt>
                      <dd className="m-0 font-mono text-[0.75rem] text-ink-950">
                        {reading.value}
                      </dd>
                    </div>
                  ))}
                </dl>
              </div>
            </div>
          </div>

          <div className="min-w-0 p-5 md:p-6">
            <div className="flex flex-wrap items-baseline justify-between gap-x-4 gap-y-1">
              <Label>What the server did</Label>
              <span className="font-mono text-[0.75rem] text-slate-500">
                POST /w/acme-health/production/api/v1/mcp
              </span>
            </div>

            <ol className="mt-5 list-none space-y-0 p-0">
              {TRACE.map((step, index) => {
                const delay = stepDelay(TRACE_AT + index * TRACE_STEP);
                const last = index === TRACE.length - 1;
                return (
                  <li
                    key={step.title}
                    className={clsx(styles.step, "relative flex gap-3.5 pb-4")}
                    style={delay}
                  >
                    {last ? null : (
                      <span
                        className="absolute bottom-0 left-[0.6875rem] top-6 w-px bg-slate-200"
                        aria-hidden="true"
                      />
                    )}
                    <span
                      className={clsx(
                        styles.tick,
                        "mt-0.5 flex h-[1.4375rem] w-[1.4375rem] shrink-0 items-center justify-center rounded-full bg-brand-50 text-brand-700 ring-1 ring-brand-200",
                      )}
                      style={delay}
                    >
                      <Icon name="check" className="h-3 w-3" />
                    </span>
                    <div className="min-w-0">
                      <div className="text-[0.9375rem] font-semibold leading-6 text-ink-950">
                        {step.title}
                      </div>
                      <div className="font-mono text-[0.8125rem] leading-5 text-slate-500 [overflow-wrap:anywhere]">
                        {step.detail}
                      </div>
                    </div>
                  </li>
                );
              })}
            </ol>

            <div
              className={clsx(
                styles.step,
                "flex items-center gap-2.5 rounded-lg bg-slate-50 px-3.5 py-2.5 font-mono text-[0.8125rem] text-slate-600",
              )}
              style={stepDelay(ANSWER_AT - 150)}
            >
              <span className="h-1.5 w-1.5 rounded-full bg-brand-500" />
              200 OK
              <span className="text-slate-500">
                Bundle, searchset, 4 entries
              </span>
            </div>
          </div>
        </div>
      </div>
    </>
  );
}
