/* eslint @typescript-eslint/no-explicit-any: 0 */
import { json } from "@codemirror/lang-json";
import { PlayIcon } from "@heroicons/react/24/solid";
import classNames from "classnames";
import { basicSetup } from "codemirror";
import React, { useEffect, useMemo, useState } from "react";

import { resourceTypes } from "@haste-health/fhir-types/r4/sets";
import {
  OperationDefinition,
  Parameters,
} from "@haste-health/fhir-types/r4/types";

import { Button } from "../../base/button";
import { CodeMirror } from "../../base/codemirror";
import { Input } from "../../base/input";
import { Loading } from "../../base/loading";
import { Select } from "../../base/select";
import { Outcome, OutcomePanel, toOutcome } from "../resources/OutcomePanel";
import { ClientProps } from "../types";
import { OperationParametersEditor } from "./OperationParametersEditor";
import { OperationParametersView } from "./OperationParametersView";
import { Segmented } from "./controls";
import {
  InvocationLevel,
  InvocationTarget,
  invocationLevels,
  invocationPath,
  isTargetComplete,
  missingRequired,
  parameterDefinitions,
  pruneParameters,
} from "./parameters";

const jsonExtensions = [basicSetup, json()];

const LEVELS: { value: InvocationLevel; label: string }[] = [
  { value: "system", label: "System" },
  { value: "type", label: "Type" },
  { value: "instance", label: "Instance" },
];

export type OperationInvocationPanelProps = ClientProps & {
  /** The operation to invoke. Its parameters, levels and resource types drive the panel. */
  operationDefinition: OperationDefinition;
  /**
   * Runs an invocation. By default the input is posted to the client's FHIR
   * base by the operation's code, at the chosen level.
   */
  invoke?: (target: InvocationTarget, input: Parameters) => Promise<unknown>;
  /** Shown under the Run button, e.g. which version of the operation runs. */
  runHint?: React.ReactNode;
  /**
   * Why the operation cannot be run yet, e.g. that it has not been saved.
   * When set, Run is disabled and this is shown in its place; the parameters
   * can still be filled in.
   */
  disabledReason?: string;
};

type Result =
  | { status: "idle" }
  | { status: "running" }
  | { status: "success"; output: unknown; ms: number }
  | { status: "failure"; outcome: Outcome; ms: number; path: string };

function SectionTitle({ children }: Readonly<{ children: React.ReactNode }>) {
  return (
    <span className="text-xs font-semibold uppercase tracking-wide text-slate-500">
      {children}
    </span>
  );
}

function StatusChip({ result }: Readonly<{ result: Result }>) {
  if (result.status === "idle") return null;
  if (result.status === "running") {
    return <span className="text-xs text-slate-500">Running...</span>;
  }
  const succeeded = result.status === "success";
  return (
    <span
      className={classNames("rounded px-1.5 py-0.5 text-[11px] font-medium", {
        "bg-emerald-100 text-emerald-800": succeeded,
        "bg-red-100 text-red-800": !succeeded,
      })}
    >
      {succeeded ? "Succeeded" : "Failed"} in {result.ms} ms
    </span>
  );
}

/**
 * Invokes an operation from its OperationDefinition: choose where it runs
 * (system, a resource type or an instance), fill in its input parameters, run
 * it, and read the output parameters it answers with.
 *
 * The input is built with `OperationParametersEditor` and the output shown
 * with `OperationParametersView`, both of which can be used on their own.
 */
