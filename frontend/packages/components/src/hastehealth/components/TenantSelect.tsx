import classNames from "classnames";
import React from "react";

import { TenantClaim, TenantId } from "@haste-health/jwt/types";

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

  return (
    <div
      className={classNames(
        "w-8 h-8 flex justify-center items-center rounded-none border border-[var(--ci-line)] bg-[var(--ci-surface-raised)] text-[var(--ci-signal)] font-mono text-xs font-bold",
      )}
    >
      <span>{initial}</span>
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
      <h1 className="text-lg font-mono font-bold uppercase tracking-wider text-[var(--ci-ink)] md:text-xl">
        Choose a tenant
      </h1>
      <ul className="space-y-2">
        {tenants.map((t) => {
          return (
            <li key={t.id}>
              <a
                href={generateTenantURL(email, t.id)}
                className="p-2.5 flex cursor-pointer rounded-none border border-[var(--ci-line)] bg-[var(--ci-surface)] hover:bg-[var(--ci-surface-raised)] transition-colors"
              >
                <div className="flex items-center space-x-3 rtl:space-x-reverse">
                  <div className="flex-shrink-0">
                    <InitialDisplay value={t.id as string} />
                  </div>
                  <div className="flex-1 min-w-0">
                    <p className="text-xs font-mono font-medium text-[var(--ci-ink)] truncate">
                      {t.id}
                    </p>
                    <p className="text-[11px] font-mono text-[var(--ci-muted)] truncate">
                      {t.userRole}
                    </p>
                  </div>
                </div>
              </a>
            </li>
          );
        })}
      </ul>
    </Container>
  );
};
