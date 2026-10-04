import React, { ReactNode } from "react";
import clsx from "clsx";
import Link from "@docusaurus/Link";
import Layout from "@theme/Layout";

import {
  EMPHASIZED_TIER,
  OPERATION_POINTS,
  TIERS,
  type Tier,
  type TierId,
  isUnlimited,
  limitCount,
} from "@site/src/pricing/tiers";
import { Icon } from "@site/src/components/site/icons";
import {
  revealDelay,
  riseDelay,
  useScrollReveal,
} from "@site/src/components/site/motion";
import site from "@site/src/components/site/styles.module.css";
import {
  Button,
  Container,
  PageHero,
  SectionHeader,
} from "@site/src/components/site/ui";
import { DEMO_URL, QUICK_START_URL, SIGNUP_URL } from "@site/src/links";

/**
 * The editorial part of a tier: what we say about it, as opposed to what the
 * server enforces. Keyed by tier id so a tier cannot lose its copy silently —
 * every limit, price and SLA still comes from the generated table.
 */
type TierCopy = {
  /** Selling points that are not already one of the enforced limits. */
  includes: string[];
  /**
   * On the roadmap for this tier but not built yet. Listed apart from
   * `includes` and labelled, so the card never ticks off something a customer
   * cannot use today. Move an entry up to `includes` when it ships.
   */
  planned?: string[];
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
      "Custom search parameters, operations and Subscriptions",
      "Your own tenant name and logo",
    ],
    planned: ["Bulk $export and versioned dataset snapshots"],
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
    q: "Do you sign a BAA?",
    a: "Yes, on Production and Scale, before you send us any PHI. Our BAA template, subprocessor list, data-flow diagram and a pre-answered security questionnaire are available up front, so your security review does not start with a scheduling email. We do not have a SOC 2 report yet — ask and we will tell you exactly where that work stands.",
  },
  {
    q: "What happens if I exceed my included usage?",
    a: "On the paid tiers, nothing breaks: overage is metered at the rates above and appears on your next invoice. If your usage settles at a consistently higher level we will move you to a plan that costs less than the overage. The Developer tier is the exception it is a sandbox, so writes are refused once you reach its caps rather than billed to you.",
  },
  {
    q: "Can I move between self-hosted and hosted?",
    a: "It is the same server and the same storage schema in both places. To move from hosted to self-hosted today, ask us for an export of your data and we will provide it. A self-serve bulk export is planned.",
  },
];

const compliance = [
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
];

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
      className={clsx(
        "grid rounded-2xl border bg-white p-6",
        "row-span-11 grid-rows-subgrid",
        emphasis
          ? "border-brand-700 shadow-[0_24px_60px_-28px_rgb(2_44_40/0.5)] ring-1 ring-brand-700"
          : "border-slate-200 shadow-[0_1px_2px_rgb(15_23_42/0.04)]",
      )}
    >
      {/* Row 1: the emphasis badge. Present but empty in the other cards so
          their headings stay on the same line as the emphasized one. */}
      <div>
        {emphasis ? (
          <div className="inline-flex items-center rounded-full bg-brand-800 px-3 py-1 text-xs font-semibold uppercase tracking-[0.12em] text-white">
            Most common
          </div>
        ) : null}
      </div>

      {/* Row 2: name and audience. */}
      <div>
        <h3 className="text-xl font-semibold tracking-tight text-ink-950">
          {tier.display_name}
        </h3>
        <p className="mt-1 text-sm text-slate-500">{tier.audience}</p>
      </div>

      {/* Row 3: price. */}
      <div className="flex items-baseline gap-1">
        <span className="text-3xl font-semibold tracking-tight text-ink-950">
          {tier.price}
        </span>
        {tier.cadence ? (
          <span className="text-sm text-slate-500">{tier.cadence}</span>
        ) : null}
      </div>

      {/* Row 4: the pitch. */}
      <p className="text-[0.9375rem] leading-relaxed text-slate-600">
        {copy.body}
      </p>

      {/* Row 5: what you get. Starts at the top of its band so the bullets
          align across cards even when the paragraphs above differ in length. */}
      <ul className="list-none space-y-2.5 self-start pl-0">
        {copy.includes.map((item) => (
          <li
            key={item}
            className="flex gap-2.5 text-[0.9375rem] leading-snug text-slate-700"
          >
            <Icon name="check" className="mt-0.5 h-4 w-4 text-brand-700" />
            <span>{item}</span>
          </li>
        ))}
        {/* Not built yet: a hollow marker and a label, never a tick. */}
        {copy.planned?.map((item) => (
          <li
            key={item}
            className="flex gap-2.5 text-[0.9375rem] leading-snug text-slate-500"
          >
            <span
              className="mt-0.5 flex h-4 w-4 shrink-0 items-center justify-center"
              aria-hidden="true"
            >
              <span className="h-1.5 w-1.5 rounded-full border border-slate-400" />
            </span>
            <span>
              {item}{" "}
              <span className="ml-0.5 whitespace-nowrap rounded-full border border-slate-300 px-2 py-0.5 text-[0.6875rem] font-semibold uppercase tracking-[0.08em] text-slate-500">
                Planned
              </span>
            </span>
          </li>
        ))}
      </ul>

      {/* Rows 6-9: the enforced limits.
          The `dl` is itself a subgrid spanning four card rows — one band per
          label and one per value — so a value that wraps to two lines grows
          that band in every card at once. Nesting these in a plain grid instead
          lets each card size its own cells, which is what knocks the Support
          and Uptime SLA labels out of line between cards. */}
      <dl className="row-span-4 grid grid-cols-2 grid-rows-subgrid gap-x-4 rounded-xl border border-slate-200 bg-slate-50 p-4">
        {limitsFor(tier).map((limit, index) => {
          // Two pairs per row of the card's subgrid: the label sits in the band
          // above its value, so `dt` and `dd` are placed explicitly rather than
          // flowing (which would put them side by side).
          const band = index < 2 ? 0 : 2;

          return (
            <React.Fragment key={limit.label}>
              <dt
                className="text-xs font-semibold uppercase tracking-[0.08em] text-slate-500"
                style={{ gridRow: band + 1, gridColumn: (index % 2) + 1 }}
              >
                {limit.label}
              </dt>
              <dd
                className="ml-0 text-sm font-semibold text-ink-950"
                style={{ gridRow: band + 2, gridColumn: (index % 2) + 1 }}
              >
                {limit.value}
              </dd>
            </React.Fragment>
          );
        })}
      </dl>

      {/* Row 10: what the budget buys. */}
      <p className="text-xs leading-4 text-slate-500">{hint}</p>

      {/* Row 11: the call to action, pinned to the bottom of the card. */}
      <Button
        to={copy.cta.href}
        size="sm"
        variant={emphasis ? "primary" : "secondary"}
        className="w-full self-end"
      >
        {copy.cta.label}
      </Button>
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
  {
    label: "BAA available",
    value: (tier) => (tier.baa_available ? "Yes" : "—"),
  },
];

