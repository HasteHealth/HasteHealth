import {
  ArrowRightIcon,
  ArrowTopRightOnSquareIcon,
  BoltIcon,
  CommandLineIcon,
  PencilSquareIcon,
  PlusIcon,
  TrashIcon,
} from "@heroicons/react/24/outline";
import React, { useCallback, useEffect, useMemo, useState } from "react";
import { useAtomValue } from "jotai";

import { Loading, Toaster } from "@haste-health/components";
import { R4 } from "@haste-health/fhir-types/versions";
import { id, Project } from "@haste-health/fhir-types/lib/generated/r4/types";

import { getClient } from "../../db/client";
import {
  getErrorMessage,
  openProject,
  slugifyProjectName,
  uniqueProjectSlug,
} from "../../utilities";
import { generatePath, useNavigate } from "react-router";

/**
 * `ValueSet/SupportedFHIRVersion` holds a single code, so this is a constant
 * rather than a choice in the form. A second version would change that.
 */
const DEFAULT_FHIR_VERSION = "r4";

/** The system project is this console itself, so it is never opened or edited. */
function isSystemProject(project: Project) {
  return project.id === "system";
}

/** What the typed name will be addressed as, or why it cannot be. */
function SlugHint({
  adjusted,
  finalSlug,
  slug,
  typed,
}: Readonly<{
  adjusted: boolean;
  finalSlug: string;
  slug: string;
  typed: boolean;
}>) {
  if (typed && !slug) {
    return (
      <span className="text-amber-700">
        Add a letter or number: a name needs one to make an address.
      </span>
    );
  }
  if (!finalSlug) return null;

  return (
    <span className="text-slate-500">
      Address: <span className="font-mono text-slate-700">{finalSlug}</span>
      {adjusted && (
        <span className="text-amber-700">
          {" "}
          ({slug} is taken, so this one is free)
        </span>
      )}
    </span>
  );
}

/**
 * The inline create form on the landing page.
 *
 * Posts the Project directly, so a first sandbox needs no trip through the
 * raw editor. That editor still handles what this leaves out, such as
 * identity providers.
 */
function CreateProjectForm({
  onCreated,
  takenIds,
}: Readonly<{
  onCreated: (project: Project) => void;
  /** Ids already in use, so the slug shown is one that is actually free. */
  takenIds: string[];
}>) {
  const client = useAtomValue(getClient);
  const [name, setName] = useState("");
  const [creating, setCreating] = useState(false);

  const trimmed = name.trim();
  const slug = useMemo(() => slugifyProjectName(trimmed), [trimmed]);
  const finalSlug = useMemo(
    () => (slug ? uniqueProjectSlug(slug, takenIds) : ""),
    [slug, takenIds],
  );
  /** Set when the name's own slug was taken and a suffix had to be added. */
  const adjusted = Boolean(slug) && finalSlug !== slug;

  const submit = useCallback(
    (event: React.FormEvent) => {
      event.preventDefault();
      if (!trimmed || !finalSlug || creating) {
        return;
      }

      setCreating(true);
      Toaster.promise(
        client
          // The id travels in the body: the Project middleware keeps
          // `project.id` and only generates one when absent. A PUT cannot be
          // used, as updating a Project requires it to exist.
          .create({}, R4, {
            resourceType: "Project",
            id: finalSlug,
            name: trimmed,
            fhirVersion: DEFAULT_FHIR_VERSION,
          } as Project)
          .then((created) => {
            const project = created as Project;
            setName("");
            onCreated(project);
            // A project is its own subdomain, so it opens in a new tab.
            if (project.id) {
              openProject(project.id);
            }
            return project;
          })
          .finally(() => {
            setCreating(false);
          }),
        {
          loading: "Creating project…",
          success: (created) =>
            `Opening ${(created as Project).name ?? "project"} in a new tab`,
          error: (err) => getErrorMessage(err),
        },
      );
    },
    [client, creating, finalSlug, onCreated, trimmed],
  );

  return (
    <div className="space-y-2">
      <form className="flex flex-col gap-2 sm:flex-row" onSubmit={submit}>
        <label className="sr-only" htmlFor="new-project-name">
          Project name
        </label>
        <input
          aria-describedby="new-project-slug"
          className="min-w-0 flex-1 rounded-md border border-slate-300 bg-white px-3 py-2 text-sm text-slate-900 placeholder:text-slate-400 focus:border-brand-500 focus:outline-none focus:ring-1 focus:ring-brand-500"
          disabled={creating}
          id="new-project-name"
          onChange={(event) => setName(event.target.value)}
          placeholder="My sandbox"
          value={name}
        />
        <button
          className="inline-flex shrink-0 items-center justify-center gap-2 rounded-md bg-brand-600 px-4 py-2 text-sm font-semibold text-white hover:bg-brand-500 disabled:cursor-not-allowed disabled:bg-slate-300"
          disabled={creating || !finalSlug}
          type="submit"
        >
          {creating ? "Creating…" : "Create and open"}
          <ArrowRightIcon className="h-4 w-4" />
        </button>
      </form>

      {/* The id is the address, so it is shown before the write. */}
      <p className="min-h-5 text-xs" id="new-project-slug">
        <SlugHint
          adjusted={adjusted}
          finalSlug={finalSlug}
          slug={slug}
          typed={Boolean(trimmed)}
        />
      </p>
    </div>
  );
}

