import type { Meta, StoryObj } from "@storybook/react";
import { basicSetup } from "codemirror";
import React from "react";

import { CodeMirror, CodeMirrorProps } from "./codemirror";

const BasicCodeMirror = ({
  readOnly,
  value,
  autoFocus,
  theme,
  onChange,
}: Pick<
  CodeMirrorProps,
  "value" | "autoFocus" | "theme" | "onChange" | "readOnly"
>) => {
  return (
    <CodeMirror
      readOnly={readOnly}
      value={value}
      autoFocus={autoFocus}
      theme={theme}
      onChange={onChange}
      extensions={[basicSetup]}
    />
  );
};

const meta = {
  title: "Base/CodeMirror",
  component: BasicCodeMirror,
  tags: ["autodocs"],
  parameters: {
    layout: "centered",
  },
} satisfies Meta<typeof BasicCodeMirror>;

export default meta;
type Story = StoryObj<typeof meta>;

export const BasicSetup: Story = {
  args: {
    value: "primary",
    onChange: (value: string) => console.log(value),
    autoFocus: true,
    theme: {
      "&": {
        height: "400px",
        width: "400px",
      },
    },
  },
};

export const ReadOnly: Story = {
  args: {
    value: "primary",
    onChange: (value: string) => console.log(value),
    readOnly: true,
    theme: {
      "&": {
        height: "400px",
        width: "400px",
      },
    },
  },
};
