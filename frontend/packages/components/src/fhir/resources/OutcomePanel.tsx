import {
  ExclamationCircleIcon,
  ExclamationTriangleIcon,
  InformationCircleIcon,
} from "@heroicons/react/24/outline";
import classNames from "classnames";
import React from "react";

import { isResponseError } from "@haste-health/client/http";
import {
  OperationOutcome,
  OperationOutcomeIssue,
} from "@haste-health/fhir-types/r4/types";

/** What a failed request gives us to show. */
export interface Outcome {
  /** HTTP status, when the error came from the server. */
  status?: number;
  outcome?: OperationOutcome;
  /** Fallback for a failure with no outcome, e.g. a network error. */
  message: string;
}

/**
 * Normalises what was thrown into something displayable: an OperationOutcome
 * under a 4xx or 5xx, or a bare message for a transport failure.
 */
/** `Request failed with 404.`, or without the status when there is none. */
function failureMessage(status: number | undefined): string {
  const suffix = status === undefined ? "" : ` with ${status}`;
  return `Request failed${suffix}.`;
}

/** The panel heading: the status and how to read it. */
function headerLabel(status: number | undefined): string {
  if (status === undefined) return "Request failed";
  const reason = status >= 500 ? "Server Error" : "Request Rejected";
  return `${status} ${reason}`;
}

export function toOutcome(error: unknown): Outcome {
  // Matched by shape, not `instanceof`: the client is reached as source, as a
  // built bundle and through middleware, so the thrown error is not always the
  // class this module imported.
  const response = (
    error as {
      response?: {
        body?: unknown;
        http?: { status?: number };
      };
    }
  )?.response;

  const body = response?.body as OperationOutcome | undefined;
  const status = response?.http?.status;

  if (body?.resourceType === "OperationOutcome") {
    return {
      status,
      outcome: body,
      message:
        body.issue?.[0]?.diagnostics ??
        body.issue?.[0]?.details?.text ??
        failureMessage(status),
    };
  }

  // A `ResponseError` with no outcome still knows its status.
  if (response !== undefined || isResponseError(error)) {
    return {
      status,
      message: failureMessage(status),
    };
  }

  return {
    message: error instanceof Error ? error.message : "Request failed.",
  };
}

/** Icon and colour for a severity. */
function severityStyle(severity: string | undefined) {
  switch (severity) {
    case "fatal":
    case "error":
      return {
        Icon: ExclamationCircleIcon,
        row: "text-red-800",
        chip: "bg-red-100 text-red-800",
      };
    case "warning":
      return {
        Icon: ExclamationTriangleIcon,
        row: "text-amber-800",
        chip: "bg-amber-100 text-amber-800",
      };
    default:
      return {
        Icon: InformationCircleIcon,
        row: "text-slate-700",
        chip: "bg-slate-100 text-slate-700",
      };
  }
}

/**
 * A key for an issue. `OperationOutcome.issue` has no id, so the content
 * stands in; the list is static once rendered, so duplicates are harmless.
 */
function issueKey(issue: OperationOutcomeIssue): string {
  return [
    issue.severity ?? "",
    issue.code ?? "",
    issue.diagnostics ?? issue.details?.text ?? "",
    ...(issue.expression ?? []),
    ...(issue.location ?? []),
  ].join("|");
}

/** One issue: what went wrong, then where. */
function Issue({ issue }: Readonly<{ issue: OperationOutcomeIssue }>) {
  const { Icon, row, chip } = severityStyle(issue.severity);
  // A server may send either `expression` or its R4 predecessor `location`.
  const where = [...(issue.expression ?? []), ...(issue.location ?? [])];

  return (
    <li className="border-t border-slate-200 px-3 py-2 first:border-t-0">
      <div className={classNames("flex items-start gap-2 text-sm", row)}>
        <Icon className="mt-0.5 h-4 w-4 shrink-0" />
        <div className="min-w-0 flex-1">
          <div className="flex flex-wrap items-center gap-1.5">
            <span
              className={classNames(
                "rounded px-1.5 py-0.5 text-[11px] font-medium uppercase",
                chip,
              )}
            >
              {issue.severity ?? "information"}
            </span>
            {issue.code && (
              <span className="rounded bg-slate-100 px-1.5 py-0.5 font-mono text-[11px] text-slate-600">
                {issue.code}
              </span>
            )}
          </div>

          <p className="mt-1 whitespace-pre-wrap">
            {issue.diagnostics ??
              issue.details?.text ??
              "No further detail was provided."}
          </p>

          {where.length > 0 && (
            <p className="mt-1 font-mono text-xs text-slate-500">
              at {where.join(", ")}
            </p>
          )}
        </div>
      </div>
    </li>
  );
}

export interface OutcomePanelProps {
  outcome: Outcome;
  /** The request that produced it, shown alongside. */
  request?: { method: string; path: string };
}

/**
 * Shows a failed request in place of the results. Each issue names a severity,
 * a code, a message and often the element at fault, which is more than a toast
 * can carry and exactly what someone debugging needs.
 */
export function OutcomePanel({
  outcome,
  request,
}: Readonly<OutcomePanelProps>) {
  const issues = outcome.outcome?.issue ?? [];

  return (
    <div className="overflow-hidden rounded-lg border border-red-200 bg-white">
      <header className="flex flex-wrap items-center gap-2 border-b border-red-200 bg-red-50 px-3 py-2">
        <ExclamationCircleIcon className="h-4 w-4 text-red-700" />
        <span className="text-sm font-medium text-red-900">
          {headerLabel(outcome.status)}
        </span>
        {request && (
          <span className="rounded bg-white/70 px-1.5 py-0.5 font-mono text-xs text-red-800">
            {request.method} {request.path}
          </span>
        )}
      </header>

      {issues.length > 0 ? (
        <ul>
          {issues.map((issue) => (
            <Issue key={issueKey(issue)} issue={issue} />
          ))}
        </ul>
      ) : (
        <p className="px-3 py-3 text-sm text-slate-700">{outcome.message}</p>
      )}

      {outcome.outcome && (
        <details className="border-t border-slate-200">
          <summary className="cursor-pointer px-3 py-2 text-xs text-slate-500 hover:bg-slate-50">
            Raw OperationOutcome
          </summary>
          <pre className="max-h-64 overflow-auto bg-slate-50 px-3 py-2 font-mono text-xs text-slate-700">
            {JSON.stringify(outcome.outcome, null, 2)}
          </pre>
        </details>
      )}
    </div>
  );
}
