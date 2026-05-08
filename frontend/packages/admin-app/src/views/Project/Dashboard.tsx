import {
  ArrowPathIcon,
  CheckCircleIcon,
  CircleStackIcon,
  ClockIcon,
  ExclamationTriangleIcon,
  ServerStackIcon,
  ShieldCheckIcon,
} from "@heroicons/react/24/outline";
import { useAtomValue } from "jotai";
import React, { useCallback, useEffect, useMemo, useState } from "react";
import { generatePath, useNavigate } from "react-router-dom";

import { isResponseError } from "@haste-health/client/http";
import { Loading, Toaster } from "@haste-health/components";
import { R4 } from "@haste-health/fhir-types/versions";

import { getCapabilities } from "../../db/capabilities";
import { getClient } from "../../db/client";
import { getEndpointMetadata } from "../../db/endpointMeta";
import { Bundle, code, uri } from "@haste-health/fhir-types/r4/types";

type StatKey =
  | "patient"
  | "encounter"
  | "observation"
  | "condition"
  | "medication"
  | "medicationRequest"
  | "practitioner"
  | "careTeam"
  | "carePlan"
  | "claim"
  | "explanationOfBenefit"
  | "questionnaire"
  | "questionnaireResponse"
  | "operationDefinition"
  | "subscription"
  | "membership"
  | "accessPolicy"
  | "clientApplication"
  | "auditEvent";

type DashboardCounts = Partial<Record<StatKey, number>>;

type DashboardSnapshot = {
  counts: DashboardCounts;
  latestAuditTimestamp?: string;
  activeSubscriptionCount?: number;
  refreshedAt: Date;
};

const COUNT_QUERIES: ReadonlyArray<{ key: StatKey; resourceType: string }> = [
  { key: "patient", resourceType: "Patient" },
  { key: "encounter", resourceType: "Encounter" },
  { key: "observation", resourceType: "Observation" },
  { key: "condition", resourceType: "Condition" },
  { key: "medication", resourceType: "Medication" },
  { key: "medicationRequest", resourceType: "MedicationRequest" },
  { key: "practitioner", resourceType: "Practitioner" },
  { key: "careTeam", resourceType: "CareTeam" },
  { key: "carePlan", resourceType: "CarePlan" },
  { key: "claim", resourceType: "Claim" },
  { key: "explanationOfBenefit", resourceType: "ExplanationOfBenefit" },
  { key: "questionnaire", resourceType: "Questionnaire" },
  { key: "questionnaireResponse", resourceType: "QuestionnaireResponse" },
  { key: "operationDefinition", resourceType: "OperationDefinition" },
  { key: "subscription", resourceType: "Subscription" },
  { key: "membership", resourceType: "Membership" },
  { key: "accessPolicy", resourceType: "AccessPolicyV2" },
  { key: "clientApplication", resourceType: "ClientApplication" },
  { key: "auditEvent", resourceType: "AuditEvent" },
];

const KPI_CARD_CLASS =
  "rounded-lg border border-slate-200 bg-white p-4 shadow-sm space-y-2";

function asBundle(resource: unknown): Bundle | undefined {
  if (!resource || typeof resource !== "object") {
    return undefined;
  }

  return resource as Bundle;
}

function readBundleTotal(resource: unknown): number | undefined {
  const bundle = asBundle(resource);

  return typeof bundle?.total === "number" ? bundle.total : undefined;
}

function readLatestTimestamp(resource: unknown): string | undefined {
  const bundle = asBundle(resource);
  const latestResource = bundle?.entry?.[0]?.resource as
    | { meta?: { lastUpdated?: string } }
    | undefined;

  return latestResource?.meta?.lastUpdated;
}

function toNumber(value: number | undefined): number {
  return typeof value === "number" ? value : 0;
}

function formatRelativeTime(timestamp?: string): string {
  if (!timestamp) {
    return "No events found";
  }

  const parsed = new Date(timestamp);
  if (Number.isNaN(parsed.getTime())) {
    return "Unknown";
  }

  const now = Date.now();
  const minutes = Math.max(0, Math.floor((now - parsed.getTime()) / 60000));
  if (minutes < 1) {
    return "Just now";
  }
  if (minutes < 60) {
    return `${minutes}m ago`;
  }

  const hours = Math.floor(minutes / 60);
  if (hours < 24) {
    return `${hours}h ago`;
  }

  const days = Math.floor(hours / 24);
  return `${days}d ago`;
}

