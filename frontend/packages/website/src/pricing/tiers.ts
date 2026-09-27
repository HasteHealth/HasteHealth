/**
 * The subscription tiers, as the backend defines them.
 *
 * The data is generated from `backend/crates/subscription` by
 * `cargo run subscription export` and committed to
 * `static/pricing/tiers.json`. Nothing here restates a limit or a price: if a
 * number on the pricing page is wrong, it is wrong in the Rust crate, and the
 * server is enforcing the wrong thing too.
 */
import tiersJson from "@site/static/pricing/tiers.json";

/** The tier's value in the `subscription_tier` JWT claim. */
export type TierId = "free" | "professional" | "team" | "unlimited";

/**
 * A cap on how many resources a tenant may store.
 *
 * Serde renders the Rust enum externally tagged, so a variant carrying a value
 * becomes an object and a unit variant becomes the bare tag string.
 */
export type ResourceLimit = { count: number } | "unlimited";

/** A tier's request allowance for the rate limit window. */
export type RequestBudget = { points: number } | "unmetered";

/** What one request of each kind costs against the budget. */
export type OperationPoints = {
  read: number;
  write: number;
  search: number;
  invocation: number;
};

/** What a budget buys per window, in whole requests of one kind. */
export type BudgetExamples = {
  reads_per_day: number | null;
  writes_per_day: number | null;
  searches_per_day: number | null;
};

export type Tier = {
  tier: TierId;

  // What the server enforces.
  request_budget: RequestBudget;
  total_resources: ResourceLimit;
  projects: ResourceLimit;
  search_parameters: ResourceLimit;
  operation_definitions: ResourceLimit;
  subscriptions: ResourceLimit;
  identity_providers: ResourceLimit;
  tenant_customization: boolean;

  // How the tier is sold.
  display_name: string;
  price: string;
  cadence: string | null;
  audience: string;
  support: string;
  uptime_sla: string;
  baa_available: boolean;
  self_serve: boolean;

  // Rendered and derived by the exporter.
  request_budget_label: string;
  total_resources_label: string;
  budget_examples: BudgetExamples;
};

export type TierTable = {
  operation_points: OperationPoints;
  window: string;
  tiers: Tier[];
};

const table = tiersJson as unknown as TierTable;

export const OPERATION_POINTS: OperationPoints = table.operation_points;
export const WINDOW: string = table.window;

/** Whether a limit is uncapped. */
export function isUnlimited(limit: ResourceLimit): boolean {
  return limit === "unlimited";
}

/** A limit's count, or `null` when uncapped. */
export function limitCount(limit: ResourceLimit): number | null {
  return limit === "unlimited" ? null : limit.count;
}

function tierById(id: TierId): Tier {
  const tier = table.tiers.find((candidate) => candidate.tier === id);
  if (!tier) {
    throw new Error(
      `No '${id}' tier in static/pricing/tiers.json — regenerate it with \`cargo run subscription export\`.`,
    );
  }
  return tier;
}

/**
 * The tiers in the order the page presents them: self-hosted first, because the
 * server being free to run yourself is the lead claim, then the hosted plans
 * cheapest-first.
 */
export const TIERS: Tier[] = [
  tierById("unlimited"),
  tierById("free"),
  tierById("professional"),
  tierById("team"),
];

/** The hosted tier we steer most buyers to; its card gets the emphasis. */
export const EMPHASIZED_TIER: TierId = "professional";
