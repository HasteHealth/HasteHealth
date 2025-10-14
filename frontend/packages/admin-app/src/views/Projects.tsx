import React, { useEffect, useState } from "react";

import { Button, useOxidizedHealth } from "@oxidized-health/components";
import { useAtomValue } from "jotai";
import { getClient } from "../db/client";
import { R4 } from "@oxidized-health/fhir-types/versions";
import { Project } from "@oxidized-health/fhir-types/lib/generated/r4/types";
import { deriveProjectId, deriveTenantId } from "../utilities";

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
      <div className=" flex justify-center items-center flex-col px-4 py-4  -top-[15px] mt-16">
        {projects.length === 0 ? (
          <div className="shadow-md">
            <h1 className="text-3xl font-bold mb-4">No Projects Found</h1>
            <p className="text-lg mb-8 text-center">
              It looks like you don't have any projects yet. Click the button
              below to create your first project.
            </p>
          </div>
        ) : (
          <>
            <h1 className="text-3xl font-bold mb-4">Projects</h1>
            {projects.map((project) => (
              <div
                onClick={(_e) => {
                  const currentTenant = deriveTenantId();
                  const currentProject = deriveProjectId();

                  const newUrl = window.location.origin.replace(
                    `${currentTenant}_${currentProject}`,
                    `${currentTenant}_${project.id}`
                  );

                  alert(newUrl);
                }}
                key={project.id}
                className="cursor-pointer block max-w-sm p-6 bg-white border border-gray-200 rounded-lg shadow-sm hover:bg-gray-100 dark:bg-gray-800 dark:border-gray-700 dark:hover:bg-gray-700"
              >
                <h5 className="mb-2 text-2xl font-bold tracking-tight text-gray-900 dark:text-white">
                  {project.name}
                </h5>
                <p className="font-normal text-gray-700 dark:text-gray-400">
                  {project.fhirVersion}
                </p>
              </div>
            ))}
          </>
        )}
      </div>
    </div>
  );
}
