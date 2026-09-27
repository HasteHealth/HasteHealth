import React, { ReactNode } from "react";
import Link from "@docusaurus/Link";
import Layout from "@theme/Layout";
import Heading from "@theme/Heading";

import {
  EMPHASIZED_TIER,
  OPERATION_POINTS,
  TIERS,
  type Tier,
  type TierId,
  isUnlimited,
  limitCount,
} from "@site/src/pricing/tiers";

const SIGNUP_URL = "https://api.haste.health/auth/signup";
const DEMO_URL = "https://calendly.com/rp-haste/book-a-demo";
const QUICK_START_URL = "/docs/getting_started/quick_start";

/**
 * The editorial part of a tier: what we say about it, as opposed to what the
 * server enforces. Keyed by tier id so a tier cannot lose its copy silently —
 * every limit, price and SLA still comes from the generated table.
 */
type TierCopy = {
  /** Selling points that are not already one of the enforced limits. */
  includes: string[];
  body: string;
  cta: { label: string; href: string };
};

const COPY: Record<TierId, TierCopy> = {
  unlimited: {
    body: "The whole server under Apache-2.0. No feature gates, no license key, no seat count, no call with us. Run it on your own hardware and keep your data where it already is.",
    cta: { label: "Read the quick start", href: QUICK_START_URL },
    includes: [
      "Every feature in the open-source repository",
      "Multi-tenant Tenant and Project scoping",
      "MCP server, SQL-on-FHIR, custom operations",
      "Postgres-backed search — no Elasticsearch required",
      "Community support via GitHub Issues and Discussions",
    ],
  },
  free: {
    body: "A hosted tenant, live in under a minute, with no credit card. Sized so you can load a real dataset and query it the same day — built for the three days you spend comparing backends, and for the prototype that follows.",
    cta: { label: "Start for free", href: SIGNUP_URL },
    includes: [
      "Room for ~300 Synthea patients, ~13 loadable per day",
      "Full REST API, MCP endpoint and Admin App",
      "Shared infrastructure, no uptime guarantee",
      "Community support",
    ],
  },
  professional: {
    body: "A dedicated hosted tenant with a BAA and a support commitment you can put in front of your own security reviewer. This is the tier most funded digital-health startups land on. Start on the free tier and email us when you are ready to move up.",
    cta: { label: "Start for free", href: SIGNUP_URL },
    includes: [
      "Signed BAA before any PHI reaches us",
      "Daily backups with point-in-time recovery",
      "Bulk $export and versioned dataset snapshots",
      "Custom search parameters, operations and Subscriptions",
      "Your own tenant name and logo",
    ],
  },
  team: {
    body: "For hundreds of millions of resources, machine-volume ingest, or a compliance posture that needs its own conversation. Priced on your actual footprint rather than a seat count.",
    cta: { label: "Book a demo", href: DEMO_URL },
    includes: [
      "Everything in Production",
      "Elasticsearch scale-out for search",
      "Dedicated database and tunable read replicas",
      "Single-tenant deployment or your own VPC",
      "Shared Slack channel and named engineer",
    ],
  },
};

/**
 * The limits block each card shows, drawn from the generated table so the
 * published figure is the enforced one.
 */
function limitsFor(tier: Tier): { label: string; value: string }[] {
  return [
    { label: "Resources", value: tier.total_resources_label },
    { label: "API budget", value: tier.request_budget_label },
    { label: "Support", value: tier.support },
    { label: "Uptime SLA", value: tier.uptime_sla },
  ];
}

/**
 * The one-line explanation of what a tier's budget buys, so "250,000 points /
 * day" means something without leaving the card.
 */
function budgetHint(tier: Tier): string | null {
  const { searches_per_day, writes_per_day } = tier.budget_examples;

  if (searches_per_day === null || writes_per_day === null) {
    return null;
  }

  return `≈ ${searches_per_day.toLocaleString()} searches or ${writes_per_day.toLocaleString()} writes per day`;
}

const overages = [
  {
    what: "Additional resources",
    cost: "$0.20 per 10,000 / month",
    note: "Counted as stored current versions. History is not billed.",
  },
  {
    what: "Additional request points",
    cost: "$0.50 per 100,000",
    note: `A read costs ${OPERATION_POINTS.read} point, a search ${OPERATION_POINTS.search}, a write ${OPERATION_POINTS.write}.`,
  },
  {
    what: "Bulk $export egress",
    cost: "$0.09 per GB",
    note: "First 100 GB each month is included on Production and Scale.",
  },
  {
    what: "Attachment storage",
    cost: "$0.03 per GB / month",
    note: "Binary and DocumentReference content in object storage.",
  },
];

