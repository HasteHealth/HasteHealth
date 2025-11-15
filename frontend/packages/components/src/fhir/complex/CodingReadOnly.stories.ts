import type { Meta, StoryObj } from "@storybook/react";

import { Coding } from "@haste-health/fhir-types/r4/types";

import { FHIRCodingReadOnly } from "./CodingReadOnly";

// More on how to set up stories at: https://storybook.js.org/docs/react/writing-stories/introduction#default-export
const meta = {
  title: "Complex/FHIRCodingReadOnly",
  component: FHIRCodingReadOnly,
  parameters: {
    // Optional parameter to center the component in the Canvas. More info: https://storybook.js.org/docs/react/configure/story-layout
    layout: "centered",
  },
  // This component will have an automatically generated Autodocs entry: https://storybook.js.org/docs/react/writing-docs/autodocs
  tags: ["autodocs"],
} satisfies Meta<typeof FHIRCodingReadOnly>;

export default meta;

type Story = StoryObj<typeof meta>;

export const Primary: Story = {
  args: { value: { code: "testing2", display: "test2" } as Coding },
};

export const NoDisplay: Story = {
  args: { value: { code: "testing2", system: "test.com" } as Coding },
};

export const BadValue: Story = {
  args: {
    // @ts-ignore
    value: "test",
    issue: "Bad value",
  },
};
