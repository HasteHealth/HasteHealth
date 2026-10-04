import React, { type ReactNode } from "react";
import clsx from "clsx";
import Head from "@docusaurus/Head";
import Link from "@docusaurus/Link";
import Layout from "@theme/Layout";
import Heading from "@theme/Heading";

import DataFlowDiagram from "@site/src/components/DataFlowDiagram";
import McpPreview from "@site/src/components/McpPreview";
import AgentSession from "@site/src/components/home/AgentSession";
import ConformanceGrid from "@site/src/components/home/ConformanceGrid";
import { useHomepageFacts } from "@site/src/components/home/facts";
import {
  GitHubMark,
  Icon,
  type IconName,
} from "@site/src/components/home/icons";
import {
  CountUp,
  revealDelay,
  riseDelay,
  useScrollReveal,
} from "@site/src/components/home/motion";
import styles from "@site/src/components/home/styles.module.css";
import {
  DEMO_URL,
  GITHUB_URL,
  QUICK_START_URL,
  SECURITY_EMAIL,
  SIGNUP_URL,
} from "@site/src/links";
import { TIERS, type Tier, type TierId } from "@site/src/pricing/tiers";

// ---------------------------------------------------------------------------
// Content
// ---------------------------------------------------------------------------

/** The interoperability standards the server implements, as a trust bar. */
const standards = [
  "HL7 FHIR R4",
  "SMART on FHIR",
  "OAuth 2.0",
  "OpenID Connect",
  "Model Context Protocol",
  "SQL on FHIR",
  "HL7v2",
  "FHIRPath",
];

const flowSteps = [
  {
    title: "Bring data in",
    body: "Load from EHR FHIR APIs, HL7v2 feeds over MLLP, or plain FHIR transactions and batches. Everything is validated and stored as FHIR R4.",
  },
  {
    title: "Govern it once",
    body: "Tenants and projects isolate data. OAuth scopes and access policies decide who can see what, and every write is versioned.",
  },
  {
    title: "Serve every consumer",
    body: "Apps use REST and SMART on FHIR, analysts use SQL on FHIR, and agents use MCP. All of them read the same governed data.",
  },
];

type Feature = { icon: IconName; title: string; body: string };

function agentFeatures(mcpTools: number): Feature[] {
  return [
    {
      icon: "tools",
      title: "Tools generated from your server",
      body: `${mcpTools} MCP tools cover search, read, write, history, batch and transaction. Their schemas come from your live CapabilityStatement, so agents only see what your deployment supports.`,
    },
    {
      icon: "badge",
      title: "An identity for every agent",
      body: "Each agent is its own OAuth client with SMART scopes and access policies. Give one read-only access to three resource types, not the keys to the dataset.",
    },
    {
      icon: "shieldCheck",
      title: "The same guardrails as people",
      body: "Tool calls pass through the same scope checks, policy engine and audit trail as a REST request. There is no side door for AI.",
    },
    {
      icon: "alert",
      title: "Errors an agent can fix",
      body: "Errors carry the FHIR OperationOutcome behind them wherever there is one, so an agent can see what was wrong, correct its request and try again.",
    },
  ];
}

/** What an agent with governed FHIR access is typically asked to do. */
const agentUseCases = [
  "Clinical decision support",
  "Clinical documentation",
  "Care coordination",
  "Patient education",
  "Data quality audits",
];

const platform: (Feature & { href: string })[] = [
  {
    icon: "database",
    title: "A complete FHIR R4 API",
    body: "Every R4 resource type with create, read, update, patch, delete, history, transactions and batches. Search supports modifiers and prefixes, _include, _revinclude and Patient/$everything.",
    href: "/docs/api/rest_api/fhir/intro",
  },
  {
    icon: "key",
    title: "Authentication built in",
    body: "An OAuth 2.0 and OpenID Connect provider with SMART on FHIR standalone launch, plus federated sign-in through Okta, Azure, Auth0, Keycloak or Google.",
    href: "/docs/category/authentication",
  },
  {
    icon: "shieldCheck",
    title: "Fine-grained access control",
    body: "Attribute-based policies written in FHIRPath decide who can read or write what. Assign them to people, client applications and custom operations.",
    href: "/docs/auth/authorization/access_control",
  },
  {
    icon: "layers",
    title: "Multi-tenant by design",
    body: "Tenants and projects are enforced at the storage and search layers, so one deployment can serve every customer, environment or business unit.",
    href: "/docs/core_concepts/platform_architecture",
  },
  {
    icon: "table",
    title: "Analytics without a warehouse",
    body: "SQL on FHIR ViewDefinitions flatten resources into CSV, JSON or NDJSON on demand. Point a BI tool at clinical data without building an ETL pipeline.",
    href: "/docs/guides/sql_on_fhir",
  },
  {
    icon: "code",
    title: "Custom logic without a fork",
    body: "Custom operations run as sandboxed TypeScript in an embedded Deno runtime, stored and versioned as FHIR OperationDefinition resources.",
    href: "/docs/reference/fhir/model/resources/OperationDefinition",
  },
];

