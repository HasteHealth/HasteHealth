import type { Meta, StoryObj } from "@storybook/react-webpack5";
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
        <p className="text-xs font-mono text-[var(--ci-muted)]">Modal content</p>
      </div>

      <div className="mt-4">
        <button
          type="button"
          className="inline-flex justify-center rounded-none border border-[var(--ci-line)] bg-[var(--ci-surface-raised)] px-4 py-1.5 font-mono text-xs uppercase tracking-wider text-[var(--ci-ink)] hover:bg-[var(--ci-line)] focus:outline-none focus-visible:ring-1 focus-visible:ring-[var(--ci-signal)]"
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
