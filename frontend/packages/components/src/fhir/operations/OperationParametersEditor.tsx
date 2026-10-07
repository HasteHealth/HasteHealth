import { XMarkIcon } from "@heroicons/react/24/outline";
import React, { useState } from "react";

import {
  OperationDefinition,
  OperationDefinitionParameter,
  Parameters,
  ParametersParameter,
} from "@haste-health/fhir-types/r4/types";

import { Add } from "../../base/Add";
import { JSONResourceEditor } from "../resources/JSONResourceEditor";
import { ClientProps } from "../types";
import { ParameterValueEditor } from "./ParameterValue";
import { ParameterDocumentation, ParameterHeader, Segmented } from "./controls";
import {
  ParameterUse,
  entryValue,
  isEmptyEntry,
  isEmptyValue,
  isPartParameter,
  isRepeating,
  parameterDefinitions,
  pruneParameters,
  replaceEntries,
  undeclaredNames,
  withEntryValue,
} from "./parameters";

export type OperationParametersEditorProps = ClientProps & {
  /** The operation whose parameters are edited. */
  operationDefinition: OperationDefinition;
  /** Which parameters to edit. Defaults to the input parameters. */
  use?: ParameterUse;
  /** The Parameters resource being built. */
  value: Parameters;
  onChange: (value: Parameters) => void;
  /** Heading shown beside the Form and JSON switch. */
  title?: string;
};

/** The definitions of a parameter's parts. */
function partDefinitions(
  definition: OperationDefinitionParameter,
): OperationDefinitionParameter[] {
  return definition.part?.filter((part) => part.max !== "0") ?? [];
}

type FieldsProps = ClientProps & {
  definitions: OperationDefinitionParameter[];
  list: ParametersParameter[];
  onChange: (list: ParametersParameter[]) => void;
};

/** One field per definition, editing the entries of `list` they declare. */
function ParameterFields({
  definitions,
  list,
  onChange,
  ...client
}: FieldsProps) {
  return (
    <div className="space-y-4">
      {definitions.map((definition) => (
        <ParameterField
          {...client}
          key={definition.name}
          definition={definition}
          entries={list.filter((entry) => entry.name === definition.name)}
          onEntries={(entries) =>
            onChange(
              replaceEntries(list, definitions, definition.name, entries),
            )
          }
        />
      ))}
    </div>
  );
}

type EntryEditorProps = ClientProps & {
  definition: OperationDefinitionParameter;
  entry: ParametersParameter | undefined;
  /** `undefined` removes the entry. */
  onEntry: (entry: ParametersParameter | undefined) => void;
  /**
   * Keep the entry when its value is cleared: a row in a repeating parameter
   * stays until it is removed, while a single value is dropped.
   */
  keepWhenEmpty: boolean;
};

/** Edits one entry: its value, or for a group of parts, each part. */
function EntryEditor({
  definition,
  entry,
  onEntry,
  keepWhenEmpty,
  ...client
}: EntryEditorProps) {
  if (isPartParameter(definition)) {
    return (
      <div className="border-l-2 border-slate-200 pl-3">
        <ParameterFields
          {...client}
          definitions={partDefinitions(definition)}
          list={entry?.part ?? []}
          onChange={(parts) =>
            onEntry(
              parts.length === 0 && !keepWhenEmpty
                ? undefined
                : withEntryValue(definition, entry, parts),
            )
          }
        />
      </div>
    );
  }

  return (
    <ParameterValueEditor
      {...client}
      definition={definition}
      value={entryValue(definition, entry)}
      onChange={(value) =>
        onEntry(
          isEmptyValue(value) && !keepWhenEmpty
            ? undefined
            : withEntryValue(definition, entry, value),
        )
      }
    />
  );
}

type FieldProps = ClientProps & {
  definition: OperationDefinitionParameter;
  entries: ParametersParameter[];
  onEntries: (entries: ParametersParameter[]) => void;
};

