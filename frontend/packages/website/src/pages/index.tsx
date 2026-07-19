import React, { ReactNode } from "react";
import Link from "@docusaurus/Link";
import useDocusaurusContext from "@docusaurus/useDocusaurusContext";
import Layout from "@theme/Layout";
import Heading from "@theme/Heading";

const capabilityCards = [
  {
    title: "FHIR Native",
    body: "Interoperate out of the box with FHIR APIs, search, and operations.",
  },
  {
    title: "Extensible Platform",
    body: "Extend server behavior with TypeScript plugins for custom business logic and APIs.",
  },
  {
    title: "SMART and OIDC",
    body: "Use standards-based authN/authZ for provider apps, patient apps, and backend services.",
  },
  {
    title: "High-Performance Core",
    body: "Run on a Rust-native core optimized for low latency and predictable high throughput.",
  },
];

const architectureSteps = [
  {
    title: "Ingest",
    body: "Accept data through FHIR APIs, HL7v2 messages, and system-to-system pipelines.",
  },
  {
    title: "Normalize",
    body: "Store resources in a consistent model and index them for fast retrieval and search.",
  },
  {
    title: "Serve",
    body: "Deliver secure, real-time data to EHR workflows, analytics, and AI-driven products.",
  },
];

const outcomeStats = [
  { value: "<10ms", label: "Create and update latency" },
  { value: ">25k/s", label: "Writes per second on 10 threads" },
  { value: "<50ms", label: "Typical search response" },
  { value: "<100MB", label: "Memory footprint per instance" },
];

const useCases = [
  {
    title: "Provider and EHR Platforms",
    body: "Power patient timelines, clinician workflows, and care coordination with normalized clinical data.",
    href: "/docs/integration/healthcare_systems/ehr",
  },
  {
    title: "Payers and Utilization Systems",
    body: "Support eligibility, prior authorization, and claims-adjacent workflows with FHIR-first APIs.",
    href: "/docs/integration/healthcare_systems/payers_insurance",
  },
  {
    title: "AI and Decision Support",
    body: "Expose governed clinical context for LLM applications and model-driven automation.",
    href: "/docs/category/ai",
  },
];

function SectionTitle(props: Readonly<{ title: string; subtitle?: string }>) {
  return (
    <div className="space-y-3">
      <Heading
        as="h2"
        className="text-3xl md:text-4xl font-bold text-brand-950"
      >
        {props.title}
      </Heading>
      {props.subtitle ? (
        <p className="text-base md:text-lg text-slate-700 max-w-4xl">
          {props.subtitle}
        </p>
      ) : null}
    </div>
  );
}

