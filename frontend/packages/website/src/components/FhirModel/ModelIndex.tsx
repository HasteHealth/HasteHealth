import React, { useDeferredValue, useState } from "react";
import Link from "@docusaurus/Link";

import { BACKENDS, combine } from "../TestCoverage/data";
import { StatusIcon } from "../TestCoverage";
import index from "@site/src/fhir-model/index.json";
import type { IndexEntry } from "./types";
import styles from "./styles.module.css";

const entries = index as IndexEntry[];

type Group = { title: string; entries: IndexEntry[] };

// HL7's own ordering of its resource categories.
const MODULES = ["Foundation", "Base", "Clinical", "Financial", "Specialized"];

function resourceGroup(entry: IndexEntry): string {
  if (entry.source === "haste-health") return "Haste Health";
  if (entry.source === "sql-on-fhir") return "SQL on FHIR";
  return entry.category ?? "Foundation.Abstract";
}

function groupOrder(title: string): number {
  const module = MODULES.indexOf(title.split(".")[0]);
  return module === -1 ? MODULES.length : module;
}

function resourceGroups(list: IndexEntry[]): Group[] {
  const groups = new Map<string, IndexEntry[]>();
  for (const entry of list) {
    const title = resourceGroup(entry);
    groups.set(title, [...(groups.get(title) ?? []), entry]);
  }
  return [...groups]
    .map(([title, grouped]) => ({ title, entries: grouped }))
    .sort(
      (a, b) =>
        groupOrder(a.title) - groupOrder(b.title) ||
        a.title.localeCompare(b.title),
    );
}

function typeGroups(list: IndexEntry[]): Group[] {
  return [
    {
      title: "Primitive types",
      entries: list.filter((e) => e.kind === "primitive-type"),
    },
    {
      title: "Complex types",
      entries: list.filter((e) => e.kind === "complex-type"),
    },
  ].filter((group) => group.entries.length > 0);
}

/** Lets long names wrap between words: Medicinal<wbr>Product<wbr>Authorization. */
function wrappable(name: string): React.ReactNode[] {
  const nodes: React.ReactNode[] = [];
  let start = 0;
  for (let i = 1; i <= name.length; i++) {
    const boundary = i === name.length || (/[a-z]/.test(name[i - 1]) && /[A-Z]/.test(name[i]));
    if (!boundary) continue;
    if (start > 0) nodes.push(<wbr key={`wbr-${start}`} />);
    nodes.push(name.slice(start, i));
    start = i;
  }
  return nodes;
}

function Card({ entry }: Readonly<{ entry: IndexEntry }>) {
  const overall = entry.overall;
  return (
    <Link to={entry.href} className={styles.card}>
      <span className={styles.cardTitle}>
        <span className={styles.cardName}>{wrappable(entry.name)}</span>
        {overall && (
          <span
            className={styles.cardIcon}
            title="Passes its conformance TestScript on PostgreSQL and Elasticsearch"
          >
            <StatusIcon
              result={combine(BACKENDS.map((b) => overall[b]))}
              className="h-3.5 w-3.5"
            />
          </span>
        )}
      </span>
      <span className={styles.cardText}>{entry.short}</span>
    </Link>
  );
}

function matches(entry: IndexEntry, query: string): boolean {
  const q = query.trim().toLowerCase();
  return (
    !q ||
    entry.name.toLowerCase().includes(q) ||
    entry.short.toLowerCase().includes(q)
  );
}

export default function ModelIndex({
  kind,
}: Readonly<{ kind: "resources" | "types" }>) {
  const [query, setQuery] = useState("");
  const deferred = useDeferredValue(query);
  const all = entries.filter(
    (e) => (kind === "resources") === (e.kind === "resource"),
  );
  const shown = all.filter((e) => matches(e, deferred));
  const groups =
    kind === "resources" ? resourceGroups(shown) : typeGroups(shown);

  return (
    <div className={styles.root}>
      <div className={styles.toolbar}>
        <input
          type="search"
          className={styles.filter}
          placeholder={`Filter ${all.length} ${kind === "resources" ? "resources" : "data types"}`}
          aria-label={`Filter ${kind}`}
          value={query}
          onChange={(e) => setQuery(e.target.value)}
        />
        {deferred && (
          <span className={styles.muted}>
            {shown.length} of {all.length}
          </span>
        )}
      </div>
      {groups.map((group) => (
        <section key={group.title} className={styles.group}>
          <h2 className={styles.groupTitle}>
            {group.title.replace(".", " · ")}
          </h2>
          <div className={styles.cards}>
            {group.entries.map((entry) => (
              <Card key={entry.name} entry={entry} />
            ))}
          </div>
        </section>
      ))}
      {groups.length === 0 && (
        <p className={styles.muted}>Nothing matches “{deferred}”.</p>
      )}
    </div>
  );
}
