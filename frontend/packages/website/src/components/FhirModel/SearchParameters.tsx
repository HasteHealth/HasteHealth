import React, { useDeferredValue, useState } from "react";

import { BACKENDS, BACKEND_NAMES } from "../TestCoverage/data";
import { StatusIcon } from "../TestCoverage";
import Markdown from "./Markdown";
import type { ModelData, SearchParameter } from "./types";
import styles from "./styles.module.css";

function TypeBadge({ type }: Readonly<{ type: string }>) {
  const className = [styles.paramType, styles[`type-${type}`]].filter(Boolean).join(" ");
  return <span className={className}>{type}</span>;
}

function matches(parameter: SearchParameter, query: string): boolean {
  const q = query.trim().toLowerCase();
  if (!q) return true;
  return [parameter.code, parameter.type, parameter.description, parameter.expression ?? ""].some(
    (text) => text.toLowerCase().includes(q),
  );
}

function Tested({ parameter }: Readonly<{ parameter: SearchParameter }>) {
  return (
    <>
      {BACKENDS.map((backend) => (
        <td key={backend} className={styles.center}>
          <StatusIcon result={parameter.coverage?.[backend] ?? "not-run"} />
        </td>
      ))}
    </>
  );
}

export default function SearchParameters({ data }: Readonly<{ data: ModelData }>) {
  const [query, setQuery] = useState("");
  const deferred = useDeferredValue(query);
  const parameters = data.searchParameters ?? [];
  const common = data.commonParameters ?? [];
  const tested = Boolean(data.coverage);
  const shown = parameters.filter((p) => matches(p, deferred));

  return (
    <div className={styles.root}>
      <p>
        Query with <code>GET [base]/{data.name}?[parameter]=[value]</code>. Type decides which
        modifiers and prefixes apply, see{" "}
        <a href="/docs/reference/conformance/test-coverage#search-features">search features</a>.
      </p>
      {parameters.length > 8 && (
        <div className={styles.toolbar}>
          <input
            type="search"
            className={styles.filter}
            placeholder={`Filter ${parameters.length} parameters`}
            aria-label="Filter search parameters"
            value={query}
            onChange={(e) => setQuery(e.target.value)}
          />
          {deferred && (
            <span className={styles.muted}>
              {shown.length} of {parameters.length}
            </span>
          )}
        </div>
      )}
      {parameters.length === 0 ? (
        <p className={styles.muted}>
          {data.name} defines no search parameters of its own; the common parameters below apply.
        </p>
      ) : (
        <div className={styles.scroll}>
          <table className={styles.table}>
            <thead>
              <tr>
                <th>Parameter</th>
                <th>Type</th>
                <th>Description</th>
                {tested &&
                  BACKENDS.map((backend) => (
                    <th key={backend} className={styles.center} title={`Tested on ${BACKEND_NAMES[backend]}`}>
                      {backend === "postgres" ? "PG" : "ES"}
                    </th>
                  ))}
              </tr>
            </thead>
            <tbody>
              {shown.map((parameter) => (
                <tr key={parameter.code} id={`param-${parameter.code}`}>
                  <td className={styles.paramCode}>
                    <code>{parameter.code}</code>
                  </td>
                  <td>
                    <TypeBadge type={parameter.type} />
                  </td>
                  <td>
                    <Markdown text={parameter.description} className={styles.paramDescription} />
                    {parameter.expression && (
                      <code className={styles.expression}>{parameter.expression}</code>
                    )}
                  </td>
                  {tested && <Tested parameter={parameter} />}
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
      {tested && (
        <p className={styles.legend}>
          PG: PostgreSQL, ES: Elasticsearch. <StatusIcon result="pass" className="h-3.5 w-3.5" /> passes
          its TestScript checks, <StatusIcon result="fail" className="h-3.5 w-3.5" /> fails,{" "}
          <StatusIcon result="not-run" className="h-3.5 w-3.5" /> not tested yet.
        </p>
      )}
      {common.length > 0 && (
        <details className={styles.common}>
          <summary>Common parameters on every resource ({common.length})</summary>
          <div className={styles.scroll}>
            <table className={styles.table}>
              <tbody>
                {common.map((parameter) => (
                  <tr key={parameter.code}>
                    <td className={styles.paramCode}>
                      <code>{parameter.code}</code>
                    </td>
                    <td>
                      <TypeBadge type={parameter.type} />
                    </td>
                    <td>
                      <Markdown text={parameter.description} className={styles.paramDescription} />
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </details>
      )}
    </div>
  );
}
