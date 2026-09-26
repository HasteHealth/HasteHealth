import React from "react";
import Link from "@docusaurus/Link";

import { BACKENDS, BACKEND_NAMES, INTERACTIONS, combine } from "../TestCoverage/data";
import { INTERACTION_NAMES } from "../TestCoverage/labels";
import { StatusIcon } from "../TestCoverage";
import Markdown from "./Markdown";
import type { ModelData } from "./types";
import styles from "./styles.module.css";

const KIND_LABELS: Record<ModelData["kind"], string> = {
  resource: "Resource",
  "complex-type": "Data type",
  "primitive-type": "Primitive type",
};

const STATUS_LABELS: Record<string, string> = {
  normative: "Normative",
  "trial-use": "Trial use",
  informative: "Informative",
  draft: "Draft",
};

const SOURCE_LABELS: Record<ModelData["source"], string | undefined> = {
  hl7: undefined,
  "haste-health": "Haste Health",
  "sql-on-fhir": "SQL on FHIR",
};

export function Badge({
  children,
  title,
  accent = false,
}: Readonly<{ children: React.ReactNode; title?: string; accent?: boolean }>) {
  return (
    <span className={accent ? `${styles.badge} ${styles.badgeAccent}` : styles.badge} title={title}>
      {children}
    </span>
  );
}

function Badges({ data }: Readonly<{ data: ModelData }>) {
  const source = SOURCE_LABELS[data.source];
  return (
    <div className={styles.badges}>
      <Badge accent>{KIND_LABELS[data.kind]}</Badge>
      {source && <Badge accent>{source}</Badge>}
      {data.category && <Badge>{data.category.replace(".", " · ")}</Badge>}
      {data.status && <Badge>{STATUS_LABELS[data.status] ?? data.status}</Badge>}
      {data.maturity !== undefined && (
        <Badge title="FHIR Maturity Model level, 0 (draft) to 5 (widely implemented)">
          Maturity {data.maturity}
        </Badge>
      )}
    </div>
  );
}

function Coverage({ data }: Readonly<{ data: ModelData }>) {
  const coverage = data.coverage;
  if (!coverage) return null;

  const parameters = data.searchParameters ?? [];
  const tested = parameters.filter((p) => p.coverage).length;

  return (
    <div className={styles.coverage}>
      <div className={styles.coverageHead}>
        <span className={styles.coverageTitle}>Tested support</span>
        {/* A plain anchor: the coverage page renders its rows client-side, so
            the build's anchor check can't see #resource-<Type>. */}
        <a
          href={`/docs/reference/conformance/test-coverage#resource-${data.name}`}
          className={styles.coverageLink}
        >
          Full test results →
        </a>
      </div>
      <dl className={styles.coverageGrid}>
        <dt>Backends</dt>
        <dd>
          {BACKENDS.map((backend) => (
            <span key={backend} className={styles.coverageItem}>
              <StatusIcon result={coverage.overall[backend]} />
              {BACKEND_NAMES[backend]}
            </span>
          ))}
        </dd>
        <dt>Interactions</dt>
        <dd>
          {INTERACTIONS.map((interaction) => {
            const results = coverage.interactions[interaction];
            const result = results ? combine(BACKENDS.map((b) => results[b])) : "not-run";
            return (
              <span key={interaction} className={styles.coverageItem}>
                <StatusIcon result={result} />
                {INTERACTION_NAMES[interaction]}
              </span>
            );
          })}
        </dd>
        <dt>Search</dt>
        <dd>
          <Link to="#search-parameters">
            {tested} of {parameters.length} parameters tested
          </Link>
        </dd>
      </dl>
    </div>
  );
}

export default function ModelHeader({ data }: Readonly<{ data: ModelData }>) {
  return (
    <div className={styles.root}>
      <Badges data={data} />
      <Markdown text={data.definition} className={styles.definition} />
      <div className={styles.links}>
        <Link to="#structure">Structure</Link>
        {data.searchParameters && <Link to="#search-parameters">Search parameters</Link>}
        {data.hl7 && <Link href={data.hl7}>HL7 specification</Link>}
        <Link href={data.raw} target="_blank">
          StructureDefinition JSON
        </Link>
      </div>
      <Coverage data={data} />
    </div>
  );
}
