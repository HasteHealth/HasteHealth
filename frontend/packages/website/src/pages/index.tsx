import React, { type ReactNode } from "react";
import clsx from "clsx";
import Head from "@docusaurus/Head";
import Link from "@docusaurus/Link";
import Layout from "@theme/Layout";
import Heading from "@theme/Heading";

import DataFlowDiagram from "@site/src/components/DataFlowDiagram";
import AgentSession from "@site/src/components/home/AgentSession";
import ConformanceGrid from "@site/src/components/home/ConformanceGrid";
import ProductGrid from "@site/src/components/home/ProductGrid";
import { useHomepageFacts } from "@site/src/components/home/facts";
import { GitHubMark, type IconName } from "@site/src/components/site/icons";
import {
  CountUp,
  revealDelay,
  useScrollReveal,
} from "@site/src/components/site/motion";
import site from "@site/src/components/site/styles.module.css";
import {
  ArrowLink,
  Button,
  Container,
  Eyebrow,
  IconTile,
  LinkCard,
  SectionHeader,
} from "@site/src/components/site/ui";
import {
  DEMO_URL,
  GITHUB_URL,
  QUICK_START_URL,
  SIGNUP_URL,
} from "@site/src/links";
import { EMPHASIZED_TIER, TIERS } from "@site/src/pricing/tiers";

// ---------------------------------------------------------------------------
// Content
// ---------------------------------------------------------------------------

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

/** A feature that links somewhere. The label defaults to "Read the docs". */
type LinkedFeature = Feature & { href: string; label?: string };

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

/**
 * The hosted tier most buyers land on. The homepage quotes its price from the
 * generated tier table (src/pricing/tiers.ts), the same one the pricing page
 * and the server use, so the figure is not restated here.
 */
const production = TIERS.find((tier) => tier.tier === EMPHASIZED_TIER);

function productionTerms(): string {
  if (!production) return "";
  const price = `${production.price}${production.cadence ?? ""}`;
  return production.baa_available
    ? ` Production plans add a signed BAA and start at ${price}.`
    : ` Production plans start at ${price}.`;
}

/** The ways to take the project, and where its development can be followed. */
const ways: LinkedFeature[] = [
  {
    icon: "server",
    title: "Run it yourself",
    body: "The whole server, free under Apache-2.0. Docker Compose brings up the full stack locally, and container images or a single binary from npm take it to production.",
    href: QUICK_START_URL,
    label: "Self-host in 5 minutes",
  },
  {
    icon: "cloud",
    title: "Or have it hosted",
    body: `The same server, operated for you. Start on a free tenant with no credit card.${productionTerms()}`,
    href: "/pricing",
    label: "See pricing",
  },
  {
    icon: "branch",
    title: "Built in the open",
    body: "Development happens in the public repository, and each month's changes are written up in a feature update. The conformance tests are public too, known gaps included.",
    href: "/blog",
    label: "Read the latest updates",
  },
];

// ---------------------------------------------------------------------------
// Building blocks
// ---------------------------------------------------------------------------

/**
 * A large feature card. Below the product grid the page is a column of
 * these, each one subject on a white panel, instead of full-bleed bands.
 */
function Panel({
  children,
  className,
}: Readonly<{ children: ReactNode; className?: string }>) {
  return (
    <div
      className={clsx(
        "overflow-hidden rounded-3xl border border-slate-200 bg-white",
        className,
      )}
      data-reveal=""
    >
      {children}
    </div>
  );
}

/** A heading inside a panel, one size down from a section header. */
function PanelHeader({
  eyebrow,
  title,
  children,
  centered = false,
}: Readonly<{
  eyebrow: string;
  title: string;
  children?: ReactNode;
  centered?: boolean;
}>) {
  return (
    <div className={clsx("max-w-[40rem]", centered && "mx-auto text-center")}>
      <Eyebrow>{eyebrow}</Eyebrow>
      <Heading
        as="h2"
        className="mt-3 text-[1.75rem] font-semibold leading-[1.15] tracking-[-0.02em] text-ink-950 text-balance md:text-[2.25rem]"
      >
        {title}
      </Heading>
      {children ? (
        <p className="mt-4 text-[1.0625rem] leading-relaxed text-slate-600 md:text-lg">
          {children}
        </p>
      ) : null}
    </div>
  );
}

function FeatureCard({ feature }: Readonly<{ feature: LinkedFeature }>) {
  return (
    <LinkCard
      to={feature.href}
      icon={<IconTile name={feature.icon} />}
      title={feature.title}
      label={feature.label ?? "Read the docs"}
    >
      {feature.body}
    </LinkCard>
  );
}

