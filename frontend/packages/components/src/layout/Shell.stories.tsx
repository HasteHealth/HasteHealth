import type { Meta, StoryObj } from "@storybook/react";
import React from "react";

import { Shell } from "./Shell";

const Test = () => <span>Test</span>;

const meta = {
  title: "Layout/Shell",
  component: Shell,
  // This component will have an automatically generated Autodocs entry: https://storybook.js.org/docs/react/writing-docs/autodocs
  tags: ["autodocs"],
  parameters: {
    // More on how to position stories at: https://storybook.js.org/docs/react/configure/story-layout
    layout: "fullscreen",
  },
} satisfies Meta<typeof Shell>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Primary: Story = {
  args: {
    children: React.createElement(Test),
  },
};
