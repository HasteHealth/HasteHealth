import type { Meta, StoryObj } from "@storybook/react";

import { Login } from "./Login";

// More on how to set up stories at: https://storybook.js.org/docs/react/writing-stories/introduction#default-export
const meta = {
  title: "HasteHealth/Login",
  component: Login,
  parameters: {
    // Optional parameter to center the component in the Canvas. More info: https://storybook.js.org/docs/react/configure/story-layout
    layout: "fullscreen",
  },
  // This component will have an automatically generated Autodocs entry: https://storybook.js.org/docs/react/writing-docs/autodocs
  tags: ["autodocs"],
} satisfies Meta<typeof Login>;

export default meta;

type Story = StoryObj<typeof meta>;

export const Primary: Story = {
  args: { action: "#" },
};

export const WithErrors: Story = {
  args: {
    action: "#",
    errors: ["Invalid email or password"],
  },
};

export const JustEmail: Story = {
  args: {
    action: "#",
    hidePassword: true,
    signupURL: "singup",
    forgotPasswordURL: "forgot",
  },
};

export const Federated: Story = {
  args: {
    action: "#",
    signupURL: "singup",
    forgotPasswordURL: "forgot",
    federatedProviders: [
      {
        url: "#",
        title: "Google",
        logo: "https://lh3.googleusercontent.com/COxitqgJr1sJnIDe8-jiKhxDx1FrYbtRHKJ9z_hELisAlapwE9LUPh6fcXIfb5vwpbMl4xl9H9TRFPc5NOO8Sb3VSgIBrfRYvW6cUA",
      },
      {
        url: "#",
        title: "Okta",
      },
    ],
  },
};
