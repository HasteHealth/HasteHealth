import React from "react";

import { OperationOutcome } from "@oxidized-health/fhir-types/r4/types";

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
  title = "OxidizedHealth",
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
              className="text-sm text-red-600 dark:text-red-400"
            >
              {issue.diagnostics || issue.code}
            </div>
          );
        })}
      </div>
      <h1 className="text-xl font-bold leading-tight tracking-tight text-gray-900 md:text-2xl dark:text-white">
        {header}
      </h1>
      <form className="space-y-4 md:space-y-6" action={action} method="POST">
        <input type="hidden" name="code" id="code" value={code} />
        <div>
          <label
            htmlFor="password"
            className="block mb-2 text-sm font-medium text-gray-900 dark:text-white"
          >
            Enter your Password
          </label>
          <input
            type="password"
            name="password"
            id="password"
            placeholder="••••••••"
            className="bg-gray-50 border border-gray-300 text-gray-900 sm:text-sm rounded-lg focus:ring-blue-600 focus:border-blue-600 block w-full p-2.5 dark:bg-gray-700 dark:border-gray-600 dark:placeholder-gray-400 dark:text-white dark:focus:ring-blue-500 dark:focus:border-blue-500"
            required={true}
          />
        </div>
        <div>
          <label
            htmlFor="passwordConfirm"
            className="block mb-2 text-sm font-medium text-gray-900 dark:text-white"
          >
            Confirm your Password
          </label>
          <input
            type="password"
            name="passwordConfirm"
            id="passwordConfirm"
            placeholder="••••••••"
            className="bg-gray-50 border border-gray-300 text-gray-900 sm:text-sm rounded-lg focus:ring-blue-600 focus:border-blue-600 block w-full p-2.5 dark:bg-gray-700 dark:border-gray-600 dark:placeholder-gray-400 dark:text-white dark:focus:ring-blue-500 dark:focus:border-blue-500"
            required={true}
          />
        </div>
        <button
          type="submit"
          className="w-full text-white bg-blue-600 hover:bg-blue-700 focus:ring-4 focus:outline-none focus:ring-blue-300 font-medium rounded-lg text-sm px-5 py-2.5 text-center dark:bg-blue-600 dark:hover:bg-blue-700 dark:focus:ring-blue-800"
        >
          Continue
        </button>
      </form>
    </Container>
  );
};
