import { useAtomValue } from "jotai";
import React, { useEffect, useState } from "react";

import { R4 } from "@haste-health/fhir-types/versions";
import { HasteHealthTenantBranding } from "@haste-health/generated-ops/lib/r4/ops";

import { getClient } from "../db/client";
import { Logo } from "./Logo";

export const AppLogo = ({
  className,
  onClick,
}: Readonly<{ className?: string; onClick?: () => void }>) => {
  const client = useAtomValue(getClient);
  const [logoDataUrl, setLogoDataUrl] = useState<string>();

  useEffect(() => {
    let cancelled = false;

    client
      .invoke_system(HasteHealthTenantBranding.Op, {}, R4, {})
      .then((res) => {
        if (cancelled) {
          return;
        }

        setLogoDataUrl(
          res.logo?.contentType && res.logo?.data
            ? `data:${res.logo.contentType};base64,${res.logo.data}`
            : undefined,
        );
      })
      .catch(() => {
        // Fall back to the default logo below if branding can't be loaded.
      });

    return () => {
      cancelled = true;
    };
  }, [client]);

  if (logoDataUrl) {
    return (
      <img
        src={logoDataUrl}
        alt="Tenant logo"
        className={className}
        onClick={onClick}
      />
    );
  }

  return <Logo className={className} onClick={onClick} />;
};
