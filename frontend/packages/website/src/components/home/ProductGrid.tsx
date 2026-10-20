import React, { type CSSProperties, type ReactNode } from "react";
import clsx from "clsx";
import Link from "@docusaurus/Link";

import { Icon, type IconName } from "../site/icons";
import { IconTile } from "../site/ui";
import { useHomepageFacts } from "./facts";
import styles from "./styles.module.css";

/*
 * The product grid under the hero: one tile per part of the platform, on four
 * columns, with the FHIR API, the part everything else is built on, taking a
 * double tile. Each tile is a link to its docs, with a small illustration
 * drawn from real request and resource shapes. Hovering a tile lets the
 * illustration respond (see styles.module.css); nothing in it depends on the
 * hover to be understood.
 */

type Product = {
  id: string;
  icon: IconName;
  title: string;
  body: string;
  href: string;
  art: ReactNode;
  /** Where the tile sits in the grid beyond a single cell. */
  className?: string;
};

/** The element's place in a tile's hover sequence: `style={seq(i)}`. */
function seq(index: number): CSSProperties {
  return { "--i": index } as CSSProperties;
}

const MONO = "font-mono text-[0.75rem] leading-5";

// ---------------------------------------------------------------------------
// Illustration parts
// ---------------------------------------------------------------------------

/** The white panel an illustration is drawn on. */
function Panel({
  children,
  className,
}: Readonly<{ children: ReactNode; className?: string }>) {
  return (
    <div
      className={clsx(
        "min-w-0 overflow-hidden rounded-xl border border-slate-200 bg-white",
        className,
      )}
    >
      {children}
    </div>
  );
}

function PanelBar({
  label,
  aside,
}: Readonly<{ label: string; aside?: string }>) {
  return (
    <div
      className={clsx(
        MONO,
        "flex items-center gap-3 border-b border-slate-200 bg-slate-50 px-3 py-1.5 text-[0.6875rem] text-slate-500",
      )}
    >
      <span className="min-w-0 truncate">{label}</span>
      {aside ? (
        <span className="ml-auto shrink-0 whitespace-nowrap">{aside}</span>
      ) : null}
    </div>
  );
}

function PanelFoot({ children }: Readonly<{ children: ReactNode }>) {
  return (
    <div
      className={clsx(
        MONO,
        "border-t border-slate-200 px-3 py-1.5 text-[0.6875rem] text-slate-500",
      )}
    >
      {children}
    </div>
  );
}

function Chip({
  children,
  index,
  className,
}: Readonly<{ children: ReactNode; index?: number; className?: string }>) {
  return (
    <span
      className={clsx(
        index === undefined ? null : styles.accent,
        "inline-flex items-center rounded-md border border-slate-200 bg-white px-2 py-0.5 text-[0.75rem] font-medium text-slate-600",
        className,
      )}
      style={index === undefined ? undefined : seq(index)}
    >
      {children}
    </span>
  );
}

/** Code inside a panel, free of the theme's block styling. */
function Code({
  children,
  size = "md",
}: Readonly<{ children: ReactNode; size?: "md" | "sm" }>) {
  return (
    <pre
      className={clsx(
        size === "sm" ? "font-mono text-[0.6875rem] leading-5" : MONO,
        "m-0 overflow-x-auto rounded-none border-0 bg-transparent px-3 py-2.5 text-slate-700",
      )}
    >
      <code className="border-0 bg-transparent p-0 font-mono text-[length:inherit] text-inherit">
        {children}
      </code>
    </pre>
  );
}

const KEYWORD = "font-semibold text-ink-950";
const STRING = "text-brand-800";

// ---------------------------------------------------------------------------
// Illustrations
// ---------------------------------------------------------------------------

const REQUESTS = [
  {
    method: "GET",
    path: "/Patient?name=williams&_revinclude=Observation:patient",
    status: "200",
  },
  { method: "POST", path: "/", note: "Bundle, transaction", status: "200" },
  { method: "GET", path: "/Patient/kin5pxcy/$everything", status: "200" },
  { method: "PATCH", path: "/Observation/f3n8wq2k", status: "200" },
  { method: "GET", path: "/Observation/f3n8wq2k/_history", status: "200" },
  { method: "DELETE", path: "/Encounter/9hd2kq7m", status: "204" },
];