/** A parameter: its name and type, then one editor per value it may take. */
function ParameterField({
  definition,
  entries,
  onEntries,
  ...client
}: FieldProps) {
  if (isRepeating(definition)) {
    return (
      <div className="space-y-1.5">
        <ParameterHeader definition={definition} />
        <ParameterDocumentation definition={definition} />
        {entries.map((entry, index) => (
          // Rows have no identity of their own; their position is their key.
          <div key={index} className="flex items-start gap-2">
            <div className="min-w-0 flex-1">
              <EntryEditor
                {...client}
                definition={definition}
                entry={entry}
                keepWhenEmpty={true}
                onEntry={(next) =>
                  onEntries(
                    entries.map((current, i) =>
                      i === index ? (next ?? current) : current,
                    ),
                  )
                }
              />
            </div>
            <button
              type="button"
              aria-label={`Remove ${definition.name} ${index + 1}`}
              className="mt-1.5 text-slate-400 hover:text-red-600"
              onClick={() => onEntries(entries.filter((_, i) => i !== index))}
            >
              <XMarkIcon className="h-4 w-4" />
            </button>
          </div>
        ))}
        <Add
          onChange={() => onEntries([...entries, { name: definition.name }])}
        >
          Add {definition.name}
        </Add>
      </div>
    );
  }

  const [entry, ...rest] = entries;
  return (
    <div className="space-y-1.5">
      <ParameterHeader
        definition={definition}
        action={
          entry &&
          !isEmptyEntry(entry) && (
            <button
              type="button"
              className="text-xs text-slate-500 hover:text-slate-800"
              onClick={() => onEntries(rest)}
            >
              Clear
            </button>
          )
        }
      />
      <ParameterDocumentation definition={definition} />
      <EntryEditor
        {...client}
        definition={definition}
        entry={entry}
        keepWhenEmpty={false}
        onEntry={(next) => onEntries(next ? [next, ...rest] : rest)}
      />
    </div>
  );
}

/**
 * Builds the Parameters resource an operation is invoked with, from the
 * parameters its OperationDefinition declares.
 *
 * Each parameter gets the editor for its type, repeats when its cardinality
 * allows, and nests its parts. The JSON view shows (and edits) the resource
 * that is sent. Values not filled in yet are left in `value` as empty entries;
 * pass the result through `pruneParameters` before sending it.
 */
export function OperationParametersEditor({
  operationDefinition,
  use = "in",
  value,
  onChange,
  title = "Parameters",
  client,
  fhirVersion,
}: Readonly<OperationParametersEditorProps>) {
  const [mode, setMode] = useState<"form" | "json">("form");
  const definitions = parameterDefinitions(operationDefinition, use);
  const list = value.parameter ?? [];
  const undeclared = undeclaredNames(list, definitions);

  const setList = (next: ParametersParameter[]) => {
    const { parameter: _dropped, ...rest } = value;
    onChange(
      next.length > 0
        ? { ...rest, resourceType: "Parameters", parameter: next }
        : { ...rest, resourceType: "Parameters" },
    );
  };

  return (
    <div className="flex flex-col gap-3">
      <div className="flex items-center gap-2">
        <span className="text-xs font-semibold uppercase tracking-wide text-slate-500">
          {title}
        </span>
        <div className="flex-1" />
        <Segmented
          ariaLabel={`${title} view`}
          value={mode}
          onChange={setMode}
          options={[
            { value: "form", label: "Form" },
            { value: "json", label: "JSON" },
          ]}
        />
      </div>

      {mode === "json" ? (
        <div className="flex h-72 flex-col">
          <JSONResourceEditor
            resource={pruneParameters(value)}
            onChange={(resource) => {
              if (resource.resourceType === "Parameters") onChange(resource);
            }}
          />
        </div>
      ) : definitions.length === 0 ? (
        <p className="text-sm text-slate-500">
          This operation declares no {use === "in" ? "input" : "output"}{" "}
          parameters.
        </p>
      ) : (
        <ParameterFields
          client={client}
          fhirVersion={fhirVersion}
          definitions={definitions}
          list={list}
          onChange={setList}
        />
      )}

      {undeclared.length > 0 && (
        <p className="rounded-md border border-amber-200 bg-amber-50 px-2 py-1.5 text-xs text-amber-800">
          Not declared by the operation, sent as written in the JSON view:{" "}
          <span className="font-mono">{undeclared.join(", ")}</span>
        </p>
      )}
    </div>
  );
}
