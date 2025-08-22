import type { Meta, StoryObj } from "@storybook/react";
import React from "react";

import { Button } from "./button";
import { Toaster, promise } from "./toaster";

const TestPromise = ({
  success,
  loading,
}: {
  success: string;
  loading: string;
}) => {
  return (
    <div>
      <Button
        children="Click Me"
        onClick={() => {
          promise(
            new Promise((resolve) => {
              setTimeout(() => {
                resolve(true);
              }, 1000);
            }),
            {
              loading: loading,
              success: () => success,
              error: (err) => `This just happened: ${err.toString()}`,
            },
          );
        }}
      />
      <Toaster />
    </div>
  );
};

const meta = {
  title: "Base/Toaster",
  component: TestPromise,
  // This component will have an automatically generated Autodocs entry: https://storybook.js.org/docs/react/writing-docs/autodocs
  tags: ["autodocs"],
  parameters: {
    // More on how to position stories at: https://storybook.js.org/docs/react/configure/story-layout
    layout: "centered",
  },
} satisfies Meta<typeof TestPromise>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Primary: Story = {
  args: {
    success: "Success!",
    loading: "Loading...",
  },
};