/**
 * Every interaction the CapabilityStatement declares (static/fhir/R4/
 * metadata.json), by level, and the operations a reader is likely to know.
 */
const INTERACTIONS = [
  {
    level: "Instance",
    items: "read · vread · update · patch · delete · history",
  },
  { level: "Type", items: "create · search · history" },
  {
    level: "System",
    items: "capabilities · batch · transaction · search · history",
  },
  {
    level: "Operations",
    items: "$everything · $expand · $validate-code · custom",
  },
];

function ApiArt({ resourceTypes }: Readonly<{ resourceTypes: number }>) {
  return (
    <div className="grid gap-3 sm:grid-cols-[minmax(0,3fr)_minmax(0,2fr)]">
      <Panel>
        <PanelBar
          label="/w/acme-health/production/api/v1/fhir"
          aside="FHIR R4"
        />
        <ul className="m-0 list-none divide-y divide-slate-100 p-0">
          {REQUESTS.map((request, index) => (
            <li
              key={`${request.method} ${request.path}`}
              className={clsx(MONO, "flex items-center gap-3 px-3 py-2.5")}
            >
              <span className="w-[3.25rem] shrink-0 font-semibold text-ink-900">
                {request.method}
              </span>
              <span className="min-w-0 flex-1 truncate text-slate-600">
                {request.path}
                {request.note ? (
                  <span className="text-slate-400"> {request.note}</span>
                ) : null}
              </span>
              <span
                className={clsx(styles.status, "shrink-0")}
                style={seq(index)}
              >
                {request.status}
              </span>
            </li>
          ))}
        </ul>
      </Panel>
      <Panel>
        <PanelBar label="Interactions" aside={`${resourceTypes} types`} />
        <dl className="m-0 space-y-2.5 px-3 py-2.5">
          {INTERACTIONS.map((group) => (
            <div key={group.level}>
              <dt className="text-[0.625rem] font-semibold uppercase tracking-[0.12em] text-slate-500">
                {group.level}
              </dt>
              <dd className={clsx(MONO, "m-0 mt-0.5 text-slate-700")}>
                {group.items}
              </dd>
            </div>
          ))}
        </dl>
      </Panel>
    </div>
  );
}

const PROVIDERS = ["Okta", "Azure", "Auth0", "Keycloak", "Google", "TOTP MFA"];

function AuthArt() {
  return (
    <Panel>
      <PanelBar label="Sign in to acme-health" />
      <div className="grid grid-cols-2 gap-1.5 p-3">
        {PROVIDERS.map((provider, index) => (
          <span
            key={provider}
            className={clsx(
              styles.lift,
              "block truncate rounded-lg border border-slate-200 bg-white px-2 py-1.5 text-center text-[0.75rem] font-medium text-slate-700",
            )}
            style={seq(index)}
          >
            {provider}
          </span>
        ))}
      </div>
      <PanelFoot>OAuth 2.0 · OpenID Connect · SMART on FHIR</PanelFoot>
    </Panel>
  );
}

function PolicyArt() {
  return (
    <Panel>
      <PanelBar label="AccessPolicyV2 · own-record" />
      <Code size="sm">
        <span className="text-slate-400">// FHIRPath condition</span>
        {"\n"}
        <span className={STRING}>%request</span>.type = &apos;read&apos; and
        {"\n"}
        <span className={STRING}>%target</span>.subject.reference
        {"\n"}
        {"  "}= &apos;Patient/&apos; + <span className={STRING}>%user</span>.id
      </Code>
      <div className="flex items-center justify-between gap-3 border-t border-slate-200 px-3 py-2">
        <span className={clsx(MONO, "truncate text-slate-500")}>
          Observation read
        </span>
        <Chip index={0}>permit</Chip>
      </div>
    </Panel>
  );
}

const PROJECTS = ["production", "staging", "sandbox"];

