import React from "react";
import Link from "@docusaurus/Link";

import { Icon, LogoMark } from "./site/icons";
import styles from "./DataFlowDiagram.module.css";

type Endpoint = {
  label: string;
  /** The protocol the data travels over. Shown only when there is room. */
  via: string;
  href: string;
  /** The AI consumers, which the diagram picks out in the brand colour. */
  accent?: boolean;
};

const EHR = "/docs/integration/healthcare_systems/ehr";
const FHIR_API = "/docs/api/rest_api/fhir/intro";

const sources: Endpoint[] = [
  { label: "Epic", via: "FHIR R4", href: EHR },
  { label: "Oracle Health", via: "FHIR R4", href: EHR },
  { label: "MEDITECH", via: "FHIR, HL7v2", href: EHR },
  { label: "athenahealth", via: "FHIR R4", href: EHR },
  { label: "HL7v2 feeds", via: "MLLP", href: "/docs/guides/hl7v2" },
  { label: "Any FHIR R4 API", via: "REST", href: FHIR_API },
];

const consumers: Endpoint[] = [
  {
    label: "AI agents",
    via: "MCP",
    href: "/docs/api/rest_api/model_context_protocol/endpoint",
    accent: true,
  },
  {
    label: "Claude, Gemini",
    via: "MCP",
    href: "/docs/category/ai",
    accent: true,
  },
  {
    label: "Apps and portals",
    via: "SMART",
    href: "/docs/auth/authentication/smart_on_fhir",
  },
  {
    label: "Backend services",
    via: "OAuth 2.0",
    href: "/docs/auth/authentication/grant_types/client_credentials",
  },
  {
    label: "Analytics and BI",
    via: "SQL on FHIR",
    href: "/docs/guides/sql_on_fhir",
  },
  { label: "Partner systems", via: "FHIR R4", href: FHIR_API },
];

const core = [
  "Repository and search",
  "OAuth 2.0 and SMART",
  "Access policies",
  "Audit trail and history",
];

/*
 * The connector geometry, in SVG units of 1/16 rem.
 *
 * Each chip is 2.75rem tall with a 0.625rem gap, so chip `i` is centred at
 * ROW / 2 + i * PITCH. The connector is drawn at exactly that size and scales
 * with the root font size, which keeps every line landing on the middle of its
 * chip without measuring anything in the browser.
 */
const ROW = 44;
const PITCH = 54;
const WIDTH = 96;
/** How far apart the lines sit where they meet the core. */
const FAN = 9;

function stackHeight(count: number): number {
  return count * PITCH - (PITCH - ROW);
}

/**
 * This component is also used in the docs (overview/what_is_haste_health),
 * outside the homepage's Tailwind reset and in a much narrower column. So it
 * lays itself out with container queries, not viewport breakpoints, and it
 * leans on no reset: every element sets its own margin, padding and colour.
 */
export default function DataFlowDiagram() {
  return (
    <div className="@container">
      {/* Stacked, the grid is one column in source order. Side by side, every
          item is placed by hand: labels on row 1, the diagram on row 2. */}
      <div className="grid grid-cols-1 items-center gap-y-4 @3xl:grid-cols-[minmax(0,1fr)_6rem_auto_6rem_minmax(0,1fr)] @3xl:gap-y-3">
        <ColumnLabel className="@3xl:col-start-1 @3xl:row-start-1">
          Comes in from
        </ColumnLabel>
        <Endpoints
          items={sources}
          className="@3xl:col-start-1 @3xl:row-start-2"
        />
        <Connector
          count={sources.length}
          direction="in"
          className="@3xl:col-start-2 @3xl:row-start-2"
        />

        <div className="flex justify-center @3xl:col-start-3 @3xl:row-start-2">
          <Core />
        </div>

        <Connector
          count={consumers.length}
          direction="out"
          className="@3xl:col-start-4 @3xl:row-start-2"
        />
        <ColumnLabel className="@3xl:col-start-5 @3xl:row-start-1">
          Goes out to
        </ColumnLabel>
        <Endpoints
          items={consumers}
          className="@3xl:col-start-5 @3xl:row-start-2"
        />
      </div>
    </div>
  );
}

function ColumnLabel({
  children,
  className = "",
}: Readonly<{ children: React.ReactNode; className?: string }>) {
  return (
    <div
      className={`text-center text-[0.75rem] font-semibold uppercase tracking-[0.14em] text-slate-500 @3xl:text-left ${className}`}
    >
      {children}
    </div>
  );
}

