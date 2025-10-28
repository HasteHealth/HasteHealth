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
            className="cursor-pointer font-medium hover:text-teal-400 space-x-1 text-wrap"
            key={statKey}
            onClick={() => {
              navigate(
                generatePath("/resources/:resourceType", {
                  resourceType: statKey,
                })
              );
            }}
          >
            <span className="font-medium">{statKey}s:</span>
            <span className="text-slate-600">{stats[statKey] ?? ""}</span>
          </div>
        ))}
      </div>
    </div>
  );

  return (
    <div className="w-full">
      <div className="grid md:grid-cols-3 lg:grid-cols-4 2xl:grid-cols-5  sm:grid-cols-2 gap-4 grid-flow-row-dense auto-cols-max">
        <StatCard
          title="Clinical Resources"
          stats={{
            Patient: stats?.patient,
            Encounter: stats?.encounter,
            Observation: stats?.observation,
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
