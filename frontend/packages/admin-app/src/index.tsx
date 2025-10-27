import {
  ArrowLeftOnRectangleIcon,
  Cog6ToothIcon,
} from "@heroicons/react/24/outline";
import classNames from "classnames";
import { useAtom } from "jotai";
import React, { useEffect, useMemo } from "react";
import ReactDOM from "react-dom/client";
import {
  Link,
  Navigate,
  Outlet,
  RouterProvider,
  createBrowserRouter,
  generatePath,
  useMatches,
  useNavigate,
  useParams,
} from "react-router-dom";

import {
  OxidizedHealthProvider,
  Loading,
  ProfileDropdown,
  SideBar,
  Toaster,
  useOxidizedHealth,
} from "@oxidized-health/components";
import "@oxidized-health/components/dist/index.css";

import Search from "./components/Search";
import SearchModal from "./components/SearchModal";
import { REACT_APP_CLIENT_ID, REACT_APP_FHIR_BASE_URL } from "./config";
import { createAdminAppClient, getClient } from "./db/client";
import "./index.css";
import reportWebVitals from "./reportWebVitals";
import BundleImport from "./views/Project/BundleImport";
import Dashboard from "./views/Project/Dashboard";
import EmptyWorkspace from "./views/Project/EmptyWorkspace";
import ResourceEditor from "./views/ResourceEditor/index";
import ResourceType from "./views/Project/ResourceType";
import Resources from "./views/Project/Resources";
import Settings from "./views/Project/Settings";
import Projects from "./views/System/Projects";
import { deriveProjectId, deriveTenantId } from "./utilities";
import * as r4Types from "@oxidized-health/fhir-types/r4/types";
import SystemResources from "./views/System";
import { ProjectInformation } from "@oxidized-health/generated-ops/r4";
import { R4 } from "@oxidized-health/fhir-types/versions";

const capitalize = (s: string) => s.charAt(0).toUpperCase() + s.slice(1);

// Could potentially use HOST=oxidized-health.localhost but instead just going to default redirect to system.oxidized-health.localhost
if (
  process.env.NODE_ENV === "development" &&
  window.location.hostname === "localhost"
) {
  window.location.href = `http://system.oxidized-health.localhost:${window.location.port}`;
}

function LoginWrapper() {
  const oxidizedHealth = useOxidizedHealth();

  return (
    <>
      {oxidizedHealth.loading ? (
        <div className="h-screen flex flex-1 justify-center items-center flex-col">
          <Loading />
          <div className="mt-1 ">Loading...</div>
        </div>
      ) : oxidizedHealth.error ? (
        <div className="h-screen flex">
          <div className="flex-1 flex items-center justify-center">
            <div className="p-4 bg-red-100 text-red-800 border border-red-400 rounded-md space-y-2">
              <div className="font-bold">
                {oxidizedHealth.error.code.split("_").map(capitalize).join(" ")}
              </div>
              <div>{oxidizedHealth.error.description}</div>
            </div>
          </div>
        </div>
      ) : (
        <div className="flex flex-col flex-1">
          <Outlet />
        </div>
      )}
    </>
  );
}

function ServiceSetup() {
  const oxidizedHealth = useOxidizedHealth();
  const client = oxidizedHealth.isAuthenticated
    ? oxidizedHealth.client
    : undefined;
  const [c, setClient] = useAtom(getClient);

  React.useEffect(() => {
    if (client) {
      setClient(createAdminAppClient(client));
    }
  }, [setClient, oxidizedHealth.isAuthenticated, oxidizedHealth.client]);

  return (
    <>
      {c ? (
        <>
          <Outlet />
        </>
      ) : undefined}
    </>
  );
}

function OxidizedHealthWrapper() {
  const navigate = useNavigate();

  return (
    <OxidizedHealthProvider
      refresh
      authorize_method="GET"
      scope="offline_access openid email profile fhirUser user/*.*"
      domain={REACT_APP_FHIR_BASE_URL || ""}
      tenant={deriveTenantId()}
      project={deriveProjectId()}
      clientId={REACT_APP_CLIENT_ID}
      redirectUrl={window.location.origin}
      onRedirectCallback={(initialPath: string) => {
        navigate(initialPath);
      }}
    >
      <Outlet />
    </OxidizedHealthProvider>
  );
}

