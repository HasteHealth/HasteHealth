import type { Meta, StoryObj } from "@storybook/react";

import { OperationOutcome } from "@oxidized-health/fhir-types/r4/types";

import { FHIROperationOutcomeDisplay } from "./OperationOutcome";

// More on how to set up stories at: https://storybook.js.org/docs/react/writing-stories/introduction#default-export
const meta = {
  title: "Resources/FHIROperationOutcomeDisplay",
  component: FHIROperationOutcomeDisplay,
  parameters: {
    // Optional parameter to center the component in the Canvas. More info: https://storybook.js.org/docs/react/configure/story-layout
    layout: "fullscreen",
  },
  // This component will have an automatically generated Autodocs entry: https://storybook.js.org/docs/react/writing-docs/autodocs
  tags: ["autodocs"],
} satisfies Meta<typeof FHIROperationOutcomeDisplay>;

export default meta;

type Story = StoryObj<typeof meta>;

export const Primary: Story = {
  args: {
    title: "OxidizedHealth",
    operationOutcome: {
      resourceType: "OperationOutcome",
      issue: [
        {
          severity: "error",
          code: "invalid",
          diagnostics: "This is a test error message",
        },
      ],
    } as OperationOutcome,
  },
};
