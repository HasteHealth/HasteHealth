import { Bars3Icon } from "@heroicons/react/24/outline";
import classNames from "classnames";
import { useAtom, useAtomValue } from "jotai";
import { EditorView } from "@codemirror/view";
import React, { useEffect } from "react";
import ReactDOM from "react-dom/client";
import {
  Link,
  Navigate,
  Outlet,
  RouterProvider,
  createBrowserRouter,
  generatePath,
  useLocation,
  useNavigate,
  useParams,
} from "react-router";

import {
  HasteHealthProvider,
  Loading,
  ProfileDropdown,
  Toaster,
  useHasteHealth,
} from "@haste-health/components";

import reportWebVitals from "./reportWebVitals";
import Search from "./components/Search";
import SearchModal from "./components/SearchModal";
import { VITE_CLIENT_ID, VITE_FHIR_BASE_URL } from "./config";
import { createAdminAppClient, getClient } from "./db/client";
import { StructureCache, getStructures } from "./db/structures";
import { useTenantBranding } from "./hooks/useTenantBranding";

import BundleImport from "./views/Project/BundleImport";
import EmptyWorkspace from "./views/Project/EmptyWorkspace";
import IndexingErrors from "./views/Project/IndexingErrors";
import ResourceEditor from "./views/ResourceEditor/index";
import Console from "./views/Console";
import { ConsoleSidebar } from "./views/Console/ConsoleSidebar";
import { getCapabilities } from "./db/capabilities";
import { Shortcut, useShortcuts } from "./hooks/useShortcuts";
import { ShortcutHelp } from "./components/ShortcutHelp";
import Settings from "./views/Project/Settings";
import Projects from "./views/System/Projects";
// import ViewDefinitionEditor from "./views/Analytics/ViewDefinitionEditor";
import { deriveProjectId, deriveTenantId } from "./utilities";
import * as r4Types from "@haste-health/fhir-types/r4/types";
import SystemResources from "./views/System";
import { ProjectInformation } from "@haste-health/generated-ops/r4";
import { R4 } from "@haste-health/fhir-types/versions";
import { AppLogo } from "./components/AppLogo";

import "@haste-health/components/dist/index.css";
import "./index.css";

const capitalize = (s: string) => s.charAt(0).toUpperCase() + s.slice(1);

function LoginWrapper() {
  const hasteHealth = useHasteHealth({ searchMethod: "POST" });

  return (
    <>
      {hasteHealth.loading ? (
        <div className="h-screen flex flex-1 justify-center items-center flex-col">
          <Loading />
          <div className="mt-1 ">Loading...</div>
        </div>
      ) : hasteHealth.error ? (
        <div className="h-screen flex">
          <div className="flex-1 flex items-center justify-center">
            <div className="p-4 bg-red-100 text-red-800 border border-red-400 rounded-md space-y-2">
              <div className="font-bold">
                {hasteHealth.error.code.split("_").map(capitalize).join(" ")}
              </div>
              <div>{hasteHealth.error.description}</div>
            </div>
          </div>
        </div>
      ) : (
        <div className="flex flex-1">
          <Outlet />
        </div>
      )}
    </>
  );
}

