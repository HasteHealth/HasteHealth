import { useAtomValue } from "jotai";
import React from "react";

import { deriveHasteHealthVersionedURL } from "@haste-health/client/http";
import {
  Loading,
  Table,
  Toaster,
  useHasteHealth,
} from "@haste-health/components";
import { OperationOutcome, id } from "@haste-health/fhir-types/r4/types";
import { R4 } from "@haste-health/fhir-types/versions";
import {
  HasteHealthDeleteRefreshToken,
  HasteHealthDeleteScope,
  HasteHealthListRefreshTokens,
  HasteHealthListScopes,
} from "@haste-health/generated-ops/lib/r4/ops";
import { IDTokenPayload } from "@haste-health/jwt/types";

import { getClient } from "../../db/client";

function copytoClipboard(token: string | undefined) {
  if (token) {
    navigator.clipboard.writeText(token);
    Toaster.success("Value copied");
  }
}

interface SettingProps {
  user?: IDTokenPayload<string>;
}

function Copyable({
  value,
  label,
}: Readonly<{ value?: string; label?: string }>) {
  return (
    <div>
      <label className="text-sm text-slate-800">{label}</label>
      <div className="flex flex-1 flex-row items-center">
        <span
          className="text-sm block w-full whitespace-nowrap border px-2 py-1 cursor-pointer hover:bg-orange-100 truncate"
          onClick={(e) => {
            e.preventDefault();
            copytoClipboard(value);
          }}
        >
          {value}
        </span>
      </div>
    </div>
  );
}

function Scopes() {
  const hasteHealth = useHasteHealth();
  const client = useAtomValue(getClient);
  const [scopes, setScopes] = React.useState<
    HasteHealthListScopes.Output["scopes"]
  >([]);
  const fetchScopes = React.useMemo(() => {
    return () => {
      client.invoke_system(HasteHealthListScopes.Op, {}, R4, {}).then((res) => {
        setScopes(res.scopes);
      });
    };
  }, [hasteHealth]);
  React.useEffect(() => {
    fetchScopes();
  }, []);
  const deleteScopes = React.useMemo(() => {
    return (client_id: id) => {
      const deletePromise = client
        .invoke_system(HasteHealthDeleteScope.Op, {}, R4, {
          client_id,
        })
        .then((res) => {
          if (res.issue[0]?.code !== "informational") {
            throw new Error("Failed to delete");
          }
          return res;
        });
      Toaster.promise(deletePromise, {
        loading: "Deleting Resource",
        success: (res) => {
          const result = res as OperationOutcome;
          fetchScopes();
          return result.issue[0].diagnostics ?? "Deleted";
        },
        error: (error) => {
          console.error(error);
          return "Failed to delete authorization.";
        },
      });
    };
  }, [hasteHealth]);

  return (
    <div className="space-y-2">
      <h2 className="text-lg font-semibold underline">Authorized Apps</h2>
      <div className="flex flex-col space-y-2">
        <Table
          columns={[
            {
              id: "client_id",
              content: "Client ID",
              selectorType: "fhirpath",
              selector: "$this.client_id",
            },
            {
              id: "scopes",
              content: "Scopes",
              selectorType: "fhirpath",
              selector: "$this.scopes",
            },
            {
              id: "created_at",
              content: "Authorized At",
              selectorType: "fhirpath",
              selector: "$this.created_at",
            },
            {
              id: "actions",
              content: "Actions",
              selectorType: "fhirpath",
              selector: "$this",
              renderer: (data) => {
                const scope = data[0] as NonNullable<
                  HasteHealthListScopes.Output["scopes"]
                >[number];

                return (
                  <div
                    onClick={() => {
                      deleteScopes(scope.client_id);
                    }}
                    className="cursor-pointer font-semibold text-red-600 hover:text-red-700"
                  >
                    Revoke
                  </div>
                );
              },
            },
          ]}
          data={scopes ?? []}
        />
      </div>
    </div>
  );
}