/**
 * The banner above the project grid, carrying the create form so the first
 * thing on the page is also the first thing to do.
 */
function GettingStarted({
  onCreated,
  hasProjects,
  takenIds,
}: Readonly<{
  onCreated: (project: Project) => void;
  hasProjects: boolean;
  takenIds: string[];
}>) {
  return (
    <section className="overflow-hidden rounded-lg border border-brand-100 bg-gradient-to-br from-brand-50 to-white p-6 shadow-sm">
      <div className="flex items-start gap-3">
        <span className="hidden rounded-md bg-brand-600/10 p-2 text-brand-700 sm:block">
          <BoltIcon className="h-5 w-5" />
        </span>
        <div className="min-w-0 flex-1 space-y-4">
          <div className="space-y-1">
            <h2 className="text-lg font-semibold text-slate-900">
              {hasProjects
                ? "Spin up another sandbox"
                : "Try the FHIR API in a sandbox"}
            </h2>
            <p className="max-w-2xl text-sm text-slate-600">
              A project is an isolated FHIR server with its own data, its own
              users and the full REST API. Name one and it opens in a new tab,
              ready for creating resources and running searches.
            </p>
          </div>

          <div className="max-w-xl">
            <CreateProjectForm onCreated={onCreated} takenIds={takenIds} />
          </div>

          <p className="flex flex-wrap items-center gap-x-4 gap-y-1 text-xs text-slate-500">
            <span className="inline-flex items-center gap-1">
              <CommandLineIcon className="h-3.5 w-3.5" />
              Opens the console, where every resource type is browsable and
              editable
            </span>
            <a
              className="text-brand-700 hover:underline"
              href="https://haste.health"
              rel="noreferrer"
              target="_blank"
            >
              Read the docs
            </a>
          </p>
        </div>
      </div>
    </section>
  );
}