export default function Pricing(): ReactNode {
  const page = useScrollReveal<HTMLElement>();

  return (
    <Layout
      wrapperClassName="bg-white"
      title="Pricing"
      description="Haste Health pricing: the FHIR server is free and Apache-2.0 forever when self-hosted. Hosted tiers start free and run from $1,500/month with a BAA and an uptime SLA."
    >
      <main id="tw-scope" ref={page} className="text-ink-950">
        <PageHero
          badge="Published pricing"
          title="Pricing"
          overlap
          actions={
            <>
              <Button to={SIGNUP_URL} on="dark">
                Start for free
              </Button>
              <Button to={DEMO_URL} variant="secondary" on="dark">
                Book a demo
              </Button>
            </>
          }
        >
          The server is free with self-hosting and licensed under Apache-2.0.
          The hosted tiers cover the operational work of running it in
          production: backups, upgrades, a signed BAA, and dedicated support.
        </PageHero>

        {/* The cards ride up over the bottom of the hero. They are subgrids of
            this grid, which is what keeps each section of every card on a
            shared baseline. */}
        <section className="relative z-10 -mt-28 pb-16 md:-mt-32 md:pb-24">
          <Container>
            <div className={site.rise} style={riseDelay(300)}>
              <div
                className={`grid gap-5 lg:grid-cols-2 xl:grid-cols-4 ${CARD_ROWS}`}
              >
                {TIERS.map((tier) => (
                  <TierCard key={tier.tier} tier={tier} />
                ))}
              </div>
            </div>

            <p className="mt-8 text-center text-base text-slate-600">
              Doesn&apos;t fit your situation? Tell us at{" "}
              <Link
                href="mailto:business@haste.health"
                className="font-semibold text-brand-800 underline decoration-brand-300 underline-offset-4 hover:text-brand-900 hover:decoration-brand-600"
              >
                business@haste.health
              </Link>
              .
            </p>
          </Container>
        </section>

        <section className="border-y border-slate-200/80 bg-slate-50 py-20 md:py-28">
          <Container>
            <SectionHeader eyebrow="Side by side" title="Every limit, per tier">
              Generated from the same table the server enforces, so a figure
              here is the figure your tenant is held to.
            </SectionHeader>
            <div
              className="mt-10 overflow-x-auto rounded-2xl border border-slate-200 bg-white md:mt-12"
              data-reveal=""
            >
              <table className="table w-full min-w-[48rem] text-left text-[0.9375rem]">
                <thead>
                  <tr className="border-b border-slate-200 bg-transparent">
                    <th className="px-5 py-4 font-semibold text-ink-950">
                      Limit
                    </th>
                    {TIERS.map((tier) => (
                      <th
                        key={tier.tier}
                        className={clsx(
                          "px-5 py-4 font-semibold text-ink-950",
                          tier.tier === EMPHASIZED_TIER && "bg-brand-50/60",
                        )}
                      >
                        {tier.display_name}
                      </th>
                    ))}
                  </tr>
                </thead>
                <tbody>
                  {comparisonRows.map((row) => (
                    <tr
                      key={row.label}
                      className="border-b border-slate-200 bg-transparent last:border-b-0"
                    >
                      <th
                        scope="row"
                        className="px-5 py-3.5 text-left font-medium text-ink-950"
                      >
                        {row.label}
                      </th>
                      {TIERS.map((tier) => (
                        <td
                          key={tier.tier}
                          className={clsx(
                            "px-5 py-3.5 text-slate-600",
                            tier.tier === EMPHASIZED_TIER && "bg-brand-50/60",
                          )}
                        >
                          {row.value(tier)}
                        </td>
                      ))}
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          </Container>
        </section>

        <section className="bg-white py-20 md:py-28">
          <Container>
            <SectionHeader
              eyebrow="Metering"
              title="Usage beyond what your plan includes"
            >
              On the paid tiers overage is metered, never throttled. If your
              usage settles above your plan, we move you to the plan that costs
              less.
            </SectionHeader>
            <div
              className="mt-10 overflow-x-auto rounded-2xl border border-slate-200 bg-white md:mt-12"
              data-reveal=""
            >
              <table className="table w-full min-w-[42rem] text-left text-[0.9375rem]">
                <thead>
                  <tr className="border-b border-slate-200 bg-transparent">
                    <th className="px-5 py-4 font-semibold text-ink-950">
                      What
                    </th>
                    <th className="px-5 py-4 font-semibold text-ink-950">
                      Rate
                    </th>
                    <th className="px-5 py-4 font-semibold text-ink-950">
                      How it is counted
                    </th>
                  </tr>
                </thead>
                <tbody>
                  {overages.map((row) => (
                    <tr
                      key={row.what}
                      className="border-b border-slate-200 bg-transparent last:border-b-0"
                    >
                      <td className="px-5 py-3.5 font-medium text-ink-950">
                        {row.what}
                      </td>
                      <td className="px-5 py-3.5 font-mono text-[0.875rem] text-ink-900">
                        {row.cost}
                      </td>
                      <td className="px-5 py-3.5 text-slate-600">{row.note}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          </Container>
        </section>

        <section className="relative isolate py-20 text-white md:py-28">
          <div
            className={clsx(site.darkBackdrop, "-z-10")}
            aria-hidden="true"
          />
          <Container>
            <SectionHeader
              eyebrow="Compliance"
              title="What your security reviewer gets on day one"
              on="dark"
            >
              None of this is gated behind a sales call, and we are straight
              about what we do not have yet.
            </SectionHeader>
            <div className="mt-12 grid gap-5 md:grid-cols-3 lg:mt-14">
              {compliance.map((card, index) => (
                <article
                  key={card.title}
                  className="flex flex-col rounded-2xl border border-white/10 bg-white/[0.04] p-6 md:p-7"
                  data-reveal=""
                  style={revealDelay(index)}
                >
                  <h3 className="text-xl font-semibold tracking-tight text-white">
                    {card.title}
                  </h3>
                  <p className="mt-2 flex-1 text-base leading-relaxed text-slate-300">
                    {card.body}
                  </p>
                </article>
              ))}
            </div>
            <p className="mt-8 text-base text-slate-300" data-reveal="">
              Request the pack at{" "}
              <Link
                href="mailto:security@haste.health"
                className="font-semibold text-brand-300 underline decoration-brand-300/40 underline-offset-4 hover:text-brand-200 hover:decoration-brand-200"
              >
                security@haste.health
              </Link>
              .
            </p>
          </Container>
        </section>

        <section className="bg-white py-20 md:py-28">
          <Container>
            <div className="grid gap-10 lg:grid-cols-12 lg:gap-12">
              <div className="lg:col-span-4">
                <SectionHeader eyebrow="Questions" title="Frequently asked" />
              </div>
              <div className="divide-y divide-slate-200 border-y border-slate-200 lg:col-span-8">
                {faqs.map((faq) => (
                  <div key={faq.q} className="py-7" data-reveal="">
                    <h3 className="text-xl font-semibold tracking-tight text-ink-950">
                      {faq.q}
                    </h3>
                    <p className="mt-3 text-[1.0625rem] leading-relaxed text-slate-600">
                      {faq.a}
                    </p>
                  </div>
                ))}
              </div>
            </div>
          </Container>
        </section>
      </main>
    </Layout>
  );
}
