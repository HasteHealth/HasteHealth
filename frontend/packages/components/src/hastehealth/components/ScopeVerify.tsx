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
        <div className="flex flex-col justify-center items-center mb-6 text-2xl font-semibold text-gray-900 dark:text-white space-y-2">
          <div>
            {client.logoUri ? (
              <img
                className="w-12 h-12 rounded-full"
                src={client.logoUri}
                alt="logo"
              />
            ) : (
              <div
                className={classNames(
                  "flex  justify-center items-center w-12 h-12 rounded-full",
                  `bg-${generateTailwindColorFromValue(client.name)}-100`,
                  `text-${generateTailwindColorFromValue(client.name)}-800`,
                  `dark:bg-${generateTailwindColorFromValue(client.name)}-800`,
                  `dark:text-${generateTailwindColorFromValue(client.name)}-100`
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
        <span className="text-sm text-gray-500">
          The above app is requesting the following permissions. Please review
          and either consent or deny access for the app.
        </span>
      </div>

      <div className="max-h-72 overflow-auto">
        <table className="border-collapse  list-inside list-disc w-full">
          {authorizeParameters.scope?.split(" ").map((scope) => (
            <tr className="border" key={scope}>
              <td className="p-4">{scope}</td>
              <td>
                <div className="items-center justify-center flex">
                  <ExclamationCircleIcon className="w-6 h-6 text-gray-300" />
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
            className="w-full text-white bg-orange-600 hover:bg-orange-700 focus:ring-4 focus:outline-none focus:ring-orange-300 font-medium rounded-lg text-sm px-5 py-2.5 text-center dark:bg-orange-600 dark:hover:bg-orange-700 dark:focus:ring-orange-800"
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
            className="w-full text-gray-900 bg-gray-100 hover:bg-gray-200 focus:ring-4 focus:outline-none  font-medium rounded-lg text-sm px-5 py-2.5 text-center dark:text-white dark:bg-gray-600 dark:hover:bg-gray-700 dark:focus:ring-gray-800"
          >
            Deny
          </button>
        </form>
      </div>
    </Container>
  );
};
