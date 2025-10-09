import React, { useEffect, useState } from "react";

import { Button, useOxidizedHealth } from "@oxidized-health/components";
import { useAtomValue } from "jotai";
import { getClient } from "../db/client";
import { R4 } from "@oxidized-health/fhir-types/versions";

export default function EmptyWorkspace() {
  const [stats, setStats] = useState<any[]>([]);
  const client = useAtomValue(getClient);

  //   useEffect(() => {
  //     client.search_type({}, R4, "Project", []).then((res) => {});
  //   });

  return (
    <div className="h-screen w-screen flex  flex-col items-center">
      <div className=" flex justify-center items-center flex-col px-4 py-4 shadow-md -top-[15px] mt-16">
        <h1 className="text-3xl font-bold mb-4">No Projects</h1>
      </div>
    </div>
  );
}