const faqs = [
  {
    q: "Is the open-source version crippled?",
    a: "No. There is no feature gate and no license key. Everything we build lands in the Apache-2.0 repository, including the Postgres-only search backend, the MCP server and SQL-on-FHIR. The hosted tiers sell operations and compliance — backups, upgrades, a BAA and someone on call — not access to code.",
  },
  {
    q: "What is a request point?",
    a: `The unit your API budget is spent in, because not every request costs the server the same. A read by id is ${OPERATION_POINTS.read} point, a search ${OPERATION_POINTS.search}, a history read ${OPERATION_POINTS.history}, an operation ${OPERATION_POINTS.invocation}, and a write ${OPERATION_POINTS.write}. A batch or transaction costs the sum of its entries. Writes carry the most weight because storage is append-only: a write adds a history row and re-indexes that resource across every search parameter it touches, and unlike a search it cannot be served from a read replica. Budgets are per day, and every figure on this page is generated from the same table the server enforces.`,
  },
  {
    q: "Why is a price on this page at all, this early?",
    a: "Because you cannot evaluate a backend you cannot budget for. These numbers are real and you can sign up against them today. They will also move as we learn; we would rather publish and revise in public than make you book a call to find out whether we are in your range.",
  },
  {
    q: "Do you sign a BAA?",
    a: "Yes, on Production and Scale, before you send us any PHI. Our BAA template, subprocessor list, data-flow diagram and a pre-answered security questionnaire are available up front, so your security review does not start with a scheduling email. We do not have a SOC 2 report yet — ask and we will tell you exactly where that work stands.",
  },
  {
    q: "What happens if I exceed my included usage?",
    a: "On the paid tiers, nothing breaks: overage is metered at the rates above and appears on your next invoice. If your usage settles at a consistently higher level we will move you to a plan that costs less than the overage. The Developer tier is the exception — it is a sandbox, so writes are refused once you reach its caps rather than billed to you.",
  },
  {
    q: "Can I move between self-hosted and hosted?",
    a: "In both directions. It is the same server and the same storage schema, so a migration is a bulk export and a bulk import. We will not hold your data to keep your business.",
  },
  {
    q: "Is there a discount for non-profits or research?",
    a: "Yes. Academic, non-profit and open-source projects get Production at no cost. Email business@haste.health with a sentence about the work.",
  },
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
        className="text-2xl md:text-3xl font-bold tracking-tight text-brand-950"
      >
        {props.title}
      </Heading>
      {props.subtitle ? (
        <p className="max-w-3xl text-base text-slate-700 leading-relaxed">
          {props.subtitle}
        </p>
      ) : null}
    </div>
  );
}

/**
 * One tier card.
 *
 * The card is a subgrid spanning the eleven rows the parent defines, so every
 * section sits in the same band in every card regardless of how much copy a tier
 * carries. The eleven bands are:
 *
 *   1 badge   2 heading   3 price   4 body   5 features
 *   6-9 the limits block, one band per label and per value
 *   10 budget hint   11 call to action
 *
 * The limits block gets four of its own bands rather than being a nested grid:
 * `"50,000,000 points / day"` and `"4 business hours"` wrap to two lines in a
 * narrow card while `"Unmetered"` and `"None"` do not, and a nested grid would
 * let each card size those cells independently — which pushes the Support and
 * Uptime SLA labels out of line between cards.
 */
