import React, { ReactNode, useEffect, useState } from "react";
import Link from "@docusaurus/Link";
import Layout from "@theme/Layout";
import Heading from "@theme/Heading";
import DataFlowDiagram from "@site/src/components/DataFlowDiagram";
import McpPreview from "@site/src/components/McpPreview";

const buildCards = [
  {
    title: "Open Source, Self-Hosted",
    body: "Apache-2.0 licensed. docker compose up runs the full stack locally, with pre-built binaries and container images for production. Own your infrastructure and your data from day one.",
    href: "/docs/getting_started/quick_start",
  },
  {
    title: "MCP-Native AI Access",
    body: "14 tools generated live from your server's CapabilityStatement give agents structured search, read, write, and schema-discovery over FHIR data.",
    href: "/docs/api/rest_api/model_context_protocol/tools",
  },
  {
    title: "Ship Logic Without Forking the Server",
    body: "Custom operations run as sandboxed TypeScript in an embedded Deno runtime, stored and versioned as FHIR OperationDefinition resources. Extend the platform, not a private fork of it.",
    href: "/docs/reference/fhir/model/resources/OperationDefinition",
  },
  {
    title: "Analytics Without a Data Warehouse",
    body: "SQL-on-FHIR ViewDefinitions flatten resources into CSV, JSON, or NDJSON on demand, with a visual editor in the Admin App. Point a BI tool at clinical data without building an ETL pipeline.",
    href: "/docs/guides/sql_on_fhir",
  },
  {
    title: "Fine-Grained, Auditable Access",
    body: "Attribute-based policies scope agents and users down to compartments and conditions. Every request can emit a real FHIR AuditEvent, and resource history is immutable at the storage layer.",
    href: "/docs/auth/authorization/access_control",
  },
  {
    title: "Multi-Tenant From Day One",
    body: "Tenant and Project scoping reach all the way down. Every query is filtered by tenant and project at the storage and search layers, so one deployment can serve every customer without re-architecting later.",
    href: "/docs/core_concepts/platform_architecture",
  },
];

const audiences = [
  {
    title: "Digital Health Startups",
    body: "Skip months of FHIR plumbing and compliance scaffolding. Self-host under Apache-2.0 and spend your runway on your product, not your data layer.",
    href: "/docs/getting_started/quick_start",
  },
  {
    title: "Provider & Care Platforms",
    body: "Normalize Epic, Cerner, and HL7v2 feeds into one API for patient timelines, care coordination, and clinical workflows.",
    href: "/docs/integration/healthcare_systems/ehr",
  },
  {
    title: "Payers & Risk Platforms",
    body: "Support eligibility, prior authorization, and claims-adjacent workflows on FHIR-first APIs instead of brittle X12 glue code.",
    href: "/docs/integration/healthcare_systems/payers_insurance",
  },
  {
    title: "AI & Agent Builders",
    body: "A headless data layer means agents get first-class, structured access — not a scraped UI or a bolted-on integration.",
    href: "/docs/category/ai",
  },
];

const outcomeStats = [
  { value: "<10ms", label: "Create and update latency" },
  { value: ">25k/s", label: "Writes per second, 10 threads" },
  { value: "<50ms", label: "Typical search response" },
  { value: "<100MB", label: "Memory footprint per instance" },
];

function SectionTitle(
  props: Readonly<{ title: string; subtitle?: string; eyebrow?: string }>,
) {
  return (
    <div className="space-y-4">
      {props.eyebrow ? (
        <div className="inline-flex items-center rounded-full border border-brand-200 bg-brand-50 px-3 py-1 text-xs font-semibold uppercase tracking-[0.08em] text-brand-800">
          {props.eyebrow}
        </div>
      ) : null}
      <Heading
        as="h2"
        className="text-3xl md:text-4xl font-bold tracking-[-0.02em] leading-[1.15] text-brand-950"
      >
        {props.title}
      </Heading>
      {props.subtitle ? (
        <p className="text-base md:text-lg leading-relaxed text-slate-700 max-w-2xl">
          {props.subtitle}
        </p>
      ) : null}
    </div>
  );
}

