import type { Meta, StoryObj } from "@storybook/react";

import { Identifier } from "@oxidized-health/fhir-types/r4/types";

import { FHIRIdentifierReadOnly } from "./IdentifierReadOnly";

// More on how to set up stories at: https://storybook.js.org/docs/react/writing-stories/introduction#default-export
const meta = {
  title: "Complex/FHIRIdentifierReadOnly",
  component: FHIRIdentifierReadOnly,
  parameters: {
    // Optional parameter to center the component in the Canvas. More info: https://storybook.js.org/docs/react/configure/story-layout
    layout: "centered",
  },
  // This component will have an automatically generated Autodocs entry: https://storybook.js.org/docs/react/writing-docs/autodocs
  tags: ["autodocs"],
} satisfies Meta<typeof FHIRIdentifierReadOnly>;

export default meta;

type Story = StoryObj<typeof meta>;

export const Primary: Story = {
  args: {
    value: {
      system: "test.com",
      value: "test",
    } as Identifier,
  },
};

export const BadValue: Story = {
  args: {
    // @ts-ignore
    value: "test",
    issue: "Bad value",
  },
};
