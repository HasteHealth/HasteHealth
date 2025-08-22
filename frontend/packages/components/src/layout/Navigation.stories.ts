import type { Meta, StoryObj } from "@storybook/react";

import { Navigation } from "./Navigation";

const meta = {
  title: "Layout/Navigation",
  component: Navigation,
  // This component will have an automatically generated Autodocs entry: https://storybook.js.org/docs/react/writing-docs/autodocs
  tags: ["autodocs"],
  parameters: {
    // More on how to position stories at: https://storybook.js.org/docs/react/configure/story-layout
    layout: "fullscreen",
  },
} satisfies Meta<typeof Navigation>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Primary: Story = {
  args: {
    navigation: [{ name: "Dashboard" }, { name: "Resources" }],
    userNavigation: [{ name: "Settings" }],
    active: "Dashboard",
    user: {
      email: "jane@doe.com",
      name: "Jane Doe",
    },
  },
};
