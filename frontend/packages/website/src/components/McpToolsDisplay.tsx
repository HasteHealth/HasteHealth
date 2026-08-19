import React from "react";
import CodeBlock from "@theme/CodeBlock";

// eslint-disable-next-line @typescript-eslint/no-explicit-any
type JSONSchema = Record<string, any>;

type MCPTool = {
  name: string;
  title?: string;
  description?: string;
  inputSchema: JSONSchema;
  outputSchema?: JSONSchema;
};

type CategoryKey = "read" | "create" | "update" | "delete" | "bulk" | "other";

const CATEGORIES: Record<CategoryKey, { label: string; badgeClassName: string }> = {
  read: {
    label: "Read",
    badgeClassName: "border-emerald-200 bg-emerald-50 text-emerald-700",
  },
  create: {
    label: "Create",
    badgeClassName: "border-blue-200 bg-blue-50 text-blue-700",
  },
  update: {
    label: "Update",
    badgeClassName: "border-amber-200 bg-amber-50 text-amber-700",
  },
  delete: {
    label: "Delete",
    badgeClassName: "border-red-200 bg-red-50 text-red-700",
  },
  bulk: {
    label: "Bulk",
    badgeClassName: "border-purple-200 bg-purple-50 text-purple-700",
  },
  other: {
    label: "Other",
    badgeClassName: "border-slate-200 bg-slate-50 text-slate-700",
  },
};

const CATEGORY_ORDER: CategoryKey[] = [
  "read",
  "create",
  "update",
  "delete",
  "bulk",
  "other",
];

function categorize(name: string): CategoryKey {
  if (/delete/i.test(name)) return "delete";
  if (/create/i.test(name)) return "create";
  if (/update|patch/i.test(name)) return "update";
  if (/transaction|batch/i.test(name)) return "bulk";
  if (/read|search|history|capabilities|get_/i.test(name)) return "read";
  return "other";
}

function refLabel(ref: string): string {
  const parts = ref.split("/");
  return parts[parts.length - 1] || ref;
}

function EnumSummary({ values }: Readonly<{ values: unknown[] }>) {
  const [expanded, setExpanded] = React.useState(false);
  const strings = values.map((v) => String(v));

  if (strings.length <= 6) {
    return (
      <span className="text-xs text-slate-500">
        {strings.map((v, i) => (
          <React.Fragment key={v}>
            {i > 0 ? " · " : ""}
            <code className="text-[11px]">{v}</code>
          </React.Fragment>
        ))}
      </span>
    );
  }

  return (
    <div className="inline-block align-middle text-xs">
      <button
        type="button"
        onClick={() => setExpanded((e) => !e)}
        className="cursor-pointer font-medium text-brand-600 hover:underline"
      >
        {expanded ? "Hide options ▲" : `${strings.length} supported types ▾`}
      </button>
      {expanded ? (
        <div className="mt-2 flex max-h-48 flex-wrap gap-1.5 overflow-y-auto rounded-lg border border-slate-100 bg-slate-50 p-2">
          {strings.map((v) => (
            <code
              key={v}
              className="rounded-md border border-brand-100 bg-white px-1.5 py-0.5 text-[11px] text-brand-800"
            >
              {v}
            </code>
          ))}
        </div>
      ) : null}
    </div>
  );
}

function TypeSummary({ schema }: Readonly<{ schema: JSONSchema | undefined }>) {
  if (!schema) return <span className="text-xs text-slate-400">any</span>;

  if (schema.const !== undefined) {
    return <code className="text-[11px] text-brand-700">"{String(schema.const)}"</code>;
  }

  if (typeof schema.$ref === "string") {
    return (
      <a
        href={schema.$ref}
        target="_blank"
        rel="noreferrer"
        className="text-xs font-medium text-brand-600 hover:underline"
      >
        {refLabel(schema.$ref)} schema ↗
      </a>
    );
  }

  if (Array.isArray(schema.enum)) {
    return <EnumSummary values={schema.enum} />;
  }

  if (schema.type === "array" && schema.items) {
    return (
      <span className="text-xs text-slate-500">
        array of <TypeSummary schema={schema.items} />
      </span>
    );
  }

  return <span className="text-xs text-slate-500">{schema.type ?? "object"}</span>;
}

