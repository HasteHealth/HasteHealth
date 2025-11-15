import type { Meta, StoryObj } from "@storybook/react";

import { Range } from "@haste-health/fhir-types/r4/types";

import { FHIRRangeReadOnly } from "./RangeReadOnly";

// More on how to set up stories at: https://storybook.js.org/docs/react/writing-stories/introduction#default-export
const meta = {
  title: "Complex/FHIRRangeReadOnly",
  component: FHIRRangeReadOnly,
  parameters: {
    // Optional parameter to center the component in the Canvas. More info: https://storybook.js.org/docs/react/configure/story-layout
    layout: "centered",
  },
  // This component will have an automatically generated Autodocs entry: https://storybook.js.org/docs/react/writing-docs/autodocs
  tags: ["autodocs"],
} satisfies Meta<typeof FHIRRangeReadOnly>;

export default meta;

type Story = StoryObj<typeof meta>;

export const Primary: Story = {
  args: {
    value: {
      low: {
        value: 15,
        unit: "mg",
      },
      high: {
        value: 30,
        unit: "mg",
      },
    } as Range,
  },
};

export const BadValue: Story = {
  args: {
    // @ts-ignore
    value: "test",
    issue: "Bad value",
  },
};
