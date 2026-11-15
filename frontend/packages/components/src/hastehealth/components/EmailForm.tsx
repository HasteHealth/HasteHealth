import React from "react";

import { Container } from "./Container";

export type EmailFormProps = {
  email?: string;
  title?: string;
  logo?: string;
  header?: string;
  action: string;
  errors?: string[];
};

export const EmailForm = ({
  email,
  errors = [],
  title = "HasteHealth",
  header = "Signup",
  logo,
  action,
}: EmailFormProps) => {
  return (
    <Container logo={logo} title={title}>
      <div>
        {Array.isArray(errors) &&
          errors.map((error) => {
            return (
              <div
                key={error}
                className="text-xs font-mono text-red-500"
              >
                {error}
              </div>
            );
          })}
      </div>
      <h1 className="text-lg font-mono font-bold uppercase tracking-wider text-[var(--ci-ink)] md:text-xl">
        {header}
      </h1>
      <form className="space-y-4 md:space-y-6" action={action} method="POST">
        <div>
          <label
            htmlFor="email"
            className="block mb-1.5 font-mono text-xs uppercase tracking-wider text-[var(--ci-muted)]"
          >
            Enter your email
          </label>
          <input
            type="email"
            name="email"
            id="email"
            defaultValue={email}
            className="w-full rounded-none border border-[var(--ci-line)] bg-[var(--ci-surface-raised)] p-2.5 font-mono text-xs text-[var(--ci-ink)] placeholder-[var(--ci-muted)] focus:border-[var(--ci-signal)] focus:outline-none focus:ring-1 focus:ring-[var(--ci-signal)]"
            placeholder="name@company.com"
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