function TierCard(props: Readonly<{ tier: Tier }>): ReactNode {
  const { tier } = props;
  const copy = COPY[tier.tier];
  const emphasis = tier.tier === EMPHASIZED_TIER;
  const hint = budgetHint(tier);

  return (
    <article
      className={[
        "grid rounded-2xl border bg-white p-6",
        "row-span-11 grid-rows-subgrid",
        emphasis
          ? "border-brand-700 shadow-lg ring-1 ring-brand-700"
          : "border-brand-200",
      ].join(" ")}
    >
      {/* Row 1: the emphasis badge. Present but empty in the other cards so
          their headings stay on the same line as the emphasized one. */}
      <div>
        {emphasis ? (
          <div className="inline-flex items-center rounded-full bg-brand-700 px-3 py-1 text-xs font-semibold uppercase tracking-[0.08em] text-white">
            Most common
          </div>
        ) : null}
      </div>

      {/* Row 2: name and audience. */}
      <div>
        <h3 className="text-xl font-bold text-brand-950">
          {tier.display_name}
        </h3>
        <p className="mt-1 text-sm text-slate-600">{tier.audience}</p>
      </div>

      {/* Row 3: price. */}
      <div className="flex items-baseline gap-1">
        <span className="text-3xl font-bold tracking-tight text-brand-950">
          {tier.price}
        </span>
        {tier.cadence ? (
          <span className="text-sm text-slate-600">{tier.cadence}</span>
        ) : null}
      </div>

      {/* Row 4: the pitch. */}
      <p className="text-sm text-slate-700 leading-6">{copy.body}</p>

      {/* Row 5: what you get. Starts at the top of its band so the bullets
          align across cards even when the paragraphs above differ in length. */}
      <ul className="space-y-2 list-none pl-0 self-start">
        {copy.includes.map((item) => (
          <li key={item} className="flex gap-2 text-sm text-slate-700 leading-6">
            <span aria-hidden="true" className="mt-0.5 text-brand-700">
              &#10003;
            </span>
            <span>{item}</span>
          </li>
        ))}
      </ul>

      {/* Rows 6-9: the enforced limits.
          The `dl` is itself a subgrid spanning four card rows — one band per
          label and one per value — so a value that wraps to two lines grows
          that band in every card at once. Nesting these in a plain grid instead
          lets each card size its own cells, which is what knocks the Support
          and Uptime SLA labels out of line between cards. */}
      <dl className="row-span-4 grid grid-cols-2 grid-rows-subgrid gap-x-4 rounded-xl border border-brand-200 bg-brand-50/40 p-4">
        {limitsFor(tier).map((limit, index) => {
          // Two pairs per row of the card's subgrid: the label sits in the band
          // above its value, so `dt` and `dd` are placed explicitly rather than
          // flowing (which would put them side by side).
          const band = index < 2 ? 0 : 2;

          return (
            <React.Fragment key={limit.label}>
              <dt
                className="text-xs font-semibold uppercase tracking-[0.06em] text-slate-500"
                style={{ gridRow: band + 1, gridColumn: (index % 2) + 1 }}
              >
                {limit.label}
              </dt>
              <dd
                className="ml-0 text-sm font-semibold text-brand-900"
                style={{ gridRow: band + 2, gridColumn: (index % 2) + 1 }}
              >
                {limit.value}
              </dd>
            </React.Fragment>
          );
        })}
      </dl>

      {/* Row 10: what the budget buys. */}
      <p className="text-xs text-slate-500 leading-4">{hint}</p>

      {/* Row 11: the call to action, pinned to the bottom of the card. */}
      <Link
        href={copy.cta.href}
        className={[
          "inline-flex w-full items-center justify-center self-end rounded-lg px-4 py-2.5 text-sm font-semibold no-underline hover:no-underline",
          emphasis
            ? "bg-brand-700 text-white hover:bg-brand-800"
            : "border border-brand-300 bg-white text-brand-800 hover:bg-brand-50",
        ].join(" ")}
      >
        {copy.cta.label}
      </Link>
    </article>
  );
}

/**
 * The row bands every card's subgrid aligns to: badge, heading, price, body,
 * features, then the limits-and-CTA footer.
 *
 * Each card spans six rows, so the parent needs six rows per row of cards —
 * twelve in the two-column layout, where the four cards wrap onto two rows. The
 * `1fr` band is the feature list, which absorbs the slack so the footer band
 * lands at the same height in every card.
 *
 * Written out literally rather than composed: Tailwind scans the source for
 * class names and will not emit one that is built by interpolation.
 */
// Bands 1-4 auto (badge, heading, price, body), 5 is the feature list and takes
// the slack, 6-11 auto (four limit bands, hint, CTA).
const CARD_ROWS = [
  // One column: each card is its own row of cards, so one band tiles down.
  "grid-rows-[auto_auto_auto_auto_1fr_auto_auto_auto_auto_auto_auto]",
  // Two columns: the four cards wrap onto two rows of cards, so two bands.
  "lg:grid-rows-[auto_auto_auto_auto_1fr_auto_auto_auto_auto_auto_auto_auto_auto_auto_auto_1fr_auto_auto_auto_auto_auto_auto]",
  // Four columns: a single row of cards.
  "xl:grid-rows-[auto_auto_auto_auto_1fr_auto_auto_auto_auto_auto_auto]",
].join(" ");

/**
 * A comparison row in the limits table. `values` is keyed by tier id, or
 * derived from the generated table.
 */
type ComparisonRow = {
  label: string;
  value: (tier: Tier) => string;
};

