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
  title = "OxidizedHealth",
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
                className="text-sm text-red-600 dark:text-red-400"
              >
                {error}
              </div>
            );
          })}
      </div>
      <h1 className="text-xl font-bold leading-tight tracking-tight text-gray-900 md:text-2xl dark:text-white">
        {header}
      </h1>
      <form className="space-y-4 md:space-y-6" action={action} method="POST">
        <div>
          <label
            htmlFor="email"
            className="block mb-2 text-sm font-medium text-gray-900 dark:text-white"
          >
            Enter your email
          </label>
          <input
            type="email"
            name="email"
            id="email"
            defaultValue={email}
            className="bg-gray-50 border border-gray-300 text-gray-900 sm:text-sm rounded-lg focus:ring-teal-600 focus:border-teal-600 block w-full p-2.5 dark:bg-gray-700 dark:border-gray-600 dark:placeholder-gray-400 dark:text-white dark:focus:ring-teal-500 dark:focus:border-teal-500"
            placeholder="name@company.com"
            required={true}
          />
        </div>
        <button
          type="submit"
          className="w-full text-white bg-teal-600 hover:bg-teal-700 focus:ring-4 focus:outline-none focus:ring-teal-300 font-medium rounded-lg text-sm px-5 py-2.5 text-center dark:bg-teal-600 dark:hover:bg-teal-700 dark:focus:ring-teal-800"
        >
          Continue
        </button>
      </form>
    </Container>
  );
};
