import type { Decorator, Meta, StoryObj } from "@storybook/react-webpack5";
import React, { useState } from "react";

import {
  OperationDefinition,
  Parameters,
} from "@haste-health/fhir-types/r4/types";
import { R4 } from "@haste-health/fhir-types/versions";

import { createStorybookClient } from "../stories.client";
import { OperationInvocationPanel } from "./OperationInvocationPanel";
import { OperationParametersEditor } from "./OperationParametersEditor";
import { OperationParametersView } from "./OperationParametersView";
import { InvocationTarget } from "./parameters";

const client = createStorybookClient();

const operationDefinition = {
  resourceType: "OperationDefinition",
  id: "patient-summary",
  name: "PatientSummary",
  status: "active",
  kind: "operation",
  code: "summary",
  system: true,
  type: true,
  instance: true,
  resource: ["Patient", "Encounter"],
  parameter: [
    {
      name: "limit",
      use: "in",
      min: 1,
      max: "1",
      type: "integer",
      documentation: "Most entries to return per section.",
    },
    {
      name: "since",
      use: "in",
      min: 0,
      max: "1",
      type: "date",
      documentation: "Only include data recorded on or after this date.",
    },
    {
      name: "includeNotes",
      use: "in",
      min: 0,
      max: "1",
      type: "boolean",
    },
    {
      name: "category",
      use: "in",
      min: 0,
      max: "*",
      type: "string",
      documentation: "Sections to include. Leave empty for all of them.",
    },
    {
      name: "code",
      use: "in",
      min: 0,
      max: "1",
      type: "Coding",
    },
    {
      name: "filter",
      use: "in",
      min: 0,
      max: "*",
      documentation:
        "Extra conditions, each a path and the value it must have.",
      part: [
        { name: "path", use: "in", min: 1, max: "1", type: "string" },
        { name: "value", use: "in", min: 1, max: "1", type: "string" },
      ],
    },
    {
      name: "patient",
      use: "in",
      min: 0,
      max: "1",
      type: "Patient",
      documentation: "A patient to summarize instead of a stored one.",
    },
    {
      name: "summary",
      use: "out",
      min: 1,
      max: "1",
      type: "Bundle",
      documentation: "The summary as a searchset Bundle.",
    },
    { name: "count", use: "out", min: 1, max: "1", type: "integer" },
    { name: "generated", use: "out", min: 1, max: "1", type: "dateTime" },
    {
      name: "issue",
      use: "out",
      min: 0,
      max: "*",
      part: [
        { name: "severity", use: "out", min: 1, max: "1", type: "code" },
        { name: "message", use: "out", min: 1, max: "1", type: "string" },
      ],
    },
  ],
} as unknown as OperationDefinition;

/** Answers as the operation would, after a short delay. */
async function fakeInvoke(
  target: InvocationTarget,
  input: Parameters,
): Promise<Parameters> {
  await new Promise((resolve) => setTimeout(resolve, 400));
  return {
    resourceType: "Parameters",
    parameter: [
      {
        name: "summary",
        resource: {
          resourceType: "Bundle",
          type: "searchset",
          total: 1,
          entry: [
            {
              resource: {
                resourceType: "Patient",
                id: target.level === "instance" ? target.id : "example",
                name: [{ family: "Chalmers", given: ["Peter"] }],
              },
            },
          ],
        },
      },
      { name: "count", valueInteger: 1 },
      { name: "generated", valueDateTime: "2026-10-06T21:30:00Z" },
      {
        name: "issue",
        part: [
          { name: "severity", valueCode: "warning" },
          {
            name: "message",
            valueString: "No encounters since the date given.",
          },
        ],
      },
      { name: "received", resource: input },
    ],
  } as Parameters;
}

/** The panel fills its container, so it gets one the size of a side panel. */
const panelFrame: Decorator = (Story) => (
  <div style={{ height: "900px", width: "460px" }} className="border">
    <Story />
  </div>
);

const meta: Meta<typeof OperationInvocationPanel> = {
  title: "Operations/OperationInvocationPanel",
  component: OperationInvocationPanel,
  parameters: { layout: "fullscreen" },
  tags: ["autodocs"],
};

export default meta;

type Story = StoryObj<typeof meta>;

export const Default: Story = {
  decorators: [panelFrame],
  args: {
    operationDefinition,
    client,
    fhirVersion: R4,
    invoke: fakeInvoke,
    runHint:
      "Runs the saved operation: save first so your latest code and parameters are used.",
  },
};

export const NotSaved: Story = {
  decorators: [panelFrame],
  args: {
    operationDefinition,
    client,
    fhirVersion: R4,
    invoke: fakeInvoke,
    disabledReason: "Create the operation to run it.",
  },
};

export const SystemOnly: Story = {
  decorators: [panelFrame],
  args: {
    operationDefinition: {
      ...operationDefinition,
      type: false,
      instance: false,
    },
    client,
    fhirVersion: R4,
    invoke: fakeInvoke,
  },
};

export const Failure: Story = {
  decorators: [panelFrame],
  args: {
    operationDefinition,
    client,
    fhirVersion: R4,
    invoke: async () => {
      await new Promise((resolve) => setTimeout(resolve, 200));
      // Shaped like the client's ResponseError, which the panel reads by shape.
      throw Object.assign(new Error("Parameter 'limit' is required."), {
        response: {
          http: { status: 400 },
          body: {
            resourceType: "OperationOutcome",
            issue: [
              {
                severity: "error",
                code: "required",
                diagnostics: "Parameter 'limit' is required.",
                expression: ["Parameters.parameter"],
              },
            ],
          },
        },
      });
    },
  },
};

/** The input editor on its own, holding its Parameters in state. */
function EditorOnly() {
  const [value, setValue] = useState<Parameters>({
    resourceType: "Parameters",
    parameter: [
      { name: "limit", valueInteger: 10 },
      { name: "category", valueString: "labs" },
      { name: "category", valueString: "medications" },
    ],
  } as Parameters);
  return (
    <div className="space-y-4 p-4" style={{ width: "460px" }}>
      <OperationParametersEditor
        operationDefinition={operationDefinition}
        client={client}
        fhirVersion={R4}
        value={value}
        onChange={setValue}
      />
      <pre className="text-xs">{JSON.stringify(value, null, 2)}</pre>
    </div>
  );
}

export const Editor: StoryObj = {
  render: () => <EditorOnly />,
};

export const OutputView: StoryObj = {
  render: function Render() {
    const [value, setValue] = useState<Parameters>();
    React.useEffect(() => {
      void fakeInvoke({ level: "system" }, { resourceType: "Parameters" }).then(
        setValue,
      );
    }, []);
    return (
      <div className="p-4" style={{ width: "460px" }}>
        <OperationParametersView
          operationDefinition={operationDefinition}
          value={value}
        />
      </div>
    );
  },
};