/** The rest of the box, one link each. */
const alsoIncluded = [
  { label: "Admin console", href: "/docs/guides/admin_app" },
  { label: "CLI", href: "/docs/tutorials/cli" },
  { label: "TypeScript, Rust and React SDKs", href: "/docs/category/sdks" },
  { label: "Terminology services", href: "/docs/category/terminology" },
  { label: "HL7v2 ingestion (experimental)", href: "/docs/guides/hl7v2" },
];

const performance = [
  { prefix: "<", value: 10, unit: "ms", label: "Create and update latency" },
  { prefix: ">", value: 25, unit: "k/s", label: "Writes per second" },
  { prefix: "<", value: 50, unit: "ms", label: "Typical search response" },
  {
    prefix: "<",
    value: 100,
    unit: "MB",
    label: "Memory footprint per instance",
  },
];

const controls: Feature[] = [
  {
    icon: "users",
    title: "Single sign-on",
    body: "Federate login to Okta, Azure, Auth0, Keycloak or Google over OpenID Connect.",
  },
  {
    icon: "lock",
    title: "Hardened sign-in",
    body: "TOTP-based MFA, argon2 password hashing and CSRF-protected authentication flows.",
  },
  {
    icon: "key",
    title: "Scoped tokens",
    body: "OAuth 2.0 with PKCE, client credentials and refresh tokens. SMART scopes are checked on every request.",
  },
  {
    icon: "shieldCheck",
    title: "Attribute-based policies",
    body: "FHIRPath rules evaluated per request, for users, client applications and custom operations.",
  },
  {
    icon: "clipboard",
    title: "Audit trail in FHIR",
    body: "Requests can be recorded as AuditEvent resources that name the caller, the action and the outcome.",
  },
  {
    icon: "history",
    title: "Immutable history",
    body: "Every write creates a new version. Resource history is immutable at the storage layer.",
  },
];

const managedAssurances = [
  "A signed BAA before any PHI reaches us",
  "Daily backups with point-in-time recovery",
  "An uptime SLA and a committed support response time",
  "BAA template, subprocessor list, data-flow diagram and pre-answered CAIQ and SIG Lite, on request",
];

/**
 * The editorial half of a deployment card. Price, SLA and support terms are
 * read from the generated tier table (src/pricing/tiers.ts), the same one the
 * pricing page and the server use, so they are not restated here.
 */
type DeploymentOption = {
  tier: TierId;
  icon: IconName;
  where: string;
  body: string;
  points: (tier: Tier) => string[];
  cta: { label: string; href: string };
};

function baa(tier: Tier): string[] {
  return tier.baa_available ? ["Signed BAA"] : [];
}

const deployments: DeploymentOption[] = [
  {
    tier: "unlimited",
    icon: "server",
    where: "Your cloud or data center",
    body: "The whole server under Apache-2.0. Run it with Docker Compose, container images or a single binary from npm, and keep your data where it already is.",
    points: () => [
      "Every feature, with no license key",
      "PostgreSQL only, Elasticsearch optional",
      "Community support on GitHub",
    ],
    cta: { label: "Read the quick start", href: QUICK_START_URL },
  },
  {
    tier: "professional",
    icon: "cloud",
    where: "Managed by Haste Health",
    body: "A dedicated hosted tenant that we run, back up and upgrade. Start on the free Developer tier and move up when you carry real patient data.",
    points: (tier) => [
      ...baa(tier),
      `${tier.uptime_sla} uptime SLA`,
      `Support response in ${tier.support}`,
    ],
    cta: { label: "Start for free", href: SIGNUP_URL },
  },
  {
    tier: "team",
    icon: "building",
    where: "Dedicated, or in your VPC",
    body: "Single-tenant infrastructure for population-scale data: a dedicated database, tunable read replicas and Elasticsearch scale-out for search.",
    points: (tier) => [
      ...baa(tier),
      `${tier.uptime_sla} uptime SLA`,
      `Support response in ${tier.support}, with a named engineer`,
    ],
    cta: { label: "Book a demo", href: DEMO_URL },
  },
];

// ---------------------------------------------------------------------------
// Building blocks
// ---------------------------------------------------------------------------

function Container({
  children,
  className,
}: Readonly<{ children: ReactNode; className?: string }>) {
  return (
    <div
      className={clsx("mx-auto w-full max-w-[84rem] px-5 sm:px-8", className)}
    >
      {children}
    </div>
  );
}