type CommandSegment = {
  text: string;
  highlight?: boolean;
};

const COMMAND_SEGMENTS: CommandSegment[] = [
  { text: " haste-health api search-type " },
  { text: "Patient", highlight: true },
  { text: " " },
  { text: '"name=chen&_count=1"', highlight: true },
];

const COMMAND_LENGTH = COMMAND_SEGMENTS.reduce(
  (sum, seg) => sum + seg.text.length,
  0,
);

const TYPE_MIN_DELAY_MS = 8;
const TYPE_MAX_DELAY_MS = 18;
const RESPOND_DELAY_MS = 120;
const START_DELAY_MS = 400;

function renderCommand(typed: number) {
  let offset = 0;
  let cursorPlaced = false;
  return COMMAND_SEGMENTS.map((seg, i) => {
    const start = offset;
    offset += seg.text.length;
    const localTyped = Math.max(0, Math.min(seg.text.length, typed - start));
    const visible = seg.text.slice(0, localTyped);
    const hidden = seg.text.slice(localTyped);
    const showCursorHere = !cursorPlaced && hidden.length > 0;
    if (showCursorHere) cursorPlaced = true;
    return (
      <React.Fragment key={i}>
        {seg.highlight ? (
          <span className="text-brand-300">{visible}</span>
        ) : (
          visible
        )}
        {showCursorHere ? (
          <span className="hero-cursor" aria-hidden="true">
            ▋
          </span>
        ) : null}
        {hidden ? <span className="opacity-0">{hidden}</span> : null}
      </React.Fragment>
    );
  });
}

function ResponseBody(props: Readonly<{ visible: boolean }>) {
  return (
    <span
      className={
        props.visible
          ? "hero-response-in inline-block"
          : "inline-block opacity-0"
      }
    >
      {"{"}
      {"\n"}
      {"  "}
      <span className="text-brand-300">"resourceType"</span>: "Bundle",
      {"\n"}
      {"  "}
      <span className="text-brand-300">"type"</span>: "searchset",{"\n"}
      {"  "}
      <span className="text-brand-300">"total"</span>: 1,{"\n"}
      {"  "}
      <span className="text-brand-300">"entry"</span>: [{"{"}
      {"\n"}
      {"    "}
      <span className="text-brand-300">"resource"</span>: {"{"}
      {"\n"}
      {"      "}
      <span className="text-brand-300">"resourceType"</span>: "Patient",
      {"\n"}
      {"      "}
      <span className="text-brand-300">"id"</span>: "xufn84vpa69b_998c1",
      {"\n"}
      {"      "}
      <span className="text-brand-300">"name"</span>: [{"{"}{" "}
      <span className="text-brand-300">"family"</span>: "Chen",{" "}
      <span className="text-brand-300">"given"</span>: ["Maya"] {"}"}]{"\n"}
      {"    "}
      {"}"}
      {"\n"}
      {"  "}
      {"}"}]{"\n"}
      {"}"}
    </span>
  );
}

