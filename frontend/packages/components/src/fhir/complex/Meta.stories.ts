import type { Meta, StoryObj } from "@storybook/react";

import { Extension, id, instant } from "@haste-health/fhir-types/r4/types";
import { R4 } from "@haste-health/fhir-types/versions";

import { createStorybookClient } from "../stories.client";
import { FHIRMetaReadOnly } from "./Meta";

// More on how to set up stories at: https://storybook.js.org/docs/react/writing-stories/introduction#default-export
const meta = {
  title: "Complex/FHIRMetaReadOnly",
  component: FHIRMetaReadOnly,
  parameters: {
    // Optional parameter to center the component in the Canvas. More info: https://storybook.js.org/docs/react/configure/story-layout
    layout: "centered",
  },
  // This component will have an automatically generated Autodocs entry: https://storybook.js.org/docs/react/writing-docs/autodocs
  tags: ["autodocs"],
} satisfies Meta<typeof FHIRMetaReadOnly>;

export default meta;

type Story = StoryObj<typeof meta>;

export const Primary: Story = {
  args: {
    client: createStorybookClient(),
    fhirVersion: R4,
    value: {
      versionId: "1" as id,
      lastUpdated: "1980-01-01" as instant,
      extension: [
        {
          url: "https://haste-health.app/author",
          valueReference: { reference: "Patient/123" },
        },
      ] as Extension[],
    },
  },
};

export const OnError: Story = {
  args: {
    // @ts-ignore
    value: "test",
    issue: "Bad value",
  },
};
