import React, { useEffect, useState } from "react";
import { useAtomValue } from "jotai";

import {
  Button,
  Toaster,
  Loading,
  FHIRGenerativeForm,
} from "@haste-health/components";
import { R4 } from "@haste-health/fhir-types/versions";
import {
  code,
  id,
  Project,
  StructureDefinition,
} from "@haste-health/fhir-types/lib/generated/r4/types";

import { getClient } from "../../db/client";
import {
  deriveProjectId,
  deriveTenantId,
  getErrorMessage,
} from "../../utilities";
import Modal from "../../components/Modal";

import { getResource } from "../../db/resource";
import { generatePath, useNavigate } from "react-router-dom";

function ProjectCreateModal({
  open,
  setOpen,
  setProjects,
}: {
  open: boolean;
  setOpen: (open: boolean) => void;
  setProjects: React.Dispatch<React.SetStateAction<Project[]>>;
}) {
  const sd = useAtomValue(
    getResource({
      resourceType: "StructureDefinition",
      id: "Project" as id,
    })
  );
  const [project, setProject] = useState<Project>({
    resourceType: "Project",
    fhirVersion: "r4" as code,
  });
  const client = useAtomValue(getClient);
  return (
    <Modal open={open} setOpen={() => setOpen(false)}>
      <div className="p-2">
        <FHIRGenerativeForm
          fhirVersion={R4}
          value={project}
          structureDefinition={sd as StructureDefinition}
          setValue={(v) => {
            const newProject = v(project);
            setProject(newProject as Project);
          }}
          client={client}
        />

        <Button
          className="mt-4"
          onClick={(_e) => {
            const createPromise = client.create({}, R4, project).then((res) => {
              setProjects((projects) => [...projects, res]);
            });

            Toaster.promise(createPromise, {
              loading: "Creating Project",
              success: (success) => `Project created`,
              error: (error) => {
                return getErrorMessage(error);
              },
            });
            setOpen(false);
          }}
        >
          Create
        </Button>
      </div>
    </Modal>
  );
}

function ProjectUpdateModal({
  id,
  open,
  setOpen,
  setProjects,
}: {
  id: id;
  open: boolean;
  setOpen: (open: boolean) => void;
  setProjects: React.Dispatch<React.SetStateAction<Project[]>>;
}) {
  const client = useAtomValue(getClient);
  const [project, setProject] = useState<Project | null>(null);
  const sd = useAtomValue(
    getResource({
      resourceType: "StructureDefinition",
      id: "Project" as id,
    })
  );
  useEffect(() => {
    client.read({}, R4, "Project", id).then((res) => {
      setProject(res as Project);
    });
  }, [id]);

  return (
    <Modal open={open} setOpen={() => setOpen(false)}>
      <div className="p-2">
        <FHIRGenerativeForm
          fhirVersion={R4}
          value={project ?? undefined}
          structureDefinition={sd as StructureDefinition}
          setValue={(v) => {
            const newProject = v(project!);
            setProject(newProject as Project);
          }}
          client={client}
        />

        <Button
          className="mt-4"
          onClick={(_e) => {
            const createPromise = client
              .update({}, R4, "Project", id, project!)
              .then((res) => {
                setProjects((projects) => [
                  ...projects.filter((p) => p.id !== res.id),
                  res,
                ]);
              });

            Toaster.promise(createPromise, {
              loading: "Updating Project",
              success: (success) => `Project updated`,
              error: (error) => {
                return getErrorMessage(error);
              },
            });
            setOpen(false);
          }}
        >
          Update
        </Button>
      </div>
    </Modal>
  );
}

export default function Projects() {
  const [projects, setProjects] = useState<Project[]>([]);
  const client = useAtomValue(getClient);
  const [openCreateModal, setOpenCreateModal] = useState(false);
  const [openUpdateModalId, setOpenUpdateIdModal] = useState<string>("");
  const navigate = useNavigate();

  useEffect(() => {
    client
      .search_type({}, R4, "Project", [
        { name: "_sort", value: ["_lastUpdated"] },
      ])
      .then((res) => {
        setProjects(res.resources);
      });
  }, []);

  return (
    <React.Suspense
      fallback={
        <div className="h-screen flex flex-1 justify-center items-center flex-col">
          <Loading />
          <div className="mt-1 ">Loading...</div>
        </div>
      }
    >
      <div className="flex flex-col flex-1">
        {/* <ProjectCreateModal
          open={openCreateModal}
          setOpen={(bool) => {
            setOpenCreateModal(bool);
          }}
          setProjects={setProjects}
        />
        <ProjectUpdateModal
          open={openUpdateModalId !== ""}
          setOpen={(bool) => {
            if (!bool) setOpenUpdateIdModal("");
          }}
          id={openUpdateModalId as id}
          setProjects={setProjects}
        /> */}
        <div className=" flex justify-center flex-col px-4  -top-[15px] mt-16">
          <div className="flex items-center space-x-2 mb-8">
            <h1 className="text-3xl font-bold text-center">Projects</h1>
            <Button
              onClick={(e) => {
                navigate(generatePath("/resources/Project/new", {}));
                e.stopPropagation();
              }}
            >
              Create
            </Button>
          </div>
          {projects.length === 0 ? (
            <div className="shadow-md">
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

                    window.open(newUrl, "_blank");
                  }}
                  key={project.id}
                  className="hover:bg-slate-100 cursor-pointer p-6 bg-white border border-slate-200 rounded-lg shadow-sm space-y-1"
                >
                  <div className="flex items-center space-x-1 mb-2 flex-wrap">
                    <div className="flex-1 ">
                      <h5 className="text-2xl font-bold tracking-tight text-slate-900 dark:text-white text-ellipsis">
                        {project.name}
                      </h5>
                    </div>
                    <div>
                      <span className="text-slate-300 text-sm">
                        {project.id}
                      </span>
                    </div>
                  </div>
                  <p className="font-normal text-slate-400">
                    FHIR Version: {project.fhirVersion}
                  </p>
                  <div className="flex justify-center items-center">
                    <div>
                      <span
                        onClick={(e) => {
                          if (
                            confirm(
                              "Do you want to delete project " + project.name
                            )
                          ) {
                            if (confirm("Are you sure?")) {
                              const deletePromise = client
                                .delete_instance({}, R4, "Project", project.id!)
                                .then(() => {
                                  setProjects(
                                    projects.filter((p) => p.id !== project.id)
                                  );
                                });
                              Toaster.promise(deletePromise, {
                                loading: "Deleting Project",
                                success: (success) => `Project deleted`,
                                error: (error) => {
                                  return getErrorMessage(error);
                                },
                              });
                            }
                          } else {
                            console.log("You pressed Cancel!");
                          }
                          // Don't bubble up.
                          e.stopPropagation();
                        }}
                        className="text-red-500 hover:text-red-600 cursor-pointer"
                      >
                        Delete
                      </span>
                    </div>
                    <div className="flex-1" />
                    <div>
                      <span
                        onClick={(e) => {
                          navigate(
                            generatePath("/resources/Project/:id", {
                              id: project.id as string,
                            })
                          );
                          e.stopPropagation();
                        }}
                        className="text-orange-600 hover:text-orange-700 cursor-pointer"
                      >
                        Update
                      </span>
                    </div>
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>
      </div>
    </React.Suspense>
  );
}
