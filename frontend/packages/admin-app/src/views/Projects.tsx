import React, { useEffect, useState } from "react";

import { Button, useOxidizedHealth } from "@oxidized-health/components";
import { useAtomValue } from "jotai";
import { getClient } from "../db/client";
import { R4 } from "@oxidized-health/fhir-types/versions";
import { Project } from "@oxidized-health/fhir-types/lib/generated/r4/types";

export default function Projects() {
  const [projects, setProjects] = useState<Project[]>([]);
  const client = useAtomValue(getClient);

  useEffect(() => {
    client.search_type({}, R4, "Project", []).then((res) => {
      setProjects(res.resources);
    });
  }, []);

  return (
    <div className="h-screen w-screen flex  flex-col items-center">
      <div className=" flex justify-center items-center flex-col px-4 py-4 shadow-md -top-[15px] mt-16">
        {projects.length === 0 ? (
          <>
            <h1 className="text-3xl font-bold mb-4">No Projects Found</h1>
            <p className="text-lg mb-8 text-center">
              It looks like you don't have any projects yet. Click the button
              below to create your first project.
            </p>
          </>
        ) : (
          <>
            <h1 className="text-3xl font-bold mb-4">Projects</h1>
            <p className="text-lg mb-8 text-center">
              You have {projects.length}{" "}
              {projects.length === 1 ? "project" : "projects"}. Click the button
              below to create a new project.
            </p>
          </>
        )}
      </div>
    </div>
  );
}
