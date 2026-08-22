import { useAtomValue } from "jotai";
import { useEffect, useState } from "react";

import { R4 } from "@haste-health/fhir-types/versions";
import { HasteHealthTenantBranding } from "@haste-health/generated-ops/lib/r4/ops";

import { getClient } from "../db/client";

export type TenantBrandingInfo = {
  name?: string;
  logoDataUrl?: string;
};

export function useTenantBranding(): TenantBrandingInfo {
  const client = useAtomValue(getClient);
  const [branding, setBranding] = useState<TenantBrandingInfo>({});

  useEffect(() => {
    let cancelled = false;

    client
      .invoke_system(HasteHealthTenantBranding.Op, {}, R4, {})
      .then((res) => {
        if (cancelled) {
          return;
        }

        setBranding({
          name: res.name,
          logoDataUrl:
            res.logo?.contentType && res.logo?.data
              ? `data:${res.logo.contentType};base64,${res.logo.data}`
              : undefined,
        });
      })
      .catch(() => {
        // Leave branding empty so callers fall back to their defaults.
      });

    return () => {
      cancelled = true;
    };
  }, [client]);

  return branding;
}
