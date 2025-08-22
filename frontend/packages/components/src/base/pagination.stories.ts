import type { Meta, StoryObj } from "@storybook/react";

import { Pagination } from "./pagination";

const meta = {
  title: "Base/Pagination",
  component: Pagination,
  tags: ["autodocs"],
  parameters: {
    layout: "centered",
  },
} satisfies Meta<typeof Pagination>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Primary: Story = {
  args: {
    currentPage: 5,
    totalPages: 10,
    onPagination: (e) => {
      console.log("pagination");
    },
  },
};
