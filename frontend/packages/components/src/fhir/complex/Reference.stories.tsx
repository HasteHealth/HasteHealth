import type { Meta, StoryObj } from "@storybook/react";
import React from "react";

import { Reference } from "@haste-health/fhir-types/r4/types";
import { R4 } from "@haste-health/fhir-types/versions";

import { createStorybookClient } from "../stories.client";
import { FHIRReferenceEditable } from "./Reference";

const FHIRReferenceStorie = (
  props: Parameters<typeof FHIRReferenceEditable>[0]
) => {
  const [reference, setReference] = React.useState<Reference | undefined>(
    props.value
  );
  return (
    <FHIRReferenceEditable
      {...props}
      value={reference}
      onChange={setReference}
    />
  );
};

// More on how to set up stories at: https://storybook.js.org/docs/react/writing-stories/introduction#default-export
const meta = {
  title: "Complex/FHIRReferenceEditable",
  component: FHIRReferenceStorie,
  parameters: {
    // Optional parameter to center the component in the Canvas. More info: https://storybook.js.org/docs/react/configure/story-layout
    layout: "centered",
  },
  // This component will have an automatically generated Autodocs entry: https://storybook.js.org/docs/react/writing-docs/autodocs
  tags: ["autodocs"],
} satisfies Meta<typeof FHIRReferenceStorie>;

export default meta;

type Story = StoryObj<typeof meta>;

export const Primary: Story = {
  args: {
    fhirVersion: R4,
    client: createStorybookClient(),
    value: {
      reference: "Patient/123",
    },
    label: "Subject",
    onChange: (value) => console.log(value),
  },
};

export const OnError: Story = {
  args: {
    fhirVersion: R4,
    client: createStorybookClient(),
    // @ts-ignore
    value: "test",
    issue: "Bad value",
    onChange: (value) => console.log(value),
  },
};

export const Readonly: Story = {
  args: {
    disabled: true,
    fhirVersion: R4,
    client: createStorybookClient(),
    value: {
      reference: "Patient/123",
    },
    label: "Subject",
    onChange: (value) => console.log(value),
  },
};
