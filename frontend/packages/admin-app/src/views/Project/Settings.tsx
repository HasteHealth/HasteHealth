import {
  CheckCircleIcon,
  ClipboardDocumentIcon,
  ExclamationTriangleIcon,
  LockClosedIcon,
  ShieldCheckIcon,
  UserCircleIcon,
  XMarkIcon,
} from "@heroicons/react/24/outline";
import { useAtomValue } from "jotai";
import React, { useCallback, useEffect, useMemo, useState } from "react";

import {
  Button,
  Input,
  Loading,
  Table,
  Toaster,
  useHasteHealth,
} from "@haste-health/components";
import {
  Attachment,
  OperationOutcome,
  base64Binary,
  code,
  id,
} from "@haste-health/fhir-types/r4/types";
import { R4 } from "@haste-health/fhir-types/versions";
import {
  HasteHealthDeleteRefreshToken,
  HasteHealthDeleteScope,
  HasteHealthListRefreshTokens,
  HasteHealthListScopes,
  HasteHealthTenantBranding,
  HasteHealthTenantCustomization,
} from "@haste-health/generated-ops/lib/r4/ops";
import { IDTokenPayload } from "@haste-health/jwt/types";

import { VITE_FHIR_BASE_URL } from "../../config";
import { getClient } from "../../db/client";
import { getEndpointMetadata } from "../../db/endpointMeta";
import { getErrorMessage } from "../../utilities";

type SettingsProps = {
  user?: IDTokenPayload<string>;
};

type ScopeRecord = NonNullable<HasteHealthListScopes.Output["scopes"]>[number];
type RefreshTokenRecord = NonNullable<
  HasteHealthListRefreshTokens.Output["refresh-tokens"]
>[number];

function copyToClipboard(value?: string) {
  if (!value) {
    return;
  }

  navigator.clipboard
    .writeText(value)
    .then(() => {
      Toaster.success("Copied to clipboard");
    })
    .catch(() => {
      Toaster.error("Failed to copy value");
    });
}

function MetricCard({
  title,
  value,
  detail,
  icon,
}: Readonly<{
  title: string;
  value: string;
  detail: string;
  icon: React.ReactNode;
}>) {
  return (
    <div className="rounded-lg border border-slate-200 bg-white p-4 shadow-sm">
      <div className="flex items-center justify-between text-slate-500">
        <span className="text-sm font-medium">{title}</span>
        <span className="h-5 w-5">{icon}</span>
      </div>
      <div className="mt-2 text-2xl font-semibold text-slate-900">{value}</div>
      <div className="mt-1 text-xs text-slate-500">{detail}</div>
    </div>
  );
}

function SectionCard({
  title,
  description,
  children,
  action,
}: Readonly<{
  title: string;
  description: string;
  children: React.ReactNode;
  action?: React.ReactNode;
}>) {
  return (
    <section className="rounded-lg border border-slate-200 bg-white p-5 shadow-sm">
      <div className="mb-4 flex items-start justify-between gap-3">
        <div>
          <h2 className="text-lg font-semibold text-slate-900">{title}</h2>
          <p className="mt-1 text-sm text-slate-500">{description}</p>
        </div>
        {action}
      </div>
      {children}
    </section>
  );
}

function CopyField({
  label,
  value,
}: Readonly<{
  label: string;
  value?: string;
}>) {
  return (
    <div className="space-y-1 w-full ">
      <label className="text-xs font-medium uppercase tracking-wide text-slate-500">
        {label}
      </label>
      <button
        className="flex  w-full items-center justify-between gap-3 rounded-md border border-slate-200 bg-slate-50 px-3 py-2 text-left hover:bg-slate-100"
        onClick={() => {
          copyToClipboard(value);
        }}
      >
        <span className="block truncate text-sm text-slate-800">
          {value ?? "Not available"}
        </span>
        <ClipboardDocumentIcon className="h-4 w-4 shrink-0 text-slate-500" />
      </button>
    </div>
  );
}

function readFileAsBase64(file: File): Promise<string> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.readAsDataURL(file);

    reader.onload = () => {
      const data = reader.result;
      if (typeof data === "string") {
        resolve(data.split(",")[1]);
      } else {
        reject(new Error("FileReader result was not a string"));
      }
    };
    reader.onerror = () => reject(new Error("Failed to read file"));
  });
}

