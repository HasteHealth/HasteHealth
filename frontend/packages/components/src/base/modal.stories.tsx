import type { Meta, StoryObj } from "@storybook/react";
import React from "react";

import { Modal } from "./modal";

const meta = {
  title: "Base/Modal",
  component: Modal,
  tags: ["autodocs"],
  parameters: {
    layout: "centered",
  },
} satisfies Meta<typeof Modal>;

const ModalContent = ({ setOpen }: { setOpen: (v: boolean) => void }) => {
  return (
    <>
      <div className="mt-2">
        <p className="text-sm text-gray-500">Modal content</p>
      </div>

      <div className="mt-4">
        <button
          type="button"
          className="inline-flex justify-center rounded-md border border-transparent bg-teal-200 px-4 py-2 text-sm font-medium text-teal-900 hover:bg-teal-200 focus:outline-none focus-visible:ring-2 focus-visible:ring-teal-500 focus-visible:ring-offset-2"
          onClick={() => setOpen(false)}
        >
          Close
        </button>
      </div>
    </>
  );
};

export default meta;
type Story = StoryObj<typeof meta>;

export const DropdownMenuStory: Story = {
  args: {
    ModalContent: (setIsOpen) => <ModalContent setOpen={setIsOpen} />,
    children: (setIsOpen) => (
      <span
        className="cursor-pointer"
        onClick={() => {
          setIsOpen(true);
        }}
      >
        Open Modal
      </span>
    ),
  },
};