function PropertyRow({
  name,
  schema,
  required,
  depth = 0,
}: Readonly<{
  name: string;
  schema: JSONSchema;
  required: boolean;
  depth?: number;
}>) {
  const nestedEntries: [string, JSONSchema][] | null =
    schema.type === "object" && schema.properties
      ? Object.entries(schema.properties)
      : schema.type === "array" &&
          schema.items?.type === "object" &&
          schema.items?.properties
        ? Object.entries(schema.items.properties)
        : null;

  const nestedRequired: string[] =
    schema.type === "array" ? (schema.items?.required ?? []) : (schema.required ?? []);

  return (
    <div className={depth > 0 ? "border-l border-slate-150 pl-4" : ""}>
      <div className="flex flex-wrap items-baseline gap-x-2 gap-y-1 py-2.5">
        <code className="text-[13px] font-semibold text-slate-900">{name}</code>
        {required ? (
          <span className="rounded bg-red-50 px-1.5 py-0.5 text-[10px] font-semibold uppercase tracking-wide text-red-600">
            required
          </span>
        ) : null}
        <TypeSummary schema={schema} />
      </div>
      {schema.description ? (
        <p className="-mt-1 pb-2.5 text-sm leading-6 text-slate-600">
          {schema.description}
        </p>
      ) : null}
      {nestedEntries ? (
        <div className="pb-2.5">
          {nestedEntries.map(([childName, childSchema]) => (
            <PropertyRow
              key={childName}
              name={childName}
              schema={childSchema}
              required={nestedRequired.includes(childName)}
              depth={depth + 1}
            />
          ))}
        </div>
      ) : null}
    </div>
  );
}

function ParametersSection({ schema }: Readonly<{ schema: JSONSchema }>) {
  const properties: [string, JSONSchema][] = schema?.properties
    ? Object.entries(schema.properties)
    : [];
  const required: string[] = schema?.required ?? [];

  if (properties.length === 0) {
    return <p className="text-sm text-slate-500">This tool takes no arguments.</p>;
  }

  return (
    <div className="divide-y divide-slate-100">
      {properties.map(([name, propSchema]) => (
        <PropertyRow
          key={name}
          name={name}
          schema={propSchema}
          required={required.includes(name)}
        />
      ))}
    </div>
  );
}

function ReturnsSection({ schema }: Readonly<{ schema?: JSONSchema }>) {
  if (!schema) {
    return <p className="text-sm text-slate-500">No structured output.</p>;
  }
  return (
    <div className="space-y-1.5 py-1">
      <TypeSummary schema={schema} />
      {schema.description ? (
        <p className="text-sm leading-6 text-slate-600">{schema.description}</p>
      ) : null}
    </div>
  );
}

function RawSchemaToggle({
  label,
  schema,
}: Readonly<{ label: string; schema: JSONSchema }>) {
  const [show, setShow] = React.useState(false);
  return (
    <div className="mt-1">
      <button
        type="button"
        onClick={() => setShow((s) => !s)}
        className="cursor-pointer text-xs font-medium text-slate-400 hover:text-brand-600 hover:underline"
      >
        {show ? "Hide" : "View"} raw {label} schema
      </button>
      {show ? (
        <div className="mt-2">
          <CodeBlock language="json">{JSON.stringify(schema, null, 2)}</CodeBlock>
        </div>
      ) : null}
    </div>
  );
}

const ChevronIcon = ({ open }: Readonly<{ open: boolean }>) => (
  <svg
    className={`mt-1 h-4 w-4 shrink-0 text-slate-400 transition-transform duration-200 ${
      open ? "rotate-90" : ""
    }`}
    viewBox="0 0 20 20"
    fill="currentColor"
    aria-hidden="true"
  >
    <path
      fillRule="evenodd"
      d="M7.21 14.77a.75.75 0 01.02-1.06L11.168 10 7.23 6.29a.75.75 0 111.04-1.08l4.5 4.25a.75.75 0 010 1.08l-4.5 4.25a.75.75 0 01-1.06-.02z"
      clipRule="evenodd"
    />
  </svg>
);

function ToolCard({
  tool,
  open,
  onToggle,
}: Readonly<{ tool: MCPTool; open: boolean; onToggle: () => void }>) {
  const category = CATEGORIES[categorize(tool.name)];

  return (
    <div
      id={tool.name}
      className="scroll-mt-24 rounded-xl border border-slate-200 bg-white transition-colors hover:border-brand-300"
    >
      <button
        type="button"
        onClick={onToggle}
        aria-expanded={open}
        className="flex w-full cursor-pointer items-start justify-between gap-4 px-5 py-4 text-left"
      >
        <div className="min-w-0">
          <div className="flex flex-wrap items-center gap-2">
            <code className="text-[13px] font-semibold text-slate-900">
              {tool.name}
            </code>
            <span
              className={`rounded-full border px-2 py-0.5 text-[10px] font-semibold uppercase tracking-wide ${category.badgeClassName}`}
            >
              {category.label}
            </span>
          </div>
          {tool.description ? (
            <p className="mt-1.5 text-sm leading-6 text-slate-600">
              {tool.description}
            </p>
          ) : null}
        </div>
        <ChevronIcon open={open} />
      </button>

      <div
        className={`grid transition-[grid-template-rows] duration-300 ease-out ${
          open ? "grid-rows-[1fr]" : "grid-rows-[0fr]"
        }`}
      >
        <div className="overflow-hidden">
          <div className="space-y-5 border-t border-slate-100 px-5 py-5">
            <section>
              <h4 className="mb-1 text-xs font-semibold uppercase tracking-wide text-slate-400">
                Parameters
              </h4>
              <ParametersSection schema={tool.inputSchema} />
              <RawSchemaToggle label="input" schema={tool.inputSchema} />
            </section>
            <section>
              <h4 className="mb-1 text-xs font-semibold uppercase tracking-wide text-slate-400">
                Returns
              </h4>
              <ReturnsSection schema={tool.outputSchema} />
              {tool.outputSchema ? (
                <RawSchemaToggle label="output" schema={tool.outputSchema} />
              ) : null}
            </section>
          </div>
        </div>
      </div>
    </div>
  );
}

