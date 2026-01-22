import Link from "@docusaurus/Link";
import { useEffect, useRef } from "react";

export default function HealthcareDataFlow() {
  const containerRef = useRef<HTMLDivElement>(null);
  const svgRef = useRef<SVGSVGElement>(null);

  const connections = [
    ["ehr", "center"],
    ["app", "center"],
    ["cli", "center"],
    ["center", "web"],
    ["center", "backend"],
    ["center", "ai"],
  ];

  const drawLines = () => {
    if (!containerRef.current || !svgRef.current) return;

    const container = containerRef.current.getBoundingClientRect();
    const svg = svgRef.current;

    svg.innerHTML = `
      <defs>
        <marker id="arrow" markerWidth="10" markerHeight="10" refX="9" refY="3" orient="auto">
          <polygon points="0 0, 10 3, 0 6" fill="#ea580c"/>
        </marker>
      </defs>
    `;

    const centerPoint = (el: HTMLElement) => {
      const box = el.getBoundingClientRect();
      return {
        x: box.left - container.left + box.width / 2,
        y: box.top - container.top + box.height / 2,
      };
    };

    connections.forEach(([from, to]) => {
      const a = document.getElementById(from);
      const b = document.getElementById(to);
      if (!a || !b) return;

      const p1 = centerPoint(a);
      const p2 = centerPoint(b);

      const line = document.createElementNS(
        "http://www.w3.org/2000/svg",
        "line",
      );
      line.setAttribute("x1", String(p1.x));
      line.setAttribute("y1", String(p1.y));
      line.setAttribute("x2", String(p2.x));
      line.setAttribute("y2", String(p2.y));
      line.setAttribute("stroke", "#ea580c");
      line.setAttribute("stroke-width", "3");
      line.setAttribute("marker-end", "url(#arrow)");
      line.setAttribute("class", "flow-line");

      svg.appendChild(line);
    });
  };

  useEffect(() => {
    drawLines();
    window.addEventListener("resize", drawLines);
    return () => window.removeEventListener("resize", drawLines);
  }, []);

  return (
    <div
      ref={containerRef}
      className="relative w-full h-[250px] mx-auto  rounded-xl"
    >
      {/* SVG Overlay */}
      <svg
        ref={svgRef}
        className="absolute inset-0 w-full h-full pointer-events-none"
      />

      {/* Center */}
      <div
        id="center"
        className="absolute left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2"
      >
        <Link to="/docs/overview/what_is_haste_health">
          <div className="pulse cursor-pointer bg-orange-50 hover:bg-orange-100 rounded-2xl shadow-xl w-40 h-40 flex items-center justify-center border border-orange-200">
            <img src="/img/logo.svg" alt="Haste Health" className="h-30" />
          </div>
        </Link>
      </div>

      {/* Inputs */}
      <div className="left-0 absolute top-[calc(0%-38px)]">
        <Link to="/docs/integration/healthcare_systems/ehr">
          <Endpoint
            id="ehr"
            icon="🏥"
            title="EHR System"
            subtitle="Epic, Cerner etc..."
          />
        </Link>
      </div>
      <div className="left-0 absolute top-[calc(50%-38px)]">
        <Link to="/docs/category/fhir-compatibility">
          <Endpoint
            id="app"
            icon="🔥"
            title="FHIR"
            subtitle="FHIR compatible servers or apis."
          />
        </Link>
      </div>
      <div className="left-0 absolute top-[calc(100%-38px)]">
        <Link to="/docs/tutorials/cli">
          <Endpoint
            id="cli"
            icon="💻"
            title="Developer access"
            subtitle="Automated scripts and tools."
          />
        </Link>
      </div>

      {/* Outputs */}
      <div className="absolute  left-[calc(100%-220px)] top-[calc(0%-38px)]">
        <Link to="/docs/api/authentication/grant_types/authorization_code">
          <Endpoint
            id="web"
            icon="🌐"
            title="Web Application"
            subtitle="Patient Portals, Dashboards, Analytics."
          />
        </Link>
      </div>
      <div className="absolute  left-[calc(100%-220px)] top-[calc(50%-38px)]">
        <Link to="/docs/api/authentication/grant_types/client_credentials">
          <Endpoint
            id="backend"
            icon="⚙️"
            title="Backend Services"
            subtitle="External web services."
          />
        </Link>
      </div>
      <div className="absolute  left-[calc(100%-220px)] top-[calc(100%-38px)]">
        <Link to="/docs/category/ai">
          <Endpoint
            id="ai"
            icon="🤖"
            title="AI"
            subtitle="Feed and query data in AI Applications."
          />
        </Link>
      </div>

      {/* Animations */}
      <style>{`
        .flow-line {
          stroke-dasharray: 12 10;
          animation: dash 34s linear infinite;
        }

        @keyframes dash {
          to {
            stroke-dashoffset: -1000;
          }
        }

        .pulse {
          animation: pulse 12.5s infinite;
        }

        @keyframes pulse {
          0%,100% { transform: scale(1); }
          50% { transform: scale(1.06); }
        }
      `}</style>
    </div>
  );
}

function Endpoint({
  id,
  icon,
  title,
  subtitle,
}: Readonly<{
  id: string;
  icon: string;
  title: string;
  subtitle: string;
}>) {
  return (
    <div
      id={id}
      className="cursor-pointer hover:bg-orange-100 bg-orange-50 rounded-xl shadow-md p-4 w-64 flex items-center gap-4 border border-orange-200"
    >
      <div className="text-4xl">{icon}</div>
      <div>
        <div className="font-semibold text-orange-900">{title}</div>
        <div className="text-sm text-orange-900">{subtitle}</div>
      </div>
    </div>
  );
}
