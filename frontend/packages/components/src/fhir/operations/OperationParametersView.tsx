import React from "react";

import {
  OperationDefinition,
  OperationDefinitionParameter,
  ParametersParameter,
} from "@haste-health/fhir-types/r4/types";

import { JSONValueView, ParameterValueView } from "./ParameterValue";
import { ParameterDocumentation, ParameterHeader } from "./controls";
import {
  ParameterUse,
  entryValueField,
  isParameters,
  isPartParameter,
  isResourceParameter,
  parameterDefinitions,
  valueField,
} from "./parameters";

export type OperationParametersViewProps = {
  /** The operation whose parameters are shown. */
  operationDefinition: OperationDefinition;
  /** Which parameters to show. Defaults to the output parameters. */
  use?: ParameterUse;
  /**
   * The Parameters resource to show, or the bare resource an operation with a
   * single `return` parameter may answer with. Before there is one, the
   * declared parameters are listed without values.
   */
  value?: unknown;
};

/**
 * The complex type a `value[x]` field carries: `Coding` for `valueCoding`. A
 * primitive comes out capitalised (`String`), which is fine: primitives are
 * shown as their text whatever the type.
 */
function typeOfField(field: string): string {
  return field.slice("value".length);
}

/** An entry shown from what it carries, for one no definition describes. */
function EntryValue({ entry }: Readonly<{ entry: ParametersParameter }>) {
  if (entry.resource !== undefined)
    return <JSONValueView value={entry.resource} />;
  if (entry.part) {
    return (
      <div className="space-y-3 border-l-2 border-slate-200 pl-3">
        {entry.part.map((part, index) => (
          // Parts have no identity of their own; their position is their key.
          <div key={index} className="space-y-1">
            <span className="font-mono text-sm font-medium text-slate-900">
              {part.name}
            </span>
            <EntryValue entry={part} />
          </div>
        ))}
      </div>
    );
  }
  const field = entryValueField(entry);
  if (!field) return <ParameterValueView value={undefined} />;
  return <ParameterValueView type={typeOfField(field)} value={entry[field]} />;
}

/** An entry shown as its definition describes it. */
function DeclaredEntryValue({
  definition,
  entry,
}: Readonly<{
  definition: OperationDefinitionParameter;
  entry: ParametersParameter;
}>) {
  if (isPartParameter(definition)) {
    return (
      <div className="border-l-2 border-slate-200 pl-3">
        <ParameterList
          definitions={definition.part ?? []}
          list={entry.part ?? []}
          ran={true}
        />
      </div>
    );
  }
  if (isResourceParameter(definition)) {
    return <JSONValueView value={entry.resource} />;
  }
  if (definition.type) {
    const value = entry[valueField(definition.type)];
    // A value in another field than the declared type's is still shown.
    if (value !== undefined) {
      return <ParameterValueView type={definition.type} value={value} />;
    }
  }
  return <EntryValue entry={entry} />;
}

function ParameterList({
  definitions,
  list,
  ran,
}: Readonly<{
  definitions: OperationDefinitionParameter[];
  list: ParametersParameter[];
  /** Whether `list` is a response, so a missing value means none came back. */
  ran: boolean;
}>) {
  return (
    <div className="space-y-4">
      {definitions.map((definition) => {
        const entries = list.filter((entry) => entry.name === definition.name);
        return (
          <div key={definition.name} className="space-y-1.5">
            <ParameterHeader definition={definition} />
            <ParameterDocumentation definition={definition} />
            {entries.length === 0 ? (
              <span className="text-sm text-slate-400">
                {ran ? "Not returned" : "No value yet"}
              </span>
            ) : (
              entries.map((entry, index) => (
                // Entries have no identity of their own; position is the key.
                <DeclaredEntryValue
                  key={index}
                  definition={definition}
                  entry={entry}
                />
              ))
            )}
          </div>
        );
      })}
    </div>
  );
}

/**
 * Shows an operation's parameters read-only, as the OperationDefinition
 * declares them: each parameter with its type and documentation, then the
 * values a response carried for it. Values the definition does not declare
 * are listed after them.
 */
export function OperationParametersView({
  operationDefinition,
  use = "out",
  value,
}: Readonly<OperationParametersViewProps>) {
  const definitions = parameterDefinitions(operationDefinition, use);
  const ran = value !== undefined;

  // An operation whose only output is `return` may send the resource bare.
  if (ran && !isParameters(value)) {
    const returned = definitions.find(
      (definition) => definition.name === "return",
    );
    return returned ? (
      <ParameterList
        definitions={[returned]}
        list={[
          {
            name: "return",
            resource: value as ParametersParameter["resource"],
          },
        ]}
        ran={true}
      />
    ) : (
      <JSONValueView value={value} />
    );
  }

  const list = (isParameters(value) ? value.parameter : undefined) ?? [];
  const declared = new Set<string>(
    definitions.map((definition) => definition.name),
  );
  const undeclared = list.filter((entry) => !declared.has(entry.name));

  if (definitions.length === 0 && undeclared.length === 0) {
    return (
      <p className="text-sm text-slate-500">
        {ran
          ? "The response carried no parameters."
          : `This operation declares no ${use === "in" ? "input" : "output"} parameters.`}
      </p>
    );
  }

  return (
    <div className="space-y-4">
      <ParameterList definitions={definitions} list={list} ran={ran} />
      {undeclared.length > 0 && (
        <div className="space-y-3 border-t border-slate-200 pt-3">
          <p className="text-xs font-medium text-slate-500">
            Not declared by the operation
          </p>
          {undeclared.map((entry, index) => (
            <div key={index} className="space-y-1">
              <span className="font-mono text-sm font-medium text-slate-900">
                {entry.name}
              </span>
              <EntryValue entry={entry} />
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
