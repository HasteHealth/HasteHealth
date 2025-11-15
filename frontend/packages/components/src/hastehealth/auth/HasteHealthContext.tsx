import { createContext } from "react";

import {
  TenantId,
  AccessToken,
  IDToken,
  IDTokenPayload,
  ProjectId,
} from "@haste-health/jwt/types";

import { OIDC_WELL_KNOWN } from "./reducer";

export type AccessTokenResponse = {
  access_token: AccessToken<string>;
  id_token: IDToken<string>;
  token_type: string;
  expires_in: number;
  refresh_token?: string;
};

export type HasteHealthContextState = {
  tenant?: TenantId;
  project?: ProjectId;
  rootURL?: string;
  well_known_uri?: string;
  well_known?: OIDC_WELL_KNOWN;
  logout: (redirect: string) => void;
  isAuthenticated: boolean;
  payload?: AccessTokenResponse;
  user?: IDTokenPayload<string>;
  loading: boolean;
  error?: {
    code: string;
    description: string;
    uri?: string;
    state?: string;
  };
  reAuthenticate: (state: HasteHealthContextState) => void;
};

const stub = (): never => {
  throw new Error("HasteHealth has not been initiated.");
};

export const InitialContext: HasteHealthContextState = {
  tenant: undefined,
  project: undefined,
  logout: stub,
  reAuthenticate: stub,
  well_known_uri: undefined,
  isAuthenticated: false,
  payload: undefined,
  user: undefined,
  loading: false,
  error: undefined,
};

const HasteHealthContext = createContext<HasteHealthContextState>({
  ...InitialContext,
});

export default HasteHealthContext;
