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
            className="bg-gray-50 border border-gray-300 text-gray-900 sm:text-sm rounded-lg focus:ring-brand-600 focus:border-brand-600 block w-full p-2.5 dark:bg-gray-700 dark:border-gray-600 dark:placeholder-gray-400 dark:text-white dark:focus:ring-brand-500 dark:focus:border-brand-500"
            placeholder="name@company.com"
            required={true}
          />
        </div>
        <button
          type="submit"
          className="w-full text-white bg-brand-500 hover:bg-brand-500 focus:ring-4 focus:outline-none focus:ring-brand-300 font-medium rounded-lg text-sm px-5 py-2.5 text-center dark:bg-brand-500 dark:hover:bg-brand-500 dark:focus:ring-brand-800"
        >
          Continue
        </button>
      </form>
    </Container>
  );
};
