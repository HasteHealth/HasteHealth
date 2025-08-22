import type { Meta, StoryObj } from "@storybook/react";

import { code, uri } from "@oxidized-health/fhir-types/r4/types";
import { R4 } from "@oxidized-health/fhir-types/versions";

import { createStorybookClient } from "../stories.client";
import { FHIRCodeEditable } from "./code";

// More on how to set up stories at: https://storybook.js.org/docs/react/writing-stories/introduction#default-export
const meta = {
  title: "Primitives/FHIRCodeEditable",
  component: FHIRCodeEditable,
  parameters: {
    // Optional parameter to center the component in the Canvas. More info: https://storybook.js.org/docs/react/configure/story-layout
    layout: "centered",
  },
  // This component will have an automatically generated Autodocs entry: https://storybook.js.org/docs/react/writing-docs/autodocs
  tags: ["autodocs"],
} satisfies Meta<typeof FHIRCodeEditable>;

export default meta;

type Story = StoryObj<typeof meta>;

export const Primary: Story = {
  args: {
    fhirVersion: R4,
    client: createStorybookClient(),
    value: "test" as code,
    onChange: (value: string | undefined) => console.log(value),
    system: "http://hl7.org/fhir/ValueSet/gender-identity" as uri,
  },
};

export const OnError: Story = {
  args: {
    fhirVersion: R4,
    client: createStorybookClient(),
    value: "test" as code,
    issue: "Bad value",
    onChange: (value) => console.log(value),
  },
};
