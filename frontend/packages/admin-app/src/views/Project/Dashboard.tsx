import { useAtomValue } from "jotai";
import React, { useEffect, useState } from "react";

import { isResponseError } from "@haste-health/client/lib/http";
import { Toaster } from "@haste-health/components";
import { Loading } from "@haste-health/components";
import { R4 } from "@haste-health/fhir-types/versions";

import { getClient } from "../../db/client";
import {
  Bundle,
  code,
  uri,
} from "@haste-health/fhir-types/lib/generated/r4/types";
import { generatePath, useNavigate } from "react-router-dom";

type Statistics = {
  patient?: number;
  observation?: number;
  encounter?: number;
  operationDefinition?: number;
  subscription?: number;
  questionnaire?: number;
  questionnaireResponse?: number;
  auditEvent?: number;
  membership?: number;
  accessPolicy?: number;
  clientApplication?: number;
  claim?: number;
  explanationOfBenefit?: number;
  medication?: number;
  medicationRequest?: number;
  condition?: number;
  careTeam?: number;
  carePlan?: number;
  practitioner?: number;
};

type CategoryAccentColor =
  | "blue"
  | "emerald"
  | "violet"
  | "amber"
  | "rose"
  | "orange"
  | "slate";

const ACCENT_CLASSES: Record<
  CategoryAccentColor,
  { border: string; badge: string; heading: string }
> = {
  blue: {
    border: "border-l-blue-500",
    badge: "bg-blue-50 text-blue-700",
    heading: "text-blue-700",
  },
  emerald: {
    border: "border-l-emerald-500",
    badge: "bg-emerald-50 text-emerald-700",
    heading: "text-emerald-700",
  },
  violet: {
    border: "border-l-violet-500",
    badge: "bg-violet-50 text-violet-700",
    heading: "text-violet-700",
  },
  amber: {
    border: "border-l-amber-500",
    badge: "bg-amber-50 text-amber-700",
    heading: "text-amber-700",
  },
  rose: {
    border: "border-l-rose-500",
    badge: "bg-rose-50 text-rose-700",
    heading: "text-rose-700",
  },
  orange: {
    border: "border-l-orange-500",
    badge: "bg-orange-50 text-orange-700",
    heading: "text-orange-700",
  },
  slate: {
    border: "border-l-slate-400",
    badge: "bg-slate-100 text-slate-600",
    heading: "text-slate-600",
  },
};

const StatCard = ({
  title,
  accent = "slate",
  stats,
}: {
  title: string;
  accent?: CategoryAccentColor;
  stats: Record<string, number | undefined>;
}) => {
  const colors = ACCENT_CLASSES[accent];
  const total = Object.values(stats).reduce(
    (sum, v) => (sum ?? 0) + (v ?? 0),
    0,
  );
  const navigate = useNavigate();

  return (
    <div
      className={`bg-white border border-slate-200 border-l-4 ${colors.border} rounded-lg shadow-sm flex flex-col`}
    >
      <div className="px-5 pt-4 pb-3 border-b border-slate-100 flex items-center justify-between">
        <h3
          className={`text-sm font-semibold uppercase tracking-wide ${colors.heading}`}
        >
          {title}
        </h3>
        <span
          className={`text-xs font-medium px-2 py-0.5 rounded-full ${colors.badge}`}
        >
          {total?.toLocaleString() ?? "—"} total
        </span>
      </div>
      <div className="divide-y divide-slate-100">
        {Object.entries(stats).map(([resourceType, count]) => (
          <button
            key={resourceType}
            className="w-full flex items-center justify-between px-5 py-2.5 text-left hover:bg-slate-50 transition-colors group"
            onClick={() =>
              navigate(
                generatePath("/resources/:resourceType", { resourceType }),
              )
            }
          >
            <span className="text-sm text-slate-700 group-hover:text-slate-900 font-medium">
              {resourceType}
            </span>
            <span className="text-sm font-semibold text-slate-900 tabular-nums">
              {typeof count === "number" ? (
                count.toLocaleString()
              ) : (
                <span className="text-slate-300">—</span>
              )}
            </span>
          </button>
        ))}
      </div>
    </div>
  );
};

