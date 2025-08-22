import type { Meta, StoryObj } from "@storybook/react";

import { Feedback } from "./Feedback";

// More on how to set up stories at: https://storybook.js.org/docs/react/writing-stories/introduction#default-export
const meta = {
  title: "OxidizedHealth/Feedback",
  component: Feedback,
  parameters: {
    // Optional parameter to center the component in the Canvas. More info: https://storybook.js.org/docs/react/configure/story-layout
    layout: "fullscreen",
  },
  // This component will have an automatically generated Autodocs entry: https://storybook.js.org/docs/react/writing-docs/autodocs
  tags: ["autodocs"],
} satisfies Meta<typeof Feedback>;

export default meta;

type Story = StoryObj<typeof meta>;

export const Form: Story = {
  args: {
    title: "OxidizedHealth",
    header: "Email Verification",
    content:
      "We have sent an email to your email address. Please verify your email address to login.",
  },
};