function ProjectCard({
  project,
  highlight,
  onDelete,
}: Readonly<{
  project: Project;
  /** Set on a project created in this session, so it is easy to spot. */
  highlight: boolean;
  onDelete: (id: string) => void;
}>) {
  const navigate = useNavigate();
  const [confirmingDelete, setConfirmingDelete] = useState(false);
  const system = isSystemProject(project);

  return (
    <article
      className={
        highlight
          ? "flex flex-col rounded-lg border-2 border-brand-500 bg-white shadow-sm ring-2 ring-brand-100"
          : "flex flex-col rounded-lg border border-slate-200 bg-white shadow-sm transition hover:border-brand-300 hover:shadow-md"
      }
    >
      <button
        className="group flex-1 rounded-t-lg p-5 text-left hover:bg-brand-50/40 disabled:cursor-not-allowed disabled:bg-gray-50 disabled:hover:bg-gray-50"
        disabled={system}
        onClick={() => openProject(project.id as string)}
        title={
          system ? "The system project has no console" : "Open in a new tab"
        }
      >
        <div className="flex items-start justify-between gap-2">
          <h3 className="truncate text-base font-semibold text-slate-900 group-disabled:text-slate-400">
            {project.name ?? "Unnamed Project"}
          </h3>
          <ArrowTopRightOnSquareIcon className="mt-0.5 h-4 w-4 shrink-0 text-slate-400 group-hover:text-brand-600 group-disabled:invisible" />
        </div>
        <p className="mt-1 truncate font-mono text-xs text-slate-500">
          {project.id}
        </p>
        <div className="mt-3 flex flex-wrap items-center gap-2">
          <span className="inline-flex items-center rounded-full bg-brand-50 px-2 py-0.5 text-xs font-medium text-brand-700 group-disabled:bg-slate-100 group-disabled:text-slate-500">
            FHIR {project.fhirVersion?.toUpperCase()}
          </span>
          {system && (
            <span className="inline-flex items-center rounded-full bg-slate-100 px-2 py-0.5 text-xs font-medium text-slate-500">
              This console
            </span>
          )}
          {project.identityProvider?.length ? (
            <span className="inline-flex items-center rounded-full bg-slate-100 px-2 py-0.5 text-xs font-medium text-slate-600">
              {project.identityProvider.length} identity provider
              {project.identityProvider.length === 1 ? "" : "s"}
            </span>
          ) : null}
        </div>
        {/* Kept even when hidden, so cards in a row stay aligned. */}
        <span className="mt-4 inline-flex items-center gap-1 text-xs font-medium text-brand-700 group-hover:underline group-disabled:invisible">
          Open console
          <ArrowRightIcon className="h-3.5 w-3.5" />
        </span>
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
              title="Edit project settings"
              disabled={system}
              className="rounded p-1.5 text-slate-500 hover:bg-slate-100 hover:text-slate-800 disabled:cursor-not-allowed disabled:text-slate-300"
              onClick={() =>
                navigate(
                  generatePath("/r/Project/:id", {
                    id: project.id as string,
                  }),
                )
              }
            >
              <PencilSquareIcon className="h-4 w-4" />
            </button>
            <button
              title="Delete project"
              disabled={system}
              className="rounded p-1.5 text-slate-500 hover:bg-red-50 hover:text-red-600 disabled:cursor-not-allowed disabled:text-slate-300"
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

/** The tile closing the grid; the full editor, for what the form omits. */
function NewProjectCard() {
  const navigate = useNavigate();

  return (
    <button
      className="flex min-h-36 flex-col items-center justify-center gap-1 rounded-lg border-2 border-dashed border-slate-300 bg-white/50 p-5 text-slate-500 hover:border-brand-400 hover:text-brand-700"
      onClick={() => navigate(generatePath("/r/Project/new", {}))}
      type="button"
    >
      <PlusIcon className="h-6 w-6" />
      <span className="text-sm font-medium">New project</span>
      <span className="text-center text-xs text-slate-400">
        With identity providers and access policies
      </span>
    </button>
  );
}

export default function Projects() {
  const [projects, setProjects] = useState<Project[]>([]);
  const client = useAtomValue(getClient);
  const [loading, setLoading] = useState(true);
  /** The project created in this session, highlighted in the grid. */
  const [createdId, setCreatedId] = useState<string | undefined>(undefined);

  /**
   * Reloads the list. Writes are followed by one of these: the grid reflects
   * a search, and the index is written asynchronously.
   */
  const refresh = useCallback(
    () =>
      client
        .search_type({}, R4, "Project", [
          { name: "_sort", value: ["_lastUpdated"] },
        ])
        .then((res) => {
          setProjects(res.resources);
        })
        .catch(() => {
          Toaster.error("Failed to load projects.");
        }),
    [client],
  );

  useEffect(() => {
    setLoading(true);
    refresh().finally(() => {
      setLoading(false);
    });
  }, [refresh]);

  const handleDelete = useCallback(
    (projectId: string) => {
      Toaster.promise(
        client.delete_instance({}, R4, "Project", projectId as id).then(() => {
          setProjects((prev) => prev.filter((p) => p.id !== projectId));
          void refresh();
        }),
        {
          loading: "Deleting project…",
          success: () => "Project deleted",
          error: (err) => getErrorMessage(err),
        },
      );
    },
    [client, refresh],
  );

  const handleCreated = useCallback(
    (project: Project) => {
      setCreatedId(project.id);
      // Show the card at once, then reconcile with the index.
      setProjects((prev) => [project, ...prev]);
      void refresh();
    },
    [refresh],
  );

  const { ordered, sandboxCount, takenIds } = useMemo(() => {
    const sandboxes = projects.filter((p) => !isSystemProject(p));
    return {
      // Sandboxes first, the system project last. On a first run it is
      // dropped too: a card that cannot be opened is noise beside the form.
      ordered:
        sandboxes.length > 0
          ? [...sandboxes, ...projects.filter(isSystemProject)]
          : [],
      sandboxCount: sandboxes.length,
      // Only as fresh as the last search, so the server can still reject a
      // slug this believes is free.
      takenIds: projects.flatMap((p): string[] =>
        p.id ? [p.id as string] : [],
      ),
    };
  }, [projects]);

  return (
    <div className="flex w-full flex-col gap-6">
      <header className="space-y-1">
        <h1 className="text-2xl font-semibold text-slate-900">Projects</h1>
        <p className="text-sm text-slate-500">
          Isolated FHIR sandboxes. Open one to use its API, or manage the users
          and identity providers that reach it.
        </p>
      </header>

      {!loading && (
        <GettingStarted
          hasProjects={sandboxCount > 0}
          takenIds={takenIds}
          onCreated={handleCreated}
        />
      )}

      {loading ? (
        <div className="flex items-center gap-2 text-sm text-slate-500">
          <Loading />
          <span>Loading projects…</span>
        </div>
      ) : (
        // On a first run the form above is the whole page.
        sandboxCount > 0 && (
          <section className="space-y-3">
            <h2 className="text-sm font-medium text-slate-700">
              Your projects ({sandboxCount})
            </h2>
            <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4">
              {ordered.map((project) => (
                <ProjectCard
                  key={project.id}
                  highlight={project.id === createdId}
                  project={project}
                  onDelete={handleDelete}
                />
              ))}
              <NewProjectCard />
            </div>
          </section>
        )
      )}
    </div>
  );
}
