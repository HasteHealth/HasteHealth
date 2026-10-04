import React, { Fragment, ReactNode } from "react";

/** Colours JSON keys, and dims `//` comment lines. */
function ColorizedJson({ code }: Readonly<{ code: string }>) {
  const nodes: ReactNode[] = [];
  const keyPattern = /"([^"]+)":/g;
  let i = 0;

  for (const line of code.split("\n")) {
    if (line.trimStart().startsWith("//")) {
      nodes.push(
        <span key={i++} className="text-slate-400">
          {line}
        </span>,
        "\n",
      );
      continue;
    }

    let lastIndex = 0;
    let match: RegExpExecArray | null;
    keyPattern.lastIndex = 0;
    while ((match = keyPattern.exec(line)) !== null) {
      if (match.index > lastIndex) {
        nodes.push(
          <Fragment key={i++}>{line.slice(lastIndex, match.index)}</Fragment>,
        );
      }
      nodes.push(
        <span key={i++} className="text-brand-300">{`"${match[1]}"`}</span>,
        <Fragment key={i++}>:</Fragment>,
      );
      lastIndex = keyPattern.lastIndex;
    }
    nodes.push(<Fragment key={i++}>{line.slice(lastIndex)}</Fragment>, "\n");
  }

  // Drop the newline after the last line.
  nodes.pop();
  return <>{nodes}</>;
}

// The tool's real input shape: see static/mcp/tools.json.
const request = `{
  "method": "tools/call",
  "params": {
    "name": "fhir_r4_search",
    "arguments": {
      "resourceType": "Observation",
      "search_parameters": {
        "code": "85354-9",
        "date": "ge2026-09-04",
        "patient": "kin5pxcy85546rjhqjwzk9l9bn"
      }
    }
  }
}`;

const response = `{
  "resourceType": "Bundle",
  "type": "searchset",
  "total": 4,
  "entry": [{
    "resource": {
      "resourceType": "Observation",
      "id": "f3n8wq2ktz6m1yc9d4hv7xbj5a",
      "status": "final",
      "code": { "coding": [
        { "system": "http://loinc.org", "code": "85354-9" }
      ] },
      "subject": { "reference": "Patient/kin5pxcy85546rjhqjwzk9l9bn" },
      "effectiveDateTime": "2026-10-02T09:12:00Z",
      "component": [{
        "code": { "coding": [
          { "system": "http://loinc.org", "code": "8480-6" }
        ] },
        "valueQuantity": { "value": 122, "unit": "mmHg" }
      }
      // + diastolic (LOINC 8462-4), 80 mmHg
      ]
    }
  }
  // + 3 more entries, same shape
  ]
}`;

function Code({ code }: Readonly<{ code: string }>) {
  return (
    <pre className="m-0 overflow-x-auto rounded-none border-0 bg-transparent p-0 font-mono text-[0.8125rem] leading-[1.7] text-slate-300">
      <code className="border-0 bg-transparent p-0 font-mono text-[length:inherit] text-inherit">
        <ColorizedJson code={code} />
      </code>
    </pre>
  );
}

function PaneLabel({ children }: Readonly<{ children: ReactNode }>) {
  return (
    <div className="text-[0.75rem] font-semibold uppercase tracking-[0.14em] text-slate-400">
      {children}
    </div>
  );
}

/** One MCP tool call, as the agent sends it and as the server answers it. */
export default function McpPreview() {
  return (
    <div className="overflow-hidden rounded-2xl border border-white/10 bg-ink-950 text-left shadow-[0_30px_70px_-30px_rgb(0_0_0/0.7)]">
      <div className="flex flex-wrap items-center gap-x-4 gap-y-1 border-b border-white/10 bg-white/[0.03] px-5 py-3.5 font-mono text-[0.8125rem]">
        <span className="font-semibold text-brand-300">POST</span>
        <span className="text-slate-300 [overflow-wrap:anywhere]">
          /w/acme-health/production/api/v1/mcp
        </span>
        <span className="ml-auto whitespace-nowrap text-slate-400">
          JSON-RPC 2.0
        </span>
      </div>

      {/* min-w-0 on each pane: a grid item is otherwise as wide as its
          longest line of code, and pushes the window off a phone screen. */}
      <div className="grid md:grid-cols-[minmax(0,5fr)_minmax(0,7fr)]">
        <div className="min-w-0 space-y-6 border-b border-white/10 p-5 md:border-b-0 md:border-r md:p-7">
          <div className="space-y-3">
            <PaneLabel>Agent asks</PaneLabel>
            <p className="m-0 rounded-xl rounded-tl-sm bg-white/[0.07] px-4 py-3 text-[1.0625rem] leading-relaxed text-white">
              How has David Williams&apos; blood pressure trended over the last
              30 days?
            </p>
          </div>
          <div className="space-y-3">
            <PaneLabel>Which becomes one tool call</PaneLabel>
            <Code code={request} />
          </div>
          <div className="space-y-3">
            <PaneLabel>And one grounded answer</PaneLabel>
            <p className="m-0 rounded-xl rounded-tl-sm border border-brand-400/20 bg-brand-500/10 px-4 py-3 text-[1.0625rem] leading-relaxed text-white">
              Four readings since September 4, trending down from 134/88 to
              122/80 mmHg.
            </p>
          </div>
        </div>

        <div className="min-w-0 space-y-3 p-5 md:p-7">
          <div className="flex items-center justify-between gap-4">
            <PaneLabel>Haste Health responds</PaneLabel>
            <span className="inline-flex items-center gap-2 whitespace-nowrap font-mono text-[0.8125rem] text-slate-300">
              <span className="h-1.5 w-1.5 rounded-full bg-brand-400" />
              200 OK
            </span>
          </div>
          <Code code={response} />
        </div>
      </div>
    </div>
  );
}