function TenantArt() {
  return (
    <Panel>
      <div className="flex items-center justify-between border-b border-slate-200 bg-slate-50 px-3 py-2">
        <span className="text-[0.8125rem] font-semibold text-ink-950">
          acme-health
        </span>
        <span className={clsx(MONO, "text-[0.6875rem] text-slate-500")}>
          tenant
        </span>
      </div>
      <ul className="m-0 ml-5 list-none space-y-1 border-l border-slate-200 py-2.5 pl-2 pr-2">
        {PROJECTS.map((project, index) => (
          <li
            key={project}
            className={clsx(
              styles.accent,
              "flex items-center justify-between rounded-lg border border-transparent px-2 py-1.5 text-[0.8125rem] font-medium text-slate-700",
            )}
            style={seq(index)}
          >
            {project}
            <span className={clsx(MONO, "text-[0.6875rem] text-slate-500")}>
              project
            </span>
          </li>
        ))}
      </ul>
      <PanelFoot>Own users · own policies · own data</PanelFoot>
    </Panel>
  );
}

const COLUMNS = [
  { name: "id", width: "w-[28%]" },
  { name: "birth_date", width: "w-[42%]" },
  { name: "family", width: "w-[30%]" },
];
const ROWS = [
  ["kin5px", "1961-04-12", "Williams"],
  ["8dm3qt", "1974-09-30", "Okafor"],
  ["2sh9rk", "1988-01-05", "Lindqvist"],
];

function SqlArt() {
  return (
    <Panel>
      <PanelBar label="ViewDefinition · demographics" />
      <table className="m-0 table w-full table-fixed border-collapse font-mono text-[0.6875rem] leading-5">
        <thead>
          <tr className="bg-transparent">
            {COLUMNS.map((column) => (
              <th
                key={column.name}
                className={clsx(
                  column.width,
                  "border-b border-slate-200 px-1.5 py-1.5 text-left font-medium text-slate-500",
                )}
              >
                {column.name}
              </th>
            ))}
          </tr>
        </thead>
        <tbody>
          {ROWS.map((row, index) => (
            <tr
              key={row[0]}
              className={clsx(styles.accent, "bg-transparent")}
              style={seq(index)}
            >
              {row.map((cell) => (
                <td
                  key={cell}
                  className="truncate border-b border-slate-100 px-1.5 py-1.5 text-slate-700"
                >
                  {cell}
                </td>
              ))}
            </tr>
          ))}
        </tbody>
      </table>
      <div className="flex items-center gap-1.5 px-3 py-2">
        <span className={clsx(MONO, "mr-1 text-[0.6875rem] text-slate-500")}>
          _format
        </span>
        {["csv", "json", "ndjson"].map((format) => (
          <Chip key={format}>{format}</Chip>
        ))}
      </div>
    </Panel>
  );
}

const TOOLS = [
  "fhir_r4_search",
  "fhir_r4_read",
  "fhir_r4_create",
  "fhir_r4_update",
  "fhir_r4_patch",
  "fhir_r4_transaction",
  "fhir_r4_history_instance",
  "fhir_op_everything",
  "fhir_op_validate_code",
  "fhir_op_expand",
];

const CLIENTS = ["Claude", "Gemini", "OpenAI", "Any MCP client"];

function McpArt({ tools }: Readonly<{ tools: number }>) {
  const more = tools - TOOLS.length;
  return (
    <div className="grid gap-3 sm:grid-cols-[minmax(0,3fr)_minmax(0,2fr)]">
      <Panel>
        <PanelBar label="tools/list" aside={`${tools} tools`} />
        <div className="flex flex-wrap gap-1.5 p-3">
          {TOOLS.map((tool, index) => (
            <span
              key={tool}
              className={clsx(
                styles.accent,
                MONO,
                "rounded-md border border-slate-200 px-2 py-0.5 text-slate-600",
              )}
              style={seq(index)}
            >
              {tool}
            </span>
          ))}
          {more > 0 ? (
            <span className={clsx(MONO, "px-1 py-0.5 text-slate-400")}>
              +{more} more
            </span>
          ) : null}
        </div>
      </Panel>
      <Panel>
        <PanelBar label="Connects from" />
        <ul className="m-0 list-none space-y-0.5 p-2">
          {CLIENTS.map((client) => (
            <li
              key={client}
              className="flex items-center justify-between rounded-lg px-2 py-1.5 text-[0.8125rem] font-medium text-slate-700"
            >
              {client}
              <span
                className="h-1.5 w-1.5 rounded-full bg-brand-500"
                aria-hidden="true"
              />
            </li>
          ))}
        </ul>
        <PanelFoot>OAuth 2.0 · SMART scopes · AuditEvent</PanelFoot>
      </Panel>
    </div>
  );
}

