import { useAtomValue } from "jotai";
import React, { useEffect, useState } from "react";

import { isResponseError } from "@oxidized-health/client/lib/http";
import { Toaster } from "@oxidized-health/components";
import { Loading } from "@oxidized-health/components";
import { R4 } from "@oxidized-health/fhir-types/versions";

import { getClient } from "../../db/client";
import {
  Bundle,
  code,
  uri,
} from "@oxidized-health/fhir-types/lib/generated/r4/types";
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

const Dashboard = () => {
  const [stats, setStats] = useState<Statistics | null>(null);

  const navigate = useNavigate();

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
      })
      .catch((e) => {
        if (isResponseError(e))
          Toaster.error(
            e.response.body.issue?.[0]?.diagnostics ?? "Failed to fetch stats."
          );
        else {
          Toaster.error("Failed to usage stats.");
        }
      });
  }, [setStats]);

  const StatCard = ({
    title,
    stats,
  }: {
    title: string;
    stats: Record<string, number | undefined>;
  }) => (
    <div className=" p-6 bg-white border border-slate-200 rounded-lg shadow-sm text-slate-800">
      <h3 className="text-lg font-semibold underline mb-2">{title}</h3>
      <div className="space-y-1">
        {Object.keys(stats).map((statKey) => (
          <div
            className="cursor-pointer font-medium hover:text-teal-400 space-x-1"
            key={statKey}
            onClick={() => {
              navigate(
                generatePath("/resources/:resourceType", {
                  resourceType: statKey,
                })
              );
            }}
          >
            <span className="font-medium text-pretty">
              {statKey}s:{" "}
              <span className="text-slate-600">{stats[statKey] ?? ""}</span>
            </span>
          </div>
        ))}
      </div>
    </div>
  );

  return (
    <div className="w-full">
      <div className="grid md:grid-cols-3 lg:grid-cols-4 2xl:grid-cols-5  sm:grid-cols-2 gap-4 grid-flow-row-dense auto-cols-max">
        <StatCard
          title="Patient Resources"
          stats={{
            Patient: stats?.patient,
            Encounter: stats?.encounter,
            Observation: stats?.observation,
            Medication: stats?.medication,
            MedicationRequest: stats?.medicationRequest,
            Condition: stats?.condition,
          }}
        />
        <StatCard
          title="Practitioner Resources"
          stats={{
            CareTeam: stats?.careTeam,
            CarePlan: stats?.carePlan,
            Practitioner: stats?.practitioner,
          }}
        />
        <StatCard
          title="Configuration Resources"
          stats={{
            OperationDefinition: stats?.operationDefinition,
            Subscription: stats?.subscription,
          }}
        />
        <StatCard
          title="UI Resources"
          stats={{
            Questionnaire: stats?.questionnaire,
            QuestionnaireResponse: stats?.questionnaireResponse,
          }}
        />
        <StatCard
          title="Security"
          stats={{
            Membership: stats?.membership,
            AccessPolicy: stats?.accessPolicy,
            ClientApplication: stats?.clientApplication,
          }}
        />
        <StatCard
          title="Insurance"
          stats={{
            Claim: stats?.claim,
            ExplanationOfBenefit: stats?.explanationOfBenefit,
          }}
        />
        <StatCard
          title="Monitoring Resources"
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
