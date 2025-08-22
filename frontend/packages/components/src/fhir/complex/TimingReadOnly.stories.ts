import type { Meta, StoryObj } from "@storybook/react";

import { Timing } from "@oxidized-health/fhir-types/r4/types";

import { FHIRTimingReadOnly } from "./TimingReadOnly";

// More on how to set up stories at: https://storybook.js.org/docs/react/writing-stories/introduction#default-export
const meta = {
  title: "Complex/FHIRTimingReadOnly",
  component: FHIRTimingReadOnly,
  parameters: {
    // Optional parameter to center the component in the Canvas. More info: https://storybook.js.org/docs/react/configure/story-layout
    layout: "centered",
  },
  // This component will have an automatically generated Autodocs entry: https://storybook.js.org/docs/react/writing-docs/autodocs
  tags: ["autodocs"],
} satisfies Meta<typeof FHIRTimingReadOnly>;

export default meta;

type Story = StoryObj<typeof meta>;

export const Duration: Story = {
  args: {
    value: {
      repeat: {
        boundsDuration: {
          value: 15,
          unit: "seconds",
        },
      },
    } as Timing,
  },
};

export const Period: Story = {
  args: {
    value: {
      repeat: {
        boundsPeriod: {
          start: "2021-01-01",
          end: "2021-01-15",
        },
      },
    } as Timing,
  },
};

export const Range: Story = {
  args: {
    value: {
      repeat: {
        boundsRange: {
          low: {
            value: 15,
            unit: "seconds",
          },
          high: {
            value: 30,
            unit: "seconds",
          },
        },
      },
    } as Timing,
  },
};

export const BadValue: Story = {
  args: {
    // @ts-ignore
    value: "test",
    issue: "Bad value",
  },
};
