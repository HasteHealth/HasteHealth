import { useContext, useMemo } from "react";

import createHTTPClient from "@oxidized-health/client/lib/http";

import OxidizedHealthContext, {
  OxidizedHealthContextState,
} from "../OxidizedHealthContext";

export function useOxidizedHealth(): OxidizedHealthContextState & {
  client: ReturnType<typeof createHTTPClient>;
} {
  const context = useContext(OxidizedHealthContext);

  const client = useMemo(() => {
    return createHTTPClient({
      authenticate: () => context.reAuthenticate(context),
      getAccessToken: () =>
        Promise.resolve(context.payload?.access_token as string),
      url: context.rootURL as string,
    });
  }, [context.payload]);

  return {
    ...context,
    client,
  };
}
