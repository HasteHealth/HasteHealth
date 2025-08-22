import type { Meta, StoryObj } from "@storybook/react";

import { Table } from "./table";

const meta = {
  title: "Base/Table",
  component: Table,
  // This component will have an automatically generated Autodocs entry: https://storybook.js.org/docs/react/writing-docs/autodocs
  tags: ["autodocs"],
  parameters: {
    // More on how to position stories at: https://storybook.js.org/docs/react/configure/story-layout
    layout: "centered",
  },
} satisfies Meta<typeof Table>;

export default meta;
type Story = StoryObj<typeof meta>;

export const ClosedSelect: Story = {
  args: {
    data: [
      { name: "Bob" },
      { name: "Bob2" },
      { name: "Bob3" },
      { name: "Bob4" },
      { name: "Bob5" },
      { name: "Bob6" },
    ],
    columns: [
      {
        id: "v1",
        content: "Name",
        selector: "$this.name",
        selectorType: "fhirpath",
      },
    ],
  },
};