function HeroSnippet() {
  const [typed, setTyped] = useState(COMMAND_LENGTH);
  const [phase, setPhase] = useState<"typing" | "responding" | "output">(
    "output",
  );

  useEffect(() => {
    const prefersReducedMotion = window.matchMedia(
      "(prefers-reduced-motion: reduce)",
    ).matches;
    if (prefersReducedMotion) return;

    let timeoutId: ReturnType<typeof setTimeout>;
    let cancelled = false;

    function typeStep(current: number) {
      if (cancelled) return;
      if (current >= COMMAND_LENGTH) {
        setPhase("responding");
        timeoutId = setTimeout(() => {
          if (cancelled) return;
          setPhase("output");
        }, RESPOND_DELAY_MS);
        return;
      }
      setTyped(current + 1);
      const delay =
        TYPE_MIN_DELAY_MS +
        Math.random() * (TYPE_MAX_DELAY_MS - TYPE_MIN_DELAY_MS);
      timeoutId = setTimeout(() => typeStep(current + 1), delay);
    }

    setTyped(0);
    setPhase("typing");
    timeoutId = setTimeout(() => typeStep(0), START_DELAY_MS);

    return () => {
      cancelled = true;
      clearTimeout(timeoutId);
    };
  }, []);

  return (
    <div className="overflow-hidden rounded-xl border border-white/15 bg-brand-950/50 shadow-2xl backdrop-blur">
      <div className="flex items-center gap-2 border-b border-white/10 bg-white/5 px-4 py-2.5">
        <span className="h-2.5 w-2.5 rounded-full bg-red-400/70" />
        <span className="h-2.5 w-2.5 rounded-full bg-yellow-400/70" />
        <span className="h-2.5 w-2.5 rounded-full bg-green-400/70" />
        <span className="ml-2 font-mono text-xs text-brand-200">
          Haste Health
        </span>
      </div>
      <pre
        className="whitespace-pre-wrap px-4 py-4 font-mono text-xs leading-6"
        style={{ background: "transparent", margin: 0, border: 0 }}
      >
        <code
          style={{ background: "transparent", color: "var(--color-brand-100)" }}
        >
          <span className="text-brand-300">$</span>
          {renderCommand(typed)}
          {"\n\n"}
          <ResponseBody visible={phase === "output"} />
        </code>
      </pre>
    </div>
  );
}

