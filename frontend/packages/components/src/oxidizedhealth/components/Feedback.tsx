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
      <h1 className="text-xl font-bold leading-tight tracking-tight text-gray-900 md:text-2xl dark:text-white">
        {header}
      </h1>
    )}

    <p className="mb-2 text-sm font-medium text-gray-900 dark:text-white">
      {content}
    </p>

    {children}
  </Container>
);
