import React, { type CSSProperties } from "react";

import { useHomepageFacts } from "./facts";
import styles from "./styles.module.css";

type CellResult = "pass" | "warn" | "fail";

/**
 * One square per FHIR resource type in the conformance report, coloured by
 * its result. The squares fill in as a wave when the block scrolls into view.
 *
 * Colour never carries the result alone: the legend pairs each one with an
 * icon, a label and a count, and the full table is the conformance page this
 * block links to. The squares themselves are hidden from assistive technology,
 * because the legend and the figure above them already say all they show.
 */
export default function ConformanceGrid() {
  const { conformance } = useHomepageFacts();

  // Passing types first, so the exceptions sit together at the end of the grid.
  const results: CellResult[] = [
    ...Array.from({ length: conformance.passing }, () => "pass" as const),
    ...conformance.knownIssues.map(() => "warn" as const),
    ...conformance.failing.map(() => "fail" as const),
  ];
  const cells = results.map((result, position) => ({
    id: `cell-${position}`,
    position,
    result,
  }));

  return (
    <div>
      <div className={styles.cells} aria-hidden="true" data-reveal="">
        {cells.map((cell) => (
          <span
            key={cell.id}
            className={styles.cell}
            data-result={cell.result}
            style={{ "--i": cell.position } as CSSProperties}
          />
        ))}
      </div>

      <ul className="mt-4 flex list-none flex-wrap gap-x-6 gap-y-2 p-0 text-sm text-slate-600">
        <li className="flex items-center gap-2">
          <LegendIcon result="pass" />
          Passes every check ({conformance.passing})
        </li>
        {conformance.knownIssues.length > 0 ? (
          <li className="flex items-center gap-2">
            <LegendIcon result="warn" />
            Known issue, published: {conformance.knownIssues.join(", ")}
          </li>
        ) : null}
        {conformance.failing.length > 0 ? (
          <li className="flex items-center gap-2">
            <LegendIcon result="fail" />
            Failing: {conformance.failing.join(", ")}
          </li>
        ) : null}
      </ul>
    </div>
  );
}

// The same three shapes the conformance page uses for these results.
const LEGEND: Record<CellResult, { tone: string; path: string }> = {
  pass: {
    tone: "text-brand-700",
    path: "M10 18a8 8 0 1 0 0-16 8 8 0 0 0 0 16Zm3.857-9.809a.75.75 0 0 0-1.214-.882l-3.483 4.79-1.88-1.88a.75.75 0 1 0-1.06 1.061l2.5 2.5a.75.75 0 0 0 1.137-.089l4-5.5Z",
  },
  warn: {
    tone: "text-amber-500",
    path: "M8.485 2.495c.673-1.167 2.357-1.167 3.03 0l6.28 10.875c.673 1.167-.17 2.625-1.516 2.625H3.72c-1.347 0-2.189-1.458-1.515-2.625L8.485 2.495ZM10 5a.75.75 0 0 1 .75.75v3.5a.75.75 0 0 1-1.5 0v-3.5A.75.75 0 0 1 10 5Zm0 9a1 1 0 1 0 0-2 1 1 0 0 0 0 2Z",
  },
  fail: {
    tone: "text-red-600",
    path: "M10 18a8 8 0 1 0 0-16 8 8 0 0 0 0 16ZM8.28 7.22a.75.75 0 0 0-1.06 1.06L8.94 10l-1.72 1.72a.75.75 0 1 0 1.06 1.06L10 11.06l1.72 1.72a.75.75 0 1 0 1.06-1.06L11.06 10l1.72-1.72a.75.75 0 0 0-1.06-1.06L10 8.94 8.28 7.22Z",
  },
};

function LegendIcon({ result }: Readonly<{ result: CellResult }>) {
  return (
    <svg
      viewBox="0 0 20 20"
      fill="currentColor"
      aria-hidden="true"
      className={`h-4 w-4 shrink-0 ${LEGEND[result].tone}`}
    >
      <path fillRule="evenodd" d={LEGEND[result].path} clipRule="evenodd" />
    </svg>
  );
}
