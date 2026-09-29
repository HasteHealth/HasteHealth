import { useContext, useMemo } from "react";

import createHTTPClient from "@haste-health/client/lib/http";

import HasteHealthContext, {
  HasteHealthContextState,
} from "../HasteHealthContext";

interface UseHasteHealthConfig {
  searchMethod: "GET" | "POST";
}

export function useHasteHealth(
  config: UseHasteHealthConfig | undefined,
): HasteHealthContextState & {
  client: ReturnType<typeof createHTTPClient>;
} {
  const context = useContext(HasteHealthContext);

  const client = useMemo(() => {
    return createHTTPClient({
      authenticate: () => context.reAuthenticate(context),
      getAccessToken: () =>
        Promise.resolve(context.payload?.access_token as string),
      url: context.rootURL as string,
      // Haste Health serves `_search`; keeps identifiers out of URLs.
      searchMethod: config?.searchMethod ?? "GET",
    });
  }, [context.payload]);

  return {
    ...context,
    client,
  };
}
