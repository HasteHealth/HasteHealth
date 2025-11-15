import type { Meta, StoryObj } from "@storybook/react";

import { Annotation } from "@haste-health/fhir-types/r4/types";

import { FHIRAnnotationEditable } from "./Annotation";

const meta = {
  title: "Complex/FHIRAnnotationEditable",
  component: FHIRAnnotationEditable,
  parameters: {
    layout: "centered",
  },
  tags: ["autodocs"],
} satisfies Meta<typeof FHIRAnnotationEditable>;

export default meta;

type Story = StoryObj<typeof meta>;

export const Primary: Story = {
  args: {
    value: { text: "# Testing" } as Annotation,
    onChange: (value) => console.log(value),
  },
};

export const OnError: Story = {
  args: {
    value: { text: "# Testing" } as Annotation,
    issue: "Issue",
    onChange: (value) => console.log(value),
  },
};
