import React from "react";

import {
  OperationOutcome,
  OperationOutcomeIssue,
} from "@haste-health/fhir-types/r4/types";

function capitalize(string: string) {
  return string.charAt(0).toUpperCase() + string.slice(1);
}

export const OperationOutcomeIssueDisplay = ({
  hideCode = false,
  issue,
}: {
  hideCode?: boolean;
  issue: OperationOutcomeIssue;
}) => (
  <div>
    {!hideCode && (
      <div className="mt-2 px-4 py-2 space-y-2 md:space-y-2 ">
        <h1 className="text-xl font-bold leading-tight tracking-tight text-red-500 md:text-2xl">
          {capitalize(issue.code)}
        </h1>
      </div>
    )}

    <div className="space-y-4 md:space-y-6">
      <p className="p-4 mb-2 text-sm font-medium text-red-500">
        {issue.diagnostics}
      </p>
    </div>
  </div>
);

export const FHIROperationOutcomeDisplay = ({
  title,
  logo,
  operationOutcome,
  children,
}: {
  logo?: string;
  title: string;
  operationOutcome: OperationOutcome;
  children?: React.ReactNode;
}) => (
  <section className="bg-gray-50 dark:bg-gray-900">
    <div className="flex flex-col items-center justify-center px-6 py-8 mx-auto md:h-screen lg:py-0">
      <a
        href="#"
        className="flex items-center mb-6 text-2xl font-semibold text-gray-900 dark:text-white"
      >
        {logo && <img className="w-8 h-8 mr-2" src={logo} alt="logo" />}
        {title}
      </a>
      <div className="w-full bg-white rounded-lg shadow dark:border md:mt-0 sm:max-w-md xl:p-0 dark:bg-gray-800 dark:border-gray-700 spacing-y-4">
        {operationOutcome.issue.map((i, index) => (
          <OperationOutcomeIssueDisplay key={index} issue={i} />
        ))}
        {children}
      </div>
    </div>
  </section>
);
