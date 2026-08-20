import React, { ReactNode } from "react";
import Link from "@docusaurus/Link";
import Layout from "@theme/Layout";
import Heading from "@theme/Heading";

const contactChannels = [
  {
    title: "Business",
    body: "Partnerships, sales, and general business inquiries.",
    label: "business@haste.health",
    href: "mailto:business@haste.health",
  },
  {
    title: "Developer",
    body: "Questions about integrating with or contributing to Haste Health.",
    label: "dev@haste.health",
    href: "mailto:dev@haste.health",
  },
  {
    title: "Security",
    body: "Report a vulnerability or security concern.",
    label: "security@haste.health",
    href: "mailto:security@haste.health",
  },
];

const communityLinks = [
  {
    title: "GitHub Issues",
    body: "Report a bug or track ongoing work.",
    label: "Open an issue",
    href: "https://github.com/HasteHealth/HasteHealth/issues",
  },
  {
    title: "GitHub Discussions",
    body: "Ask questions and discuss ideas with the community.",
    label: "Join the discussion",
    href: "https://github.com/HasteHealth/HasteHealth/discussions",
  },
];

export default function Contact(): ReactNode {
  return (
    <Layout
      wrapperClassName="bg-background"
      title="Contact"
      description="Get in touch with the Haste Health team."
    >
      <main
        id="tw-scope"
        className="container mx-auto px-4 py-8 md:py-12 text-brand-950"
      >
        <section className="rounded-3xl border border-brand-200 bg-white px-6 py-12 md:px-10 md:py-16">
          <div className="max-w-4xl space-y-4">
            <Heading
              as="h1"
              className="text-4xl md:text-5xl font-bold tracking-tight text-brand-950"
            >
              Contact Us
            </Heading>
            <p className="max-w-3xl text-lg text-slate-700 leading-relaxed">
              Reach out to the right team, or connect with us on GitHub.
            </p>
          </div>
        </section>

        <section className="mt-10 rounded-2xl border border-brand-200 bg-white p-6 md:p-8">
          <Heading
            as="h2"
            className="text-2xl md:text-3xl font-bold text-brand-950"
          >
            Email Us
          </Heading>
          <div className="mt-6 grid gap-4 md:grid-cols-3">
            {contactChannels.map((channel) => (
              <article
                key={channel.title}
                className="flex flex-col rounded-xl border border-brand-200 bg-brand-50/40 p-5"
              >
                <h3 className="text-lg font-semibold text-brand-900">
                  {channel.title}
                </h3>
                <p className="mt-2 flex-1 text-sm text-slate-700 leading-6">
                  {channel.body}
                </p>
                <Link
                  href={channel.href}
                  className="mt-4 inline-flex items-center justify-center rounded-lg bg-brand-700 px-4 py-2 text-sm font-semibold text-white hover:bg-brand-800"
                >
                  {channel.label}
                </Link>
              </article>
            ))}
          </div>
        </section>

        <section className="mt-10 rounded-2xl border border-brand-200 bg-white p-6 md:p-8">
          <Heading
            as="h2"
            className="text-2xl md:text-3xl font-bold text-brand-950"
          >
            Connect on GitHub
          </Heading>
          <div className="mt-6 grid gap-4 md:grid-cols-2">
            {communityLinks.map((link) => (
              <article
                key={link.title}
                className="flex flex-col rounded-xl border border-brand-200 bg-brand-50/40 p-5"
              >
                <h3 className="text-lg font-semibold text-brand-900">
                  {link.title}
                </h3>
                <p className="mt-2 flex-1 text-sm text-slate-700 leading-6">
                  {link.body}
                </p>
                <Link
                  href={link.href}
                  className="mt-4 inline-flex items-center justify-center rounded-lg bg-brand-700 px-4 py-2 text-sm font-semibold text-white hover:bg-brand-800"
                >
                  {link.label}
                </Link>
              </article>
            ))}
          </div>
        </section>
      </main>
    </Layout>
  );
}
