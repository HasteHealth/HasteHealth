import type { Meta, StoryObj } from "@storybook/react-webpack5";

import { SampledData } from "@haste-health/fhir-types/r4/types";

import { FHIRSampledDataEditable } from "./SampledData";

// More on how to set up stories at: https://storybook.js.org/docs/react/writing-stories/introduction#default-export
const meta = {
  title: "Complex/FHIRSampledDataEditable",
  component: FHIRSampledDataEditable,
  parameters: {
    // Optional parameter to center the component in the Canvas. More info: https://storybook.js.org/docs/react/configure/story-layout
    layout: "centered",
  },
  // This component will have an automatically generated Autodocs entry: https://storybook.js.org/docs/react/writing-docs/autodocs
  tags: ["autodocs"],
} satisfies Meta<typeof FHIRSampledDataEditable>;

export default meta;

type Story = StoryObj<typeof meta>;

export const Primary: Story = {
  args: {
    value: {
      origin: {
        value: 0,
        unit: "mV",
      },
      data: "1 2 3 4",
    } as SampledData,
    label: "Sampled Data",
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
