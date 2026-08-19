import React, { Fragment, ReactNode } from "react";

function ColorizedJson({ code }: Readonly<{ code: string }>) {
  const nodes: ReactNode[] = [];
  const keyPattern = /"([^"]+)":/g;
  let lastIndex = 0;
  let match: RegExpExecArray | null;
  let i = 0;

  while ((match = keyPattern.exec(code)) !== null) {
    if (match.index > lastIndex) {
      nodes.push(
        <Fragment key={i++}>{code.slice(lastIndex, match.index)}</Fragment>,
      );
    }
    nodes.push(
      <span key={i++} className="text-brand-300">{`"${match[1]}"`}</span>,
    );
    nodes.push(<Fragment key={i++}>:</Fragment>);
    lastIndex = keyPattern.lastIndex;
  }
  nodes.push(<Fragment key={i++}>{code.slice(lastIndex)}</Fragment>);

  return <>{nodes}</>;
}

const request = `{
  "tool": "fhir_r4_search",
  "arguments": {
    "resourceType": "Observation",
    "code": "85354-9",
    "date": "ge2026-07-19",
    "patient": "xufn84vpa69b…998c1"
  }
}`;

const response = `{
  "resourceType": "Bundle",
  "type": "searchset",
  "total": 1,
  "entry": [{
    "resource": {
      "resourceType": "Observation",
      "id": "bp-8f2a1c",
      "meta": { "profile": [
        "http://hl7.org/fhir/us/core/StructureDefinition/us-core-blood-pressure"
      ] },
      "status": "final",
      "category": [{ "coding": [
        { "system": ".../observation-category", "code": "vital-signs" }
      ] }],
      "code": { "coding": [
        { "system": "http://loinc.org", "code": "85354-9" }
      ] },
      "subject": { "reference": "Patient/xufn84vpa69b…998c1" },
      "effectiveDateTime": "2026-08-12T09:14:00Z",
      "component": [
        {
          "code": { "coding": [
            { "system": "http://loinc.org", "code": "8480-6" }
          ] },
          "valueQuantity": {
            "value": 128, "unit": "mmHg",
            "system": "http://unitsofmeasure.org", "code": "mm[Hg]"
          }
        }
        // + 1 more item: diastolic (LOINC 8462-4), same shape
      ]
    }
  }]
}`;

export default function McpPreview() {
  return (
    <div className="overflow-hidden rounded-2xl border border-brand-800 bg-brand-950 shadow-[0_20px_60px_rgba(15,23,42,0.25)]">
      <div className="flex items-center gap-2 border-b border-white/10 bg-white/5 px-4 py-3">
        <span className="h-3 w-3 rounded-full bg-red-400/70" />
        <span className="h-3 w-3 rounded-full bg-yellow-400/70" />
        <span className="h-3 w-3 rounded-full bg-green-400/70" />
        <span className="ml-3 text-xs font-medium text-brand-200">
          MCP · fhir_r4_search
        </span>
      </div>
      <div className="grid md:grid-cols-2">
        <div className="border-b border-white/10 p-5 md:border-b-0 md:border-r">
          <p className="mb-3 text-xs font-semibold uppercase tracking-[0.08em] text-brand-300">
            Agent asks
          </p>
          <p className="mb-4 rounded-lg bg-white/5 p-3 text-sm italic text-white leading-relaxed">
            "Find this patient's blood pressure readings from the last 30
            days."
          </p>
          <pre
            className="overflow-x-auto text-xs leading-6"
            style={{ background: "transparent", margin: 0, padding: 0, border: 0 }}
          >
            <code style={{ background: "transparent", color: "var(--color-brand-100)" }}>
              <ColorizedJson code={request} />
            </code>
          </pre>
        </div>
        <div className="p-5">
          <p className="mb-3 text-xs font-semibold uppercase tracking-[0.08em] text-brand-300">
            Haste Health responds
          </p>
          <pre
            className="overflow-x-auto text-xs leading-6"
            style={{ background: "transparent", margin: 0, padding: 0, border: 0 }}
          >
            <code style={{ background: "transparent", color: "var(--color-brand-100)" }}>
              <ColorizedJson code={response} />
            </code>
          </pre>
        </div>
      </div>
    </div>
  );
}