function ToolsSkeleton() {
  return (
    <div className="animate-pulse space-y-3">
      {[0, 1, 2, 3].map((i) => (
        <div key={i} className="h-16 rounded-xl border border-slate-100 bg-slate-50" />
      ))}
    </div>
  );
}

export default function McpToolsDisplay() {
  const [tools, setTools] = React.useState<MCPTool[] | null>(null);
  const [error, setError] = React.useState<string | null>(null);
  const [query, setQuery] = React.useState("");
  const [openTools, setOpenTools] = React.useState<Set<string>>(new Set());

  React.useEffect(() => {
    fetch("/mcp/tools.json")
      .then((res) => {
        if (!res.ok) throw new Error(`${res.status} ${res.statusText}`);
        return res.json();
      })
      .then((fetchedTools) => setTools(fetchedTools))
      .catch((err) => setError(err.message));
  }, []);

  const toggle = (name: string) => {
    setOpenTools((prev) => {
      const next = new Set(prev);
      if (next.has(name)) {
        next.delete(name);
      } else {
        next.add(name);
      }
      return next;
    });
  };

  if (error) {
    return (
      <div className="rounded-xl border border-red-200 bg-red-50 p-4 text-sm text-red-700">
        Couldn&apos;t load the MCP tools list ({error}). This page renders from a
        static <code className="text-[11px]">tools.json</code> generated by{" "}
        <code className="text-[11px]">pnpm generate-mcp</code> against a running
        Haste Health instance.
      </div>
    );
  }

  if (!tools) return <ToolsSkeleton />;

  const q = query.trim().toLowerCase();
  const filtered = q
    ? tools.filter(
        (t) =>
          t.name.toLowerCase().includes(q) ||
          (t.description ?? "").toLowerCase().includes(q),
      )
    : tools;

  const grouped = CATEGORY_ORDER.map((key) => ({
    key,
    category: CATEGORIES[key],
    tools: filtered.filter((t) => categorize(t.name) === key),
  })).filter((g) => g.tools.length > 0);

  return (
    <div>
      <div className="mb-5 flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
        <p className="text-sm text-slate-500">
          {tools.length} tools available over MCP
        </p>
        <input
          type="text"
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          placeholder="Filter tools by name or description…"
          className="w-full rounded-lg border border-slate-200 px-3 py-2 text-sm text-slate-900 focus:border-brand-400 focus:outline-none sm:w-72"
        />
      </div>

      {grouped.length > 1 ? (
        <div className="mb-6 flex flex-wrap gap-2">
          {grouped.map(({ key, category, tools: groupTools }) => (
            <a
              key={key}
              href={`#mcp-category-${key}`}
              className={`rounded-full border px-3 py-1 text-xs font-semibold no-underline ${category.badgeClassName}`}
            >
              {category.label} ({groupTools.length})
            </a>
          ))}
        </div>
      ) : null}

      <div className="space-y-8">
        {grouped.map(({ key, category, tools: groupTools }) => (
          <div key={key} id={`mcp-category-${key}`} className="scroll-mt-24">
            <h3 className="mb-3 text-xs font-semibold uppercase tracking-wide text-slate-400">
              {category.label}
            </h3>
            <div className="space-y-3">
              {groupTools.map((tool) => (
                <ToolCard
                  key={tool.name}
                  tool={tool}
                  open={openTools.has(tool.name)}
                  onToggle={() => toggle(tool.name)}
                />
              ))}
            </div>
          </div>
        ))}
        {grouped.length === 0 ? (
          <p className="text-sm text-slate-500">
            No tools match &quot;{query}&quot;.
          </p>
        ) : null}
      </div>
    </div>
  );
}
