import type { Meta, StoryObj } from "@storybook/react";

import { R4 } from "@haste-health/fhir-types/versions";

import { createStorybookClient } from "../stories.client";
import { FHIRGenerativeSearchTable } from "./searchTable";

const meta = {
  title: "Generated/FHIRGenerativeSearchTable",
  component: FHIRGenerativeSearchTable,
  // This component will have an automatically generated Autodocs entry: https://storybook.js.org/docs/react/writing-docs/autodocs
  tags: ["autodocs"],
  parameters: {
    // More on how to position stories at: https://storybook.js.org/docs/react/configure/story-layout
    // layout: "",
  },
} satisfies Meta<typeof FHIRGenerativeSearchTable>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Primary: Story = {
  args: {
    fhirVersion: R4,
    client: createStorybookClient(),
    resourceType: "Patient",
  },
};