// ---------------------------------------------------------------------------
// Sections
// ---------------------------------------------------------------------------

function Hero() {
  return (
    <section>
      <Container className="pb-6 pt-10 md:pb-8 md:pt-16">
        {/* Headline left, the pitch and the buttons right. The headline is
            centred on the pitch and the buttons: its bottom padding offsets
            the small line under the buttons, which would otherwise pull it
            down. The grid follows at the same gap it uses between its tiles. */}
        <div
          className={clsx(
            site.rise,
            "grid grid-cols-1 gap-6 lg:grid-cols-[minmax(0,7fr)_minmax(0,5fr)] lg:items-center lg:gap-10",
          )}
        >
          {/* The category noun a buyer actually shortlists on ("FHIR
              server") leads; AI-native is the audience, not the category.
              The two lines are the social card's (scripts/generate/
              social-card.mjs), with the same accent word. */}
          <Heading
            as="h1"
            className="m-0 text-[2.5rem] font-semibold leading-[1.02] tracking-[-0.035em] text-ink-950 text-balance md:text-5xl lg:pb-10 xl:text-[3.25rem]"
          >
            <span className="lg:block">
              The <span className="whitespace-nowrap">Open-Source</span>{" "}
              <span className="whitespace-nowrap">FHIR Server</span>
            </span>{" "}
            <span className="lg:block">
              for{" "}
              <span className="whitespace-nowrap text-brand-700">
                AI-Native
              </span>{" "}
              Health Apps
            </span>
          </Heading>

          <div className="max-w-[36rem]">
            {/* The first sentence is the social card's subhead, word for
                word, so a link preview and the page it opens say the same
                thing. */}
            <p className="m-0 text-lg leading-relaxed text-slate-600 text-balance">
              <span className="font-medium text-ink-950">
                Epic, Oracle Health and HL7v2 in. FHIR, SQL and MCP out.
              </span>{" "}
              One governed clinical data layer for your applications and your AI
              agents. Self-host it under Apache-2.0, or let us run it for you
              with a signed BAA.
            </p>

            {/* The hosted free tenant is the lowest-friction start and the
                path to a paid tier; the demo is for the buyer who wants a
                person. Self-hosting stays one click away on the line below. */}
            <div className="mt-5 flex flex-wrap items-center gap-2.5">
              <Button to={SIGNUP_URL} size="sm">
                Start for free
              </Button>
              <Button to={DEMO_URL} variant="secondary" size="sm">
                Book a demo
              </Button>
            </div>
            <p className="m-0 mt-4 text-sm text-slate-500">
              Free hosted tenant, no credit card. Or{" "}
              <Link
                to={QUICK_START_URL}
                className="font-medium text-slate-700 underline decoration-slate-300 underline-offset-4 hover:text-ink-950 hover:decoration-slate-500"
              >
                self-host in 5 minutes
              </Link>
              .
            </p>
          </div>
        </div>
      </Container>
    </section>
  );
}

function Products() {
  return (
    <section className="pb-6 md:pb-8">
      <Container>
        <ProductGrid />
      </Container>
    </section>
  );
}

function AiNative() {
  const { mcpTools } = useHomepageFacts();

  return (
    <section className="py-4 md:py-5">
      <Container>
        <Panel>
          <div className="grid lg:grid-cols-12">
            <div className="p-7 md:p-10 lg:col-span-5 lg:p-12">
              <PanelHeader
                eyebrow="AI-native"
                title="Agents are first-class clients, held to first-class rules"
              >
                Every project serves a Model Context Protocol endpoint beside
                its REST API. Agents get typed tools and real FHIR schemas
                instead of a scraped UI, and each call they make is
                authenticated, authorized and audited like any other request.
              </PanelHeader>

              <div className="mt-8 flex flex-wrap items-center gap-2">
                <span className="mr-1 text-[0.9375rem] font-medium text-slate-500">
                  What you can build:
                </span>
                {agentUseCases.map((useCase) => (
                  <span
                    key={useCase}
                    className="rounded-full border border-slate-200 bg-white px-3 py-1 text-[0.8125rem] font-medium text-slate-600"
                  >
                    {useCase}
                  </span>
                ))}
              </div>

              <div className="mt-8 flex flex-col gap-3 sm:flex-row">
                <Button to="/docs/category/ai" size="sm">
                  Connect Claude or Gemini
                </Button>
                <Button
                  to="/docs/api/rest_api/model_context_protocol/tools"
                  variant="secondary"
                  size="sm"
                >
                  Browse the MCP tools
                </Button>
              </div>
            </div>

            <div className="flex items-center border-t border-slate-200 bg-slate-50 p-4 sm:p-6 lg:col-span-7 lg:border-l lg:border-t-0 lg:p-8">
              <div className="w-full">
                <AgentSession />
              </div>
            </div>
          </div>

          {/* Hairlines between the cells: a 1px gap over a slate background. */}
          <div className="grid gap-px border-t border-slate-200 bg-slate-200 sm:grid-cols-2 lg:grid-cols-4">
            {agentFeatures(mcpTools).map((feature) => (
              <div key={feature.title} className="bg-white p-6 md:p-7">
                <IconTile name={feature.icon} size="sm" />
                <h3 className="m-0 mt-4 text-[1.0625rem] font-semibold tracking-tight text-ink-950">
                  {feature.title}
                </h3>
                <p className="m-0 mt-1.5 text-[0.9375rem] leading-relaxed text-slate-600">
                  {feature.body}
                </p>
              </div>
            ))}
          </div>
        </Panel>
      </Container>
    </section>
  );
}