function RefreshTokens() {
  const hasteHealth = useHasteHealth();
  const client = useAtomValue(getClient);
  const [refreshTokens, setRefreshTokens] = React.useState<
    HasteHealthListRefreshTokens.Output["refresh-tokens"]
  >([]);
  const fetchRefreshTokens = React.useMemo(() => {
    return () => {
      client
        .invoke_system(HasteHealthListRefreshTokens.Op, {}, R4, {})
        .then((res) => {
          setRefreshTokens(res["refresh-tokens"]);
        });
    };
  }, [hasteHealth]);
  React.useEffect(() => {
    fetchRefreshTokens();
  }, []);
  const deleteRefreshToken = React.useMemo(() => {
    return (id: id) => {
      const deletePromise = client
        .invoke_system(HasteHealthDeleteRefreshToken.Op, {}, R4, {
          id,
        })
        .then((res) => {
          if (res.issue[0]?.code !== "informational") {
            throw new Error("Failed to delete");
          }
          return res;
        });
      Toaster.promise(deletePromise, {
        loading: "Deleting Resource",
        success: (res) => {
          const result = res as OperationOutcome;
          fetchRefreshTokens();
          return result.issue[0].diagnostics ?? "Deleted";
        },
        error: (error) => {
          console.error(error);
          return "Failed to delete authorization.";
        },
      });
    };
  }, [hasteHealth]);

  return (
    <div className="space-y-2">
      <h2 className="text-lg font-semibold underline">Active Refresh Tokens</h2>
      <div className="flex flex-col space-y-2">
        <Table
          columns={[
            {
              id: "id",
              content: "ID",
              selectorType: "fhirpath",
              selector: "$this.id",
            },
            {
              id: "client_id",
              content: "Client ID",
              selectorType: "fhirpath",
              selector: "$this.client_id",
            },
            {
              id: "created_at",
              content: "Authorized At",
              selectorType: "fhirpath",
              selector: "$this.created_at",
            },
            {
              id: "actions",
              content: "Actions",
              selectorType: "fhirpath",
              selector: "$this",
              renderer: (data) => {
                const refreshToken = data[0] as NonNullable<
                  HasteHealthListRefreshTokens.Output["refresh-tokens"]
                >[number];

                return (
                  <div
                    onClick={() => {
                      deleteRefreshToken(refreshToken.id);
                    }}
                    className="cursor-pointer font-semibold text-red-600 hover:text-red-700"
                  >
                    Revoke
                  </div>
                );
              },
            },
          ]}
          data={refreshTokens ?? []}
        />
      </div>
    </div>
  );
}

function Card({ children }: Readonly<{ children: React.ReactNode }>) {
  return (
    <div className="p-6 bg-white border border-slate-200 rounded-lg shadow-md space-y-1">
      {children}
    </div>
  );
}

function UserData({ user }: Readonly<SettingProps>) {
  return (
    <div className="space-y-2">
      <h2 className="text-lg font-semibold underline">User Information</h2>
      <div className="flex flex-col space-y-2">
        <div>
          <Copyable label="Sub" value={user?.sub} />
        </div>
        <div>
          <Copyable
            label="Role"
            value={user?.["https://haste-health.app/user_role"]}
          />
        </div>
        <div>
          <Copyable
            label="Tenant"
            value={user?.["https://haste-health.app/tenant"]}
          />
        </div>
        <div>
          <Copyable label="Aud" value={user?.["aud"]} />
        </div>
        <div>
          <Copyable label="Scope" value={user?.["scope"]} />
        </div>
      </div>
    </div>
  );
}

function FHIRSettings() {
  const hasteHealth = useHasteHealth();
  return (
    <div className="space-y-2">
      <h3 className="text-md font-semibold underline">FHIR</h3>
      <div className=" space-y-2">
        <div className="flex flex-col">
          <Copyable
            label="R4 Root"
            value={
              hasteHealth.rootURL
                ? deriveHasteHealthVersionedURL(hasteHealth.rootURL, R4)
                : ""
            }
          />
        </div>
        <div className="flex flex-col">
          <Copyable
            label="R4 Metdata"
            value={
              hasteHealth.rootURL
                ? deriveHasteHealthVersionedURL(hasteHealth.rootURL, R4) +
                  "/metadata"
                : ""
            }
          />
        </div>
      </div>
    </div>
  );
}

function OpenIDConnectSettings() {
  const hasteHealth = useHasteHealth();
  return (
    <div className="space-y-2">
      <h3 className="text-lg font-semibold underline">OpenID Connect</h3>
      <div className=" space-y-2">
        <div className="flex flex-col">
          <Copyable label="Discovery" value={hasteHealth.well_known_uri} />
        </div>
        <div className="flex flex-col">
          <Copyable
            label="Token"
            value={hasteHealth.well_known?.token_endpoint}
          />
        </div>
        <div className="flex flex-col">
          <Copyable
            label="Authorization"
            value={hasteHealth.well_known?.authorization_endpoint}
          />
        </div>
      </div>
    </div>
  );
}

function SettingDisplay({ user }: Readonly<SettingProps>) {
  return (
    <div className="flex flex-col flex-1 space-y-4 w-full">
      <h2 className="text-2xl font-semibold mb-0">Settings</h2>

      <div className="grid md:grid-cols-3 lg:grid-cols-3 sm:grid-cols-2 gap-4 grid-flow-row-dense auto-cols-max">
        <Card>
          <UserData user={user} />
        </Card>
        <Card>
          <FHIRSettings />
        </Card>
        <Card>
          <OpenIDConnectSettings />
        </Card>
        <Card>
          <Scopes />
        </Card>
        <Card>
          <RefreshTokens />
        </Card>
      </div>
    </div>
  );
}

export default function Settings() {
  const hasteHealth = useHasteHealth();
  return (
    <React.Suspense
      fallback={
        <div className="h-screen flex flex-1 justify-center items-center flex-col">
          <Loading />
          <div className="mt-1 ">Loading...</div>
        </div>
      }
    >
      <SettingDisplay user={hasteHealth.user} />
    </React.Suspense>
  );
}
