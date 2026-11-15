import React from "react";

import { OperationOutcome } from "@haste-health/fhir-types/r4/types";

import { Container } from "./Container";

export type PasswordResetProps = {
  title?: string;
  header?: string;
  error?: OperationOutcome;
  logo?: string;
  action: string;
  code: string;
};

export const PasswordResetForm = ({
  title = "HasteHealth",
  header = "Password Reset",
  code,
  logo,
  error,
  action,
}: PasswordResetProps) => {
  return (
    <Container logo={logo} title={title}>
      <div>
        {error?.issue?.map((issue) => {
          return (
            <div
              key={issue.diagnostics || issue.code}
              className="text-xs font-mono text-red-500"
            >
              {issue.diagnostics || issue.code}
            </div>
          );
        })}
      </div>
      <h1 className="text-lg font-mono font-bold uppercase tracking-wider text-[var(--ci-ink)] md:text-xl">
        {header}
      </h1>
      <form className="space-y-4 md:space-y-6" action={action} method="POST">
        <input type="hidden" name="code" id="code" value={code} />
        <div>
          <label
            htmlFor="password"
            className="block mb-1.5 font-mono text-xs uppercase tracking-wider text-[var(--ci-muted)]"
          >
            Enter your Password
          </label>
          <input
            type="password"
            name="password"
            id="password"
            placeholder="••••••••"
            className="w-full rounded-none border border-[var(--ci-line)] bg-[var(--ci-surface-raised)] p-2.5 font-mono text-xs text-[var(--ci-ink)] placeholder-[var(--ci-muted)] focus:border-[var(--ci-signal)] focus:outline-none focus:ring-1 focus:ring-[var(--ci-signal)]"
            required={true}
          />
        </div>
        <div>
          <label
            htmlFor="passwordConfirm"
            className="block mb-1.5 font-mono text-xs uppercase tracking-wider text-[var(--ci-muted)]"
          >
            Confirm your Password
          </label>
          <input
            type="password"
            name="passwordConfirm"
            id="passwordConfirm"
            placeholder="••••••••"
            className="w-full rounded-none border border-[var(--ci-line)] bg-[var(--ci-surface-raised)] p-2.5 font-mono text-xs text-[var(--ci-ink)] placeholder-[var(--ci-muted)] focus:border-[var(--ci-signal)] focus:outline-none focus:ring-1 focus:ring-[var(--ci-signal)]"
            required={true}
          />
        </div>
        <button
          type="submit"
          className="w-full rounded-none border border-[var(--ci-signal)] bg-[var(--ci-signal)] hover:bg-[var(--ci-signal-hover)] py-2.5 font-mono text-xs font-semibold uppercase tracking-wider text-[var(--ci-on-signal)] transition-colors"
        >
          Continue
        </button>
      </form>
    </Container>
  );
};
