import React from "react";
import clsx from "clsx";

import { Icon, LogoMark } from "../site/icons";
import { riseDelay } from "../site/motion";
import site from "../site/styles.module.css";
import styles from "./styles.module.css";

/**
 * The hero's product panel: one agent question, followed from the MCP call
 * through every check the server applies before it answers.
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

// When each piece arrives after page load, in milliseconds.
const QUESTION_AT = 700;
const TOOL_AT = 1150;
const TRACE_AT = 1450;
const TRACE_STEP = 240;
const ANSWER_AT = TRACE_AT + TRACE.length * TRACE_STEP + 150;

function Label({ children }: Readonly<{ children: React.ReactNode }>) {
  return (
    <div className="text-[0.75rem] font-semibold uppercase tracking-[0.14em] text-slate-300">
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
        className="overflow-hidden rounded-2xl border border-white/10 bg-deep-900 text-left shadow-[0_40px_80px_-30px_rgb(2_20_22/0.75)] ring-1 ring-black/20"
      >
        <div className="flex items-center gap-3 border-b border-white/10 bg-white/[0.03] px-5 py-3.5 text-sm">
          <LogoMark className="h-4 w-4 text-brand-400" />
          <span className="font-semibold text-white">care-copilot</span>
          <span className="hidden font-mono text-[0.8125rem] text-slate-300 sm:inline">
            acme-health / production
          </span>
          <span className="ml-auto inline-flex items-center gap-2.5 text-[0.8125rem] font-medium text-slate-300">
            <span className={styles.live} aria-hidden="true" />
            MCP connected
          </span>
        </div>

        <div className="grid md:grid-cols-[minmax(0,5fr)_minmax(0,6fr)]">
          <div className="flex min-w-0 flex-col gap-4 border-b border-white/10 p-5 md:border-b-0 md:border-r md:p-7">
            <div
              className={clsx(site.rise, "space-y-2")}
              style={riseDelay(QUESTION_AT)}
            >
              <Label>Clinician</Label>
              <p className="rounded-xl rounded-tl-sm bg-white/[0.07] px-4 py-3 text-[1.0625rem] leading-relaxed text-white">
                How has David Williams&apos; blood pressure trended over the
                last 30 days?
              </p>
            </div>

            <div
              className={clsx(
                site.rise,
                "flex items-center gap-2.5 rounded-lg border border-white/10 px-3 py-2 text-[0.8125rem] text-slate-300",
              )}
              style={riseDelay(TOOL_AT)}
            >
              <Icon name="tools" className="h-4 w-4 text-brand-400" />
              <span className="font-mono text-brand-200">fhir_r4_search</span>
              <span className="text-slate-300">Observation</span>
              <span className="ml-auto whitespace-nowrap text-slate-300">
                4 results
              </span>
            </div>

            <div
              className={clsx(site.rise, "space-y-2")}
              style={riseDelay(ANSWER_AT)}
            >
              <Label>Agent</Label>
              <div className="rounded-xl rounded-tl-sm border border-brand-400/20 bg-brand-500/10 px-4 py-3">
                <p className="text-[1.0625rem] leading-relaxed text-white">
                  Four readings since September 4, trending down from 134/88 to
                  122/80 mmHg.
                </p>
                <dl className="mt-3 grid grid-cols-4 gap-1.5 sm:gap-2">
                  {READINGS.map((reading) => (
                    <div
                      key={reading.date}
                      className="whitespace-nowrap rounded-md bg-deep-950/70 px-1 py-1.5 text-center"
                    >
                      <dt className="text-[0.6875rem] text-slate-300">
                        {reading.date}
                      </dt>
                      <dd className="m-0 font-mono text-[0.75rem] text-brand-100 sm:text-[0.8125rem]">
                        {reading.value}
                      </dd>
                    </div>
                  ))}
                </dl>
              </div>
            </div>
          </div>

          <div className="min-w-0 p-5 md:p-7">
            <div className="flex flex-wrap items-baseline justify-between gap-x-4 gap-y-1">
              <Label>What the server did</Label>
              <span className="font-mono text-[0.75rem] text-slate-300">
                POST /w/acme-health/production/api/v1/mcp
              </span>
            </div>

            <ol className="mt-5 list-none space-y-0 p-0">
              {TRACE.map((step, index) => {
                const delay = riseDelay(TRACE_AT + index * TRACE_STEP);
                const last = index === TRACE.length - 1;
                return (
                  <li
                    key={step.title}
                    className={clsx(site.rise, "relative flex gap-3.5 pb-4")}
                    style={delay}
                  >
                    {last ? null : (
                      <span
                        className="absolute bottom-0 left-[0.6875rem] top-6 w-px bg-white/10"
                        aria-hidden="true"
                      />
                    )}
                    <span
                      className={clsx(
                        styles.tick,
                        "mt-0.5 flex h-[1.4375rem] w-[1.4375rem] shrink-0 items-center justify-center rounded-full bg-brand-500/15 text-brand-300 ring-1 ring-brand-400/40",
                      )}
                      style={delay}
                    >
                      <Icon name="check" className="h-3 w-3" />
                    </span>
                    <div className="min-w-0">
                      <div className="text-[0.9375rem] font-semibold leading-6 text-white">
                        {step.title}
                      </div>
                      <div className="font-mono text-[0.8125rem] leading-5 text-slate-300 [overflow-wrap:anywhere]">
                        {step.detail}
                      </div>
                    </div>
                  </li>
                );
              })}
            </ol>

            <div
              className={clsx(
                site.rise,
                "flex items-center gap-2.5 rounded-lg bg-white/[0.05] px-3.5 py-2.5 font-mono text-[0.8125rem] text-slate-300",
              )}
              style={riseDelay(ANSWER_AT - 150)}
            >
              <span className="h-1.5 w-1.5 rounded-full bg-brand-400" />
              200 OK
              <span className="text-slate-300">
                Bundle, searchset, 4 entries
              </span>
            </div>
          </div>
        </div>
      </div>
    </>
  );
}