function ServiceSetup() {
  const hasteHealth = useHasteHealth({ searchMethod: "POST" });
  const client = hasteHealth.isAuthenticated ? hasteHealth.client : undefined;
  const [c, setClient] = useAtom(getClient);
  const [, setStructures] = useAtom(getStructures);

  React.useEffect(() => {
    if (client) {
      const adminClient = createAdminAppClient(client);
      setClient(adminClient);
      // The inspector reads element definitions through this cache.
      setStructures(new StructureCache(adminClient));
    }
  }, [
    setClient,
    setStructures,
    hasteHealth.isAuthenticated,
    hasteHealth.client,
  ]);

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

function HasteHealthWrapper() {
  const navigate = useNavigate();

  return (
    <HasteHealthProvider
      refresh
      authorize_method="GET"
      scope="offline_access openid email profile fhirUser system/*.*"
      domain={VITE_FHIR_BASE_URL || ""}
      tenant={deriveTenantId()}
      project={deriveProjectId()}
      clientId={VITE_CLIENT_ID}
      redirectUrl={window.location.origin}
      onRedirectCallback={(initialPath: string) => {
        navigate(initialPath);
      }}
    >
      <Outlet />
    </HasteHealthProvider>
  );
}

/**
 * The system console's tabs. Projects leads: opening one is what a new user
 * is here to do, while the other two administer access to it.
 */
const SYSTEM_TABS: {
  type: r4Types.ResourceType;
  label: string;
}[] = [
  { type: "Project", label: "Projects" },
  { type: "User", label: "Users" },
  { type: "IdentityProvider", label: "Identity providers" },
];

/** The types this console manages, which is also what its search covers. */
const SYSTEM_TYPES: r4Types.ResourceType[] = SYSTEM_TABS.map(
  ({ type }) => type,
);

const APP_HEADER_HEIGHT_CLASS = "h-16";
const APP_HEADER_OFFSET = "4rem";

function SystemBar() {
  const params = useParams();

  return (
    <div className="flex w-full flex-col overflow-y-auto z-10">
      {/* Quiet, so it does not compete with the create call to action. */}
      <nav className="mb-6 flex shrink-0 gap-6 border-b border-slate-200">
        {SYSTEM_TABS.map(({ type, label }) => (
          <Link
            key={type}
            to={`/r/${type}`}
            className={classNames(
              "-mb-px border-b-2 px-1 pb-3 text-sm transition-colors",
              {
                ["border-brand-600 font-semibold text-brand-700"]:
                  params.resourceType === type,
                ["border-transparent text-slate-500 hover:border-slate-300 hover:text-slate-800"]:
                  params.resourceType !== type,
              },
            )}
          >
            {label}
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
          id: "haste-health-wrapper",
          element: <HasteHealthWrapper />,
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
                      element: (
                        <div className="flex flex-col w-screen">
                          <Navbar />
                          <Page resourceTypeFilter={SYSTEM_TYPES} />
                        </div>
                      ),

                      children: [
                        {
                          id: "root",
                          element: <SystemBar />,
                          children: [
                            {
                              id: "Resources",
                              path: "/r/:resourceType",
                              element: <SystemResources />,
                            },
                            {
                              id: "Editor",
                              path: "/r/:resourceType/:id",
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
                              element: <Navigate to="/r/Project" replace />,
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
          id: "haste-health-wrapper",
          element: <HasteHealthWrapper />,
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
                          id: "console",
                          path: "",
                          element: <Console />,
                        },
                        {
                          // Everything addressable is a FHIR path under /r/.
                          id: "console-command",
                          path: "r/*",
                          element: <Console />,
                        },
                        {
                          id: "bundle-import",
                          path: "import",
                          element: <BundleImport />,
                        },
                        {
                          id: "indexing-errors",
                          path: "indexing-errors",
                          element: <IndexingErrors />,
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

function Navbar({ showLogo = true }: Readonly<{ showLogo?: boolean }>) {
  const hasteHealth = useHasteHealth({ searchMethod: "POST" });
  const navigate = useNavigate();

  return (
    <div className="px-4 sticky top-0 bg-white border-b z-20 text-sm">
      <div className={`flex items-center ${APP_HEADER_HEIGHT_CLASS}`}>
        {showLogo && (
          <AppLogo
            className="h-9 mr-4 cursor-pointer text-brand-500"
            onClick={() => {
              navigate(generatePath("/", {}));
            }}
          />
        )}
        <div className="flex grow"></div>
        <div className="flex justify-center items-center space-x-8">
          <div className="min-w-72 flex grow">
            <Search />
          </div>
          <a
            target="_blank"
            className="cursor text-slate-500 hover:text-slate-600 hover:underline"
            href="https://haste.health"
          >
            Documentation
          </a>
          <ProfileDropdown
            user={{
              email: hasteHealth.user?.email,
              name: hasteHealth.user?.given_name || hasteHealth.user?.email,
              // imageUrl: auth0.user?.picture,
            }}
          >
            <div>
              <div className="mt-2">
                <button
                  className="block w-full cursor-pointer px-4 py-2 text-left text-sm hover:bg-brand-200 hover:text-brand-800"
                  onClick={() => {
                    navigate(generatePath("/settings", {}));
                  }}
                  type="button"
                >
                  Settings
                </button>
                <button
                  className="block w-full cursor-pointer px-4 py-2 text-left text-sm text-slate-800 hover:bg-brand-200 hover:text-brand-800"
                  onClick={() => {
                    hasteHealth.logout(window.location.origin);
                  }}
                  type="button"
                >
                  Sign out
                </button>
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
    <div className="px-6">
      <div
        className="py-6 flex flex-1"
        style={{ height: `calc(100vh - ${APP_HEADER_OFFSET})` }}
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
  const hasteHealth = useHasteHealth({ searchMethod: "POST" });
  const navigate = useNavigate();
  const location = useLocation();
  const [project, setProject] = React.useState<r4Types.Project | null>(null);
  const [sidebarOpen, setSidebarOpen] = React.useState(false);
  const [shortcutsOpen, setShortcutsOpen] = React.useState(false);
  const capabilities = useAtomValue(getCapabilities);
  const branding = useTenantBranding();
  const tenantDisplayName = branding.name || hasteHealth.tenant;

  useEffect(() => {
    hasteHealth.client
      .invoke_system(ProjectInformation.Op, {}, R4, {})
      .then((res) => setProject(res.project))
      .catch(() => setProject(null));
  }, []);

  const resourceTypes = React.useMemo(
    () =>
      (capabilities?.rest?.[0]?.resource ?? [])
        .map((entry) => entry.type as string)
        .filter(Boolean)
        .sort((a, b) => a.localeCompare(b)),
    [capabilities],
  );

  // The type in view, so the sidebar can mark it.
  const activeType = React.useMemo(() => {
    const match = /^\/r\/([A-Z][A-Za-z0-9]*)/.exec(location.pathname);
    return match?.[1];
  }, [location.pathname]);

  const go = React.useCallback(
    (path: string) => {
      navigate(path);
      setSidebarOpen(false);
    },
    [navigate],
  );

  /** Moves focus to the first element matching `selector`. */
  const focusFirst = React.useCallback((selector: string) => {
    const target = document.querySelector<HTMLElement>(selector);
    target?.focus();
  }, []);

  /**
   * Focuses the command bar and selects what is in it, so the next keystroke
   * replaces the query the way a browser's address bar does.
   *
   * The bar is a CodeMirror editor, which owns its selection, so this goes
   * through the view rather than the DOM.
   */
  const focusCommandBar = React.useCallback(() => {
    const dom = document.querySelector<HTMLElement>("[data-command-input]");
    if (!dom) return;
    const view = EditorView.findFromDOM(dom);
    if (!view) {
      dom.querySelector<HTMLElement>(".cm-content")?.focus();
      return;
    }
    view.focus();
    view.dispatch({
      selection: { anchor: 0, head: view.state.doc.length },
    });
  }, []);

  const shortcuts = React.useMemo(
    (): Shortcut[] => [
      {
        key: "k",
        mod: true,
        description: "Focus the command bar",
        whileTyping: true,
        run: focusCommandBar,
      },
      {
        key: "r",
        mod: true,
        shift: true,
        description: "Jump to the results",
        whileTyping: true,
        run: () => focusFirst("[data-table-row]"),
      },
      {
        key: "b",
        mod: true,
        description: "Jump to the sidebar",
        whileTyping: true,
        run: () => focusFirst("[data-sidebar] button"),
      },
      // These carry the modifier because the command bar holds focus for
      // most of a session, and a bare letter would be typed into it.
      {
        key: "g",
        mod: true,
        shift: true,
        description: "Go to the console",
        run: () => go("/"),
      },
      {
        key: "h",
        mod: true,
        shift: true,
        description: "Go to event history",
        run: () => go("/r/_history"),
      },
      {
        key: "i",
        mod: true,
        shift: true,
        description: "Go to import bundle",
        run: () => go("/import"),
      },
      {
        key: "e",
        mod: true,
        shift: true,
        description: "Go to indexing errors",
        run: () => go("/indexing-errors"),
      },
      {
        key: "p",
        mod: true,
        shift: true,
        description: "Go to projections",
        run: () => go("/r/ViewDefinition"),
      },
      {
        key: ",",
        mod: true,
        description: "Go to settings",
        run: () => go("/settings"),
      },
      {
        // Not `?`: that is the separator before a search, so the bar has to
        // keep it. `/` with the modifier is the usual binding for help and
        // is never typed alone in a FHIR path.
        key: "/",
        mod: true,
        description: "Show this help",
        run: () => setShortcutsOpen((open) => !open),
      },
      {
        key: "escape",
        description: "Close help or the sidebar drawer",
        whileTyping: true,
        run: () => {
          setShortcutsOpen(false);
          setSidebarOpen(false);
        },
      },
    ],
    [focusFirst, go],
  );

  useShortcuts(shortcuts);

  const sidebar = (
    <ConsoleSidebar
      resourceTypes={resourceTypes}
      activeType={activeType}
      activePath={location.pathname}
      onNavigate={go}
      onShowShortcuts={() => setShortcutsOpen(true)}
    />
  );

  return (
    // `w-full` and `min-w-0` because the shell is a flex item inside the
    // login wrapper: without them it is sized by its content and leaves the
    // right of the screen empty.
    <div className="flex h-screen w-full min-w-0 flex-col overflow-hidden bg-slate-100">
      <header className="flex h-12 shrink-0 items-center gap-2 border-b border-slate-200 bg-white px-3">
        {/* Below `lg` the sidebar is a drawer rather than a column. */}
        <button
          className="rounded p-1 text-slate-500 hover:bg-slate-100 lg:hidden"
          onClick={() => setSidebarOpen((open) => !open)}
          title="Toggle navigation"
          type="button"
        >
          <Bars3Icon className="h-5 w-5" />
        </button>

        <AppLogo
          className="h-7 w-7 shrink-0 cursor-pointer text-brand-500"
          onClick={() => go("/")}
        />
        <button
          className="min-w-0 truncate text-left text-sm font-semibold text-slate-800 hover:underline"
          onClick={() => go("/")}
          type="button"
        >
          {tenantDisplayName}
          {project?.name && (
            <span className="ml-1 font-normal text-slate-400">
              / {project.name}
            </span>
          )}
        </button>

        <div className="flex flex-1 justify-end items-center gap-4">
          <a
            className="hidden text-xs text-slate-500 hover:text-slate-700 hover:underline sm:block"
            href="https://haste.health"
            rel="noreferrer"
            target="_blank"
          >
            Docs
          </a>
          <ProfileDropdown
            user={{
              email: hasteHealth.user?.email,
              name: hasteHealth.user?.given_name || hasteHealth.user?.email,
            }}
          >
            <div className="mt-2">
              <button
                className="block w-full cursor-pointer px-4 py-2 text-left text-sm hover:bg-brand-200 hover:text-brand-800"
                onClick={() => go("/settings")}
                type="button"
              >
                Settings
              </button>
              <button
                className="block w-full cursor-pointer px-4 py-2 text-left text-sm text-slate-800 hover:bg-brand-200 hover:text-brand-800"
                onClick={() => hasteHealth.logout(window.location.origin)}
                type="button"
              >
                Sign out
              </button>
            </div>
          </ProfileDropdown>
        </div>
      </header>

      <div className="flex min-h-0 flex-1">
        <div className="hidden lg:flex">{sidebar}</div>

        {sidebarOpen && (
          <div className="fixed inset-0 z-40 flex lg:hidden">
            <div
              className="absolute inset-0 bg-slate-900/30"
              onClick={() => setSidebarOpen(false)}
            />
            <div className="relative h-full">{sidebar}</div>
          </div>
        )}

        <main className="flex min-h-0 min-w-0 flex-1 flex-col p-3">
          <Toaster.Toaster />
          <React.Suspense
            fallback={
              <div className="flex flex-1 items-center justify-center">
                <Loading />
              </div>
            }
          >
            <Outlet />
          </React.Suspense>
        </main>
      </div>

      {shortcutsOpen && (
        <ShortcutHelp
          shortcuts={shortcuts}
          onClose={() => setShortcutsOpen(false)}
        />
      )}
    </div>
  );
}

function App() {
  return <RouterProvider router={router} />;
}

const root = ReactDOM.createRoot(
  document.getElementById("root") as HTMLElement,
);

root.render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);

// If you want to start measuring performance in your app, pass a function
// to log results (for example: reportWebVitals(console.log))
// or send to an analytics endpoint. Learn more: https://bit.ly/CRA-vitals
reportWebVitals();
