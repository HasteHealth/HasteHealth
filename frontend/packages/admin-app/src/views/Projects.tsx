import React, { useEffect, useState } from "react";

import { Button, useOxidizedHealth } from "@oxidized-health/components";
import { useAtomValue } from "jotai";
import { getClient } from "../db/client";
import { R4 } from "@oxidized-health/fhir-types/versions";
import {
  code,
  Project,
} from "@oxidized-health/fhir-types/lib/generated/r4/types";
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
    <div className="flex flex-col flex-1">
      <div className=" flex justify-center flex-col px-4 py-4  -top-[15px] mt-16">
        <div className="flex items-center space-x-2 mb-8">
          <h1 className="text-3xl font-bold text-center">Projects</h1>
          <Button
            onClick={(_e) => {
              client
                .create({}, R4, {
                  resourceType: "Project",
                  name: "New Project",
                  fhirVersion: "r4" as code,
                })
                .then((res) => {
                  setProjects([...projects, res]);
                });
            }}
          >
            Create
          </Button>
        </div>
        {projects.length === 0 ? (
          <div className="shadow-md">
            <h1 className="text-3xl font-bold mb-4">No Projects Found</h1>{" "}
            <p className="text-lg mb-8 text-center">
              It looks like you don't have any projects yet. Click the button
              below to create your first project.
            </p>
          </div>
        ) : (
          <div className="grid md:grid-cols-3 lg:grid-cols-4 sm:grid-cols-2 gap-4 grid-flow-row-dense auto-cols-max">
            {projects.map((project) => (
              <div
                onClick={(_e) => {
                  const currentTenant = deriveTenantId();
                  const currentProject = deriveProjectId();

                  const newUrl = window.location.origin.replace(
                    `${currentTenant}_${currentProject}`,
                    `${currentTenant}_${project.id}`
                  );

                  window.location.href = newUrl;
                }}
                key={project.id}
                className="hover:bg-slate-100 cursor-pointer p-6 bg-white border border-slate-200 rounded-lg shadow-sm"
              >
                <div className="flex items-center space-x-1 mb-2">
                  <div className="flex-1">
                    <h5 className="text-2xl font-bold tracking-tight text-slate-900 dark:text-white">
                      {project.name}
                    </h5>
                  </div>
                  <div>
                    <span className="text-slate-300 text-sm">{project.id}</span>
                  </div>
                </div>
                <p className="font-normal text-slate-400">
                  FHIR Version: {project.fhirVersion}
                </p>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