const Dashboard = () => {
  const [stats, setStats] = useState<Statistics | null>(null);
  const [loading, setLoading] = useState(true);

  const client = useAtomValue(getClient);
  useEffect(() => {
    client
      .batch({}, R4, {
        resourceType: "Bundle",
        type: "batch" as code,
        entry: [
          {
            request: {
              method: "GET" as code,
              url: "Patient?_total=estimate&_count=1" as uri,
            },
          },
          {
            request: {
              method: "GET" as code,
              url: "Observation?_total=estimate&_count=1" as uri,
            },
          },
          {
            request: {
              method: "GET" as code,
              url: "Encounter?_total=estimate&_count=1" as uri,
            },
          },
          {
            request: {
              method: "GET" as code,
              url: "OperationDefinition?_total=estimate&_count=1" as uri,
            },
          },
          {
            request: {
              method: "GET" as code,
              url: "Subscription?_total=estimate&_count=1" as uri,
            },
          },
          {
            request: {
              method: "GET" as code,
              url: "Questionnaire?_total=estimate&_count=1" as uri,
            },
          },
          {
            request: {
              method: "GET" as code,
              url: "QuestionnaireResponse?_total=estimate&_count=1" as uri,
            },
          },
          {
            request: {
              method: "GET" as code,
              url: "AuditEvent?_total=estimate&_count=1" as uri,
            },
          },
          {
            request: {
              method: "GET" as code,
              url: "Membership?_total=estimate&_count=1" as uri,
            },
          },
          {
            request: {
              method: "GET" as code,
              url: "AccessPolicyV2?_total=estimate&_count=1" as uri,
            },
          },
          {
            request: {
              method: "GET" as code,
              url: "ClientApplication?_total=estimate&_count=1" as uri,
            },
          },
          {
            request: {
              method: "GET" as code,
              url: "Claim?_total=estimate&_count=1" as uri,
            },
          },
          {
            request: {
              method: "GET" as code,
              url: "ExplanationOfBenefit?_total=estimate&_count=1" as uri,
            },
          },

          {
            request: {
              method: "GET" as code,
              url: "Medication?_total=estimate&_count=1" as uri,
            },
          },
          {
            request: {
              method: "GET" as code,
              url: "MedicationRequest?_total=estimate&_count=1" as uri,
            },
          },
          {
            request: {
              method: "GET" as code,
              url: "Condition?_total=estimate&_count=1" as uri,
            },
          },
          {
            request: {
              method: "GET" as code,
              url: "CareTeam?_total=estimate&_count=1" as uri,
            },
          },
          {
            request: {
              method: "GET" as code,
              url: "CarePlan?_total=estimate&_count=1" as uri,
            },
          },
          {
            request: {
              method: "GET" as code,
              url: "Practitioner?_total=estimate&_count=1" as uri,
            },
          },
        ],
      })
      .then((bundle) => {
        setStats({
          patient: (bundle.entry?.[0]?.resource as Bundle)?.total,
          observation: (bundle.entry?.[1]?.resource as Bundle)?.total,
          encounter: (bundle.entry?.[2]?.resource as Bundle)?.total,
          operationDefinition: (bundle.entry?.[3]?.resource as Bundle)?.total,
          subscription: (bundle.entry?.[4]?.resource as Bundle)?.total,
          questionnaire: (bundle.entry?.[5]?.resource as Bundle)?.total,
          questionnaireResponse: (bundle.entry?.[6]?.resource as Bundle)?.total,
          auditEvent: (bundle.entry?.[7]?.resource as Bundle)?.total,
          membership: (bundle.entry?.[8]?.resource as Bundle)?.total,
          accessPolicy: (bundle.entry?.[9]?.resource as Bundle)?.total,
          clientApplication: (bundle.entry?.[10]?.resource as Bundle)?.total,
          claim: (bundle.entry?.[11]?.resource as Bundle)?.total,
          explanationOfBenefit: (bundle.entry?.[12]?.resource as Bundle)?.total,
          medication: (bundle.entry?.[13]?.resource as Bundle)?.total,
          medicationRequest: (bundle.entry?.[14]?.resource as Bundle)?.total,
          condition: (bundle.entry?.[15]?.resource as Bundle)?.total,
          careTeam: (bundle.entry?.[16]?.resource as Bundle)?.total,
          carePlan: (bundle.entry?.[17]?.resource as Bundle)?.total,
          practitioner: (bundle.entry?.[18]?.resource as Bundle)?.total,
        });
        setLoading(false);
      })
      .catch((e) => {
        setLoading(false);
        if (isResponseError(e))
          Toaster.error(
            e.response.body.issue?.[0]?.diagnostics ?? "Failed to fetch stats.",
          );
        else {
          Toaster.error("Failed to load usage stats.");
        }
      });
  }, [setStats]);

  return (
    <div className="flex flex-col gap-6 w-full">
      <div className="flex flex-col gap-1">
        <h1 className="text-2xl font-bold text-slate-800">Dashboard</h1>
        <p className="text-sm text-slate-500">
          Overview of FHIR resources stored in this project.
        </p>
      </div>

      {loading && (
        <div className="flex items-center gap-2 text-sm text-slate-500">
          <Loading />
          <span>Loading statistics…</span>
        </div>
      )}

      <div className="grid sm:grid-cols-2 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 2xl:grid-cols-5 gap-4">
        <StatCard
          title="Clinical"
          accent="blue"
          stats={{
            Patient: stats?.patient,
            Encounter: stats?.encounter,
            Observation: stats?.observation,
            Condition: stats?.condition,
            Medication: stats?.medication,
            MedicationRequest: stats?.medicationRequest,
          }}
        />
        <StatCard
          title="Care Team"
          accent="emerald"
          stats={{
            Practitioner: stats?.practitioner,
            CareTeam: stats?.careTeam,
            CarePlan: stats?.carePlan,
          }}
        />
        <StatCard
          title="Insurance"
          accent="amber"
          stats={{
            Claim: stats?.claim,
            ExplanationOfBenefit: stats?.explanationOfBenefit,
          }}
        />
        <StatCard
          title="Forms & Surveys"
          accent="violet"
          stats={{
            Questionnaire: stats?.questionnaire,
            QuestionnaireResponse: stats?.questionnaireResponse,
          }}
        />
        <StatCard
          title="Security"
          accent="rose"
          stats={{
            Membership: stats?.membership,
            AccessPolicyV2: stats?.accessPolicy,
            ClientApplication: stats?.clientApplication,
          }}
        />
        <StatCard
          title="Configuration"
          accent="orange"
          stats={{
            OperationDefinition: stats?.operationDefinition,
            Subscription: stats?.subscription,
          }}
        />
        <StatCard
          title="Monitoring"
          accent="slate"
          stats={{
            AuditEvent: stats?.auditEvent,
          }}
        />
      </div>
    </div>
  );
};

export default function DashboardView() {
  return (
    <React.Suspense
      fallback={
        <div className="h-screen flex flex-1 justify-center items-center flex-col">
          <Loading />
          <div className="mt-1 ">Loading...</div>
        </div>
      }
    >
      <Dashboard />
    </React.Suspense>
  );
}