function HowItFits() {
  return (
    <section className="py-4 md:py-5">
      <Container>
        <Panel>
          <div className="px-7 pt-10 md:px-10 md:pt-12">
            <PanelHeader
              eyebrow="How it fits"
              title="One clinical data layer between your systems of record and everything you build"
              centered
            >
              Point EHRs, HL7v2 interfaces and other FHIR servers at Haste
              Health. What comes out is a single, normalized FHIR R4 API for
              your applications, your analysts and your AI agents.
            </PanelHeader>
          </div>
          <div className="p-5 sm:p-8 md:p-12">
            <DataFlowDiagram />
          </div>
        </Panel>
      </Container>
    </section>
  );
}

function Proof() {
  const { conformance, suites } = useHomepageFacts();
  // Every suite's checks, not just the core suite the grid below shows.
  const checks = suites.reduce((sum, suite) => sum + suite.assertions, 0);
  const coverage = [
    { value: checks, label: "automated checks" },
    { value: conformance.searchParameters, label: "search parameters" },
    { value: conformance.backends.length, label: "search backends" },
  ];

  return (
    <section className="py-4 md:py-5">
      <Container>
        <div className="grid gap-4 lg:grid-cols-2">
          <Panel className="flex flex-col p-7 md:p-10">
            <PanelHeader
              eyebrow="Performance"
              title="Fast under load, without oversized infrastructure"
            >
              A Rust core keeps latency low and throughput high on ordinary
              hardware.
            </PanelHeader>

            <dl className="mt-8 grid grid-cols-2 gap-x-6 gap-y-8">
              {performance.map((stat, index) => (
                // Term first in the markup, figure first on screen. The
                // column is reversed, so justify-end pins the figure to the
                // top.
                <div
                  key={stat.label}
                  className="flex flex-col-reverse justify-end border-t border-slate-200 pt-4"
                  data-reveal=""
                  style={revealDelay(index)}
                >
                  <dt className="mt-1.5 text-[0.9375rem] leading-snug text-slate-600">
                    {stat.label}
                  </dt>
                  <dd className="m-0 text-[2.25rem] font-semibold leading-none tracking-[-0.03em] text-ink-950 md:text-[2.75rem]">
                    {stat.prefix}
                    <CountUp value={stat.value} />
                    <span className="ml-1 text-xl font-semibold tracking-normal text-slate-500">
                      {stat.unit}
                    </span>
                  </dd>
                </div>
              ))}
            </dl>
            <p className="m-0 mt-auto pt-8 text-sm leading-relaxed text-slate-500">
              Benchmarked on a single machine with Postgres 18 and a
              Synthea-generated dataset, 10 threads.
            </p>
          </Panel>

          <Panel className="flex flex-col p-7 md:p-10">
            <PanelHeader
              eyebrow="Conformance"
              title="Tested against the specification, in public"
            >
              Every claim about FHIR support is backed by a test you can read.
              Known gaps are published, not hidden.
            </PanelHeader>

            <p className="m-0 mt-8 flex flex-wrap items-baseline gap-x-3 gap-y-1">
              <span className="text-[2.25rem] font-semibold leading-none tracking-[-0.03em] text-ink-950 md:text-[2.75rem]">
                <CountUp value={conformance.passing} />
                <span className="text-slate-400">
                  {" "}
                  / {conformance.resourceTypes}
                </span>
              </span>
              <span className="text-[1.0625rem] text-slate-600">
                resource types pass every check
              </span>
            </p>
            <div className="mt-5">
              <ConformanceGrid />
            </div>

            <dl className="mt-6 grid grid-cols-3 gap-4 border-t border-slate-200 pt-6">
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
            <p className="m-0 mt-5 text-sm leading-relaxed text-slate-500">
              The checks span {suites.length} test suites:{" "}
              {suites.map((suite) => suite.title).join(", ")}. Each square is
              one resource type in the core suite, run in CI against both
              PostgreSQL and Elasticsearch.
            </p>
            <div className="mt-auto pt-6">
              <ArrowLink to="/docs/reference/conformance/overview">
                See every conformance report
              </ArrowLink>
            </div>
          </Panel>
        </div>
      </Container>
    </section>
  );
}

