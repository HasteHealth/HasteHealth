import classNames from "classnames";
import React from "react";

import { TenantClaim, TenantId } from "@oxidized-health/jwt/types";

import { generateTailwindColorFromValue } from "../utilities";
import { Container } from "./Container";

export interface TenantSelectProps {
  email: string;
  tenants: TenantClaim<string>[];
  generateTenantURL: (email: string, tenantId: TenantId) => string;
  title?: string;
  logo?: string;
}

export const InitialDisplay = ({ value }: { value: string }) => {
  const initial = value.substring(0, 1).toUpperCase();
  const color = generateTailwindColorFromValue(value);

  return (
    <div
      className={classNames(
        "w-8 h-8 flex justify-center items-center rounded-full",
        `bg-${color}-100`,
        `text-${color}-800`,
        `dark:bg-${color}-800`,
        `dark:text-${color}-100`
      )}
    >
      <span className="font-bold">{initial}</span>
    </div>
  );
};

export const TenantSelect = ({
  email,
  tenants,
  title,
  logo,
  generateTenantURL,
}: TenantSelectProps) => {
  return (
    <Container logo={logo} title={title}>
      <h1 className="text-xl font-bold leading-tight tracking-tight text-gray-900 md:text-2xl dark:text-white">
        Choose a tenant
      </h1>
      <ul className="space-y-2">
        {tenants.map((t) => {
          return (
            <li key={t.id}>
              <a
                href={generateTenantURL(email, t.id)}
                className="p-2 flex cursor-pointer hover:bg-teal-50 dark:hover:bg-teal-800"
              >
                <div className="flex items-center space-x-4 rtl:space-x-reverse">
                  <div className="flex-shrink-0">
                    <InitialDisplay value={t.id as string} />
                  </div>
                  <div className="flex-1 min-w-0">
                    <p className="text-sm font-medium text-gray-900 truncate dark:text-white">
                      {t.id}
                    </p>
                    <p className="text-sm text-gray-500 truncate dark:text-gray-400">
                      {t.userRole}
                    </p>
                  </div>
                  <div className="inline-flex items-center text-base font-semibold text-gray-900 dark:text-white"></div>
                </div>
              </a>
            </li>
          );
        })}
      </ul>
    </Container>
  );
};
