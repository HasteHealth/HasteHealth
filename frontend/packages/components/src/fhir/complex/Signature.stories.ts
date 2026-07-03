import type { Meta, StoryObj } from "@storybook/react-webpack5";

import { Signature } from "@haste-health/fhir-types/r4/types";
import { R4 } from "@haste-health/fhir-types/versions";

import { createStorybookClient } from "../stories.client";
import { FHIRSignatureEditable } from "./Signature";

// More on how to set up stories at: https://storybook.js.org/docs/react/writing-stories/introduction#default-export
const meta = {
  title: "Complex/FHIRSignatureEditable",
  component: FHIRSignatureEditable,
  parameters: {
    // Optional parameter to center the component in the Canvas. More info: https://storybook.js.org/docs/react/configure/story-layout
    layout: "centered",
  },
  // This component will have an automatically generated Autodocs entry: https://storybook.js.org/docs/react/writing-docs/autodocs
  tags: ["autodocs"],
} satisfies Meta<typeof FHIRSignatureEditable>;

export default meta;

type Story = StoryObj<typeof meta>;

export const Primary: Story = {
  args: {
    fhirVersion: R4,
    client: createStorybookClient(),
    value: {
      type: [
        {
          system: "urn:iso-astm:E1762-95:2013",
          code: "1.2.840.10065.1.12.1.1",
        },
      ],
      when: "2026-05-14T12:00:00Z",
      who: {
        reference: "Practitioner/123",
      },
    } as Signature,
    label: "Signature",
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
