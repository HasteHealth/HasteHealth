import type { Meta, StoryObj } from "@storybook/react";

import { ContactDetail } from "@haste-health/fhir-types/r4/types";
import { R4 } from "@haste-health/fhir-types/versions";

import { createStorybookClient } from "../stories.client";
import { FHIRContactDetailEditable } from "./ContactDetail";

// More on how to set up stories at: https://storybook.js.org/docs/react/writing-stories/introduction#default-export
const meta = {
  title: "Complex/FHIRContactDetailEditable",
  component: FHIRContactDetailEditable,
  parameters: {
    // Optional parameter to center the component in the Canvas. More info: https://storybook.js.org/docs/react/configure/story-layout
    layout: "centered",
  },
  // This component will have an automatically generated Autodocs entry: https://storybook.js.org/docs/react/writing-docs/autodocs
  tags: ["autodocs"],
} satisfies Meta<typeof FHIRContactDetailEditable>;

export default meta;

type Story = StoryObj<typeof meta>;

const client = createStorybookClient();

export const Primary: Story = {
  args: {
    fhirVersion: R4,
    client: client,
    value: {
      name: "John Doe",
      telecom: [
        {
          system: "phone",
          value: "555-555-5555",
        },
      ],
    } as ContactDetail,
    label: "Contact Detail",
    onChange: (value) => console.log(value),
  },
};

export const OnError: Story = {
  args: {
    fhirVersion: R4,
    // @ts-ignore
    value: "test",
    issue: "Bad value",
    onChange: (value) => console.log(value),
  },
};