/** Which surface a control sits on, so it can pick legible colours. */
type Surface = "light" | "dark";

const BUTTON =
  "inline-flex items-center justify-center gap-2 whitespace-nowrap rounded-full px-6 py-3 text-[1.0625rem] font-semibold no-underline transition-colors hover:no-underline focus-visible:outline-2 focus-visible:outline-offset-2";

const BUTTON_STYLES: Record<
  "primary" | "secondary",
  Record<Surface, string>
> = {
  primary: {
    dark: "bg-white text-ink-950 hover:bg-brand-100 hover:text-ink-950 focus-visible:outline-white",
    light:
      "bg-brand-800 text-white hover:bg-brand-900 hover:text-white focus-visible:outline-brand-800",
  },
  secondary: {
    dark: "border border-white/25 text-white hover:border-white/50 hover:bg-white/10 hover:text-white focus-visible:outline-white",
    light:
      "border border-slate-300 bg-white text-ink-900 hover:border-slate-400 hover:bg-slate-50 hover:text-ink-900 focus-visible:outline-brand-800",
  },
};

function Button({
  to,
  children,
  variant = "primary",
  on = "light",
}: Readonly<{
  to: string;
  children: ReactNode;
  variant?: "primary" | "secondary";
  on?: Surface;
}>) {
  return (
    <Link to={to} className={clsx(BUTTON, BUTTON_STYLES[variant][on])}>
      {children}
    </Link>
  );
}

/** A quiet text link with an arrow that leans forward on hover. */
function ArrowLink({
  to,
  children,
  on = "light",
}: Readonly<{ to: string; children: ReactNode; on?: Surface }>) {
  return (
    <Link
      to={to}
      // A mail link opens the mail client; it should not open a blank tab too.
      {...(to.startsWith("mailto:") ? { target: "_self" } : {})}
      className={clsx(
        "inline-flex items-center gap-1.5 text-[1.0625rem] font-semibold no-underline hover:no-underline",
        on === "dark"
          ? "text-brand-300 hover:text-brand-200"
          : "text-brand-800 hover:text-brand-900",
      )}
    >
      {children}
      <Icon name="arrowRight" className={clsx("h-4 w-4", styles.arrow)} />
    </Link>
  );
}

function SectionHeader({
  eyebrow,
  title,
  children,
  on = "light",
  centered = false,
}: Readonly<{
  eyebrow: string;
  title: string;
  children?: ReactNode;
  on?: Surface;
  centered?: boolean;
}>) {
  return (
    <div
      className={clsx("max-w-[52rem]", centered && "mx-auto text-center")}
      data-reveal=""
    >
      <p
        className={clsx(
          "text-[0.8125rem] font-semibold uppercase tracking-[0.16em]",
          on === "dark" ? "text-brand-300" : "text-brand-800",
        )}
      >
        {eyebrow}
      </p>
      <Heading
        as="h2"
        className={clsx(
          "mt-4 text-[2rem] font-semibold leading-[1.12] tracking-[-0.025em] text-balance md:text-[2.75rem]",
          on === "dark" ? "text-white" : "text-ink-950",
        )}
      >
        {title}
      </Heading>
      {children ? (
        <p
          className={clsx(
            "mt-5 text-lg leading-relaxed md:text-xl",
            on === "dark" ? "text-slate-300" : "text-slate-600",
          )}
        >
          {children}
        </p>
      ) : null}
    </div>
  );
}

/** An icon in a tinted tile, the lead-in to a feature. */
function IconTile({
  name,
  on = "light",
}: Readonly<{ name: IconName; on?: Surface }>) {
  return (
    <span
      className={clsx(
        "flex h-12 w-12 shrink-0 items-center justify-center rounded-xl",
        on === "dark"
          ? "bg-white/[0.06] text-brand-300 ring-1 ring-white/10"
          : "bg-brand-50 text-brand-800 ring-1 ring-brand-200/70",
      )}
    >
      <Icon name={name} className="h-[1.375rem] w-[1.375rem]" />
    </span>
  );
}

// ---------------------------------------------------------------------------
// Sections
// ---------------------------------------------------------------------------