function Security() {
  return (
    <section className="py-4 md:py-5">
      <Container>
        <Panel className="grid lg:grid-cols-12">
          <div className="p-7 md:p-10 lg:col-span-5 lg:p-12">
            <PanelHeader
              eyebrow="Security and governance"
              title="The controls a security review asks for, on by default"
            >
              Clinical data needs more than an API key. Haste Health ships its
              own identity provider, policy engine and audit trail, and applies
              them to people, services and autonomous agents alike.
            </PanelHeader>
            <div className="mt-8 flex flex-col gap-3 sm:flex-row">
              <Button
                to="/docs/core_concepts/identity_access_control"
                size="sm"
              >
                Review the security model
              </Button>
              <Button
                to="/docs/auth/authorization/access_control"
                variant="secondary"
                size="sm"
              >
                Explore access policies
              </Button>
            </div>
          </div>

          {/* Hairlines between the cells: a 1px gap over a slate background. */}
          <div className="grid gap-px border-t border-slate-200 bg-slate-200 sm:grid-cols-2 lg:col-span-7 lg:border-l lg:border-t-0">
            {controls.map((control) => (
              <div key={control.title} className="bg-white p-6 md:p-7">
                <IconTile name={control.icon} size="sm" />
                <h3 className="m-0 mt-4 text-[1.0625rem] font-semibold tracking-tight text-ink-950">
                  {control.title}
                </h3>
                <p className="m-0 mt-1.5 text-[0.9375rem] leading-relaxed text-slate-600">
                  {control.body}
                </p>
              </div>
            ))}
          </div>
        </Panel>
      </Container>
    </section>
  );
}

function OpenSource() {
  return (
    <section className="pb-8 pt-16 md:pb-10 md:pt-24">
      <Container>
        <SectionHeader
          eyebrow="Open source"
          title="Yours to run, with or without us"
          centered
        >
          Everything we build lands in the Apache-2.0 repository, with no
          feature gates and no license key. The hosted service runs the same
          server, on the same storage schema, that you can run yourself.
        </SectionHeader>

        <div className="mt-10 grid gap-4 md:mt-12 lg:grid-cols-3">
          {ways.map((way, index) => (
            <div key={way.title} data-reveal="" style={revealDelay(index)}>
              <FeatureCard feature={way} />
            </div>
          ))}
        </div>
      </Container>
    </section>
  );
}

function FinalCta() {
  return (
    <section className="pb-20 pt-8 md:pb-28 md:pt-10">
      <Container>
        <div
          className="rounded-3xl border border-slate-200 bg-white px-6 py-14 text-center sm:px-10 md:py-20"
          data-reveal=""
        >
          <Heading
            as="h2"
            className="mx-auto max-w-[44rem] text-[2rem] font-semibold leading-[1.12] tracking-[-0.025em] text-ink-950 text-balance md:text-[2.75rem]"
          >
            Build on an open foundation for clinical data and AI
          </Heading>
          <p className="mx-auto mt-5 max-w-[40rem] text-lg leading-relaxed text-slate-600 md:text-xl">
            Start on a free hosted tenant in under a minute, self-host in five,
            or book a demo and talk it through with the founder.
          </p>
          <div className="mt-9 flex flex-col justify-center gap-3 sm:flex-row">
            <Button to={SIGNUP_URL}>Start for free</Button>
            <Button to={DEMO_URL} variant="secondary">
              Book a demo
            </Button>
            <Button to={GITHUB_URL} variant="secondary">
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
        <Products />
        <AiNative />
        <HowItFits />
        <Proof />
        <Security />
        <OpenSource />
        <FinalCta />
      </main>
    </Layout>
  );
}
