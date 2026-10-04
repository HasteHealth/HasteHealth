import React, { ReactNode } from "react";
import Layout from "@theme/Layout";

import { GitHubMark } from "@site/src/components/site/icons";
import { revealDelay, useScrollReveal } from "@site/src/components/site/motion";
import {
  Container,
  IconTile,
  LinkCard,
  PageHero,
  SectionHeader,
} from "@site/src/components/site/ui";

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
  const page = useScrollReveal<HTMLElement>();

  return (
    <Layout
      wrapperClassName="bg-white"
      title="Contact"
      description="Get in touch with the Haste Health team about business partnerships, developer support for the open-source FHIR server and MCP tools, or security disclosures."
    >
      <main id="tw-scope" ref={page} className="text-ink-950">
        <PageHero title="Contact Us">
          Reach out to the right team, or connect with us on GitHub.
        </PageHero>

        <section className="bg-white py-20 md:py-28">
          <Container>
            <SectionHeader title="Email Us" />
            <div className="mt-10 grid gap-5 md:mt-12 md:grid-cols-3">
              {contactChannels.map((channel, index) => (
                <div
                  key={channel.title}
                  data-reveal=""
                  style={revealDelay(index)}
                >
                  <LinkCard
                    to={channel.href}
                    icon={<IconTile name="mail" />}
                    title={channel.title}
                    label={channel.label}
                  >
                    {channel.body}
                  </LinkCard>
                </div>
              ))}
            </div>
          </Container>
        </section>

        <section className="border-t border-slate-200/80 bg-slate-50 py-20 md:py-28">
          <Container>
            <SectionHeader title="Connect on GitHub" />
            <div className="mt-10 grid gap-5 md:mt-12 md:grid-cols-2">
              {communityLinks.map((link, index) => (
                <div key={link.title} data-reveal="" style={revealDelay(index)}>
                  <LinkCard
                    to={link.href}
                    icon={
                      <IconTile>
                        <GitHubMark className="h-[1.375rem] w-[1.375rem]" />
                      </IconTile>
                    }
                    title={link.title}
                    label={link.label}
                  >
                    {link.body}
                  </LinkCard>
                </div>
              ))}
            </div>
          </Container>
        </section>
      </main>
    </Layout>
  );
}