function TenantBranding({ isOwner }: Readonly<{ isOwner: boolean }>) {
  const client = useAtomValue(getClient);

  const [tenantName, setTenantName] = useState("");
  const [logo, setLogo] = useState<Attachment>();
  const [logoCleared, setLogoCleared] = useState(false);
  const [loadingBranding, setLoadingBranding] = useState(true);
  const [logoFile, setLogoFile] = useState<File>();
  const [logoPreviewUrl, setLogoPreviewUrl] = useState<string>();
  const [logoIssues, setLogoIssues] = useState<string[]>([]);
  const [savingBranding, setSavingBranding] = useState(false);
  const [logoInputKey, setLogoInputKey] = useState(0);

  const currentLogoDataUrl =
    logo?.contentType && logo?.data
      ? `data:${logo.contentType};base64,${logo.data}`
      : undefined;

  const displayedLogoUrl =
    logoPreviewUrl ?? (logoCleared ? undefined : currentLogoDataUrl);

  let logoStatusLabel = "Current logo";
  if (logoPreviewUrl) {
    logoStatusLabel = "New logo (unsaved)";
  } else if (logoCleared) {
    logoStatusLabel = "Logo will be cleared";
  }

  const loadBranding = useCallback(() => {
    setLoadingBranding(true);

    client
      .invoke_system(HasteHealthTenantBranding.Op, {}, R4, {})
      .then((res) => {
        setTenantName(res.name ?? "");
        setLogo(res.logo);
        setLogoCleared(false);
      })
      .catch(() => {
        Toaster.error("Failed to load tenant branding.");
      })
      .finally(() => {
        setLoadingBranding(false);
      });
  }, [client]);

  useEffect(() => {
    loadBranding();
  }, [loadBranding]);

  const selectLogoFile = useCallback((file: File | undefined) => {
    setLogoIssues([]);
    setLogoPreviewUrl((previous) => {
      if (previous) {
        URL.revokeObjectURL(previous);
      }
      return undefined;
    });

    if (!file) {
      setLogoFile(undefined);
      return;
    }

    if (file.type !== "image/png") {
      setLogoIssues(["Logo must be a PNG image."]);
      setLogoFile(undefined);
      return;
    }

    setLogoFile(file);
    setLogoPreviewUrl(URL.createObjectURL(file));
    setLogoCleared(false);
  }, []);

  const clearLogo = useCallback(() => {
    selectLogoFile(undefined);
    setLogoCleared(true);
    setLogoInputKey((key) => key + 1);
  }, [selectLogoFile]);

  const clearName = useCallback(() => {
    setTenantName("");
  }, []);

  const saveBranding = useCallback(() => {
    setSavingBranding(true);

    const buildInput =
      async (): Promise<HasteHealthTenantCustomization.Input> => {
        const input: HasteHealthTenantCustomization.Input = {};

        // Omitting `name`/`logo` clears them server-side, so an unset value must be
        // resent to preserve it rather than simply left out of the request.
        if (tenantName.trim()) {
          input.name = tenantName.trim();
        }

        if (logoFile) {
          input.logo = {
            contentType: "image/png" as code,
            data: (await readFileAsBase64(logoFile)) as base64Binary,
          };
        } else if (!logoCleared && logo?.contentType && logo?.data) {
          input.logo = { contentType: logo.contentType, data: logo.data };
        }

        return input;
      };

    const savePromise = buildInput().then((input) =>
      client.invoke_system(HasteHealthTenantCustomization.Op, {}, R4, input),
    );

    Toaster.promise(savePromise, {
      loading: "Saving tenant branding",
      success: () => {
        selectLogoFile(undefined);
        setLogoInputKey((key) => key + 1);
        loadBranding();
        return "Tenant branding updated";
      },
      error: (error) => getErrorMessage(error),
    }).finally(() => {
      setSavingBranding(false);
    });
  }, [
    client,
    tenantName,
    logoFile,
    logoCleared,
    logo,
    selectLogoFile,
    loadBranding,
  ]);

  return (
    <SectionCard
      title="Tenant Branding"
      description={
        isOwner
          ? "Set the display name and logo shown on login and admin pages for this tenant."
          : "Display name and logo shown on login and admin pages for this tenant."
      }
    >
      {loadingBranding ? (
        <div className="flex items-center gap-2 py-4 text-sm text-slate-500">
          <Loading />
          <span>Loading tenant branding...</span>
        </div>
      ) : (
        <div className="flex flex-col gap-4 sm:flex-row sm:items-start">
          <div className="flex flex-col items-center gap-2">
            <div className="relative h-24 w-24">
              <div className="flex h-24 w-24 items-center justify-center overflow-hidden rounded-md border border-slate-200 bg-slate-50">
                {displayedLogoUrl && (
                  <img
                    src={displayedLogoUrl}
                    alt="Tenant logo"
                    className="h-full w-full object-contain"
                  />
                )}
              </div>
              {isOwner && displayedLogoUrl && (
                <button
                  type="button"
                  aria-label="Clear logo"
                  className="absolute -top-2 -right-2 flex h-6 w-6 items-center justify-center rounded-full border border-slate-200 bg-white text-slate-500 shadow-sm hover:text-slate-700"
                  onClick={clearLogo}
                >
                  <XMarkIcon className="h-4 w-4" />
                </button>
              )}
            </div>
            <span className="text-xs text-slate-500">{logoStatusLabel}</span>
          </div>

          {isOwner ? (
            <div className="flex flex-1 flex-col gap-3">
              <Input
                label="Display Name"
                placeholder="Leave blank to clear the display name"
                value={tenantName}
                onChange={(e) => setTenantName(e.target.value)}
                icon={
                  tenantName && (
                    <button
                      type="button"
                      aria-label="Clear display name"
                      className="text-slate-400 hover:text-slate-600"
                      onClick={clearName}
                    >
                      <XMarkIcon className="h-4 w-4" />
                    </button>
                  )
                }
              />

              <Input
                key={logoInputKey}
                label="Logo"
                type="file"
                accept="image/png"
                issues={logoIssues}
                onChange={(e) => selectLogoFile(e.target.files?.[0])}
              />
              <p className="text-xs text-slate-500">
                Logo must be a square PNG image; it will be resized to
                150x150.
              </p>

              <div className="flex justify-end">
                <Button
                  buttonSize="medium"
                  disabled={savingBranding}
                  onClick={saveBranding}
                >
                  {savingBranding ? "Saving..." : "Save Branding"}
                </Button>
              </div>
            </div>
          ) : (
            <div className="flex flex-1 flex-col gap-1">
              <span className="text-sm font-medium text-slate-700">
                {tenantName || "No display name set"}
              </span>
              <p className="text-xs text-slate-500">
                Only tenant owners can update branding.
              </p>
            </div>
          )}
        </div>
      )}
    </SectionCard>
  );
}