const OPS_FACTS = [
  "Runs in an isolated Deno sandbox",
  "Versioned like any other resource",
  "Governed by the same access policies",
];

function OpsArt() {
  return (
    <div className="grid gap-3 sm:grid-cols-[minmax(0,3fr)_minmax(0,2fr)]">
      <Panel>
        <PanelBar label="OperationDefinition" aside="$recent-bp · v3" />
        <Code>
          <span className={KEYWORD}>export default async function</span> ()
          {" {"}
          {"\n  "}
          <span className={KEYWORD}>const</span> bp ={" "}
          <span className={KEYWORD}>await</span> fhir.searchType(
          {"\n    "}
          <span className={STRING}>&quot;Observation&quot;</span>,
          {"\n    { code: "}
          <span className={STRING}>&quot;85354-9&quot;</span>
          {", _count: 4 },"}
          {"\n  );"}
          {"\n  "}
          <span className={KEYWORD}>return</span>
          {" { readings: bp.total };"}
          <span className={styles.caret} aria-hidden="true" />
          {"\n}"}
        </Code>
      </Panel>
      <Panel>
        <PanelBar label="Invoke" />
        <div className="p-3">
          <div
            className={clsx(
              styles.accent,
              MONO,
              "rounded-md border border-slate-200 bg-slate-50 px-2 py-1.5 text-slate-700",
            )}
            style={seq(0)}
          >
            POST /$recent-bp
          </div>
          <ul className="m-0 mt-3 list-none space-y-1.5 p-0">
            {OPS_FACTS.map((fact) => (
              <li
                key={fact}
                className="flex items-start gap-2 text-[0.8125rem] leading-snug text-slate-600"
              >
                <Icon
                  name="check"
                  className="mt-0.5 h-3.5 w-3.5 shrink-0 text-brand-700"
                />
                {fact}
              </li>
            ))}
          </ul>
        </div>
      </Panel>
    </div>
  );
}

const CODES = [
  ["85354-9", "Blood pressure panel"],
  ["8480-6", "Systolic blood pressure"],
  ["8462-4", "Diastolic blood pressure"],
];

function TerminologyArt() {
  return (
    <Panel>
      <PanelBar label="ValueSet/$expand" aside="LOINC" />
      <ul className="m-0 list-none divide-y divide-slate-100 p-0">
        {CODES.map(([code, display], index) => (
          <li
            key={code}
            className="flex items-center gap-3 px-3 py-2 text-[0.8125rem] text-slate-700"
          >
            <span
              className={clsx(
                styles.accent,
                MONO,
                "shrink-0 rounded-md border border-slate-200 px-1.5 text-slate-600",
              )}
              style={seq(index)}
            >
              {code}
            </span>
            <span className="min-w-0 truncate">{display}</span>
          </li>
        ))}
      </ul>
      <PanelFoot>$expand · $validate-code · CodeSystem · ValueSet</PanelFoot>
    </Panel>
  );
}

const TOOLING = [
  { name: "Admin console", detail: "tenants, users, policies, apps" },
  { name: "CLI", detail: "haste-health admin client create" },
  { name: "SDKs", detail: "TypeScript · Rust · React" },
];

function ToolingArt() {
  return (
    <Panel>
      <ul className="m-0 list-none divide-y divide-slate-100 p-0">
        {TOOLING.map((tool, index) => (
          <li
            key={tool.name}
            className={clsx(styles.accent, "px-3 py-2.5")}
            style={seq(index)}
          >
            <div className="text-[0.8125rem] font-semibold text-ink-950">
              {tool.name}
            </div>
            <div className={clsx(MONO, "truncate text-slate-500")}>
              {tool.detail}
            </div>
          </li>
        ))}
      </ul>
    </Panel>
  );
}

// ---------------------------------------------------------------------------
// Standards
// ---------------------------------------------------------------------------

/**
 * The interoperability standards the server implements, as a trust row.
 * Each one links to the page that shows it in use.
 */
const STANDARDS = [
  {
    name: "HL7 FHIR R4",
    role: "resources and REST API",
    href: "/docs/api/rest_api/fhir/intro",
  },
  {
    name: "SMART on FHIR",
    role: "app launch and scopes",
    href: "/docs/auth/authentication/smart_on_fhir",
  },
  {
    name: "OAuth 2.0",
    role: "tokens and clients",
    href: "/docs/category/oauth-grant-types",
  },
  {
    name: "OpenID Connect",
    role: "sign-in and federation",
    href: "/docs/auth/authentication/federated_login",
  },
  {
    name: "Model Context Protocol",
    role: "AI agents",
    href: "/docs/api/rest_api/model_context_protocol/endpoint",
  },
  {
    name: "SQL on FHIR",
    role: "analytics views",
    href: "/docs/guides/sql_on_fhir",
  },
  { name: "HL7v2", role: "interface feeds", href: "/docs/guides/hl7v2" },
  {
    name: "FHIRPath",
    role: "policies and views",
    href: "/docs/auth/authorization/access_control",
  },
];

/** The grid's last row: what the platform is built on, one link per cell. */
function StandardsRow() {
  return (
    <div className="grid min-w-0 overflow-hidden rounded-2xl border border-slate-200 bg-white sm:col-span-2 lg:col-span-4 lg:grid-cols-[minmax(0,1fr)_minmax(0,3fr)]">
      <div className="border-b border-slate-200 p-6 lg:border-b-0 lg:border-r">
        <p className="m-0 text-[0.8125rem] font-semibold uppercase tracking-[0.16em] text-slate-500">
          Open standards
        </p>
        <p className="m-0 mt-2 text-[0.9375rem] leading-relaxed text-slate-600">
          Built on the standards your stack already speaks, so what you
          integrate today keeps working with the next vendor.
        </p>
      </div>
      {/* Hairlines between the cells: a 1px gap over a slate background. */}
      <ul className="m-0 grid list-none grid-cols-2 gap-px bg-slate-200 p-0 sm:grid-cols-4">
        {STANDARDS.map((standard) => (
          <li key={standard.name} className="bg-white">
            <Link
              to={standard.href}
              className="group/standard flex h-full flex-col px-4 py-5 no-underline transition-colors hover:bg-brand-50/60 hover:no-underline"
            >
              <span className="flex items-center gap-1.5 text-[0.9375rem] font-semibold tracking-tight text-ink-950 group-hover/standard:text-brand-900">
                {standard.name}
                <Icon
                  name="arrowRight"
                  className="h-3.5 w-3.5 shrink-0 text-slate-300 transition-colors group-hover/standard:text-brand-700"
                />
              </span>
              <span className="mt-1 text-[0.8125rem] text-slate-500">
                {standard.role}
              </span>
            </Link>
          </li>
        ))}
      </ul>
    </div>
  );
}

// ---------------------------------------------------------------------------
// The grid
// ---------------------------------------------------------------------------

function Tile({ product }: Readonly<{ product: Product }>) {
  return (
    <Link
      to={product.href}
      className={clsx(
        styles.tile,
        "group flex h-full min-w-0 flex-col rounded-2xl border border-slate-200 bg-white no-underline hover:no-underline",
        product.className,
      )}
    >
      <div className="p-6">
        <div className="flex items-center gap-3">
          <IconTile name={product.icon} size="sm" />
          <h3 className="m-0 text-[1.125rem] font-semibold tracking-tight text-ink-950">
            {product.title}
          </h3>
          <Icon
            name="arrowRight"
            className="ml-auto h-4 w-4 shrink-0 text-slate-300 transition-colors group-hover:text-brand-700"
          />
        </div>
        <p className="m-0 mt-3 text-[0.9375rem] leading-relaxed text-slate-600">
          {product.body}
        </p>
      </div>
      <div className="flex flex-1 flex-col justify-end px-6 pb-6">
        {product.art}
      </div>
    </Link>
  );
}

export default function ProductGrid() {
  const { mcpTools, conformance } = useHomepageFacts();

  // Four columns. The rows: API (wide), auth, access control; MCP (wide),
  // tenancy, SQL on FHIR; custom operations (wide), terminology, tooling;
  // then the standards row across all four.
  const products: Product[] = [
    {
      id: "api",
      icon: "database",
      title: "FHIR R4 API",
      body: "Every R4 resource type with create, read, update, patch, delete, history, transactions and batches. Search supports modifiers, chains, _include, _revinclude and Patient/$everything.",
      href: "/docs/api/rest_api/fhir/intro",
      art: <ApiArt resourceTypes={conformance.resourceTypes} />,
      className: "sm:col-span-2",
    },
    {
      id: "auth",
      icon: "key",
      title: "Authentication",
      body: "An OAuth 2.0 and OpenID Connect provider with SMART on FHIR launch, plus federated sign-in through Okta, Azure, Auth0, Keycloak or Google.",
      href: "/docs/category/authentication",
      art: <AuthArt />,
    },
    {
      id: "access",
      icon: "shieldCheck",
      title: "Access control",
      body: "Attribute-based policies written in FHIRPath decide who can read or write what, for people, client applications and custom operations.",
      href: "/docs/auth/authorization/access_control",
      art: <PolicyArt />,
    },
    {
      id: "mcp",
      icon: "tools",
      title: "AI agents over MCP",
      body: `Every project serves a Model Context Protocol endpoint beside its REST API, with ${mcpTools} typed tools generated from its CapabilityStatement. Each call is authenticated, authorized and audited like any other request.`,
      href: "/docs/api/rest_api/model_context_protocol/endpoint",
      art: <McpArt tools={mcpTools} />,
      className: "sm:col-span-2",
    },
    {
      id: "tenancy",
      icon: "layers",
      title: "Multi-tenant",
      body: "Tenants and projects are enforced at the storage and search layers, so one deployment serves every customer, environment or business unit.",
      href: "/docs/core_concepts/platform_architecture",
      art: <TenantArt />,
    },
    {
      id: "sql",
      icon: "table",
      title: "SQL on FHIR",
      body: "ViewDefinitions flatten resources into CSV, JSON or NDJSON on demand. Point a BI tool at clinical data without building an ETL pipeline.",
      href: "/docs/guides/sql_on_fhir",
      art: <SqlArt />,
    },
    {
      id: "ops",
      icon: "code",
      title: "Custom operations",
      body: "Your own FHIR operations as sandboxed TypeScript in an embedded Deno runtime, stored and versioned as OperationDefinition resources. No fork required.",
      href: "/docs/reference/fhir/model/resources/OperationDefinition",
      art: <OpsArt />,
      className: "sm:col-span-2",
    },
    {
      id: "terminology",
      icon: "clipboard",
      title: "Terminology",
      body: "CodeSystems and ValueSets are resources like any other, with $expand and $validate-code for the codes your apps and policies depend on.",
      href: "/docs/category/terminology",
      art: <TerminologyArt />,
    },
    {
      id: "tooling",
      icon: "server",
      title: "Console, CLI and SDKs",
      body: "An admin console for tenants, users and policies, a CLI for scripting and CI, and SDKs for TypeScript, Rust and React.",
      href: "/docs/guides/admin_app",
      art: <ToolingArt />,
    },
  ];

  return (
    <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
      {products.map((product) => (
        <Tile key={product.id} product={product} />
      ))}
      <StandardsRow />
    </div>
  );
}