function HealthBadge({ ok, label }: Readonly<{ ok: boolean; label: string }>) {
  return (
    <span
      className={`inline-flex items-center rounded-full px-2.5 py-1 text-xs font-medium ${
        ok ? "bg-emerald-50 text-emerald-700" : "bg-amber-50 text-amber-700"
      }`}
    >
      {label}
    </span>
  );
}

function SummaryStat({
  label,
  value,
  help,
  icon,
}: Readonly<{
  label: string;
  value: string;
  help: string;
  icon: React.ReactNode;
}>) {
  return (
    <div className={KPI_CARD_CLASS}>
      <div className="flex items-center justify-between text-slate-500">
        <span className="text-sm font-medium">{label}</span>
        <span className="h-5 w-5">{icon}</span>
      </div>
      <div className="text-2xl font-semibold text-slate-900 tabular-nums">
        {value}
      </div>
      <div className="text-xs text-slate-500">{help}</div>
    </div>
  );
}

function ResourceCategoryCard({
  title,
  description,
  accent,
  resources,
}: Readonly<{
  title: string;
  description: string;
  accent: "blue" | "emerald" | "amber" | "violet" | "rose";
  resources: Array<{ resourceType: string; count?: number }>;
}>) {
  const navigate = useNavigate();

  const accentClasses = {
    blue: "border-l-blue-500 bg-blue-50/40",
    emerald: "border-l-emerald-500 bg-emerald-50/40",
    amber: "border-l-amber-500 bg-amber-50/40",
    violet: "border-l-violet-500 bg-violet-50/40",
    rose: "border-l-rose-500 bg-rose-50/40",
  }[accent];

  const total = resources.reduce((sum, item) => sum + toNumber(item.count), 0);

  return (
    <article
      className={`rounded-lg border border-slate-200 border-l-4 ${accentClasses} p-4 shadow-sm`}
    >
      <div className="mb-3 flex items-start justify-between gap-3">
        <div>
          <h3 className="text-base font-semibold text-slate-900">{title}</h3>
          <p className="text-xs text-slate-500 mt-1">{description}</p>
        </div>
        <span className="rounded-full bg-white px-2.5 py-1 text-xs font-medium text-slate-600">
          {total.toLocaleString()} total
        </span>
      </div>

      <div className="divide-y divide-slate-200/80 rounded-md border border-slate-200 bg-white">
        {resources.map((item) => (
          <button
            key={item.resourceType}
            className="flex w-full items-center justify-between px-3 py-2 text-left text-sm hover:bg-slate-50"
            onClick={() => {
              navigate(
                generatePath("/resources/:resourceType", {
                  resourceType: item.resourceType,
                }),
              );
            }}
          >
            <span className="font-medium text-slate-700">
              {item.resourceType}
            </span>
            <span className="tabular-nums text-slate-900">
              {typeof item.count === "number"
                ? item.count.toLocaleString()
                : "-"}
            </span>
          </button>
        ))}
      </div>
    </article>
  );
}

