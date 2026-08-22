import { useAtomValue } from "jotai";
import { useEffect, useState } from "react";

import { R4 } from "@haste-health/fhir-types/versions";
import { HasteHealthTenantBranding } from "@haste-health/generated-ops/lib/r4/ops";

import { getClient } from "../db/client";

export type TenantBrandingInfo = {
  name?: string;
  logoDataUrl?: string;
  loading: boolean;
};

export function useTenantBranding(): TenantBrandingInfo {
  const client = useAtomValue(getClient);
  const [branding, setBranding] = useState<Omit<TenantBrandingInfo, "loading">>(
    {},
  );
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    let cancelled = false;
    setLoading(true);

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
      })
      .finally(() => {
        if (!cancelled) {
          setLoading(false);
        }
      });

    return () => {
      cancelled = true;
    };
  }, [client]);

  return { ...branding, loading };
}
