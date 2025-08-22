import type { Meta, StoryObj } from "@storybook/react";

import { id } from "@oxidized-health/fhir-types/r4/types";

import { FHIRIdEditable } from "./id";

// More on how to set up stories at: https://storybook.js.org/docs/react/writing-stories/introduction#default-export
const meta = {
  title: "Primitives/FHIRIdEditable",
  component: FHIRIdEditable,
  parameters: {
    // Optional parameter to center the component in the Canvas. More info: https://storybook.js.org/docs/react/configure/story-layout
    layout: "centered",
  },
  // This component will have an automatically generated Autodocs entry: https://storybook.js.org/docs/react/writing-docs/autodocs
  tags: ["autodocs"],
} satisfies Meta<typeof FHIRIdEditable>;

export default meta;

type Story = StoryObj<typeof meta>;

export const Primary: Story = {
  args: {
    value: "test" as id,
    onChange: (value) => console.log(value),
  },
};

export const InvalidId: Story = {
  args: {
    value: "*asdf" as id,
    onChange: (value) => console.log(value),
  },
};

export const OnError: Story = {
  args: {
    value: "test" as id,
    issue: "Bad value",
    onChange: (value) => console.log(value),
  },
};
