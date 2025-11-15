import type { Meta, StoryObj } from "@storybook/react";

import { ContactPoint } from "@haste-health/fhir-types/r4/types";
import { R4 } from "@haste-health/fhir-types/versions";

import { createStorybookClient } from "../stories.client";
import { FHIRContactPointEditable } from "./ContactPoint";

// More on how to set up stories at: https://storybook.js.org/docs/react/writing-stories/introduction#default-export
const meta = {
  title: "Complex/FHIRContactPointEditable",
  component: FHIRContactPointEditable,
  parameters: {
    // Optional parameter to center the component in the Canvas. More info: https://storybook.js.org/docs/react/configure/story-layout
    layout: "centered",
  },
  // This component will have an automatically generated Autodocs entry: https://storybook.js.org/docs/react/writing-docs/autodocs
  tags: ["autodocs"],
} satisfies Meta<typeof FHIRContactPointEditable>;

export default meta;

type Story = StoryObj<typeof meta>;

const client = createStorybookClient();

export const Primary: Story = {
  args: {
    fhirVersion: R4,
    client: client,
    value: {
      use: "home",
      system: "phone",
      value: "555-555-5555",
    } as ContactPoint,
    label: "ContactPoint",
    onChange: (value) => console.log(value),
  },
};

export const OnError: Story = {
  args: {
    fhirVersion: R4,
    // @ts-ignore
    value: "test",
    issue: "Bad value",
    onChange: (value) => console.log(value),
  },
};