function SettingsContent({ user }: Readonly<SettingsProps>) {
  const client = useAtomValue(getClient);
  const endpointMetadata = useAtomValue(getEndpointMetadata);

  const [scopes, setScopes] = useState<HasteHealthListScopes.Output["scopes"]>(
    [],
  );
  const [refreshTokens, setRefreshTokens] = useState<
    HasteHealthListRefreshTokens.Output["refresh-tokens"]
  >([]);
  const [loadingScopes, setLoadingScopes] = useState(true);
  const [loadingRefreshTokens, setLoadingRefreshTokens] = useState(true);

  const loadScopes = useCallback(() => {
    setLoadingScopes(true);

    client
      .invoke_system(HasteHealthListScopes.Op, {}, R4, {})
      .then((res) => {
        setScopes(res.scopes ?? []);
      })
      .catch(() => {
        Toaster.error("Failed to load authorized applications.");
      })
      .finally(() => {
        setLoadingScopes(false);
      });
  }, [client]);

  const loadRefreshTokens = useCallback(() => {
    setLoadingRefreshTokens(true);

    client
      .invoke_system(HasteHealthListRefreshTokens.Op, {}, R4, {})
      .then((res) => {
        setRefreshTokens(res["refresh-tokens"] ?? []);
      })
      .catch(() => {
        Toaster.error("Failed to load active refresh tokens.");
      })
      .finally(() => {
        setLoadingRefreshTokens(false);
      });
  }, [client]);

  useEffect(() => {
    loadScopes();
    loadRefreshTokens();
  }, [loadRefreshTokens, loadScopes]);

  const revokeScope = useCallback(
    (clientId: id) => {
      const deletePromise = client
        .invoke_system(HasteHealthDeleteScope.Op, {}, R4, {
          client_id: clientId,
        })
        .then((res) => {
          if (res.issue[0]?.code !== "informational") {
            throw new Error("Failed to revoke authorized app");
          }
          return res;
        });

      Toaster.promise(deletePromise, {
        loading: "Revoking authorization",
        success: (res) => {
          loadScopes();
          const outcome = res as OperationOutcome;
          return outcome.issue[0].diagnostics ?? "Authorization revoked";
        },
        error: () => "Failed to revoke authorization.",
      });
    },
    [client, loadScopes],
  );

  const revokeRefreshToken = useCallback(
    (clientId: id, userAgent: string) => {
      const deletePromise = client
        .invoke_system(HasteHealthDeleteRefreshToken.Op, {}, R4, {
          client_id: clientId,
          user_agent: userAgent,
        })
        .then((res) => {
          if (res.issue[0]?.code !== "informational") {
            throw new Error("Failed to revoke refresh token");
          }
          return res;
        });

      Toaster.promise(deletePromise, {
        loading: "Revoking session",
        success: (res) => {
          loadRefreshTokens();
          const outcome = res as OperationOutcome;
          return outcome.issue[0].diagnostics ?? "Session revoked";
        },
        error: () => "Failed to revoke session.",
      });
    },
    [client, loadRefreshTokens],
  );

  const oidcConfigured = Boolean(
    endpointMetadata?.["oidc-discovery-url"] &&
    endpointMetadata?.["oidc-token-endpoint"] &&
    endpointMetadata?.["oidc-authorize-endpoint"],
  );

  const tenant = user?.["https://haste.health/tenant"];
  const mfaAdminUrl = tenant
    ? `${VITE_FHIR_BASE_URL}/w/${tenant}/mfa/admin`
    : undefined;

  const isOwner = user?.["https://haste.health/user_role"] === "owner";

  const endpointCount = useMemo(() => {
    let configured = 0;

    if (endpointMetadata?.["fhir-r4-base-url"]) {
      configured += 1;
    }
    if (endpointMetadata?.["fhir-r4-capabilities-url"]) {
      configured += 1;
    }
    if (endpointMetadata?.["oidc-discovery-url"]) {
      configured += 1;
    }
    if (endpointMetadata?.["oidc-token-endpoint"]) {
      configured += 1;
    }
    if (endpointMetadata?.["oidc-authorize-endpoint"]) {
      configured += 1;
    }
    if (endpointMetadata?.["mcp-endpoint"]) {
      configured += 1;
    }

    return configured;
  }, [endpointMetadata]);

  return (
    <div className="flex w-full flex-col gap-6 overflow-y-auto">
      <header className="rounded-lg border border-slate-200 bg-white p-5 shadow-sm">
        <div className="space-y-1">
          <h1 className="text-2xl font-semibold text-slate-900">Settings</h1>
          <p className="text-sm text-slate-500">
            Manage identity, endpoints, app authorizations, and active sessions.
          </p>
        </div>
      </header>

      <TenantBranding isOwner={isOwner} />

      <section className="grid gap-4 sm:grid-cols-2 xl:grid-cols-4">
        <MetricCard
          title="Authorized Apps"
          value={(scopes?.length ?? 0).toLocaleString()}
          detail="Applications with granted scopes"
          icon={<ShieldCheckIcon className="h-5 w-5" />}
        />
        <MetricCard
          title="Active Sessions"
          value={(refreshTokens?.length ?? 0).toLocaleString()}
          detail="Refresh tokens currently active"
          icon={<UserCircleIcon className="h-5 w-5" />}
        />
        <MetricCard
          title="Endpoints Configured"
          value={endpointCount.toLocaleString()}
          detail="FHIR and OIDC URLs set"
          icon={<CheckCircleIcon className="h-5 w-5" />}
        />
        <MetricCard
          title="OIDC Health"
          value={oidcConfigured ? "Ready" : "Incomplete"}
          detail="Discovery, token, and authorize endpoints"
          icon={
            oidcConfigured ? (
              <CheckCircleIcon className="h-5 w-5" />
            ) : (
              <ExclamationTriangleIcon className="h-5 w-5" />
            )
          }
        />
      </section>

      <section className="grid gap-4 xl:grid-cols-4">
        <SectionCard
          title="User and Project Context"
          description="Values from your current authentication token. Click any value to copy."
        >
          <div className="flex flex-col space-y-2">
            <CopyField label="User Subject" value={user?.sub} />
            <CopyField
              label="Role"
              value={user?.["https://haste.health/user_role"]}
            />
            <CopyField
              label="Tenant"
              value={user?.["https://haste.health/tenant"]}
            />
            <CopyField
              label="Project"
              value={user?.["https://haste.health/project"]}
            />
            <CopyField label="Audience" value={user?.aud} />
            <CopyField label="Scope" value={user?.scope} />
          </div>
        </SectionCard>

        <SectionCard
          title="Multi-Factor Authentication"
          description="Manage the authenticator apps enrolled for your account."
        >
          <a
            href={mfaAdminUrl}
            target="_blank"
            rel="noreferrer"
            aria-disabled={!mfaAdminUrl}
            className="inline-flex items-center gap-2 rounded-md border border-slate-300 px-3 py-1.5 text-sm font-medium text-slate-700 hover:bg-slate-50 aria-disabled:pointer-events-none aria-disabled:opacity-50"
          >
            <LockClosedIcon className="h-4 w-4" />
            Manage MFA
          </a>
        </SectionCard>

        <SectionCard
          title="FHIR and OIDC Endpoints"
          description="Core endpoint URLs used by clients and integrations."
        >
          <div className="flex flex-col space-y-2">
            <CopyField
              label="FHIR R4 Base URL"
              value={endpointMetadata?.["fhir-r4-base-url"]}
            />
            <CopyField
              label="FHIR Capabilities URL"
              value={endpointMetadata?.["fhir-r4-capabilities-url"]}
            />
            <CopyField
              label="OIDC Discovery URL"
              value={endpointMetadata?.["oidc-discovery-url"]}
            />
            <CopyField
              label="OIDC Token Endpoint"
              value={endpointMetadata?.["oidc-token-endpoint"]}
            />
            <CopyField
              label="OIDC Authorization Endpoint"
              value={endpointMetadata?.["oidc-authorize-endpoint"]}
            />
            <CopyField
              label="MCP URL"
              value={endpointMetadata?.["mcp-endpoint"]}
            />
          </div>
        </SectionCard>

        <SectionCard
          title="Authorized Applications"
          description="Apps currently authorized to access this project."
          action={
            <button
              className="rounded-md border border-slate-300 px-3 py-1.5 text-xs font-medium text-slate-700 hover:bg-slate-50"
              onClick={loadScopes}
            >
              Refresh
            </button>
          }
        >
          {loadingScopes ? (
            <div className="flex items-center gap-2 py-4 text-sm text-slate-500">
              <Loading />
              <span>Loading authorized apps...</span>
            </div>
          ) : (
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
                    const scope = data[0] as ScopeRecord;
                    return (
                      <button
                        className="font-semibold text-red-600 hover:text-red-700"
                        onClick={() => {
                          revokeScope(scope.client_id);
                        }}
                      >
                        Revoke
                      </button>
                    );
                  },
                },
              ]}
              data={scopes ?? []}
            />
          )}
        </SectionCard>
      </section>

      <SectionCard
        title="Active Sessions"
        description="Refresh tokens currently issued to clients. Revoke sessions that should no longer have access."
        action={
          <button
            className="rounded-md border border-slate-300 px-3 py-1.5 text-xs font-medium text-slate-700 hover:bg-slate-50"
            onClick={loadRefreshTokens}
          >
            Refresh
          </button>
        }
      >
        {loadingRefreshTokens ? (
          <div className="flex items-center gap-2 py-4 text-sm text-slate-500">
            <Loading />
            <span>Loading active sessions...</span>
          </div>
        ) : (
          <Table
            columns={[
              {
                id: "client_id",
                content: "Client ID",
                selectorType: "fhirpath",
                selector: "$this.client_id",
              },
              {
                id: "user_agent",
                content: "User Agent",
                selectorType: "fhirpath",
                selector: "$this.user_agent",
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
                  const refreshToken = data[0] as RefreshTokenRecord;
                  return (
                    <button
                      className="font-semibold text-red-600 hover:text-red-700"
                      onClick={() => {
                        revokeRefreshToken(
                          refreshToken.client_id,
                          refreshToken.user_agent,
                        );
                      }}
                    >
                      Revoke
                    </button>
                  );
                },
              },
            ]}
            data={refreshTokens ?? []}
          />
        )}
      </SectionCard>
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
          <div className="mt-1">Loading...</div>
        </div>
      }
    >
      <SettingsContent user={hasteHealth.user} />
    </React.Suspense>
  );
}
