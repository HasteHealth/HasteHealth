import { ExclamationCircleIcon } from "@heroicons/react/20/solid";
import classNames from "classnames";
import React from "react";

import { generateTailwindColorFromValue } from "../utilities";
import { Container } from "./Container";

export interface ScopeVerifyProps {
  title?: string;
  logo?: string;

  // Pulled from authorize endpoint where scope post should ultimatley go too.
  authorizeParameters: {
    client_id: string;
    response_type: string;
    state: string;
    code_challenge: string;
    code_challenge_method: string;

    scope?: string;
    redirect_uri?: string;
  };

  client: {
    name: string;
    logoUri?: string;
  };

  postURL: string;
}

export const ScopeVerifyForm = ({
  title = "HasteHealth",
  authorizeParameters,
  client,
  logo,
  postURL,
}: ScopeVerifyProps) => {
  return (
    <Container logo={logo} title={title}>
      <div>
        <div className="flex flex-col justify-center items-center mb-6 text-xl font-mono font-semibold text-[var(--ci-ink)] space-y-2">
          <div>
            {client.logoUri ? (
              <img
                className="w-12 h-12 rounded-none border border-[var(--ci-line)]"
                src={client.logoUri}
                alt="logo"
              />
            ) : (
              <div
                className={classNames(
                  "flex justify-center items-center w-12 h-12 rounded-none border border-[var(--ci-line)] bg-[var(--ci-surface-raised)] font-mono text-base font-semibold text-[var(--ci-signal)]",
                )}
              >
                <div>{client.name.substring(0, 1).toUpperCase()}</div>
              </div>
            )}
          </div>
          <div>{client.name}</div>
        </div>
      </div>
      <div>
        <span className="font-mono text-xs text-[var(--ci-muted)]">
          The above app is requesting the following permissions. Please review
          and either consent or deny access for the app.
        </span>
      </div>

      <div className="max-h-72 overflow-auto">
        <table className="border-collapse list-inside list-disc w-full font-mono text-xs text-[var(--ci-ink)]">
          {authorizeParameters.scope?.split(" ").map((scope) => (
            <tr className="border border-[var(--ci-line)]" key={scope}>
              <td className="p-3">{scope}</td>
              <td>
                <div className="items-center justify-center flex">
                  <ExclamationCircleIcon className="w-5 h-5 text-[var(--ci-muted)] opacity-50" />
                </div>
              </td>
            </tr>
          ))}
        </table>
      </div>
      <div className="justify-center items-center flex space-x-4">
        <form action={postURL} method="POST">
          {Object.keys(authorizeParameters).map((key) => (
            <input
              readOnly
              key={key}
              name={key}
              className="hidden"
              type="text"
              value={
                authorizeParameters[key as keyof typeof authorizeParameters]
              }
            />
          ))}
          <input
            readOnly
            className="hidden"
            name="accept"
            type="checkbox"
            checked={true}
          />
          <button
            type="submit"
            className="w-full rounded-none border border-[var(--ci-signal)] bg-[var(--ci-signal)] hover:bg-[var(--ci-signal-hover)] py-2.5 font-mono text-xs font-semibold uppercase tracking-wider text-white transition-colors"
          >
            Allow
          </button>
        </form>
        <form action={postURL} method="POST">
          {Object.keys(authorizeParameters).map((key) => (
            <input
              readOnly
              key={key}
              name={key}
              className="hidden"
              type="text"
              value={
                authorizeParameters[key as keyof typeof authorizeParameters]
              }
            />
          ))}
          <input
            readOnly
            className="hidden"
            name="accept"
            type="checkbox"
            checked={false}
          />
          <button
            type="submit"
            className="w-full rounded-none border border-[var(--ci-line)] bg-[var(--ci-surface-raised)] hover:bg-[var(--ci-surface-sunken)] py-2.5 font-mono text-xs uppercase tracking-wider text-[var(--ci-ink)] transition-colors"
          >
            Deny
          </button>
        </form>
      </div>
    </Container>
  );
};