function Endpoints({
  items,
  className = "",
}: Readonly<{ items: Endpoint[]; className?: string }>) {
  return (
    <div
      className={`mx-auto grid w-full max-w-md grid-cols-2 gap-[0.625rem] @3xl:max-w-none @3xl:grid-cols-1 ${className}`}
    >
      {items.map((item) => (
        <Link
          key={item.label}
          to={item.href}
          className={`group/chip flex h-[2.75rem] items-center gap-2.5 rounded-lg border bg-white px-3 text-[0.9375rem] font-medium no-underline shadow-[0_1px_2px_rgb(15_23_42/0.04)] transition-colors hover:border-brand-500 hover:no-underline ${
            item.accent
              ? "border-brand-300 text-brand-950 hover:text-brand-950"
              : "border-slate-200 text-slate-800 hover:text-slate-950"
          }`}
        >
          <span
            className={`h-2 w-2 shrink-0 rounded-[0.1875rem] ${
              item.accent
                ? "bg-brand-600"
                : "bg-slate-300 group-hover/chip:bg-brand-500"
            }`}
            aria-hidden="true"
          />
          <span className="truncate">{item.label}</span>
          <span className="ml-auto hidden whitespace-nowrap pl-2 font-mono text-[0.75rem] uppercase tracking-wider text-slate-500 @5xl:inline">
            {item.via}
          </span>
        </Link>
      ))}
    </div>
  );
}

function Core() {
  return (
    <div className="relative">
      <div
        className="absolute -inset-6 rounded-[2rem] bg-brand-300/25 blur-2xl"
        aria-hidden="true"
      />
      <div className="relative w-[14rem] rounded-2xl bg-deep-950 p-5 text-white shadow-[0_24px_48px_-20px_rgb(2_20_22/0.55)] ring-1 ring-white/10 @5xl:w-[17.5rem] @5xl:p-6">
        <LogoMark className="h-8 w-8 text-brand-400" />
        <div className="mt-3 text-lg font-semibold leading-tight text-white">
          Haste Health
        </div>
        <div className="mt-1 text-[0.8125rem] leading-snug text-slate-400">
          FHIR R4 clinical data layer
        </div>
        <div className="mt-4 space-y-2 border-t border-white/10 pt-4">
          {core.map((line) => (
            <div
              key={line}
              className="flex items-center gap-2 text-[0.8125rem] leading-snug text-slate-200"
            >
              <Icon name="check" className="h-3.5 w-3.5 text-brand-400" />
              {line}
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}

/**
 * The lines between a column of chips and the core, with a dot travelling
 * along each in the direction the data flows. Side by side it is a fan of
 * curves; stacked on a narrow screen it is a single short vertical line.
 */
function Connector({
  count,
  direction,
  className = "",
}: Readonly<{ count: number; direction: "in" | "out"; className?: string }>) {
  const height = stackHeight(count);
  const middle = height / 2;

  const paths = Array.from({ length: count }, (_, index) => {
    const chip = ROW / 2 + index * PITCH;
    const port = middle + (index - (count - 1) / 2) * FAN;
    const [from, to] = direction === "in" ? [chip, port] : [port, chip];
    return `M0 ${from} C ${WIDTH * 0.55} ${from}, ${WIDTH * 0.45} ${to}, ${WIDTH} ${to}`;
  });

  return (
    <>
      <svg
        viewBox={`0 0 ${WIDTH} ${height}`}
        className={`hidden overflow-visible @3xl:block ${className}`}
        style={{ width: `${WIDTH / 16}rem`, height: `${height / 16}rem` }}
        fill="none"
        aria-hidden="true"
      >
        {paths.map((path, index) => (
          <g key={path}>
            <path d={path} className="stroke-slate-300" strokeWidth="1.25" />
            <circle r="2.5" className={`fill-brand-600 ${styles.dot}`}>
              <animateMotion
                dur="2.8s"
                begin={`${(index * 0.47).toFixed(2)}s`}
                repeatCount="indefinite"
                path={path}
              />
              <animate
                attributeName="opacity"
                values="0;1;1;0"
                keyTimes="0;0.12;0.88;1"
                dur="2.8s"
                begin={`${(index * 0.47).toFixed(2)}s`}
                repeatCount="indefinite"
              />
            </circle>
          </g>
        ))}
      </svg>

      <div
        className="mx-auto flex h-9 w-0.5 overflow-hidden rounded-full bg-slate-200 @3xl:hidden"
        aria-hidden="true"
      >
        <div
          className={`h-1/3 w-full rounded-full bg-brand-600 ${styles.beam}`}
        />
      </div>
    </>
  );
}
