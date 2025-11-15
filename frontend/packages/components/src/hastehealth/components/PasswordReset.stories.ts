import type { Meta, StoryObj } from "@storybook/react";

import { OperationOutcome } from "@haste-health/fhir-types/r4/types";

import { PasswordResetForm } from "./PasswordReset";

// More on how to set up stories at: https://storybook.js.org/docs/react/writing-stories/introduction#default-export
const meta = {
  title: "HasteHealth/PasswordReset",
  component: PasswordResetForm,
  parameters: {
    // Optional parameter to center the component in the Canvas. More info: https://storybook.js.org/docs/react/configure/story-layout
    layout: "fullscreen",
  },
  // This component will have an automatically generated Autodocs entry: https://storybook.js.org/docs/react/writing-docs/autodocs
  tags: ["autodocs"],
} satisfies Meta<typeof PasswordResetForm>;

export default meta;

type Story = StoryObj<typeof meta>;

export const Primary: Story = {
  args: { code: "123", action: "#" },
};

export const OnError: Story = {
  args: {
    code: "123",
    action: "#",
    error: {
      resourceType: "OperationOutcome",
      issue: [{ code: "invalid", diagnostics: "Password is invalid." }],
    } as OperationOutcome,
  },
};
