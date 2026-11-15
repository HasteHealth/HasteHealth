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
  title = "HasteHealth",
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
            className="text-xs font-mono text-emerald-500"
          >
            {message}
          </div>
        );
      })}
    </div>
    <div>
      {errors?.map((error) => {
        return (
          <div key={error} className="text-xs font-mono text-red-500">
            {error}
          </div>
        );
      })}
    </div>
    <h1 className="text-lg font-mono font-bold uppercase tracking-wider text-[var(--ci-ink)] md:text-xl">
      Sign in to your account
    </h1>
    <form className="space-y-4 md:space-y-6" action={action} method="POST">
      <div>
        <label
          htmlFor="email"
          className="block mb-1.5 font-mono text-xs uppercase tracking-wider text-[var(--ci-muted)]"
        >
          Your email
        </label>
        <input
          type="email"
          name="email"
          id="email"
          defaultValue={email ?? ""}
          className="w-full rounded-none border border-[var(--ci-line)] bg-[var(--ci-surface-raised)] p-2.5 font-mono text-xs text-[var(--ci-ink)] placeholder-[var(--ci-muted)] focus:border-[var(--ci-signal)] focus:outline-none focus:ring-1 focus:ring-[var(--ci-signal)]"
          placeholder="name@company.com"
          required={true}
        />
      </div>
      {!hidePassword && (
        <div>
          <label
            htmlFor="password"
            className="block mb-1.5 font-mono text-xs uppercase tracking-wider text-[var(--ci-muted)]"
          >
            Password
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
      )}
      <div className="flex items-center justify-between">
        <div className="flex items-start">
          <div className="flex items-center h-5">
            <input
              id="remember"
              aria-describedby="remember"
              type="checkbox"
              className="w-4 h-4 rounded-none border border-[var(--ci-line)] bg-[var(--ci-surface-raised)] text-[var(--ci-signal)] focus:ring-[var(--ci-signal)]"
              defaultChecked={true}
              required={true}
            />
          </div>
          <div className="ml-3 text-xs">
            <label
              htmlFor="remember"
              className="font-mono text-xs text-[var(--ci-muted)]"
            >
              Remember me
            </label>
          </div>
        </div>
        {forgotPasswordURL && (
          <a
            href={forgotPasswordURL}
            className="font-mono text-xs text-[var(--ci-signal)] hover:underline"
          >
            Forgot password?
          </a>
        )}
      </div>
      <button
        type="submit"
        className="w-full rounded-none border border-[var(--ci-signal)] bg-[var(--ci-signal)] hover:bg-[var(--ci-signal-hover)] py-2.5 font-mono text-xs font-semibold uppercase tracking-wider text-[var(--ci-on-signal)] transition-colors"
      >
        Sign in
      </button>
      {signupURL && (
        <p className="font-mono text-xs text-[var(--ci-muted)]">
          Don’t have an account yet?{" "}
          <a
            href={signupURL}
            className="font-semibold text-[var(--ci-signal)] hover:underline"
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
            className="flex w-full items-center justify-center gap-2 rounded-none border border-[var(--ci-line)] bg-[var(--ci-surface-raised)] hover:bg-[var(--ci-surface)] py-2.5 font-mono text-xs uppercase tracking-wider text-[var(--ci-ink)] transition-colors"
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