function DashboardContent() {
  const navigate = useNavigate();
  const client = useAtomValue(getClient);
  const capabilityStatement = useAtomValue(getCapabilities);
  const endpointMetadata = useAtomValue(getEndpointMetadata);

  const [snapshot, setSnapshot] = useState<DashboardSnapshot | null>(null);
  const [loading, setLoading] = useState(true);

  const loadDashboard = useCallback(async () => {
    setLoading(true);

    try {
      const batchResponse = await client.batch({}, R4, {
        resourceType: "Bundle",
        type: "batch" as code,
        entry: [
          ...COUNT_QUERIES.map((query) => ({
            request: {
              method: "GET" as code,
              url: `${query.resourceType}?_total=estimate&_count=1` as uri,
            },
          })),
          {
            request: {
              method: "GET" as code,
              url: "AuditEvent?_sort=-_lastUpdated&_count=1" as uri,
            },
          },
          {
            request: {
              method: "GET" as code,
              url: "Subscription?status=active&_total=estimate&_count=1" as uri,
            },
          },
        ],
      });

      const counts: DashboardCounts = {};
      COUNT_QUERIES.forEach((query, index) => {
        counts[query.key] = readBundleTotal(
          batchResponse.entry?.[index]?.resource,
        );
      });

      const latestAuditIndex = COUNT_QUERIES.length;
      const activeSubscriptionIndex = COUNT_QUERIES.length + 1;

      setSnapshot({
        counts,
        latestAuditTimestamp: readLatestTimestamp(
          batchResponse.entry?.[latestAuditIndex]?.resource,
        ),
        activeSubscriptionCount: readBundleTotal(
          batchResponse.entry?.[activeSubscriptionIndex]?.resource,
        ),
        refreshedAt: new Date(),
      });
    } catch (error) {
      if (isResponseError(error)) {
        Toaster.error(
          error.response.body.issue?.[0]?.diagnostics ??
            "Failed to load server dashboard data.",
        );
      } else {
        Toaster.error("Failed to load server dashboard data.");
      }
    } finally {
      setLoading(false);
    }
  }, [client]);

  useEffect(() => {
    loadDashboard();
  }, [loadDashboard]);

  const counts = snapshot?.counts ?? {};
  const totalClinicalRecords =
    toNumber(counts.patient) +
    toNumber(counts.encounter) +
    toNumber(counts.observation) +
    toNumber(counts.condition) +
    toNumber(counts.medication) +
    toNumber(counts.medicationRequest);

  const securityResources =
    toNumber(counts.membership) +
    toNumber(counts.accessPolicy) +
    toNumber(counts.clientApplication);

  const insuranceRecords =
    toNumber(counts.claim) + toNumber(counts.explanationOfBenefit);

  const configurableArtifacts =
    toNumber(counts.operationDefinition) + toNumber(counts.subscription);

  const supportedResourceCount =
    capabilityStatement?.rest?.[0]?.resource?.length ?? 0;

  const supportedInteractionCount =
    capabilityStatement?.rest?.[0]?.resource?.reduce((sum, current) => {
      return sum + (current.interaction?.length ?? 0);
    }, 0) ?? 0;

  const oidcConfigured = Boolean(
    endpointMetadata?.["oidc-discovery-url"] &&
    endpointMetadata?.["oidc-token-endpoint"] &&
    endpointMetadata?.["oidc-authorize-endpoint"],
  );

  const criticalAlerts = useMemo(() => {
    const alerts: Array<{ severity: "healthy" | "warn"; message: string }> = [];

    if (toNumber(snapshot?.activeSubscriptionCount) === 0) {
      alerts.push({
        severity: "warn",
        message:
          "No active Subscriptions detected. Event-driven workflows may not trigger.",
      });
    } else {
      alerts.push({
        severity: "healthy",
        message: `${toNumber(snapshot?.activeSubscriptionCount).toLocaleString()} active Subscriptions configured.`,
      });
    }

    if (toNumber(counts.accessPolicy) === 0) {
      alerts.push({
        severity: "warn",
        message:
          "No AccessPolicyV2 resources found. Validate least-privilege access controls.",
      });
    } else {
      alerts.push({
        severity: "healthy",
        message: `${toNumber(counts.accessPolicy).toLocaleString()} access policies in place.`,
      });
    }

    if (snapshot?.latestAuditTimestamp) {
      alerts.push({
        severity: "healthy",
        message: `Latest AuditEvent recorded ${formatRelativeTime(snapshot.latestAuditTimestamp)}.`,
      });
    } else {
      alerts.push({
        severity: "warn",
        message:
          "No AuditEvent activity found. Verify auditing is enabled for sensitive operations.",
      });
    }

    if (oidcConfigured) {
      alerts.push({
        severity: "healthy",
        message:
          "OIDC discovery, authorization, and token endpoints are configured.",
      });
    } else {
      alerts.push({
        severity: "warn",
        message:
          "OIDC endpoints are incomplete. SMART/OIDC authorization may be unavailable.",
      });
    }

    return alerts;
  }, [counts.accessPolicy, oidcConfigured, snapshot]);

  return (
    <div className="flex w-full flex-col gap-6 overflow-y-scroll">
      <header className="rounded-lg border border-slate-200 bg-white p-5 shadow-sm">
        <div className="flex flex-col gap-4 lg:flex-row lg:items-center lg:justify-between">
          <div className="space-y-1">
            <h1 className="text-2xl font-semibold text-slate-900">
              FHIR Server Dashboard
            </h1>
            <p className="text-sm text-slate-500">
              Operational overview for the current project environment.
            </p>
          </div>
          <div className="flex items-center gap-3">
            <span className="text-xs text-slate-500">
              Last refreshed:{" "}
              {snapshot?.refreshedAt.toLocaleTimeString() ?? "-"}
            </span>
            <button
              className="inline-flex items-center gap-2 rounded-md border border-slate-300 bg-white px-3 py-2 text-sm font-medium text-slate-700 hover:bg-slate-50 disabled:cursor-not-allowed disabled:opacity-60"
              onClick={loadDashboard}
              disabled={loading}
            >
              <ArrowPathIcon
                className={`h-4 w-4 ${loading ? "animate-spin" : ""}`}
              />
              Refresh
            </button>
          </div>
        </div>
      </header>

      {loading && !snapshot ? (
        <div className="flex items-center gap-2 text-sm text-slate-500">
          <Loading />
          <span>Loading FHIR server health snapshot...</span>
        </div>
      ) : null}

      <section className="grid gap-4 sm:grid-cols-2 xl:grid-cols-4">
        <SummaryStat
          label="Clinical Records"
          value={totalClinicalRecords.toLocaleString()}
          help="Patient, Encounter, Observation, Condition, Medication"
          icon={<CircleStackIcon className="h-5 w-5" />}
        />
        <SummaryStat
          label="Security Objects"
          value={securityResources.toLocaleString()}
          help="Membership, AccessPolicyV2, ClientApplication"
          icon={<ShieldCheckIcon className="h-5 w-5" />}
        />
        <SummaryStat
          label="Supported Resources"
          value={supportedResourceCount.toLocaleString()}
          help={`${supportedInteractionCount.toLocaleString()} declared interactions`}
          icon={<ServerStackIcon className="h-5 w-5" />}
        />
        <SummaryStat
          label="Audit Activity"
          value={formatRelativeTime(snapshot?.latestAuditTimestamp)}
          help="Most recent AuditEvent"
          icon={<ClockIcon className="h-5 w-5" />}
        />
      </section>

      <section className="grid gap-4 xl:grid-cols-3">
        <article className="rounded-lg border border-slate-200 bg-white p-5 shadow-sm xl:col-span-2">
          <div className="mb-4 flex items-center justify-between">
            <h2 className="text-lg font-semibold text-slate-900">
              Critical Signals
            </h2>
            <HealthBadge
              ok={criticalAlerts.every((alert) => alert.severity === "healthy")}
              label={
                criticalAlerts.every((alert) => alert.severity === "healthy")
                  ? "Healthy"
                  : "Attention Needed"
              }
            />
          </div>
          <div className="space-y-3">
            {criticalAlerts.map((alert) => (
              <div
                key={alert.message}
                className={`flex items-start gap-3 rounded-md border px-3 py-2 ${
                  alert.severity === "healthy"
                    ? "border-emerald-200 bg-emerald-50"
                    : "border-amber-200 bg-amber-50"
                }`}
              >
                {alert.severity === "healthy" ? (
                  <CheckCircleIcon className="h-5 w-5 shrink-0 text-emerald-600" />
                ) : (
                  <ExclamationTriangleIcon className="h-5 w-5 shrink-0 text-amber-600" />
                )}
                <p className="text-sm text-slate-700">{alert.message}</p>
              </div>
            ))}
          </div>
        </article>

        <article className="rounded-lg border border-slate-200 bg-white p-5 shadow-sm">
          <h2 className="text-lg font-semibold text-slate-900">
            Server Endpoints
          </h2>
          <div className="mt-4 space-y-3 text-sm">
            <div>
              <div className="text-slate-500">FHIR R4 Base</div>
              <div className="mt-1 break-all text-slate-800">
                {endpointMetadata?.["fhir-r4-base-url"] ?? "Unavailable"}
              </div>
            </div>
            <div>
              <div className="text-slate-500">Capability Statement</div>
              <div className="mt-1 break-all text-slate-800">
                {endpointMetadata?.["fhir-r4-capabilities-url"] ??
                  "Unavailable"}
              </div>
            </div>
            <div className="pt-2">
              <HealthBadge
                ok={oidcConfigured}
                label={oidcConfigured ? "OIDC Configured" : "OIDC Incomplete"}
              />
            </div>
            <button
              className="mt-2 w-full rounded-md border border-slate-300 px-3 py-2 text-sm font-medium text-slate-700 hover:bg-slate-50"
              onClick={() => navigate(generatePath("/settings", {}))}
            >
              Open Security Settings
            </button>
          </div>
        </article>
      </section>

      <section className="grid gap-4 lg:grid-cols-2 xl:grid-cols-3">
        <ResourceCategoryCard
          title="Clinical Data"
          description="Core records used by clinical applications and timelines."
          accent="blue"
          resources={[
            { resourceType: "Patient", count: counts.patient },
            { resourceType: "Encounter", count: counts.encounter },
            { resourceType: "Observation", count: counts.observation },
            { resourceType: "Condition", count: counts.condition },
            { resourceType: "Medication", count: counts.medication },
            {
              resourceType: "MedicationRequest",
              count: counts.medicationRequest,
            },
          ]}
        />

        <ResourceCategoryCard
          title="Care Coordination"
          description="Staffing and care workflow artifacts for delivery teams."
          accent="emerald"
          resources={[
            { resourceType: "Practitioner", count: counts.practitioner },
            { resourceType: "CareTeam", count: counts.careTeam },
            { resourceType: "CarePlan", count: counts.carePlan },
            { resourceType: "Subscription", count: counts.subscription },
          ]}
        />

        <ResourceCategoryCard
          title="Payer and Financial"
          description="Insurance and claims-adjacent records."
          accent="amber"
          resources={[
            { resourceType: "Claim", count: counts.claim },
            {
              resourceType: "ExplanationOfBenefit",
              count: counts.explanationOfBenefit,
            },
            {
              resourceType: "OperationDefinition",
              count: counts.operationDefinition,
            },
          ]}
        />

        <ResourceCategoryCard
          title="Questionnaires"
          description="Forms and responses powering intake and workflows."
          accent="violet"
          resources={[
            { resourceType: "Questionnaire", count: counts.questionnaire },
            {
              resourceType: "QuestionnaireResponse",
              count: counts.questionnaireResponse,
            },
            { resourceType: "AuditEvent", count: counts.auditEvent },
          ]}
        />

        <ResourceCategoryCard
          title="Security and Access"
          description="Identity, policy, and app authorization controls."
          accent="rose"
          resources={[
            { resourceType: "Membership", count: counts.membership },
            { resourceType: "AccessPolicyV2", count: counts.accessPolicy },
            {
              resourceType: "ClientApplication",
              count: counts.clientApplication,
            },
          ]}
        />

        <article className="rounded-lg border border-slate-200 bg-white p-4 shadow-sm">
          <h3 className="text-base font-semibold text-slate-900">
            Quick Actions
          </h3>
          <p className="mt-1 text-xs text-slate-500">
            Jump directly to the workflows used during incident response and
            daily operations.
          </p>

          <div className="mt-4 grid gap-2">
            <button
              className="rounded-md border border-slate-300 px-3 py-2 text-left text-sm font-medium text-slate-700 hover:bg-slate-50"
              onClick={() =>
                navigate(
                  generatePath("/resources/:resourceType", {
                    resourceType: "AuditEvent",
                  }),
                )
              }
            >
              Review AuditEvent Stream
            </button>
            <button
              className="rounded-md border border-slate-300 px-3 py-2 text-left text-sm font-medium text-slate-700 hover:bg-slate-50"
              onClick={() =>
                navigate(
                  generatePath("/resources/:resourceType", {
                    resourceType: "Subscription",
                  }),
                )
              }
            >
              Manage Subscriptions
            </button>
            <button
              className="rounded-md border border-slate-300 px-3 py-2 text-left text-sm font-medium text-slate-700 hover:bg-slate-50"
              onClick={() =>
                navigate(
                  generatePath("/resources/:resourceType", {
                    resourceType: "AccessPolicyV2",
                  }),
                )
              }
            >
              Inspect Access Policies
            </button>
            <button
              className="rounded-md border border-slate-300 px-3 py-2 text-left text-sm font-medium text-slate-700 hover:bg-slate-50"
              onClick={() => navigate(generatePath("/resources", {}))}
            >
              View Supported Resource Catalog
            </button>
          </div>

          <div className="mt-4 rounded-md border border-brand-200 bg-brand-50 px-3 py-2 text-sm text-brand-900">
            Insurance records tracked: {insuranceRecords.toLocaleString()} |
            Configurable artifacts: {configurableArtifacts.toLocaleString()}
          </div>
        </article>
      </section>
    </div>
  );
}

export default function DashboardView() {
  return (
    <React.Suspense
      fallback={
        <div className="h-screen flex flex-1 justify-center items-center flex-col">
          <Loading />
          <div className="mt-1">Loading...</div>
        </div>
      }
    >
      <DashboardContent />
    </React.Suspense>
  );
}
