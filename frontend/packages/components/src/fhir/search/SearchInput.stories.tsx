import type { Meta, StoryObj } from "@storybook/react-webpack5";
import React, { useState } from "react";

import { R4 } from "@haste-health/fhir-types/versions";

import { createStorybookClient } from "../stories.client";
import { FHIRSearchInput, FHIRSearchInputProps } from "./SearchInput";

/** Makes the stories editable and shows what submit would hand back. */
function SearchInputHarness(props: Readonly<FHIRSearchInputProps>) {
  const [value, setValue] = useState(props.value ?? "");
  const [submitted, setSubmitted] = useState<string>();

  return (
    <div className="flex w-[48rem] flex-col gap-3">
      <FHIRSearchInput
        {...props}
        value={value}
        onChange={setValue}
        onSubmit={setSubmitted}
      />
      <div className="font-mono text-xs text-gray-500">
        <div>value: {value || "(empty)"}</div>
        <div>submitted: {submitted ?? "(press Enter)"}</div>
      </div>
      <p className="text-xs text-gray-400">
        Completions follow the cursor: a resource type at the start, that
        resource&apos;s search parameters after <code>?</code>, the modifiers
        its type allows after <code>:</code>, and prefixes or codes after{" "}
        <code>=</code>. Ctrl/Cmd-Space opens the list anywhere, Tab accepts and
        Enter submits.
      </p>
    </div>
  );
}

const meta = {
  title: "FHIR/FHIRSearchInput",
  component: SearchInputHarness,
  tags: ["autodocs"],
  parameters: {
    layout: "centered",
  },
} satisfies Meta<typeof SearchInputHarness>;

export default meta;
type Story = StoryObj<typeof meta>;

const client = createStorybookClient();

/** Empty: completion starts with the server's resource types. */
export const Empty: Story = {
  args: {
    fhirVersion: R4,
    client,
    autoFocus: true,
  },
};

/** Part way through a query. */
export const WithQuery: Story = {
  args: {
    fhirVersion: R4,
    client,
    value: "Patient?name:exact=Smith&birthdate=ge2010-01-01&_count=20",
  },
};

/** Locked to one type: the text is parameters only. */
export const ScopedToResourceType: Story = {
  args: {
    fhirVersion: R4,
    client,
    resourceType: "Observation",
    value: "code=",
    placeholder: "status=final",
  },
};

/** A chain, where names come from the referenced resource. */
export const ChainedParameter: Story = {
  args: {
    fhirVersion: R4,
    client,
    value: "Observation?subject:Patient.name=",
  },
};

export const ReadOnly: Story = {
  args: {
    fhirVersion: R4,
    client,
    value: "Patient?gender=female&_sort=-_lastUpdated",
    readOnly: true,
  },
};
