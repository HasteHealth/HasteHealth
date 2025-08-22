import { useAtomValue } from "jotai";
import React, { useEffect, useState } from "react";

import { isResponseError } from "@oxidized-health/client/lib/http";
import { Toaster } from "@oxidized-health/components";
import { Loading } from "@oxidized-health/components";
import { R4 } from "@oxidized-health/fhir-types/versions";
import { OxidizedHealthUsageStatistics } from "@oxidized-health/generated-ops/lib/r4/ops";

import Card from "../components/Card";
import { getClient } from "../db/client";

const Dashboard = () => {
  const [stats, setStats] = useState<{
    [key: string]: OxidizedHealthUsageStatistics.Output["statistics"];
  }>({});

  const client = useAtomValue(getClient);
  useEffect(() => {
    client
      .invoke_system(OxidizedHealthUsageStatistics.Op, {}, R4, {})
      .then((stats) =>
        setStats(
          Object.groupBy(stats?.statistics ?? [], (stat) => stat.version)
        )
      )
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

  return (
    <div className="flex flex-col flex-1 overflow-auto">
      <div className="mt-6 mb-6">
        <h2 className=" px-6 text-left flex text-2xl font-semibold">
          FHIR R4 Limits
        </h2>
        <span className="text-xs px-6 mt-2">
          These are the current FHIR R4 Limits for your tenant. To see available
          upgrades click{" "}
          <a
            target="_blank"
            className="underline text-blue-400"
            href="https://oxidized-health.app/pricing"
          >
            here
          </a>
          .
        </span>
      </div>
      <div className="flex flex-wrap mb-6">
        {stats["R4"]?.map((statistic) => (
          <Card
            key={statistic.name}
            title={`${statistic.name} Limit`}
            limit={statistic.limit}
            usage={statistic.usage}
            description={statistic.description ?? ""}
          />
        ))}
      </div>
      <div className="mt-6 mb-6">
        <h2 className=" px-6 text-left flex text-2xl font-semibold">
          FHIR R4B Limits
        </h2>
        <span className="text-xs px-6 mt-2">
          These are the current FHIR R4B Limits for your tenant. To see
          available upgrades click{" "}
          <a
            target="_blank"
            className="underline text-blue-400"
            href="https://oxidized-health.app/pricing"
          >
            here
          </a>
          .
        </span>
      </div>
      <div className="flex flex-wrap mb-6">
        {stats["R4B"]?.map((statistic) => (
          <Card
            key={statistic.name}
            title={`${statistic.name} Limit`}
            limit={statistic.limit}
            usage={statistic.usage}
            description={statistic.description ?? ""}
          />
        ))}
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
