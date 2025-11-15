import type { Meta, StoryObj } from "@storybook/react";

import { Ratio } from "@haste-health/fhir-types/r4/types";

import { FHIRRatioEditable } from "./Ratio";

// More on how to set up stories at: https://storybook.js.org/docs/react/writing-stories/introduction#default-export
const meta = {
  title: "Complex/FHIRRatioEditable",
  component: FHIRRatioEditable,
  parameters: {
    // Optional parameter to center the component in the Canvas. More info: https://storybook.js.org/docs/react/configure/story-layout
    layout: "centered",
  },
  // This component will have an automatically generated Autodocs entry: https://storybook.js.org/docs/react/writing-docs/autodocs
  tags: ["autodocs"],
} satisfies Meta<typeof FHIRRatioEditable>;

export default meta;

type Story = StoryObj<typeof meta>;

export const Primary: Story = {
  args: {
    value: {
      numerator: {
        unit: "km",
        value: 55.2,
      },
      denominator: {
        unit: "km",
        value: 75.2,
      },
    } as Ratio,
    label: "Ratio",
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
