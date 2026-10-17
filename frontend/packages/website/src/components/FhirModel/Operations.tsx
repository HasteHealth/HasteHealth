import React from "react";
import Link from "@docusaurus/Link";
import useBrokenLinks from "@docusaurus/useBrokenLinks";

import Markdown from "./Markdown";
import { Badge } from "./ModelHeader";
import type { Operation, OperationLevel, OperationParameter } from "./types";
import styles from "./styles.module.css";

export const LEVEL_LABELS: Record<OperationLevel, string> = {
  system: "System",
  type: "Type",
  instance: "Instance",
};

const LEVEL_TITLES: Record<OperationLevel, string> = {
  system: "Invoked on the base URL",
  type: "Invoked on the resource type",
  instance: "Invoked on one resource",
};

const ENDPOINTS = "/docs/api/rest_api/fhir/endpoints";

/**
 * The requests that invoke `operation`, one per level it allows: on a
 * resource page for that type only, on the system page for every type it is
 * defined on. The server accepts operations by POST only.
 */
function requests(operation: Operation, type?: string): string[] {
  const types = type
    ? [type]
    : operation.resources.map((resource) => resource.name);
  const name = `$${operation.code}`;
  return operation.levels.flatMap((level) => {
    if (level === "system") return [`POST [base]/${name}`];
    const tail = level === "type" ? name : `[id]/${name}`;
    return (types.length > 0 ? types : ["[type]"]).map(
      (t) => `POST [base]/${t}/${tail}`,
    );
  });
}

function TypeLink({ name, href }: Readonly<{ name: string; href?: string }>) {
  return href ? <Link to={href}>{name}</Link> : <span>{name}</span>;
}

function ParameterType({
  parameter,
}: Readonly<{ parameter: OperationParameter }>) {
  if (!parameter.type) {
    return parameter.parts ? <span className={styles.muted}>parts</span> : null;
  }
  return (
    <span className={styles.types}>
      <TypeLink name={parameter.type} href={parameter.href} />
      {parameter.targets && (
        <>
          (
          {parameter.targets.map((target, i) => (
            <React.Fragment key={target.name}>
              {i > 0 && " | "}
              <TypeLink name={target.name} href={target.href} />
            </React.Fragment>
          ))}
          )
        </>
      )}
    </span>
  );
}

const lastSegment = (url: string) => url.split("/").pop() ?? url;

function ParameterRows({
  parameters,
  depth = 0,
  prefix = "",
}: Readonly<{
  parameters: OperationParameter[];
  depth?: number;
  prefix?: string;
}>) {
  return (
    <>
      {parameters.map((parameter) => {
        const key = `${prefix}${parameter.name}`;
        return (
          <React.Fragment key={key}>
            <tr>
              <td>
                <span
                  className={styles.paramName}
                  style={{ "--depth": depth } as React.CSSProperties}
                >
                  {parameter.name}
                </span>
              </td>
              <td className={styles.paramCard}>
                {parameter.min}..{parameter.max}
              </td>
              <td>
                <ParameterType parameter={parameter} />
              </td>
              <td>
                {parameter.documentation && (
                  <Markdown
                    text={parameter.documentation}
                    className={styles.paramDescription}
                  />
                )}
                {parameter.searchType && (
                  <span className={styles.binding}>
                    Search type: {parameter.searchType}
                  </span>
                )}
                {parameter.binding && (
                  <span className={styles.binding}>
                    Binding: {parameter.binding.strength}{" "}
                    <a
                      href={parameter.binding.valueSet}
                      target="_blank"
                      rel="noopener noreferrer"
                    >
                      {lastSegment(parameter.binding.valueSet)}
                    </a>
                  </span>
                )}
              </td>
            </tr>
            {parameter.parts && (
              <ParameterRows
                parameters={parameter.parts}
                depth={depth + 1}
                prefix={`${key}.`}
              />
            )}
          </React.Fragment>
        );
      })}
    </>
  );
}