export default function Home(): ReactNode {
  const { siteConfig } = useDocusaurusContext();

  return (
    <Layout
      wrapperClassName="bg-background"
      title={`Haste Health`}
      description="Modern healthcare clinical data repository. Built for performance and scale."
    >
      <meta name="algolia-site-verification" content="A94F28B6A640A6FE" />
      <main
        id="tw-scope"
        className="container mx-auto px-4 py-8 md:py-12 text-brand-950"
      >
        <section className="rounded-3xl border border-brand-200 bg-linear-to-br from-brand-50 via-white to-brand-100 px-6 py-12 md:px-10 md:py-16">
          <div className="max-w-5xl space-y-8">
            <div className="inline-flex items-center rounded-full border border-brand-200 bg-white px-4 py-2 text-sm font-semibold text-brand-900">
              Open-source, high-performance FHIR server
            </div>

            <Heading
              as="h1"
              className="text-4xl md:text-6xl font-bold tracking-tight text-brand-950"
            >
              {siteConfig.title}: API-First Clinical Data Infrastructure
            </Heading>

            <p className="max-w-4xl text-lg md:text-2xl text-slate-700 leading-relaxed">
              Build healthcare products on a standards-based platform designed
              for interoperability, operational scale, and enterprise-grade
              security.
            </p>

            <div className="flex flex-col gap-3 sm:flex-row sm:items-center">
              <Link
                className="inline-flex items-center justify-center rounded-lg bg-brand-600 px-7 py-3 text-lg font-semibold text-white transition-colors hover:bg-brand-500"
                to="/docs/getting_started/quick_start"
              >
                Start in 5 Minutes
              </Link>
              <Link
                className="inline-flex items-center justify-center rounded-lg border border-brand-300 bg-white px-7 py-3 text-lg font-semibold text-brand-900 transition-colors hover:bg-brand-50"
                to="/docs/overview/what_is_haste_health"
              >
                Read the Overview
              </Link>
            </div>
          </div>
        </section>

        <section className="mt-10 rounded-2xl border border-brand-200 bg-white p-6 md:p-8">
          <SectionTitle
            title="Build Your Own Healthcare Applications on API-First Platform"
            subtitle="Use the same standards-based API surface to power your own product experiences, workflows, and integrations."
          />
          <figure className="mt-6 overflow-hidden rounded-2xl border border-brand-200 bg-white shadow-[0_20px_60px_rgba(15,23,42,0.16)]">
            <img
              src="/img/admin_app.png"
              alt="Haste Health admin app screenshot"
              className="h-auto w-full"
              loading="eager"
            />
          </figure>
          <p className="ml-2 mt-3 text-xs md:text-sm text-slate-500">
            Our Admin App, which is built on the same API your custom
            applications can use.
          </p>
        </section>

        <section className="mt-10 rounded-2xl border border-brand-200 bg-white p-6 md:p-8">
          <SectionTitle
            title="Standards-Based by Design"
            subtitle="Adopt quickly with standards your enterprise teams and partners already trust."
          />
          <div className="mt-6 grid gap-4 md:grid-cols-2 xl:grid-cols-2">
            {capabilityCards.map((item) => (
              <article
                key={item.title}
                className="rounded-xl border border-brand-200 bg-brand-50/40 p-5"
              >
                <h3 className="text-lg font-semibold text-brand-900">
                  {item.title}
                </h3>
                <p className="mt-2 text-sm text-slate-700 leading-6">
                  {item.body}
                </p>
              </article>
            ))}
          </div>
        </section>

        <section className="mt-10 rounded-2xl border border-brand-200 bg-white p-6 md:p-8">
          <SectionTitle
            title="How Haste Health Works"
            subtitle="A predictable data pipeline for enterprise healthcare operations."
          />
          <div className="mt-8 relative">
            <div className="hidden md:block absolute left-10 right-10 top-5 border-t border-brand-200" />
            <ol className="grid gap-6 md:grid-cols-3">
              {architectureSteps.map((step, idx) => (
                <li key={step.title} className="relative">
                  <div className="relative z-10 mb-3 inline-flex h-10 w-10 items-center justify-center rounded-full border-2 border-brand-300 bg-white text-sm font-bold text-brand-700">
                    {idx + 1}
                  </div>
                  <article className="rounded-xl border border-brand-200 bg-brand-50/30 p-5">
                    <h3 className="text-2xl font-semibold text-brand-900">
                      {step.title}
                    </h3>
                    <p className="mt-3 text-sm text-slate-700 leading-6">
                      {step.body}
                    </p>
                  </article>
                </li>
              ))}
            </ol>
          </div>
        </section>

        <section className="mt-10 rounded-2xl border border-brand-200 bg-white p-6 md:p-8">
          <SectionTitle
            title="Performance for Real Clinical Workloads"
            subtitle="Built to keep latency low and throughput high under production traffic."
          />
          <div className="mt-6 grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
            {outcomeStats.map((stat) => (
              <article
                key={stat.label}
                className="rounded-xl border border-brand-200 bg-brand-50/30 p-5"
              >
                <p className="text-3xl font-bold text-brand-900">
                  {stat.value}
                </p>
                <p className="mt-2 text-sm text-slate-700">{stat.label}</p>
              </article>
            ))}
          </div>
          <p className="mt-4 text-sm text-slate-600">
            Write benchmarks exceeded 25k writes per second on 10 threads and
            32k reads per second on 10 threads with a Synthea-generated dataset.
          </p>
        </section>

        <section className="mt-10 rounded-2xl border border-brand-200 bg-white p-6 md:p-8">
          <SectionTitle
            title="Built for Enterprise Healthcare Teams"
            subtitle="Deploy one platform across clinical, operational, and intelligent application domains."
          />
          <div className="mt-6 divide-y divide-brand-200 rounded-xl border border-brand-200">
            {useCases.map((useCase) => (
              <article
                key={useCase.title}
                className="grid gap-3 p-5 md:grid-cols-[1fr_auto] md:items-center"
              >
                <div>
                  <h3 className="text-xl font-semibold text-brand-900">
                    {useCase.title}
                  </h3>
                  <p className="mt-2 text-sm text-slate-700 leading-6">
                    {useCase.body}
                  </p>
                </div>
                <Link
                  to={useCase.href}
                  className="inline-flex items-center justify-center rounded-lg border border-brand-300 px-4 py-2 text-sm font-semibold text-brand-700 hover:bg-brand-50 hover:text-brand-600"
                >
                  Learn more
                </Link>
              </article>
            ))}
          </div>
        </section>

        <section className="mt-10 rounded-2xl border border-brand-300 bg-brand-950 px-6 py-10 md:px-10">
          <div className="max-w-4xl space-y-4">
            <h2 className="text-3xl md:text-4xl font-bold text-white">
              Secure and Production-Ready
            </h2>
            <p className="text-brand-100 text-base md:text-lg leading-relaxed">
              Protect ePHI with enterprise controls including standards-based
              authentication, access policies, auditability, and modern
              deployment practices aligned with HIPAA and HITECH expectations.
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