function Hero() {
  return (
    <section className="relative z-10 text-white">
      <div className={styles.heroBackdrop} aria-hidden="true">
        <div className={styles.grid} />
        <svg
          className={clsx(
            styles.pulseLine,
            "absolute inset-x-0 bottom-36 h-40 w-full",
          )}
          viewBox="0 0 1600 200"
          preserveAspectRatio="none"
          fill="none"
        >
          <path
            d="M0 100H78l18 0 16-52 22 104 18-52h1296l18 0 16-44 22 88 18-44h78"
            stroke="white"
            strokeOpacity="0.16"
            strokeWidth="1.5"
            vectorEffect="non-scaling-stroke"
          />
        </svg>
      </div>

      {/* A flex column, so the panel's negative margin shortens the section
          and the panel hangs over the edge into the next one. */}
      <Container className="relative flex flex-col pt-14 md:pt-20">
        <div className="mx-auto flex max-w-[66rem] flex-col items-center text-center">
          {/* The H1 and subhead already say open source; the badge carries
              the two things a buyer comparing FHIR servers cannot get from
              most rivals without a sales call. */}
          <p
            className={clsx(
              styles.rise,
              "inline-flex items-center gap-2.5 rounded-full border border-white/15 bg-white/[0.06] px-4 py-1.5 text-[0.8125rem] font-medium tracking-wide text-brand-100",
            )}
          >
            <span
              className="h-1.5 w-1.5 rounded-full bg-brand-400"
              aria-hidden="true"
            />
            Apache-2.0 · Published pricing · BAA available
          </p>

          {/* The category noun a buyer actually shortlists on ("FHIR
              server") leads; AI-native is the audience, not the category.
              An H1 built on MCP alone competes on a feature every rival
              now ships.

              From the md breakpoint up it breaks into the same two lines as
              the social card, and the size steps with the viewport so the
              longer line, "The Open-Source FHIR Server" (about 13.7em in a
              semibold system font), always fits the container. Below that it
              wraps freely, and the nowrap spans keep the compound words
              whole. */}
          <Heading
            as="h1"
            className={clsx(
              styles.rise,
              "mt-7 text-[2.375rem] font-semibold leading-[1.05] tracking-[-0.035em] text-white text-balance md:text-5xl lg:text-6xl xl:text-7xl",
            )}
            style={riseDelay(60)}
          >
            The <span className="whitespace-nowrap">Open-Source</span>{" "}
            <span className="whitespace-nowrap">FHIR Server</span>{" "}
            <span className="md:block">
              for{" "}
              <span className="whitespace-nowrap text-brand-300">
                AI-Native
              </span>{" "}
              Health Apps
            </span>
          </Heading>

          {/* The first sentence is the social card's subhead, word for word
              (scripts/social-card.mjs), so a link preview and the page it
              opens say the same thing. */}
          <p
            className={clsx(
              styles.rise,
              "mt-7 max-w-[46rem] text-lg leading-relaxed text-slate-300 md:text-xl",
            )}
            style={riseDelay(140)}
          >
            <span className="font-medium text-white">
              Epic, Oracle Health and HL7v2 in. FHIR, SQL and MCP out.
            </span>{" "}
            One governed clinical data layer for your applications and your AI
            agents. Self-host it under Apache-2.0, or let us run it for you with
            a signed BAA.
          </p>

          {/* The hosted free tenant is the lowest-friction start and the path
              to a paid tier; the demo is for the buyer who wants a person.
              Self-hosting stays one click away on the line below. */}
          <div
            className={clsx(
              styles.rise,
              "mt-9 flex w-full flex-col gap-3 sm:w-auto sm:flex-row",
            )}
            style={riseDelay(220)}
          >
            <Button to={SIGNUP_URL} on="dark">
              Start for free
            </Button>
            <Button to={DEMO_URL} variant="secondary" on="dark">
              Book a demo
            </Button>
          </div>
          <p
            className={clsx(styles.rise, "mt-5 text-sm text-slate-400")}
            style={riseDelay(280)}
          >
            Free hosted tenant, no credit card. Or{" "}
            <Link
              to={QUICK_START_URL}
              className="font-medium text-slate-200 underline decoration-white/30 underline-offset-4 hover:text-white hover:decoration-white/70"
            >
              self-host in 5 minutes
            </Link>
            .
          </p>
        </div>

        <div
          className={clsx(
            styles.rise,
            "relative mx-auto -mb-24 mt-14 w-full max-w-[70rem] md:-mb-32 md:mt-16",
          )}
          style={riseDelay(380)}
        >
          <AgentSession />
        </div>
      </Container>
    </section>
  );
}

function Standards() {
  return (
    // The top padding clears the hero panel that hangs into this section.
    <section className="border-b border-slate-200/80 bg-white pb-12 pt-36 md:pb-14 md:pt-48">
      <Container>
        <p className="text-center text-[0.8125rem] font-semibold uppercase tracking-[0.16em] text-slate-500">
          Built on the open standards your stack already speaks
        </p>
        <ul className="mt-6 flex list-none flex-wrap items-center justify-center gap-x-9 gap-y-3 p-0">
          {standards.map((standard) => (
            <li
              key={standard}
              className="text-[1.0625rem] font-semibold tracking-tight text-slate-500"
            >
              {standard}
            </li>
          ))}
        </ul>
      </Container>
    </section>
  );
}