export default function Home(): ReactNode {
  return (
    <Layout
      wrapperClassName="bg-background"
      title={`Haste Health`}
      description="The headless EHR: an open-source FHIR R4 platform that normalizes every healthcare data source into one API, built for AI agents, self-hosted under Apache-2.0."
    >
      <meta name="algolia-site-verification" content="A94F28B6A640A6FE" />
      <main
        id="tw-scope"
        className="container mx-auto px-4 py-8 md:py-14 text-brand-950"
      >
        <section className="relative overflow-hidden rounded-3xl border border-white/10 bg-linear-to-br from-brand-950 via-brand-900 to-brand-700 px-6 py-14 md:px-14 md:py-24">
          <svg
            className="pointer-events-none absolute inset-x-0 bottom-0 h-1/2 w-full opacity-15"
            preserveAspectRatio="none"
            viewBox="0 0 1000 300"
            aria-hidden="true"
          >
            <path
              className="hero-pulse-line"
              d="M0 150 L120 150 L145 150 L165 70 L190 230 L215 150 L260 150 L285 90 L305 210 L325 150 L1000 150"
              fill="none"
              stroke="white"
              strokeWidth="2"
            />
          </svg>
          <div className="pointer-events-none absolute -right-24 -top-24 h-72 w-72 rounded-full bg-brand-400/30 blur-3xl hero-blob" />
          <div className="pointer-events-none absolute -left-16 bottom-0 h-64 w-64 rounded-full bg-brand-300/20 blur-3xl hero-blob-slow" />

          <div className="relative grid items-center gap-12 lg:grid-cols-12 lg:gap-10">
            <div className="space-y-7 lg:col-span-6">
              <div className="inline-flex items-center rounded-full border border-brand-400/60 bg-white/10 px-3 py-1 text-xs font-semibold uppercase tracking-[0.08em] text-white backdrop-blur">
                Open source · Apache-2.0 · Self-hosted
              </div>

              <Heading
                as="h1"
                className="text-4xl md:text-6xl font-bold tracking-[-0.025em] leading-[1.05] text-white"
              >
                The Headless EHR, Built for AI Agents
              </Heading>

              <p className="max-w-2xl text-lg md:text-2xl text-brand-100 leading-relaxed">
                Haste Health normalizes Epic, Cerner, HL7v2, and any
                FHIR-compatible system into one API, giving you a headless layer
                to power your apps and AI agents.
              </p>

              <div className="flex flex-col gap-3 pt-1 sm:flex-row sm:items-center">
                <Link
                  className="inline-flex items-center justify-center rounded-lg bg-white px-7 py-3 text-lg font-semibold text-brand-900 transition-colors hover:bg-brand-50"
                  to="/docs/getting_started/quick_start"
                >
                  Start in 5 Minutes
                </Link>
                <Link
                  className="inline-flex items-center justify-center rounded-lg border border-brand-300/60 bg-white/5 px-7 py-3 text-lg font-semibold text-white transition-colors hover:bg-white/15"
                  to="/docs/integration/ai/claude"
                >
                  Connect Claude
                </Link>
              </div>
            </div>

            <div className="lg:col-span-6">
              <HeroSnippet />
            </div>
          </div>

          <style>{`
            .hero-pulse-line {
              stroke-dasharray: 1400;
              stroke-dashoffset: 1400;
              animation: hero-draw 6s ease-in-out infinite;
            }
            @keyframes hero-draw {
              0% { stroke-dashoffset: 1400; opacity: 0.2; }
              45% { stroke-dashoffset: 0; opacity: 0.9; }
              55% { stroke-dashoffset: 0; opacity: 0.9; }
              100% { stroke-dashoffset: -1400; opacity: 0.2; }
            }
            .hero-blob { animation: hero-drift 14s ease-in-out infinite; }
            .hero-blob-slow { animation: hero-drift 20s ease-in-out infinite reverse; }
            @keyframes hero-drift {
              0%, 100% { transform: translate(0, 0) scale(1); }
              50% { transform: translate(-16px, 16px) scale(1.08); }
            }
            .hero-cursor {
              animation: hero-cursor-blink 0.9s step-end infinite;
            }
            @keyframes hero-cursor-blink {
              0%, 100% { opacity: 1; }
              50% { opacity: 0; }
            }
            .hero-response-in {
              animation: hero-response-in 180ms ease-out;
            }
            @keyframes hero-response-in {
              0% { opacity: 0; transform: translateY(2px); }
              100% { opacity: 1; transform: translateY(0); }
            }
            @media (prefers-reduced-motion: reduce) {
              .hero-pulse-line, .hero-blob, .hero-blob-slow, .hero-cursor, .hero-response-in { animation: none; }
            }
          `}</style>
        </section>

        <section className="mt-16 md:mt-24 rounded-2xl border border-brand-200 bg-white p-6 md:p-10">
          <SectionTitle
            eyebrow="Headless by design"
            title="Every Data Source In. Every Application Out."
            subtitle="Point EHRs, HL7v2 interfaces, and other FHIR servers at Haste Health. What comes out is one normalized FHIR R4 API that powers patient and provider apps, analytics, partner integrations, and AI agents alike."
          />
          <div className="mt-10">
            <DataFlowDiagram />
          </div>
        </section>

        <section className="mt-16 md:mt-24 rounded-2xl border border-brand-200 bg-white p-6 md:p-10">
          <SectionTitle
            eyebrow="For builders"
            title="What You Don't Have to Build Yourself"
            subtitle="The plumbing most healthcare startups end up writing from scratch, shipped as part of the platform."
          />
          <div className="mt-8 grid gap-5 sm:grid-cols-2 lg:grid-cols-3">
            {buildCards.map((item) => (
              <Link
                key={item.title}
                to={item.href}
                className="block rounded-xl border border-brand-200 bg-brand-50/40 p-5 transition hover:border-brand-400 hover:bg-brand-50"
              >
                <h3 className="text-xl font-semibold text-brand-900">
                  {item.title}
                </h3>
                <p className="mt-2 text-sm text-slate-700 leading-6">
                  {item.body}
                </p>
              </Link>
            ))}
          </div>
        </section>

        <section className="mt-20 md:mt-32 rounded-2xl border-2 border-brand-300 bg-brand-50/60 p-6 md:p-12">
          <SectionTitle
            eyebrow="See it"
            title="This Is What Headless Looks Like"
            subtitle="An agent asks a plain-English question. Haste Health answers over MCP or the same auto-generated OpenAPI spec, with spec-conformant FHIR data and clear, actionable errors in the event of failure."
          />
          <div className="mt-8">
            <McpPreview />
          </div>
        </section>

        <section className="mt-16 md:mt-24 rounded-2xl border border-brand-200 bg-white p-6 md:p-10">
          <SectionTitle
            title="Performance for Real Clinical Workloads"
            subtitle="A Rust core built to keep latency low and throughput high without oversized infrastructure."
          />
          <div className="mt-8 grid gap-5 sm:grid-cols-2 lg:grid-cols-4">
            {outcomeStats.map((stat) => (
              <article
                key={stat.label}
                className="rounded-xl border border-brand-200 bg-brand-50/30 p-5"
              >
                <p className="text-4xl md:text-5xl font-bold tracking-[-0.02em] text-brand-900">
                  {stat.value}
                </p>
                <p className="mt-2 text-sm leading-6 text-slate-700">
                  {stat.label}
                </p>
              </article>
            ))}
          </div>
          <p className="mt-4 text-sm text-slate-600">
            Benchmarked on a single machine with Postgres 18 and a
            Synthea-generated dataset, 10 threads.
          </p>
        </section>

        <section className="mt-16 md:mt-24 rounded-2xl border border-brand-200 bg-white p-6 md:p-10">
          <SectionTitle
            eyebrow="Who it's for"
            title="Built for Teams Moving Fast on Healthcare Data"
          />
          <div className="mt-8 grid gap-5 sm:grid-cols-2 lg:grid-cols-4">
            {audiences.map((item) => (
              <Link
                key={item.title}
                to={item.href}
                className="block rounded-xl border border-brand-200 bg-brand-50/40 p-5 transition hover:border-brand-400 hover:bg-brand-50"
              >
                <h3 className="text-lg font-semibold text-brand-900">
                  {item.title}
                </h3>
                <p className="mt-2 text-sm text-slate-700 leading-6">
                  {item.body}
                </p>
              </Link>
            ))}
          </div>
        </section>

        <section className="mt-20 md:mt-32 rounded-2xl border border-white/10 bg-brand-950 px-6 py-12 md:px-14 md:py-16">
          <div className="max-w-4xl space-y-4">
            <h2 className="text-3xl md:text-4xl font-bold tracking-[-0.02em] leading-[1.15] text-white">
              Secure by Default — Even for Autonomous Agents
            </h2>
            <p className="max-w-2xl text-brand-100 text-base md:text-lg leading-relaxed">
              TOTP-based MFA, argon2 password hashing, and CSRF-protected auth
              flows. Scoped OAuth clients for every agent. Attribute-based
              access policies. Immutable, versioned resource history. The
              controls a HIPAA review actually asks for, not a checkbox.
            </p>
            <div className="pt-2 flex flex-col gap-3 sm:flex-row">
              <Link
                to="/docs/category/oauth-grant-types"
                className="inline-flex items-center justify-center rounded-lg bg-white px-6 py-3 text-base font-semibold text-brand-900 hover:bg-brand-50"
              >
                Review Security Model
              </Link>
              <Link
                to="/docs/auth/authorization/access_control"
                className="inline-flex items-center justify-center rounded-lg border border-brand-400 px-6 py-3 text-base font-semibold text-brand-100 hover:bg-brand-900"
              >
                Explore Access Policies
              </Link>
            </div>
          </div>
        </section>
      </main>
    </Layout>
  );
}