function Parameters({
  parameters,
  open,
}: Readonly<{ parameters: OperationParameter[]; open: boolean }>) {
  const groups: [string, OperationParameter[]][] = [
    ["In", parameters.filter((parameter) => parameter.use === "in")],
    ["Out", parameters.filter((parameter) => parameter.use === "out")],
  ];
  if (parameters.length === 0) {
    return <p className={styles.muted}>Takes no parameters.</p>;
  }
  return (
    <details className={styles.parameters} open={open}>
      <summary>
        Parameters: {groups[0][1].length} in, {groups[1][1].length} out
      </summary>
      <div className={styles.scroll}>
        <table className={styles.table}>
          <thead>
            <tr>
              <th>Name</th>
              <th>Card.</th>
              <th>Type</th>
              <th>Description</th>
            </tr>
          </thead>
          <tbody>
            {groups
              .filter(([, list]) => list.length > 0)
              .map(([use, list]) => (
                <React.Fragment key={use}>
                  <tr className={styles.useRow}>
                    <th colSpan={4}>{use}</th>
                  </tr>
                  <ParameterRows parameters={list} />
                </React.Fragment>
              ))}
          </tbody>
        </table>
      </div>
    </details>
  );
}

function OperationCard({
  operation,
  type,
  open,
}: Readonly<{ operation: Operation; type?: string; open: boolean }>) {
  const anchor = `op-${operation.code}`;
  // The system page links here; the build's anchor check only knows anchors
  // that are declared to it.
  useBrokenLinks().collectAnchor(anchor);
  return (
    <section id={anchor} className={styles.operation}>
      <div className={styles.operationHead}>
        <h3 className={styles.operationCode}>${operation.code}</h3>
        <span className={styles.operationBadges}>
          {operation.levels.map((level) => (
            <Badge key={level} title={LEVEL_TITLES[level]} accent>
              {LEVEL_LABELS[level]}
            </Badge>
          ))}
          {operation.affectsState && (
            <Badge title="Invoking it changes data on the server">
              Affects state
            </Badge>
          )}
        </span>
      </div>
      {operation.title && operation.title !== operation.code && (
        <p className={styles.operationTitle}>{operation.title}</p>
      )}
      {operation.description && (
        <Markdown
          text={operation.description}
          className={styles.operationDescription}
        />
      )}
      <div className={styles.requests}>
        {requests(operation, type).map((line) => (
          <code key={line}>{line}</code>
        ))}
      </div>
      <Parameters parameters={operation.parameters} open={open} />
      <div className={styles.links}>
        <Link href={operation.raw} target="_blank">
          OperationDefinition JSON
        </Link>
        {operation.hl7 && <Link href={operation.hl7}>HL7 specification</Link>}
        {!type &&
          operation.resources
            .filter((resource) => resource.href)
            .map((resource) => (
              <Link key={resource.name} to={`${resource.href}#operations`}>
                On {resource.name}
              </Link>
            ))}
      </div>
    </section>
  );
}

/**
 * The operations of a resource type (`type` given) or of the system level,
 * each with how to invoke it and its parameters. Parameters start expanded
 * when the list is short enough to read top to bottom.
 */
export default function Operations({
  operations,
  type,
}: Readonly<{ operations: Operation[]; type?: string }>) {
  if (operations.length === 0) {
    return (
      <p className={styles.muted}>
        {type
          ? `${type} has no operations of its own.`
          : "No system-level operations."}
      </p>
    );
  }
  const open = operations.length <= 3;
  return (
    <div className={styles.root}>
      <p>
        {type ? (
          <>
            Invoke with <code>POST [base]/{type}/$[name]</code> on the type, or{" "}
            <code>POST [base]/{type}/[id]/$[name]</code> on one resource,
            sending a <code>Parameters</code> resource as the body. See{" "}
            <Link to={`${ENDPOINTS}/invoke-type`}>invoke-type</Link> and{" "}
            <Link to={`${ENDPOINTS}/invoke-instance`}>invoke-instance</Link>.
          </>
        ) : (
          <>
            Invoke with <code>POST [base]/$[name]</code>, sending a{" "}
            <code>Parameters</code> resource as the body. The out parameters say
            what comes back: a resource, or a <code>Parameters</code> resource
            holding them.
          </>
        )}
      </p>
      {operations.map((operation) => (
        <OperationCard
          key={operation.code}
          operation={operation}
          type={type}
          open={open}
        />
      ))}
    </div>
  );
}