const SYSTEM_TYPES: r4Types.ResourceType[] = [
  "Project",
  "User",
  "IdentityProvider",
];

function SystemBar() {
  const params = useParams();

  return (
    <div className="w-full">
      {/* Create a horizontal navbar with circular buttons for navigation to user project IdentityProvider */}
      <nav className="flex space-x-4 pb-4">
        {SYSTEM_TYPES.map((type) => (
          <Link
            key={type}
            to={`/resources/${type}`}
            className={classNames(
              "flex items-center justify-center  h-10 rounded-full px-4 text-sm text-slate-800",
              {
                ["bg-indigo-500 hover:bg-indigo-600 text-white"]:
                  params.resourceType === type,
                [" bg-gray-100 hover:bg-indigo-400 hover:text-white p-2"]:
                  params.resourceType !== type,
              }
            )}
          >
            {type}s
          </Link>
        ))}
      </nav>
      <Outlet />
    </div>
  );
}

const router =
  deriveProjectId() == "system"
    ? createBrowserRouter([
        {
          id: "oxidized-health-wrapper",
          element: <OxidizedHealthWrapper />,
          children: [
            {
              id: "empty-workspace",
              path: "/no-workspace",
              element: <EmptyWorkspace />,
            },
            {
              path: "/",
              element: <ServiceSetup />,
              children: [
                {
                  id: "login",
                  element: <LoginWrapper />,
                  children: [
                    {
                      id: "system-root",
                      element: <Page resourceTypeFilter={SYSTEM_TYPES} />,

                      children: [
                        {
                          id: "root",
                          element: <SystemBar />,
                          children: [
                            {
                              id: "Resources",
                              path: "/resources/:resourceType",
                              element: <SystemResources />,
                            },
                            {
                              id: "Editor",
                              path: "/resources/:resourceType/:id",
                              element: <ResourceEditor />,
                            },
                            {
                              id: "settings",
                              path: "settings",
                              element: <Settings />,
                            },
                            {
                              id: "redirect",
                              path: "/",
                              element: (
                                <Navigate to="/resources/Project" replace />
                              ),
                            },
                          ],
                        },
                      ],
                    },
                  ],
                },
              ],
            },
          ],
        },
      ])
    : createBrowserRouter([
        {
          id: "oxidized-health-wrapper",
          element: <OxidizedHealthWrapper />,
          children: [
            {
              id: "login",
              element: <LoginWrapper />,
              children: [
                {
                  id: "empty-workspace",
                  path: "/no-workspace",
                  element: <EmptyWorkspace />,
                },
                {
                  path: "/",
                  element: <ServiceSetup />,
                  children: [
                    {
                      id: "tenant",
                      path: "/system",
                      element: <Projects />,
                    },
                    {
                      path: "/",
                      element: <ProjectRoot />,
                      children: [
                        {
                          id: "settings",
                          path: "settings",
                          element: <Settings />,
                        },
                        {
                          id: "dashboard",
                          path: "",
                          element: <Dashboard />,
                        },
                        {
                          id: "resources",
                          path: "resources",
                          element: <Resources />,
                        },
                        {
                          id: "types",
                          path: "resources/:resourceType",
                          element: <ResourceType />,
                        },
                        {
                          id: "instance",
                          path: "resources/:resourceType/:id",
                          element: <ResourceEditor />,
                        },
                        {
                          id: "bundle-import",
                          path: "bundle-import",
                          element: <BundleImport />,
                        },
                      ],
                    },
                  ],
                },
              ],
            },
          ],
        },
      ]);

