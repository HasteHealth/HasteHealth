import { OperationOutcome, code } from "@haste-health/fhir-types/r4/types";

export class OperationError extends Error {
  public readonly operationOutcome: OperationOutcome;
  constructor(operationOutcome: OperationOutcome) {
    super();
    this.operationOutcome = operationOutcome;
  }
  get outcome() {
    return this.operationOutcome;
  }
}

export function issueToStatusCode(issue: Issue): number {
  switch (issue.severity) {
    case "fatal": {
      return 500;
    }
    case "error": {
      switch (issue.code) {
        case "not-found": {
          return 404;
        }
        case "login": {
          return 401;
        }
        case "forbidden": {
          return 403;
        }
        default: {
          return 400;
        }
      }
    }
    case "warning": {
      return 200;
    }
    case "information": {
      return 200;
    }
    default: {
      throw new Error("unknown severity");
    }
  }
}

export function isOperationError(e: unknown): e is OperationError {
  return e instanceof OperationError;
}

export function outcome(issues: OperationOutcome["issue"]): OperationOutcome {
  return { resourceType: "OperationOutcome", issue: issues };
}

type Issue = OperationOutcome["issue"][number];
type IssueSeverity = "fatal" | "error" | "warning" | "information";
type IssueType =
  | "invalid"
  | "structure"
  | "required"
  | "value"
  | "invariant"
  | "security"
  | "login"
  | "unknown"
  | "expired"
  | "forbidden"
  | "suppressed"
  | "processing"
  | "not-supported"
  | "duplicate"
  | "multiple-matches"
  | "not-found"
  | "too-long"
  | "code-invalid"
  | "extension"
  | "too-costly"
  | "business-rule"
  | "conflict"
  | "transient"
  | "lock-error"
  | "no-store"
  | "exception"
  | "timeout"
  | "incomplete"
  | "throttled"
  | "informational";

export function issue(
  severity: IssueSeverity,
  code: IssueType,
  diagnostics: Issue["diagnostics"],
  expression?: Issue["expression"]
): Issue {
  const issue = {
    severity: severity as code,
    code: code as code,
    diagnostics,
    expression,
  };
  if (expression) return { ...issue, expression };
  return issue;
}

export function issueError(
  code: IssueType,
  diagnostics: Issue["diagnostics"],
  expression?: Issue["expression"]
): Issue {
  return issue("error", code, diagnostics, expression);
}

export function issueFatal(
  code: IssueType,
  diagnostics: Issue["diagnostics"],
  expression?: Issue["expression"]
): Issue {
  return issue("fatal", code, diagnostics, expression);
}

export function issueWarning(
  code: IssueType,
  diagnostics: Issue["diagnostics"],
  expression?: Issue["expression"]
): Issue {
  return issue("warning", code, diagnostics, expression);
}

export function issueInfo(
  code: IssueType,
  diagnostics: Issue["diagnostics"],
  expression?: Issue["expression"]
): Issue {
  return issue("information", code, diagnostics, expression);
}

export function outcomeError(
  code: IssueType,
  diagnostics: Issue["diagnostics"],
  expression?: Issue["expression"]
): OperationOutcome {
  return outcome([issueError(code, diagnostics, expression)]);
}

export function outcomeFatal(
  code: IssueType,
  diagnostics: Issue["diagnostics"],
  expression?: Issue["expression"]
): OperationOutcome {
  return outcome([issueFatal(code, diagnostics, expression)]);
}

export function outcomeWarning(
  code: IssueType,
  diagnostics: Issue["diagnostics"],
  expression?: Issue["expression"]
): OperationOutcome {
  return outcome([issueWarning(code, diagnostics, expression)]);
}

export function outcomeInfo(
  code: IssueType,
  diagnostics: Issue["diagnostics"],
  expression?: Issue["expression"]
): OperationOutcome {
  return outcome([issueInfo(code, diagnostics, expression)]);
}
