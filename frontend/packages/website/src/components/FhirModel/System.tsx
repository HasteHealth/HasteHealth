import React from "react";
import Link from "@docusaurus/Link";

import { Badge } from "./ModelHeader";
import { LEVEL_LABELS } from "./Operations";
import type { SystemData } from "./types";
import styles from "./styles.module.css";

const ENDPOINTS = "/docs/api/rest_api/fhir/endpoints";

type InteractionInfo = { label: string; request: string; summary: string; docs?: string };

/**
 * How each system-level interaction is requested. `metadata` is not a code in
 * the CapabilityStatement (FHIR takes it as given) and is always listed.
 */
const INTERACTIONS: Record<string, InteractionInfo> = {
  metadata: {
    label: "Capabilities",
    request: "GET [base]/metadata",
    summary: "The CapabilityStatement this page is generated from.",
  },
  "search-system": {
    label: "Search",
    request: "GET [base]?[parameters]",
    summary: "Search across every resource type.",
    docs: `${ENDPOINTS}/search-system`,
  },
  "history-system": {
    label: "History",
    request: "GET [base]/_history",
    summary: "Every change to every resource, newest first.",
    docs: `${ENDPOINTS}/history-system`,
  },
  transaction: {
    label: "Transaction",
    request: "POST [base]",
    summary: "A Bundle of type transaction, applied as one unit: all of it or none of it.",
  },
  batch: {
    label: "Batch",
    request: "POST [base]",
    summary: "A Bundle of type batch; each entry succeeds or fails on its own.",
  },
};

const ORDER = Object.keys(INTERACTIONS);
const order = (code: string) => {
  const index = ORDER.indexOf(code);
  return index === -1 ? ORDER.length : index;
};

export function SystemHeader({ data }: Readonly<{ data: SystemData }>) {
  return (
    <div className={styles.root}>
      <div className={styles.badges}>
        <Badge accent>FHIR {data.fhirVersion}</Badge>
        {data.formats.map((format) => (
          <Badge key={format}>{format}</Badge>
        ))}
        {data.cors && <Badge title="Browsers may call the API from other origins">CORS</Badge>}
      </div>
      <p className={styles.definition}>
        What the server offers on the base URL itself,{" "}
        <code>https://[host]/w/[tenant]/[project]/api/v1/fhir</code>: the interactions that span
        resource types, the operations invoked at the system level, and which of the{" "}
        {data.resourceTypes} resource types have operations of their own. Generated from the
        server&apos;s CapabilityStatement and the OperationDefinitions it serves.
      </p>
      <div className={styles.links}>
        <Link to="#interactions">Interactions</Link>
        <Link to="#operations">Operations</Link>
        <Link to="#operations-by-resource-type">By resource type</Link>
        <Link to="/docs/reference/fhir/model/resources/">All resources</Link>
        <Link to="/docs/api/rest_api/fhir/intro">REST API</Link>
        <Link href={data.raw} target="_blank">
          CapabilityStatement JSON
        </Link>
      </div>
    </div>
  );
}

export function SystemInteractions({ data }: Readonly<{ data: SystemData }>) {
  const codes = [...new Set(["metadata", ...data.interactions])].sort(
    (a, b) => order(a) - order(b),
  );
  return (
    <div className={styles.root}>
      <div className={styles.scroll}>
        <table className={styles.table}>
          <thead>
            <tr>
              <th>Interaction</th>
              <th>Request</th>
              <th>Description</th>
            </tr>
          </thead>
          <tbody>
            {codes.map((code) => {
              const info = INTERACTIONS[code] ?? { label: code, request: "", summary: "" };
              return (
                <tr key={code} id={`interaction-${code}`}>
                  <td className={styles.paramCode}>
                    <span className={styles.interactionLabel}>{info.label}</span>
                    <code className={styles.expression}>{code}</code>
                  </td>
                  <td>{info.request && <code className={styles.request}>{info.request}</code>}</td>
                  <td>
                    {info.summary}
                    {info.docs && (
                      <>
                        {" "}
                        <Link to={info.docs}>Documentation</Link>
                      </>
                    )}
                  </td>
                </tr>
              );
            })}
          </tbody>
        </table>
      </div>
      <p className={styles.legend}>
        The interactions on a resource type (create, read, update, patch, delete, search and
        history) are on its page, with their test results.
      </p>
    </div>
  );
}

export function OperationIndex({ data }: Readonly<{ data: SystemData }>) {
  if (data.resources.length === 0) {
    return <p className={styles.muted}>No resource type has operations of its own.</p>;
  }
  return (
    <div className={styles.root}>
      <p>
        The resource types with type- or instance-level operations. Every other resource type
        supports the standard interactions only.
      </p>
      <div className={styles.scroll}>
        <table className={styles.table}>
          <thead>
            <tr>
              <th>Resource type</th>
              <th>Operations</th>
            </tr>
          </thead>
          <tbody>
            {data.resources.map((resource) => (
              <tr key={resource.name}>
                <td className={styles.paramCode}>
                  <Link to={resource.href}>{resource.name}</Link>
                </td>
                <td>
                  <span className={styles.chips}>
                    {resource.operations.map((operation) => (
                      <Link
                        key={operation.code}
                        to={`${resource.href}#op-${operation.code}`}
                        className={styles.chip}
                        title={`${operation.levels.map((level) => LEVEL_LABELS[level]).join(" and ")} level`}
                      >
                        ${operation.code}
                      </Link>
                    ))}
                  </span>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}
