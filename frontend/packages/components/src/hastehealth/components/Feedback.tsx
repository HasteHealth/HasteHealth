import React from "react";

import { Container } from "./Container";

export const Feedback = ({
  title,
  header,
  content,
  logo,
  children,
}: {
  logo?: string;
  title: string;
  header?: string;
  content: string | React.ReactNode;
  children?: React.ReactNode;
}) => (
  <Container logo={logo} title={title}>
    {header && (
      <h1 className="text-lg font-mono font-bold uppercase tracking-wider text-[var(--ci-ink)] md:text-xl">
        {header}
      </h1>
    )}

    <p className="mb-2 font-mono text-xs text-[var(--ci-ink)]">
      {content}
    </p>

    {children}
  </Container>
);
