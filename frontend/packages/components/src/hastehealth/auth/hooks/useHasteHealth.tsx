import { useContext, useMemo } from "react";

import createHTTPClient from "@haste-health/client/lib/http";

import HasteHealthContext, {
  HasteHealthContextState,
} from "../HasteHealthContext";

export function useHasteHealth(): HasteHealthContextState & {
  client: ReturnType<typeof createHTTPClient>;
} {
  const context = useContext(HasteHealthContext);

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