function HowItFits() {
  return (
    <section className="bg-white py-20 md:py-28">
      <Container>
        <SectionHeader
          eyebrow="How it fits"
          title="One clinical data layer between your systems of record and everything you build"
          centered
        >
          Point EHRs, HL7v2 interfaces and other FHIR servers at Haste Health.
          What comes out is a single, normalized FHIR R4 API for your
          applications, your analysts and your AI agents.
        </SectionHeader>

        <div
          className="mt-14 rounded-3xl border border-slate-200 bg-slate-50/70 p-5 sm:p-8 md:mt-16 md:p-12"
          data-reveal=""
        >
          <DataFlowDiagram />
        </div>

        <ol className="mt-12 grid list-none gap-10 p-0 md:mt-16 md:grid-cols-3 md:gap-12">
          {flowSteps.map((step, index) => (
            <li
              key={step.title}
              className="border-t border-slate-200 pt-6"
              data-reveal=""
              style={revealDelay(index)}
            >
              <span className="font-mono text-sm font-medium text-brand-800">
                {String(index + 1).padStart(2, "0")}
              </span>
              <h3 className="mt-3 text-xl font-semibold tracking-tight text-ink-950">
                {step.title}
              </h3>
              <p className="mt-2 text-[1.0625rem] leading-relaxed text-slate-600">
                {step.body}
              </p>
            </li>
          ))}
        </ol>
      </Container>
    </section>
  );
}

function AiNative() {
  const { mcpTools } = useHomepageFacts();

  return (
    <section className="relative isolate py-20 text-white md:py-28">
      <div className={clsx(styles.darkBackdrop, "-z-10")} aria-hidden="true" />
      <Container>
        <SectionHeader
          eyebrow="AI-native"
          title="Agents are first-class clients, held to first-class rules"
          on="dark"
        >
          Every project serves a Model Context Protocol endpoint beside its REST
          API. Agents get typed tools and real FHIR schemas instead of a scraped
          UI, and each call they make is authenticated, authorized and audited
          like any other request.
        </SectionHeader>

        <div
          className="mt-8 flex flex-wrap items-center gap-x-2.5 gap-y-2.5"
          data-reveal=""
        >
          <span className="mr-1.5 text-base font-medium text-slate-400">
            What you can build:
          </span>
          {agentUseCases.map((useCase) => (
            <span
              key={useCase}
              className="rounded-full border border-white/15 bg-white/[0.04] px-4 py-1.5 text-[0.9375rem] font-medium text-slate-200"
            >
              {useCase}
            </span>
          ))}
        </div>

        <div className="mt-12 grid gap-x-10 gap-y-10 sm:grid-cols-2 lg:mt-14 lg:grid-cols-4">
          {agentFeatures(mcpTools).map((feature, index) => (
            <div key={feature.title} data-reveal="" style={revealDelay(index)}>
              <IconTile name={feature.icon} on="dark" />
              <h3 className="mt-5 text-lg font-semibold tracking-tight text-white">
                {feature.title}
              </h3>
              <p className="mt-2 text-base leading-relaxed text-slate-400">
                {feature.body}
              </p>
            </div>
          ))}
        </div>

        <div className="mt-14" data-reveal="">
          <McpPreview />
        </div>

        <div
          className="mt-10 flex flex-col gap-6 lg:flex-row lg:items-center lg:justify-between"
          data-reveal=""
        >
          <p className="max-w-[36rem] text-base leading-relaxed text-slate-400">
            Works with Claude, Gemini and any other client that speaks MCP, and
            with the same OAuth 2.0 flows your applications already use.
          </p>
          <div className="flex shrink-0 flex-col gap-3 sm:flex-row">
            <Button to="/docs/category/ai" on="dark">
              Connect Claude or Gemini
            </Button>
            <Button
              to="/docs/api/rest_api/model_context_protocol/tools"
              variant="secondary"
              on="dark"
            >
              Browse the MCP tools
            </Button>
          </div>
        </div>
      </Container>
    </section>
  );
}

