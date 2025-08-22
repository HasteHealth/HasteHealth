import React from "react";

import { Container } from "./Container";

export type IdentityProvider = {
  url: string;
  title: string;
  logo?: string;
};

export type LoginProps = {
  email?: string;
  title?: string;
  logo?: string;
  action: string;
  errors?: string[];
  messages?: string[];
  signupURL?: string;
  hidePassword?: boolean;
  forgotPasswordURL?: string;
  federatedProviders?: IdentityProvider[];
};

export const Login = ({
  email,
  title = "OxidizedHealth",
  federatedProviders,
  errors,
  messages,
  logo,
  action,
  signupURL,
  hidePassword,
  forgotPasswordURL,
}: LoginProps) => (
  <Container logo={logo} title={title}>
    <div>
      {messages?.map((message) => {
        return (
          <div
            key={message}
            className="text-sm text-green-600 dark:text-green-400"
          >
            {message}
          </div>
        );
      })}
    </div>
    <div>
      {errors?.map((error) => {
        return (
          <div key={error} className="text-sm text-red-600 dark:text-red-400">
            {error}
          </div>
        );
      })}
    </div>
    <h1 className="text-xl font-bold leading-tight tracking-tight text-gray-900 md:text-2xl dark:text-white">
      Sign in to your account
    </h1>
    <form className="space-y-4 md:space-y-6" action={action} method="POST">
      <div>
        <label
          htmlFor="email"
          className="block mb-2 text-sm font-medium text-gray-900 dark:text-white"
        >
          Your email
        </label>
        <input
          type="email"
          name="email"
          id="email"
          defaultValue={email ?? ""}
          className="bg-gray-50 border border-gray-300 text-gray-900 sm:text-sm rounded-lg focus:ring-teal-600 focus:border-teal-600 block w-full p-2.5 dark:bg-gray-700 dark:border-gray-600 dark:placeholder-gray-400 dark:text-white dark:focus:ring-teal-500 dark:focus:border-teal-500"
          placeholder="name@company.com"
          required={true}
        />
      </div>
      {!hidePassword && (
        <div>
          <label
            htmlFor="password"
            className="block mb-2 text-sm font-medium text-gray-900 dark:text-white"
          >
            Password
          </label>
          <input
            type="password"
            name="password"
            id="password"
            placeholder="••••••••"
            className="bg-gray-50 border border-gray-300 text-gray-900 sm:text-sm rounded-lg focus:ring-teal-600 focus:border-teal-600 block w-full p-2.5 dark:bg-gray-700 dark:border-gray-600 dark:placeholder-gray-400 dark:text-white dark:focus:ring-teal-500 dark:focus:border-teal-500"
            required={true}
          />
        </div>
      )}
      <div className="flex items-center justify-between">
        <div className="flex items-start">
          <div className="flex items-center h-5">
            <input
              id="remember"
              aria-describedby="remember"
              type="checkbox"
              className="w-4 h-4 border border-gray-300 rounded bg-gray-50 focus:ring-3 focus:ring-teal-300 dark:bg-gray-700 dark:border-gray-600 dark:focus:ring-teal-600 dark:ring-offset-gray-800"
              defaultChecked={true}
              required={true}
            />
          </div>
          <div className="ml-3 text-sm">
            <label
              htmlFor="remember"
              className="text-gray-500 dark:text-gray-300"
            >
              Remember me
            </label>
          </div>
        </div>
        {forgotPasswordURL && (
          <a
            href={forgotPasswordURL}
            className="text-sm font-medium text-teal-600 hover:underline dark:text-teal-500"
          >
            Forgot password?
          </a>
        )}
      </div>
      <button
        type="submit"
        className="w-full text-white bg-teal-600 hover:bg-teal-700 focus:ring-4 focus:outline-none focus:ring-teal-300 font-medium rounded-lg text-sm px-5 py-2.5 text-center dark:bg-teal-600 dark:hover:bg-teal-700 dark:focus:ring-teal-800"
      >
        Sign in
      </button>
      {signupURL && (
        <p className="text-sm font-light text-gray-500 dark:text-gray-400">
          Don’t have an account yet?{" "}
          <a
            href={signupURL}
            className="font-medium text-teal-600 hover:underline dark:text-teal-500"
          >
            Sign up
          </a>
        </p>
      )}
    </form>
    {federatedProviders && (
      <div className="mt-4 space-y-2">
        {federatedProviders.map((provider) => (
          <a
            key={provider.title}
            href={provider.url}
            className="space-x-2 flex content-center justify-center text-white bg-slate-600 hover:bg-slate-700 focus:ring-4 focus:outline-none focus:ring-slate-300 font-medium rounded-lg text-sm px-5 py-2.5 text-center "
          >
            {provider.logo && (
              <div className="flex flex-col justify-center">
                <img alt="logo" src={provider.logo} className="block w-4 h-4" />
              </div>
            )}
            <div>Sign in with {provider.title}</div>
          </a>
        ))}
      </div>
    )}
  </Container>
);
