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
    <div className="hover:bg-slate-100 cursor-pointer p-6 bg-white border border-slate-200 rounded-lg shadow-sm">
      <h3 className="text-lg font-bold mb-2">{title}</h3>
      <div className="space-y-1">
        {Object.keys(stats).map((key) => (
          <div className="space-x-1 text-wrap" key={key}>
            <span className="font-medium">{key}:</span>
            <span className="text-slate-600">{stats[key] ?? ""}</span>
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
            Patients: stats?.patient,
            Encounters: stats?.encounter,
            Observations: stats?.observation,
          }}
        />

        <StatCard
          title="Configuration Resources"
          stats={{
            "Operation Definitions": stats?.operationDefinition,
            Subscriptions: stats?.subscription,
          }}
        />

        <StatCard
          title="UI Resources"
          stats={{
            Questionnaires: stats?.questionnaire,
            "Questionnaire Responses": stats?.questionnaireResponse,
          }}
        />
        <StatCard
          title="Monitoring Resources"
          stats={{
            AuditEvents: stats?.auditEvent,
          }}
        />
      </div>
    </div>
    // <div className="flex flex-col flex-1 overflow-auto">
    //   <div className="mt-6 mb-6">
    //     <h2 className=" px-6 text-left flex text-2xl font-semibold">
    //       FHIR R4 Limits
    //     </h2>
    //     <span className="text-xs px-6 mt-2">
    //       These are the current FHIR R4 Limits for your tenant. To see available
    //       upgrades click{" "}
    //       <a
    //         target="_blank"
    //         className="underline text-teal-400"
    //         href="https://oxidized-health.app/pricing"
    //       >
    //         here
    //       </a>
    //       .
    //     </span>
    //   </div>
    //   <div className="flex flex-wrap mb-6">
    //     {/* {stats["R4"]?.map((statistic) => (
    //       <Card
    //         key={statistic.name}
    //         title={`${statistic.name} Limit`}
    //         limit={statistic.limit}
    //         usage={statistic.usage}
    //         description={statistic.description ?? ""}
    //       />
    //     ))} */}
    //   </div>
    //   <div className="mt-6 mb-6">
    //     <h2 className=" px-6 text-left flex text-2xl font-semibold">
    //       FHIR R4B Limits
    //     </h2>
    //     <span className="text-xs px-6 mt-2">
    //       These are the current FHIR R4B Limits for your tenant. To see
    //       available upgrades click{" "}
    //       <a
    //         target="_blank"
    //         className="underline text-teal-400"
    //         href="https://oxidized-health.app/pricing"
    //       >
    //         here
    //       </a>
    //       .
    //     </span>
    //   </div>
    //   <div className="flex flex-wrap mb-6">
    //     {/* {stats["R4B"]?.map((statistic) => (
    //       <Card
    //         key={statistic.name}
    //         title={`${statistic.name} Limit`}
    //         limit={statistic.limit}
    //         usage={statistic.usage}
    //         description={statistic.description ?? ""}
    //       />
    //     ))} */}
    //   </div>
    // </div>
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