function Platform() {
  return (
    <section className="bg-white py-20 md:py-28">
      <Container>
        <SectionHeader
          eyebrow="Platform"
          title="Everything a production FHIR backend needs, already built"
        >
          The plumbing most healthcare teams end up writing themselves ships in
          the box. All of it is in the open-source repository, with no feature
          gates and no license key.
        </SectionHeader>

        <div className="mt-12 grid gap-5 sm:grid-cols-2 lg:mt-14 lg:grid-cols-3">
          {platform.map((item, index) => (
            <div key={item.title} data-reveal="" style={revealDelay(index % 3)}>
              <Link
                to={item.href}
                className={clsx(
                  styles.card,
                  "flex h-full flex-col rounded-2xl border border-slate-200 bg-white p-6 no-underline hover:no-underline md:p-7",
                )}
              >
                <IconTile name={item.icon} />
                <h3 className="mt-5 text-xl font-semibold tracking-tight text-ink-950">
                  {item.title}
                </h3>
                <p className="mt-2 flex-1 text-base leading-relaxed text-slate-600">
                  {item.body}
                </p>
                <span className="mt-5 inline-flex items-center gap-1.5 text-base font-semibold text-brand-800">
                  Read the docs
                  <Icon
                    name="arrowRight"
                    className={clsx("h-4 w-4", styles.arrow)}
                  />
                </span>
              </Link>
            </div>
          ))}
        </div>

        <div
          className="mt-10 flex flex-wrap items-center gap-x-3 gap-y-3"
          data-reveal=""
        >
          <span className="mr-1 text-base font-medium text-slate-500">
            Also in the box:
          </span>
          {alsoIncluded.map((item) => (
            <Link
              key={item.label}
              to={item.href}
              className="rounded-full border border-slate-200 bg-white px-4 py-1.5 text-[0.9375rem] font-medium text-slate-700 no-underline transition-colors hover:border-brand-400 hover:text-brand-900 hover:no-underline"
            >
              {item.label}
            </Link>
          ))}
        </div>
      </Container>
    </section>
  );
}

function Proof() {
  const { conformance } = useHomepageFacts();
  const coverage = [
    { value: conformance.checks, label: "automated checks" },
    { value: conformance.searchParameters, label: "search parameters" },
    { value: conformance.backends.length, label: "search backends" },
  ];

  return (
    <section className="border-y border-slate-200/80 bg-slate-50 py-20 md:py-28">
      <Container>
        <SectionHeader
          eyebrow="Proof"
          title="Fast under load, and tested against the specification"
        >
          A Rust core keeps latency low and throughput high without oversized
          infrastructure, and every claim about FHIR support is backed by a test
          you can read.
        </SectionHeader>

        <dl className="mt-12 grid grid-cols-2 gap-x-8 gap-y-10 lg:mt-14 lg:grid-cols-4">
          {performance.map((stat, index) => (
            // Term first in the markup, figure first on screen. The column is
            // reversed, so justify-end is what pins the figure to the top.
            <div
              key={stat.label}
              className="flex flex-col-reverse justify-end border-t border-slate-300 pt-5"
              data-reveal=""
              style={revealDelay(index)}
            >
              <dt className="mt-2 text-base leading-snug text-slate-600">
                {stat.label}
              </dt>
              <dd className="m-0 text-[2.75rem] font-semibold leading-none tracking-[-0.03em] text-ink-950 md:text-[3.5rem]">
                {stat.prefix}
                <CountUp value={stat.value} />
                <span className="ml-1 text-2xl font-semibold tracking-normal text-slate-500">
                  {stat.unit}
                </span>
              </dd>
            </div>
          ))}
        </dl>
        <p className="mt-6 text-sm leading-relaxed text-slate-500">
          Benchmarked on a single machine with Postgres 18 and a
          Synthea-generated dataset, 10 threads.
        </p>

        <div
          className="mt-12 grid gap-8 rounded-2xl border border-slate-200 bg-white p-6 shadow-[0_1px_2px_rgb(15_23_42/0.04)] md:p-10 lg:mt-14 lg:grid-cols-12 lg:gap-14"
          data-reveal=""
        >
          <div className="lg:col-span-7">
            <p className="text-[0.8125rem] font-semibold uppercase tracking-[0.16em] text-slate-500">
              Conformance
            </p>
            <p className="mt-3 flex flex-wrap items-baseline gap-x-3 gap-y-1">
              <span className="text-[2.75rem] font-semibold leading-none tracking-[-0.03em] text-ink-950 md:text-[3.5rem]">
                <CountUp value={conformance.passing} />
                <span className="text-slate-500">
                  {" "}
                  / {conformance.resourceTypes}
                </span>
              </span>
              <span className="text-lg text-slate-600">
                FHIR resource types pass every check
              </span>
            </p>
            <div className="mt-6">
              <ConformanceGrid />
            </div>
          </div>

          <div className="flex flex-col justify-center lg:col-span-5">
            <dl className="grid grid-cols-3 gap-4">
              {coverage.map((fact) => (
                <div
                  key={fact.label}
                  className="flex flex-col-reverse justify-end"
                >
                  <dt className="mt-1 text-sm leading-snug text-slate-600">
                    {fact.label}
                  </dt>
                  <dd className="m-0 text-2xl font-semibold tracking-tight text-ink-950">
                    <CountUp value={fact.value} />
                  </dd>
                </div>
              ))}
            </dl>
            <p className="mt-6 border-t border-slate-200 pt-6 text-base leading-relaxed text-slate-600">
              Each square is one resource type with its own TestScript, run in
              CI against both PostgreSQL and Elasticsearch. Known gaps are
              published, not hidden.
            </p>
            <div className="mt-5">
              <ArrowLink to="/docs/reference/conformance/test-coverage">
                See the full conformance report
              </ArrowLink>
            </div>
          </div>
        </div>
      </Container>
    </section>
  );
}

