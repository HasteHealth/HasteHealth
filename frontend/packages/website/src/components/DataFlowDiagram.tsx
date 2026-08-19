import Link from "@docusaurus/Link";

const sources = [
  { href: "/docs/integration/healthcare_systems/ehr", icon: "🏥", label: "Epic" },
  { href: "/docs/integration/healthcare_systems/ehr", icon: "🩺", label: "Cerner" },
  { href: "/docs/integration/healthcare_systems/ehr", icon: "📋", label: "Meditech" },
  { href: "/docs/integration/healthcare_systems/ehr", icon: "⚕️", label: "athenahealth" },
  { href: "/docs/guides/hl7v2", icon: "📨", label: "HL7v2 Feeds" },
  { href: "/docs/api/rest_api/fhir/intro", icon: "🔥", label: "Any FHIR API" },
];

const consumers = [
  { href: "/docs/integration/ai/claude", icon: "💬", label: "Claude" },
  { href: "/docs/api/rest_api/model_context_protocol/tools", icon: "🤖", label: "Any MCP Agent" },
  { href: "/docs/auth/authentication/grant_types/authorization_code", icon: "🌐", label: "Apps & Portals" },
  { href: "/docs/auth/authentication/grant_types/client_credentials", icon: "⚙️", label: "Backend Services" },
  { href: "/docs/guides/sql_on_fhir", icon: "📊", label: "Analytics & BI" },
  { href: "/docs/api/rest_api/fhir/intro", icon: "🔗", label: "FHIR Partners" },
];

export default function DataFlowDiagram() {
  return (
    <div className="grid grid-cols-1 gap-6 md:grid-cols-[minmax(0,1fr)_2rem_auto_2rem_minmax(0,1fr)] md:items-center md:gap-3">
      <ChipGroup label="Comes in from" items={sources} align="right" />

      <Connector />

      <div className="flex justify-center">
        <div className="flex w-40 flex-col items-center justify-center gap-1 rounded-2xl border border-brand-200 bg-brand-50 p-5 text-center shadow-md">
          <img src="/img/logo.svg" alt="Haste Health" className="h-12" />
          <span className="text-xs font-semibold uppercase tracking-[0.08em] text-brand-800">
            Headless FHIR R4 API
          </span>
        </div>
      </div>

      <Connector />

      <ChipGroup label="Goes out to" items={consumers} align="left" />
    </div>
  );
}

function Connector() {
  return (
    <>
      <div className="hidden md:flex md:h-1 md:w-8 md:items-center md:justify-self-center md:overflow-hidden md:rounded-full md:bg-brand-100">
        <div className="beam-h h-full w-1/3 rounded-full bg-brand-500" />
      </div>
      <div className="flex h-8 w-1 justify-self-center overflow-hidden rounded-full bg-brand-100 md:hidden">
        <div className="beam-v h-1/3 w-full rounded-full bg-brand-500" />
      </div>
      <style>{`
        .beam-h { animation: beam-move-h 1.8s linear infinite; }
        @keyframes beam-move-h {
          0% { transform: translateX(-100%); }
          100% { transform: translateX(400%); }
        }
        .beam-v { animation: beam-move-v 1.8s linear infinite; }
        @keyframes beam-move-v {
          0% { transform: translateY(-100%); }
          100% { transform: translateY(400%); }
        }
        @media (prefers-reduced-motion: reduce) {
          .beam-h, .beam-v { animation: none; }
        }
      `}</style>
    </>
  );
}

function ChipGroup({
  label,
  items,
  align,
}: Readonly<{
  label: string;
  items: { href: string; icon: string; label: string }[];
  align: "left" | "right";
}>) {
  return (
    <div className="flex flex-col gap-3">
      <p
        className={`text-center text-xs font-semibold uppercase tracking-[0.08em] text-slate-500 ${
          align === "right" ? "md:text-right" : "md:text-left"
        }`}
      >
        {label}
      </p>
      <div
        className={`mx-auto grid w-full max-w-sm grid-cols-2 gap-2.5 ${
          align === "right" ? "md:ml-auto md:mr-0" : "md:mr-auto md:ml-0"
        }`}
      >
        {items.map((item) => (
          <Link key={item.label} to={item.href} className="block">
            <Chip icon={item.icon} label={item.label} />
          </Link>
        ))}
      </div>
    </div>
  );
}

function Chip({
  icon,
  label,
}: Readonly<{ icon: string; label: string }>) {
  return (
    <div className="flex cursor-pointer items-center gap-2 rounded-lg border border-brand-200 bg-brand-50 px-2.5 py-2 shadow-sm hover:bg-brand-100">
      <div className="w-5 shrink-0 text-center text-base leading-none">{icon}</div>
      <div className="truncate text-xs font-medium leading-tight text-brand-900">
        {label}
      </div>
    </div>
  );
}