const comparisonRows: ComparisonRow[] = [
  { label: "Price", value: (tier) => `${tier.price}${tier.cadence ?? ""}` },
  { label: "Stored resources", value: (tier) => tier.total_resources_label },
  { label: "Request budget", value: (tier) => tier.request_budget_label },
  {
    label: "Projects",
    value: (tier) =>
      isUnlimited(tier.projects) ? "Unlimited" : `${limitCount(tier.projects)}`,
  },
  {
    label: "Custom search parameters",
    value: (tier) => (isUnlimited(tier.search_parameters) ? "Yes" : "—"),
  },
  {
    label: "Custom operations",
    value: (tier) => (isUnlimited(tier.operation_definitions) ? "Yes" : "—"),
  },
  {
    label: "FHIR Subscriptions",
    value: (tier) => (isUnlimited(tier.subscriptions) ? "Yes" : "—"),
  },
  {
    label: "External identity providers",
    value: (tier) => (isUnlimited(tier.identity_providers) ? "Yes" : "—"),
  },
  {
    label: "Tenant name and logo",
    value: (tier) => (tier.tenant_customization ? "Yes" : "—"),
  },
  { label: "Support", value: (tier) => tier.support },
  { label: "Uptime SLA", value: (tier) => tier.uptime_sla },
  { label: "BAA available", value: (tier) => (tier.baa_available ? "Yes" : "—") },
];