function Navbar() {
  const oxidizedHealth = useOxidizedHealth();
  const navigate = useNavigate();

  return (
    <div className="z-10 sticky top-0 bg-white">
      <div className="flex items-center " style={{ height: "64px" }}>
        <div className="flex grow mr-4">
          <Search />
        </div>
        <div className="flex justify-center items-center space-x-8">
          <a
            target="_blank"
            className="cursor text-slate-500 hover:text-slate-600 hover:underline"
            href="https://oxidized-health.app"
          >
            Documentation
          </a>
          <ProfileDropdown
            user={{
              email: oxidizedHealth.user?.email,
              name:
                oxidizedHealth.user?.given_name || oxidizedHealth.user?.email,
              // imageUrl: auth0.user?.picture,
            }}
          >
            <div>
              <div className="mt-2">
                <a
                  className={classNames(
                    "cursor-pointer block px-4 py-2 text-sm  hover:text-teal-800 hover:bg-teal-200"
                  )}
                  onClick={() => {
                    navigate(generatePath("/settings", {}));
                  }}
                >
                  Settings
                </a>
                <a
                  className="cursor-pointer block px-4 py-2 text-sm text-slate-800 hover:text-teal-800 hover:bg-teal-200"
                  onClick={() => {
                    oxidizedHealth.logout(window.location.origin);
                  }}
                >
                  Sign out
                </a>
              </div>
            </div>
          </ProfileDropdown>
        </div>
      </div>
    </div>
  );
}

type PageProps = {
  resourceTypeFilter?: r4Types.ResourceType[];
};

function Page(props: PageProps) {
  return (
    <div className="px-4">
      <Navbar />
      <div
        className="py-4 flex flex-1"
        style={{ height: "calc(100vh - 64px)" }}
      >
        <Toaster.Toaster />
        <Outlet />
      </div>
      <React.Suspense fallback={<div />}>
        <SearchModal resourceTypeFilter={props.resourceTypeFilter} />
      </React.Suspense>
    </div>
  );
}

