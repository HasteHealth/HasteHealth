import { Dialog, Transition } from "@headlessui/react";
import classNames from "classnames";
import React from "react";
import { Fragment, useState } from "react";

export function Modal({
  size = "medium",
  modalTitle,
  ModalContent,
  children,
}: {
  size?: "small" | "medium" | "large" | "x-large";
  modalTitle?: string;
  ModalContent?: (
    setOpen: React.Dispatch<React.SetStateAction<boolean>>
  ) => React.ReactNode;
  children: (
    setOpen: React.Dispatch<React.SetStateAction<boolean>>
  ) => React.ReactNode;
}) {
  const [isOpen, setOpen] = useState(false);

  function closeModal() {
    setOpen(false);
  }

  return (
    <>
      {children(setOpen)}
      <Transition appear show={isOpen} as={Fragment}>
        <Dialog as="div" className="relative z-50" onClose={closeModal}>
          <Transition.Child
            as={Fragment}
            enter="ease-out duration-300"
            enterFrom="opacity-0"
            enterTo="opacity-100"
            leave="ease-in duration-200"
            leaveFrom="opacity-100"
            leaveTo="opacity-0"
          >
            <div className="fixed inset-0 bg-black bg-opacity-25" />
          </Transition.Child>

          <div className="fixed inset-0 overflow-y-auto">
            <div className="flex min-h-full items-center justify-center p-4 text-center">
              <Transition.Child
                as={Fragment}
                enter="ease-out duration-300"
                enterFrom="opacity-0 scale-95"
                enterTo="opacity-100 scale-100"
                leave="ease-in duration-200"
                leaveFrom="opacity-100 scale-100"
                leaveTo="opacity-0 scale-95"
              >
                <Dialog.Panel
                  className={classNames(
                    "w-full transform overflow-hidden rounded-2xl bg-white p-6 text-left align-middle shadow-xl transition-all",
                    {
                      "max-w-md": size === "small",
                      "max-w-lg": size === "medium",
                      "max-w-2xl": size === "large",
                      "max-w-4xl": size === "x-large",
                    }
                  )}
                >
                  <Dialog.Title
                    as="h3"
                    className="text-lg font-medium leading-6 text-gray-900"
                  >
                    {modalTitle}
                  </Dialog.Title>
                  {isOpen && ModalContent && ModalContent(setOpen)}
                </Dialog.Panel>
              </Transition.Child>
            </div>
          </div>
        </Dialog>
      </Transition>
    </>
  );
}
