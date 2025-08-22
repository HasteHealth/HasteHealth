import type { Meta, StoryObj } from "@storybook/react";

import { Reference } from "@oxidized-health/fhir-types/r4/types";

import { FHIRReferenceReadOnly } from "./ReferenceReadOnly";

// More on how to set up stories at: https://storybook.js.org/docs/react/writing-stories/introduction#default-export
const meta = {
  title: "Complex/FHIRReferenceReadOnly",
  component: FHIRReferenceReadOnly,
  parameters: {
    // Optional parameter to center the component in the Canvas. More info: https://storybook.js.org/docs/react/configure/story-layout
    layout: "centered",
  },
  // This component will have an automatically generated Autodocs entry: https://storybook.js.org/docs/react/writing-docs/autodocs
  tags: ["autodocs"],
} satisfies Meta<typeof FHIRReferenceReadOnly>;

export default meta;

type Story = StoryObj<typeof meta>;

export const Primary: Story = {
  args: {
    value: {
      reference: "Patient/123",
    } as Reference,
  },
};

export const Display: Story = {
  args: {
    value: {
      display: "Some Patient",
      reference: "Patient/123",
    } as Reference,
  },
};

export const BadValue: Story = {
  args: {
    // @ts-ignore
    value: "test",
    issue: "Bad value",
  },
};