export function OperationInvocationPanel({
  operationDefinition,
  invoke,
  runHint,
  disabledReason,
  client,
  fhirVersion,
}: Readonly<OperationInvocationPanelProps>) {
  const levels = invocationLevels(operationDefinition);
  const allowedTypes = useMemo(
    () => operationDefinition.resource ?? [],
    [operationDefinition.resource],
  );

  const [level, setLevel] = useState<InvocationLevel>(levels[0] ?? "system");
  const [resourceType, setResourceType] = useState<string>(
    allowedTypes[0] ?? "",
  );
  const [id, setId] = useState("");
  const [input, setInput] = useState<Parameters>({
    resourceType: "Parameters",
  });
  const [outputMode, setOutputMode] = useState<"form" | "json">("form");
  const [result, setResult] = useState<Result>({ status: "idle" });

  // Keep the level and resource type valid while the definition is edited.
  const levelsKey = levels.join(",");
  useEffect(() => {
    if (levels.length > 0 && !levels.includes(level)) setLevel(levels[0]);
  }, [levelsKey]);

  const typesKey = allowedTypes.join(",");
  useEffect(() => {
    if (
      allowedTypes.length > 0 &&
      !allowedTypes.includes(resourceType as any)
    ) {
      setResourceType(allowedTypes[0]);
    }
  }, [typesKey]);

  // A different operation starts from a clean slate.
  useEffect(() => {
    setInput({ resourceType: "Parameters" });
    setResult({ status: "idle" });
  }, [operationDefinition.id]);

  const typeOptions = useMemo(
    () =>
      (allowedTypes.length > 0 ? allowedTypes : [...resourceTypes].sort()).map(
        (type) => ({ value: type, label: type }),
      ),
    [typesKey],
  );

  const target: InvocationTarget =
    level === "system"
      ? { level }
      : level === "type"
        ? { level, resourceType }
        : { level, resourceType, id: id.trim() };
  const path = invocationPath(target, operationDefinition.code);
  const missing = missingRequired(
    input.parameter,
    parameterDefinitions(operationDefinition, "in"),
  );
  const canRun =
    !disabledReason && isTargetComplete(target) && result.status !== "running";

  const defaultInvoke = (target: InvocationTarget, body: Parameters) => {
    const code = operationDefinition.code as any;
    switch (target.level) {
      case "system":
        return client.invoke_system(code, {}, fhirVersion, body as any);
      case "type":
        return client.invoke_type(
          code,
          {},
          fhirVersion,
          target.resourceType as any,
          body as any,
        );
      case "instance":
        return client.invoke_instance(
          code,
          {},
          fhirVersion,
          target.resourceType as any,
          target.id as any,
          body as any,
        );
    }
  };

  const run = async () => {
    const started = performance.now();
    const elapsed = () => Math.round(performance.now() - started);
    setResult({ status: "running" });
    try {
      const output = await (invoke ?? defaultInvoke)(
        target,
        pruneParameters(input),
      );
      setResult({ status: "success", output, ms: elapsed() });
    } catch (error) {
      setResult({
        status: "failure",
        outcome: toOutcome(error),
        ms: elapsed(),
        path,
      });
    }
  };

  return (
    <div className="flex h-full min-h-0 flex-col bg-white">
      <section
        aria-label="Input"
        className="min-h-0 flex-[3] space-y-4 overflow-auto p-3"
      >
        <div className="space-y-2">
          <div className="flex items-center gap-2">
            <SectionTitle>Invoke at</SectionTitle>
            <div className="flex-1" />
            <Segmented
              ariaLabel="Invocation level"
              value={level}
              onChange={setLevel}
              options={LEVELS.map((option) => {
                const disabled =
                  levels.length > 0 && !levels.includes(option.value);
                return {
                  ...option,
                  disabled,
                  title: disabled
                    ? `This operation does not allow ${option.value} invocation.`
                    : undefined,
                };
              })}
            />
          </div>
          {levels.length === 0 && (
            <p className="rounded-md border border-amber-200 bg-amber-50 px-2 py-1.5 text-xs text-amber-800">
              The definition allows no level: set system, type or instance to
              true.
            </p>
          )}
          {level !== "system" && (
            <div className="grid grid-cols-2 gap-2">
              <Select
                label="Resource type"
                value={resourceType}
                options={typeOptions}
                onChange={(option) =>
                  setResourceType(option ? String(option.value) : "")
                }
              />
              {level === "instance" && (
                <Input
                  label="Resource id"
                  placeholder="id"
                  value={id}
                  onChange={(event) => setId(event.target.value)}
                />
              )}
            </div>
          )}
        </div>

        <OperationParametersEditor
          title="Input"
          client={client}
          fhirVersion={fhirVersion}
          operationDefinition={operationDefinition}
          value={input}
          onChange={setInput}
        />
      </section>

      <div className="space-y-1 border-y border-slate-200 bg-slate-50 px-3 py-2">
        <div className="flex items-center gap-2">
          <code
            className="min-w-0 flex-1 truncate rounded border border-slate-200 bg-white px-2 py-1 font-mono text-xs text-slate-700"
            title={`POST ${path}`}
          >
            POST {path}
          </code>
          <Button
            buttonType="primary"
            buttonSize="small"
            disabled={!canRun}
            onClick={(event) => {
              event.preventDefault();
              run();
            }}
          >
            <span className="flex items-center gap-1">
              <PlayIcon className="h-3.5 w-3.5" />
              {result.status === "running" ? "Running" : "Run"}
            </span>
          </Button>
        </div>
        {disabledReason && (
          <p className="text-xs text-amber-700">{disabledReason}</p>
        )}
        {missing.length > 0 && (
          <p className="text-xs text-amber-700">
            Required parameters without a value:{" "}
            <span className="font-mono">{missing.join(", ")}</span>
          </p>
        )}
        {!disabledReason && runHint && (
          <p className="text-xs text-slate-500">{runHint}</p>
        )}
      </div>

      <section
        aria-label="Output"
        className="min-h-0 flex-[2] space-y-3 overflow-auto p-3"
      >
        <div className="flex items-center gap-2">
          <SectionTitle>Output</SectionTitle>
          <StatusChip result={result} />
          <div className="flex-1" />
          {result.status === "success" && (
            <Segmented
              ariaLabel="Output view"
              value={outputMode}
              onChange={setOutputMode}
              options={[
                { value: "form", label: "Form" },
                { value: "json", label: "JSON" },
              ]}
            />
          )}
        </div>

        {result.status === "running" && (
          <div className="flex justify-center py-6">
            <Loading />
          </div>
        )}
        {result.status === "failure" && (
          <OutcomePanel
            outcome={result.outcome}
            request={{ method: "POST", path: result.path }}
          />
        )}
        {result.status === "success" && outputMode === "json" && (
          <div className="overflow-hidden rounded-md border border-slate-200">
            <CodeMirror
              readOnly
              extensions={jsonExtensions}
              value={JSON.stringify(result.output, null, 2)}
              theme={{ "&": { width: "100%" } }}
            />
          </div>
        )}
        {(result.status === "idle" ||
          (result.status === "success" && outputMode === "form")) && (
          <OperationParametersView
            operationDefinition={operationDefinition}
            value={result.status === "success" ? result.output : undefined}
          />
        )}
      </section>
    </div>
  );
}