function Security() {
  return (
    <section className="bg-white py-20 md:py-28">
      <Container>
        <SectionHeader
          eyebrow="Security and governance"
          title="The controls a security review asks for, on by default"
        >
          Clinical data needs more than an API key. Haste Health ships its own
          identity provider, policy engine and audit trail, and applies them to
          people, services and autonomous agents alike.
        </SectionHeader>

        <div className="mt-12 grid gap-10 lg:mt-14 lg:grid-cols-12 lg:gap-12">
          <div className="grid gap-x-10 gap-y-9 sm:grid-cols-2 lg:col-span-8">
            {controls.map((control, index) => (
              <div
                key={control.title}
                className="flex gap-4"
                data-reveal=""
                style={revealDelay(index % 2)}
              >
                <IconTile name={control.icon} />
                <div>
                  <h3 className="text-lg font-semibold tracking-tight text-ink-950">
                    {control.title}
                  </h3>
                  <p className="mt-1.5 text-base leading-relaxed text-slate-600">
                    {control.body}
                  </p>
                </div>
              </div>
            ))}
          </div>

          <aside
            className="relative isolate overflow-hidden rounded-2xl p-7 text-white md:p-8 lg:col-span-4"
            data-reveal=""
            style={revealDelay(2)}
          >
            <div
              className={clsx(styles.darkBackdrop, "-z-10")}
              aria-hidden="true"
            />
            <Icon name="document" className="h-6 w-6 text-brand-300" />
            <h3 className="mt-4 text-xl font-semibold tracking-tight text-white">
              On the managed service
            </h3>
            <ul className="mt-5 list-none space-y-3.5 p-0">
              {managedAssurances.map((assurance) => (
                <li
                  key={assurance}
                  className="flex gap-3 text-base leading-snug text-slate-200"
                >
                  <Icon
                    name="check"
                    className="mt-0.5 h-4 w-4 text-brand-300"
                  />
                  {assurance}
                </li>
              ))}
            </ul>
            <div className="mt-7">
              <ArrowLink to={SECURITY_EMAIL} on="dark">
                Request the security pack
              </ArrowLink>
            </div>
          </aside>
        </div>

        <div className="mt-12 flex flex-col gap-3 sm:flex-row" data-reveal="">
          <Button to="/docs/core_concepts/identity_access_control">
            Review the security model
          </Button>
          <Button
            to="/docs/auth/authorization/access_control"
            variant="secondary"
          >
            Explore access policies
          </Button>
        </div>
      </Container>
    </section>
  );
}

function Deployment() {
  return (
    <section className="border-t border-slate-200/80 bg-slate-50 py-20 md:py-28">
      <Container>
        <SectionHeader
          eyebrow="Deployment"
          title="Your infrastructure or ours. The same server either way."
        >
          Self-hosted and managed deployments run the same open-source server on
          the same storage schema, so you can move between them in either
          direction.
        </SectionHeader>

        <div className="mt-12 grid gap-5 lg:mt-14 lg:grid-cols-3">
          {deployments.map((deployment, index) => {
            const tier = TIERS.find(
              (candidate) => candidate.tier === deployment.tier,
            );
            if (!tier) return null;
            return (
              <article
                key={deployment.tier}
                className="flex flex-col rounded-2xl border border-slate-200 bg-white p-6 shadow-[0_1px_2px_rgb(15_23_42/0.04)] md:p-8"
                data-reveal=""
                style={revealDelay(index)}
              >
                <div className="flex items-center gap-3.5">
                  <IconTile name={deployment.icon} />
                  <div>
                    <h3 className="text-xl font-semibold leading-tight tracking-tight text-ink-950">
                      {tier.display_name}
                    </h3>
                    <p className="text-sm text-slate-500">{deployment.where}</p>
                  </div>
                </div>

                <p className="mt-6 flex items-baseline gap-1.5">
                  <span className="text-3xl font-semibold tracking-tight text-ink-950">
                    {tier.price}
                  </span>
                  {tier.cadence ? (
                    <span className="text-base text-slate-500">
                      {tier.cadence}
                    </span>
                  ) : (
                    <span className="text-base text-slate-500">Apache-2.0</span>
                  )}
                </p>

                <p className="mt-4 text-base leading-relaxed text-slate-600">
                  {deployment.body}
                </p>

                <ul className="mt-6 flex-1 list-none space-y-3 border-t border-slate-200 p-0 pt-6">
                  {deployment.points(tier).map((point) => (
                    <li
                      key={point}
                      className="flex gap-3 text-base leading-snug text-slate-700"
                    >
                      <Icon
                        name="check"
                        className="mt-0.5 h-4 w-4 text-brand-700"
                      />
                      {point}
                    </li>
                  ))}
                </ul>

                <div className="mt-7">
                  <ArrowLink to={deployment.cta.href}>
                    {deployment.cta.label}
                  </ArrowLink>
                </div>
              </article>
            );
          })}
        </div>

        <p className="mt-8 text-base text-slate-600" data-reveal="">
          Prices and limits are published in full.{" "}
          <Link
            to="/pricing"
            className="font-semibold text-brand-800 underline decoration-brand-300 underline-offset-4 hover:text-brand-900 hover:decoration-brand-600"
          >
            Compare every plan
          </Link>
          .
        </p>
      </Container>
    </section>
  );
}

