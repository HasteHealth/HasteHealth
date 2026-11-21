import { usePluginData } from "@docusaurus/useGlobalData";
import type { HomepageFacts } from "@site/plugins/homepage-facts";

export type { HomepageFacts };

/**
 * The conformance and MCP totals the homepage quotes. They are computed at
 * build time by plugins/homepage-facts.ts from the generated reports, so they
 * are never typed into the page by hand.
 */
export function useHomepageFacts(): HomepageFacts {
  return usePluginData("homepage-facts") as HomepageFacts;
}
