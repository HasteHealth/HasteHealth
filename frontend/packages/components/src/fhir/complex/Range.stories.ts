import type { Meta, StoryObj } from "@storybook/react";

import { Range } from "@haste-health/fhir-types/r4/types";

import { FHIRRangeEditable } from "./Range";

// More on how to set up stories at: https://storybook.js.org/docs/react/writing-stories/introduction#default-export
const meta = {
  title: "Complex/FHIRRangeEditable",
  component: FHIRRangeEditable,
  parameters: {
    // Optional parameter to center the component in the Canvas. More info: https://storybook.js.org/docs/react/configure/story-layout
    layout: "centered",
  },
  // This component will have an automatically generated Autodocs entry: https://storybook.js.org/docs/react/writing-docs/autodocs
  tags: ["autodocs"],
} satisfies Meta<typeof FHIRRangeEditable>;

export default meta;

type Story = StoryObj<typeof meta>;

export const Primary: Story = {
  args: {
    value: {
      low: {
        unit: "km",
        value: 55.2,
      },
      high: {
        unit: "km",
        value: 75.2,
      },
    } as Range,
    label: "Range",
    onChange: (value) => console.log(value),
  },
};

export const OnError: Story = {
  args: {
    // @ts-ignore
    value: "test",
    issue: "Bad value",
    onChange: (value) => console.log(value),
  },
};
