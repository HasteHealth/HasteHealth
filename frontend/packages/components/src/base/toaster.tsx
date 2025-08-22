import { CheckCircleIcon, XCircleIcon } from "@heroicons/react/24/solid";
import React from "react";
import toast, { DefaultToastOptions, Toaster } from "react-hot-toast";

import { Loading } from "./loading";

const defaultToastOptions: DefaultToastOptions = {
  loading: {
    icon: <Loading className="w-4 h-4" />,
  },
  success: {
    icon: <CheckCircleIcon className="w-4 h-4 text-green-500" />,
  },
  error: {
    icon: <XCircleIcon className="w-4 h-4 text-red-500" />,
  },
};

export const notify = (...args: Parameters<typeof toast>) =>
  toast(args[0], { ...defaultToastOptions, ...args[1] });

export const loading = (...args: Parameters<typeof toast.loading>) =>
  toast.loading(args[0], { ...defaultToastOptions, ...args[1] });

export const error = (...args: Parameters<typeof toast.error>) =>
  toast.error(args[0], { ...defaultToastOptions, ...args[1] });

export const success = (...args: Parameters<typeof toast.success>) =>
  toast.success(args[0], { ...defaultToastOptions, ...args[1] });

export const promise = (...args: Parameters<typeof toast.promise>) =>
  toast.promise(args[0], args[1], { ...defaultToastOptions, ...args[2] });

export { Toaster };
