import Link from "@docusaurus/Link";

type Chip = { label: string; href?: string };

const SYSTEM_PROJECT_CHIPS: Chip[] = [
  { label: "IdentityProvider", href: "/docs/reference/fhir/model/resources/IdentityProvider" },
  { label: "User", href: "/docs/reference/fhir/model/resources/User" },
  { label: "Project (metadata for every project)", href: "/docs/reference/fhir/model/resources/Project" },
];

const USER_PROJECT_CHIPS: Chip[] = [
  { label: "AccessPolicyV2", href: "/docs/reference/fhir/model/resources/AccessPolicyV2" },
  { label: "ClientApplication", href: "/docs/reference/fhir/model/resources/ClientApplication" },
  { label: "Membership", href: "/docs/reference/fhir/model/resources/Membership" },
  { label: "Patient, Observation, …" },
];

export function TenantHierarchyDiagram() {
  return (
    <div className="not-prose mb-6 rounded-2xl border border-slate-200 bg-slate-50/70 p-5 md:p-6">
      <div className="flex flex-wrap items-center gap-2">
        <span className="inline-flex items-center rounded-full border border-slate-200 bg-white px-3 py-1 text-xs font-semibold uppercase tracking-[0.14em] text-brand-800">
          Tenant
        </span>
        <span className="font-mono text-sm text-slate-600">acme-health</span>
      </div>

      <div className="mt-4 grid gap-4 md:grid-cols-3">
        <ProjectCard title="System Project" badge="Reserved · id: system" accent chips={SYSTEM_PROJECT_CHIPS} />
        <ProjectCard title="Project: production" chips={USER_PROJECT_CHIPS} />
        <ProjectCard title="Project: staging" chips={USER_PROJECT_CHIPS} />
      </div>

      <p className="mt-4 text-xs text-slate-500">
        Every tenant gets exactly one System Project. It holds identity,
        login config, and the Project record for every project in the
        tenant; access rules, OAuth clients, and clinical data all live in
        regular projects like the two on the right.
      </p>
    </div>
  );
}

function ProjectCard({
  title,
  badge,
  chips,
  accent,
}: Readonly<{
  title: string;
  badge?: string;
  chips: Chip[];
  accent?: boolean;
}>) {
  return (
    <div
      className={`rounded-xl border p-4 ${
        accent
          ? "border-brand-500 bg-white shadow-sm"
          : "border-slate-200 bg-white"
      }`}
    >
      <div className="text-sm font-semibold text-ink-950">{title}</div>
      {badge ? (
        <div className="mt-0.5 text-[11px] font-medium uppercase tracking-wide text-brand-800">
          {badge}
        </div>
      ) : null}
      <div className="mt-3 flex flex-col gap-1.5">
        {chips.map((chip) =>
          chip.href ? (
            <Link
              key={chip.label}
              to={chip.href}
              className="rounded-md border border-slate-200 bg-white px-2.5 py-1.5 text-xs font-medium text-slate-800 no-underline transition-colors hover:border-brand-500 hover:text-ink-950 hover:no-underline"
            >
              {chip.label}
            </Link>
          ) : (
            <div
              key={chip.label}
              className="rounded-md border border-dashed border-slate-200 px-2.5 py-1.5 text-xs text-slate-500"
            >
              {chip.label}
            </div>
          ),
        )}
      </div>
    </div>
  );
}

const FLOW_STEPS = [
  { title: "Authenticate", body: "User signs in through that project's OAuth client." },
  { title: "Verify", body: "Credentials and Membership are checked." },
  { title: "Issue token", body: "Token is scoped to accessible project(s)." },
  { title: "Evaluate", body: "AccessPolicyV2 in that project is applied." },
  { title: "Respond", body: "FHIR response is returned." },
];

export function RequestFlowDiagram() {
  return (
    <div className="not-prose my-6 flex flex-col gap-2 md:flex-row md:items-stretch md:gap-2">
      {FLOW_STEPS.map((step, i) => (
        <div key={step.title} className="flex flex-1 flex-col items-stretch gap-2 md:flex-row md:items-center">
          <div className="flex-1 rounded-xl border border-slate-200 bg-white p-4 shadow-[0_1px_2px_rgb(15_23_42/0.04)]">
            <div className="flex items-center gap-2">
              <span className="flex h-5 w-5 shrink-0 items-center justify-center rounded-full bg-brand-800 text-[11px] font-bold text-white">
                {i + 1}
              </span>
              <span className="text-sm font-semibold text-ink-950">
                {step.title}
              </span>
            </div>
            <p className="mt-1.5 text-xs leading-5 text-slate-600">
              {step.body}
            </p>
          </div>
          {i < FLOW_STEPS.length - 1 ? (
            <>
              <span className="hidden shrink-0 text-lg text-slate-400 md:block" aria-hidden="true">
                →
              </span>
              <span className="block shrink-0 rotate-90 text-lg text-slate-400 md:hidden" aria-hidden="true">
                →
              </span>
            </>
          ) : null}
        </div>
      ))}
    </div>
  );
}