function FinalCta() {
  return (
    <section className="bg-slate-50 pb-20 md:pb-28">
      <Container>
        <div
          className="relative isolate overflow-hidden rounded-3xl px-6 py-14 text-center text-white sm:px-10 md:py-20"
          data-reveal=""
        >
          <div
            className={clsx(styles.heroBackdrop, "-z-10")}
            aria-hidden="true"
          >
            <div className={styles.grid} />
          </div>
          <Heading
            as="h2"
            className="mx-auto max-w-[44rem] text-[2rem] font-semibold leading-[1.12] tracking-[-0.025em] text-white text-balance md:text-[2.75rem]"
          >
            Build on an open foundation for clinical data and AI
          </Heading>
          <p className="mx-auto mt-5 max-w-[40rem] text-lg leading-relaxed text-slate-300 md:text-xl">
            Start on a free hosted tenant in under a minute, self-host in five,
            or talk to an engineer about your architecture.
          </p>
          <div className="mt-9 flex flex-col justify-center gap-3 sm:flex-row">
            <Button to={SIGNUP_URL} on="dark">
              Start for free
            </Button>
            <Button to={DEMO_URL} variant="secondary" on="dark">
              Book a demo
            </Button>
            <Button to={GITHUB_URL} variant="secondary" on="dark">
              <GitHubMark className="h-[1.125rem] w-[1.125rem]" />
              View on GitHub
            </Button>
          </div>
        </div>
      </Container>
    </section>
  );
}

// ---------------------------------------------------------------------------
// Page
// ---------------------------------------------------------------------------

const STRUCTURED_DATA = JSON.stringify({
  "@context": "https://schema.org",
  "@type": "SoftwareApplication",
  name: "Haste Health",
  applicationCategory: "HealthApplication",
  operatingSystem: "Linux, macOS, Windows (Docker)",
  description:
    "Open-source, self-hosted FHIR R4 server and clinical data repository for AI-native health apps, normalizing Epic, Oracle Health, HL7v2 and any FHIR R4 API into one API. Built-in OAuth2 authorization, SMART on FHIR standalone launch and MCP tools for AI agent access.",
  url: "https://haste.health",
  license: "https://www.apache.org/licenses/LICENSE-2.0",
  offers: {
    "@type": "Offer",
    price: "0",
    priceCurrency: "USD",
  },
});

export default function Home(): ReactNode {
  const page = useScrollReveal<HTMLElement>();

  return (
    <Layout
      wrapperClassName="bg-white"
      title="Open-Source FHIR Server for AI-Native Health Apps"
      description="Haste Health is an open-source, self-hosted FHIR R4 server and clinical data repository for AI-native health apps. It normalizes Epic, Oracle Health, HL7v2 and any FHIR R4 API into one API, with OAuth2, SMART standalone launch and MCP tools built in. Apache-2.0, or hosted from $1,500/month with a BAA."
    >
      <Head>
        <meta name="algolia-site-verification" content="A94F28B6A640A6FE" />
        <script type="application/ld+json">{STRUCTURED_DATA}</script>
      </Head>
      <main id="tw-scope" ref={page} className="text-ink-950">
        <Hero />
        <Standards />
        <HowItFits />
        <AiNative />
        <Platform />
        <Proof />
        <Security />
        <Deployment />
        <FinalCta />
      </main>
    </Layout>
  );
}
