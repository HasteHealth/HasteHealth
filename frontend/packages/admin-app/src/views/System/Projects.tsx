import {
  ArrowTopRightOnSquareIcon,
  PencilSquareIcon,
  PlusIcon,
  TrashIcon,
} from "@heroicons/react/24/outline";
import React, { useCallback, useEffect, useState } from "react";
import { useAtomValue } from "jotai";

import { Loading, Toaster } from "@haste-health/components";
import { R4 } from "@haste-health/fhir-types/versions";
import { id, Project } from "@haste-health/fhir-types/lib/generated/r4/types";

import { getClient } from "../../db/client";
import {
  deriveProjectId,
  deriveTenantId,
  getErrorMessage,
} from "../../utilities";
import { generatePath, useNavigate } from "react-router-dom";

function openProject(project: Project) {
  const currentTenant = deriveTenantId();
  const currentProject = deriveProjectId();
  const newUrl = window.location.origin.replace(
    `${currentTenant}_${currentProject}`,
    `${currentTenant}_${project.id}`,
  );
  window.open(newUrl, "_blank");
}

function ProjectCard({
  project,
  onDelete,
}: Readonly<{
  project: Project;
  onDelete: (id: string) => void;
}>) {
  const navigate = useNavigate();
  const [confirmingDelete, setConfirmingDelete] = useState(false);

  return (
    <article className="flex flex-col rounded-lg border border-slate-200 bg-white shadow-sm">
      <button
        className="flex-1 p-5 text-left hover:bg-slate-50 rounded-t-lg disabled:bg-gray-50 disabled:text-gray-400 disabled:cursor-not-allowed"
        disabled={project.id === "system"}
        onClick={() => openProject(project)}
      >
        <div className="flex items-start justify-between gap-2">
          <h3 className="text-base font-semibold text-slate-900 truncate">
            {project.name ?? "Unnamed Project"}
          </h3>
          <ArrowTopRightOnSquareIcon className="h-4 w-4 shrink-0 text-slate-400 mt-0.5" />
        </div>
        <p className="mt-1 text-xs text-slate-500 font-mono truncate">
          {project.id}
        </p>
        <div className="mt-3 flex items-center gap-2">
          <span className="inline-flex items-center rounded-full bg-brand-50 px-2 py-0.5 text-xs font-medium text-brand-700">
            FHIR {project.fhirVersion?.toUpperCase()}
          </span>
        </div>
      </button>

      <div className="flex items-center justify-end gap-1 border-t border-slate-100 px-4 py-2">
        {confirmingDelete ? (
          <>
            <span className="mr-auto text-xs text-slate-600">
              Delete this project?
            </span>
            <button
              className="rounded px-2 py-1 text-xs font-medium text-slate-600 hover:bg-slate-100"
              onClick={() => setConfirmingDelete(false)}
            >
              Cancel
            </button>
            <button
              className="rounded bg-red-600 px-2 py-1 text-xs font-medium text-white hover:bg-red-700"
              onClick={() => {
                onDelete(project.id!);
                setConfirmingDelete(false);
              }}
            >
              Delete
            </button>
          </>
        ) : (
          <>
            <button
              title="Edit project"
              disabled={project.id === "system"}
              className="rounded p-1.5 text-slate-500 hover:bg-slate-100 hover:text-slate-800 disabled:cursor-not-allowed"
              onClick={() =>
                navigate(
                  generatePath("/resources/Project/:id", {
                    id: project.id as string,
                  }),
                )
              }
            >
              <PencilSquareIcon className="h-4 w-4" />
            </button>
            <button
              title="Delete project"
              disabled={project.id === "system"}
              className="rounded p-1.5 text-slate-500 hover:bg-red-50 hover:text-red-600 disabled:cursor-not-allowed"
              onClick={() => setConfirmingDelete(true)}
            >
              <TrashIcon className="h-4 w-4" />
            </button>
          </>
        )}
      </div>
    </article>
  );
}

export default function Projects() {
  const [projects, setProjects] = useState<Project[]>([]);
  const client = useAtomValue(getClient);
  const [loading, setLoading] = useState(true);
  const navigate = useNavigate();

  useEffect(() => {
    setLoading(true);
    client
      .search_type({}, R4, "Project", [
        { name: "_sort", value: ["_lastUpdated"] },
      ])
      .then((res) => {
        setProjects(res.resources);
      })
      .catch(() => {
        Toaster.error("Failed to load projects.");
      })
      .finally(() => {
        setLoading(false);
      });
  }, [client]);

  const handleDelete = useCallback(
    (projectId: string) => {
      Toaster.promise(
        client.delete_instance({}, R4, "Project", projectId as id).then(() => {
          setProjects((prev) => prev.filter((p) => p.id !== projectId));
        }),
        {
          loading: "Deleting project…",
          success: () => "Project deleted",
          error: (err) => getErrorMessage(err),
        },
      );
    },
    [client],
  );

  return (
    <div className="flex w-full flex-col gap-6">
      <header className="rounded-lg border border-slate-200 bg-white p-5 shadow-sm">
        <div className="flex items-start justify-between gap-4">
          <div className="space-y-1">
            <h1 className="text-2xl font-semibold text-slate-900">Projects</h1>
            <p className="text-sm text-slate-500">
              Each project is an isolated FHIR namespace. Click a project card
              to open it in a new tab.
            </p>
          </div>
          <button
            className="inline-flex shrink-0 items-center gap-2 rounded-md bg-brand-600 px-4 py-2 text-sm font-semibold text-white hover:bg-brand-500"
            onClick={() => navigate(generatePath("/resources/Project/new", {}))}
          >
            <PlusIcon className="h-4 w-4" />
            New Project
          </button>
        </div>
      </header>

      {loading ? (
        <div className="flex items-center gap-2 text-sm text-slate-500">
          <Loading />
          <span>Loading projects…</span>
        </div>
      ) : projects.length === 0 ? (
        <div className="rounded-lg border border-dashed border-slate-300 bg-white p-12 text-center">
          <p className="text-base font-medium text-slate-700">
            No projects found
          </p>
          <p className="mt-2 text-sm text-slate-500">
            Create your first project to start storing FHIR resources.
          </p>
          <button
            className="mt-6 inline-flex items-center gap-2 rounded-md bg-brand-600 px-4 py-2 text-sm font-semibold text-white hover:bg-brand-500"
            onClick={() => navigate(generatePath("/resources/Project/new", {}))}
          >
            <PlusIcon className="h-4 w-4" />
            New Project
          </button>
        </div>
      ) : (
        <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4">
          {projects.map((project) => (
            <ProjectCard
              key={project.id}
              project={project}
              onDelete={handleDelete}
            />
          ))}
        </div>
      )}
    </div>
  );
}