function ProjectRoot() {
  const oxidizedHealth = useOxidizedHealth();
  const navigate = useNavigate();
  const matches = useMatches();
  const [project, setProject] = React.useState<r4Types.Project | null>(null);

  useEffect(() => {
    oxidizedHealth.client
      .invoke_system(ProjectInformation.Op, {}, R4, {})
      .then((res) => {
        setProject(res.project);
      });
  }, []);

  return (
    <>
      <SideBar.SidebarLayout
        sidebar={
          <SideBar.SideBar
            top={
              <div
                onClick={() => navigate(generatePath("/", {}))}
                className="font-semibold cursor-pointer p-2 mt-4 mb-4"
              >
                <div>
                  <span>{project?.name} </span>
                </div>
                <div>
                  <span className="text-slate-400">
                    {oxidizedHealth.tenant}
                  </span>
                </div>
              </div>
            }
          >
            <SideBar.SideBarItemGroup label="Configuration">
              <SideBar.SideBarItem
                active={
                  matches[0].params.resourceType === "OperationDefinition"
                }
                onClick={() => {
                  navigate(
                    generatePath("/resources/:resourceType", {
                      resourceType: "OperationDefinition",
                    })
                  );
                }}
              >
                Custom Operations
              </SideBar.SideBarItem>
              <SideBar.SideBarItem
                active={matches[0].params.resourceType === "Subscription"}
                onClick={() => {
                  navigate(
                    generatePath("/resources/:resourceType", {
                      resourceType: "Subscription",
                    })
                  );
                }}
              >
                Subscriptions
              </SideBar.SideBarItem>
            </SideBar.SideBarItemGroup>
            <SideBar.SideBarItemGroup label="UI">
              <SideBar.SideBarItem
                active={matches[0].params.resourceType === "Questionnaire"}
                onClick={() => {
                  navigate(
                    generatePath("/resources/:resourceType", {
                      resourceType: "Questionnaire",
                    })
                  );
                }}
              >
                Questionnaires
              </SideBar.SideBarItem>
              <SideBar.SideBarItem
                active={
                  matches[0].params.resourceType === "QuestionnaireResponse"
                }
                onClick={() => {
                  navigate(
                    generatePath("/resources/:resourceType", {
                      resourceType: "QuestionnaireResponse",
                    })
                  );
                }}
              >
                Questionnaire Responses
              </SideBar.SideBarItem>
            </SideBar.SideBarItemGroup>
            <SideBar.SideBarItemGroup label="Monitoring">
              <SideBar.SideBarItem
                active={matches[0].params.resourceType === "AuditEvent"}
                onClick={() => {
                  navigate(
                    generatePath("/resources/:resourceType", {
                      resourceType: "AuditEvent",
                    })
                  );
                }}
              >
                Audit Events
              </SideBar.SideBarItem>
            </SideBar.SideBarItemGroup>

            <SideBar.SideBarItemGroup label="Security">
              <SideBar.SideBarItem
                active={matches[0].params.resourceType === "Membership"}
                onClick={() => {
                  navigate(
                    generatePath("/resources/:resourceType", {
                      resourceType: "Membership",
                    })
                  );
                }}
              >
                Membership
              </SideBar.SideBarItem>
              <SideBar.SideBarItem
                active={matches[0].params.resourceType === "AccessPolicyV2"}
                onClick={() => {
                  navigate(
                    generatePath("/resources/:resourceType", {
                      resourceType: "AccessPolicyV2",
                    })
                  );
                }}
              >
                Access Policies
              </SideBar.SideBarItem>
              <SideBar.SideBarItem
                active={matches[0].params.resourceType === "ClientApplication"}
                onClick={() => {
                  navigate(
                    generatePath("/resources/:resourceType", {
                      resourceType: "ClientApplication",
                    })
                  );
                }}
              >
                Client Applications
              </SideBar.SideBarItem>
            </SideBar.SideBarItemGroup>
            <SideBar.SideBarItemGroup label="Data">
              <SideBar.SideBarItem
                active={
                  matches.find((match) => match.id === "resources") !==
                    undefined ||
                  matches.find(
                    (match) =>
                      match.id === "types" &&
                      match.params.resourceType !== "OperationDefinition" &&
                      match.params.resourceType !== "Subscription" &&
                      match.params.resourceType !== "Questionnaire" &&
                      match.params.resourceType !== "QuestionnaireResponse" &&
                      match.params.resourceType !== "AuditEvent" &&
                      match.params.resourceType !== "Membership" &&
                      match.params.resourceType !== "AccessPolicyV2" &&
                      match.params.resourceType !== "ClientApplication" &&
                      match.params.resourceType !== "IdentityProvider"
                  ) !== undefined
                }
                onClick={() => {
                  navigate(generatePath("/resources", {}));
                }}
              >
                All Resources
              </SideBar.SideBarItem>
            </SideBar.SideBarItemGroup>
            <SideBar.SideBarItemGroup label="Import">
              <SideBar.SideBarItem
                active={
                  matches.find((match) => match.id === "bundle-import") !==
                  undefined
                }
                onClick={() => {
                  navigate(generatePath("/bundle-import", {}));
                }}
              >
                Bundles
              </SideBar.SideBarItem>
            </SideBar.SideBarItemGroup>
            {/* Used because want to maintain a margin of at least 8 when shrinking. */}
            <div />
            <SideBar.SideBarItemGroup className="mt-auto" label="User">
              <SideBar.SideBarItem
                logo={<Cog6ToothIcon />}
                active={
                  matches.find((match) => match.id === "settings") !== undefined
                }
                onClick={() => navigate(generatePath("/settings", {}))}
              >
                Settings
              </SideBar.SideBarItem>
              <SideBar.SideBarItem
                logo={<ArrowLeftOnRectangleIcon />}
                onClick={() => {
                  oxidizedHealth.logout(window.location.origin);
                }}
              >
                Sign out
              </SideBar.SideBarItem>
            </SideBar.SideBarItemGroup>
          </SideBar.SideBar>
        }
      >
        <Page />
      </SideBar.SidebarLayout>
    </>
  );
}

function App() {
  return <RouterProvider router={router} />;
}

const root = ReactDOM.createRoot(
  document.getElementById("root") as HTMLElement
);

root.render(
  <React.StrictMode>
    <App />
  </React.StrictMode>
);

// If you want to start measuring performance in your app, pass a function
// to log results (for example: reportWebVitals(console.log))
// or send to an analytics endpoint. Learn more: https://bit.ly/CRA-vitals
reportWebVitals();