export default function Pricing(): ReactNode {
  return (
    <Layout
      wrapperClassName="bg-background"
      title="Pricing"
      description="Haste Health pricing: the FHIR server is free and Apache-2.0 forever when self-hosted. Hosted tiers start free and run from $1,500/month with a BAA and an uptime SLA."
    >
      <main
        id="tw-scope"
        className="container mx-auto px-4 py-8 md:py-12 text-brand-950"
      >
        <section className="rounded-3xl border border-brand-200 bg-white px-6 py-12 md:px-10 md:py-16">
          <div className="max-w-4xl space-y-4">
            <div className="inline-flex items-center rounded-full border border-brand-200 bg-brand-50 px-3 py-1 text-xs font-semibold uppercase tracking-[0.08em] text-brand-800">
              Published prices, revised in public
            </div>
            <Heading
              as="h1"
              className="text-4xl md:text-5xl font-bold tracking-tight text-brand-950"
            >
              Pricing
            </Heading>
            <p className="max-w-3xl text-lg text-slate-700 leading-relaxed">
              The server is free forever when you run it yourself — Apache-2.0,
              no feature gates. The hosted tiers sell the part you would rather
              not operate at 3 a.m.: backups, upgrades, a BAA and someone on
              call.
            </p>
            <div className="flex flex-wrap gap-3 pt-2">
              <Link
                href={SIGNUP_URL}
                className="inline-flex items-center justify-center rounded-lg bg-brand-700 px-5 py-2.5 text-sm font-semibold text-white no-underline hover:bg-brand-800 hover:no-underline"
              >
                Start for free
              </Link>
              <Link
                href={DEMO_URL}
                className="inline-flex items-center justify-center rounded-lg border border-brand-300 bg-white px-5 py-2.5 text-sm font-semibold text-brand-800 no-underline hover:bg-brand-50 hover:no-underline"
              >
                Book a demo
              </Link>
            </div>
          </div>
        </section>

        {/* The cards are subgrids of this grid, which is what keeps each
            section of every card on a shared baseline. */}
        <section
          className={`mt-10 grid gap-5 lg:grid-cols-2 xl:grid-cols-4 ${CARD_ROWS}`}
        >
          {TIERS.map((tier) => (
            <TierCard key={tier.tier} tier={tier} />
          ))}
        </section>

        <section className="mt-10 rounded-2xl border border-brand-200 bg-white p-6 md:p-8">
          <SectionTitle
            eyebrow="Side by side"
            title="Every limit, per tier"
            subtitle="Generated from the same table the server enforces, so a figure here is the figure your tenant is held to."
          />
          <div className="mt-6 overflow-x-auto">
            <table className="w-full min-w-[48rem] text-left text-sm">
              <thead>
                <tr className="border-b border-brand-200">
                  <th className="pb-3 pr-4 font-semibold text-brand-900">
                    Limit
                  </th>
                  {TIERS.map((tier) => (
                    <th
                      key={tier.tier}
                      className="pb-3 pr-4 font-semibold text-brand-900"
                    >
                      {tier.display_name}
                    </th>
                  ))}
                </tr>
              </thead>
              <tbody>
                {comparisonRows.map((row) => (
                  <tr key={row.label} className="border-b border-brand-100">
                    <th
                      scope="row"
                      className="py-3 pr-4 text-left font-medium text-brand-900"
                    >
                      {row.label}
                    </th>
                    {TIERS.map((tier) => (
                      <td key={tier.tier} className="py-3 pr-4 text-slate-700">
                        {row.value(tier)}
                      </td>
                    ))}
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </section>

        <section className="mt-10 rounded-2xl border border-brand-200 bg-white p-6 md:p-8">
          <SectionTitle
            eyebrow="Metering"
            title="Usage beyond what your plan includes"
            subtitle="On the paid tiers overage is metered, never throttled. If your usage settles above your plan, we move you to the plan that costs less."
          />
          <div className="mt-6 overflow-x-auto">
            <table className="w-full min-w-[42rem] text-left text-sm">
              <thead>
                <tr className="border-b border-brand-200">
                  <th className="pb-3 pr-4 font-semibold text-brand-900">
                    What
                  </th>
                  <th className="pb-3 pr-4 font-semibold text-brand-900">
                    Rate
                  </th>
                  <th className="pb-3 font-semibold text-brand-900">
                    How it is counted
                  </th>
                </tr>
              </thead>
              <tbody>
                {overages.map((row) => (
                  <tr key={row.what} className="border-b border-brand-100">
                    <td className="py-3 pr-4 font-medium text-brand-900">
                      {row.what}
                    </td>
                    <td className="py-3 pr-4 font-mono text-slate-800">
                      {row.cost}
                    </td>
                    <td className="py-3 text-slate-700">{row.note}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </section>

        <section className="mt-10 rounded-2xl border border-brand-200 bg-white p-6 md:p-8">
          <SectionTitle
            eyebrow="Compliance"
            title="What your security reviewer gets on day one"
            subtitle="None of this is gated behind a sales call, and we are straight about what we do not have yet."
          />
          <div className="mt-6 grid gap-4 md:grid-cols-3">
            {[
              {
                title: "BAA, ready to sign",
                body: "Our standard Business Associate Agreement, signed before any PHI reaches us. Available on Production and Scale.",
              },
              {
                title: "Pre-answered questionnaire",
                body: "CAIQ and SIG Lite answered in advance, plus our subprocessor list, data-flow diagram and incident response policy.",
              },
              {
                title: "Where we are on SOC 2",
                body: "We do not have a SOC 2 report yet, and we would rather tell you that here than in month two of your procurement. Ask us where the work stands and we will be specific about scope and timing.",
              },
            ].map((card) => (
              <article
                key={card.title}
                className="flex flex-col rounded-xl border border-brand-200 bg-brand-50/40 p-5"
              >
                <h3 className="text-lg font-semibold text-brand-900">
                  {card.title}
                </h3>
                <p className="mt-2 flex-1 text-sm text-slate-700 leading-6">
                  {card.body}
                </p>
              </article>
            ))}
          </div>
          <p className="mt-6 text-sm text-slate-700">
            Request the pack at{" "}
            <Link href="mailto:security@haste.health">
              security@haste.health
            </Link>
            .
          </p>
        </section>

        <section className="mt-10 rounded-2xl border border-brand-200 bg-white p-6 md:p-8">
          <SectionTitle eyebrow="Questions" title="Frequently asked" />
          <div className="mt-6 divide-y divide-brand-100">
            {faqs.map((faq) => (
              <div key={faq.q} className="py-5 first:pt-0 last:pb-0">
                <h3 className="text-base font-semibold text-brand-900">
                  {faq.q}
                </h3>
                <p className="mt-2 max-w-3xl text-sm text-slate-700 leading-6">
                  {faq.a}
                </p>
              </div>
            ))}
          </div>
        </section>

        <section className="mt-10 rounded-2xl border border-brand-700 bg-brand-50/60 p-6 md:p-8">
          <div className="flex flex-col gap-4 md:flex-row md:items-center md:justify-between">
            <div className="max-w-2xl space-y-2">
              <Heading
                as="h2"
                className="text-2xl font-bold tracking-tight text-brand-950"
              >
                Think a number here is wrong?
              </Heading>
              <p className="text-base text-slate-700 leading-relaxed">
                Tell us. These prices are a first draft published so you can
                budget against something real, and design-partner feedback is
                exactly how they get better.
              </p>
            </div>
            <Link
              href="mailto:business@haste.health"
              className="inline-flex shrink-0 items-center justify-center rounded-lg bg-brand-700 px-5 py-2.5 text-sm font-semibold text-white no-underline hover:bg-brand-800 hover:no-underline"
            >
              business@haste.health
            </Link>
          </div>
        </section>
      </main>
    </Layout>
  );
}
