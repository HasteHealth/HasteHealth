import { Combobox, Transition } from "@headlessui/react";
import { CheckIcon, ChevronUpDownIcon } from "@heroicons/react/20/solid";
import classNames from "classnames";
import React, { Fragment, useState } from "react";

import { DisplayIssues, Label, inputClassNames } from "./containers";

export type Option = { value: string | number; label: string };
export interface SelectProps {
  value?: string | number;
  options: Array<Option>;
  onChange?: (value: Option) => void;
  issue?: string;
  label?: string;
  open?: boolean;
  required?: boolean;
}

export const Select = ({
  value,
  options,
  label,
  issue,
  open = false,
  onChange,
  required,
}: SelectProps) => {
  const [query, setQuery] = useState("");

  const filteredOptions =
    query === ""
      ? options
      : options.filter((option) =>
          option.label
            .toLowerCase()
            .replace(/\s+/g, "")
            .includes(query.toLowerCase().replace(/\s+/g, ""))
        );

  return (
    <Combobox
      value={
        value
          ? options.filter((o) => o.value === value)[0] || {
              value,
              label: value,
            }
          : undefined
      }
      onChange={onChange}
      nullable
    >
      <div className="relative">
        <div className="flex flex-col flex-grow">
          <Label
            required={required}
            label={label}
            className={classNames("mr-1 ")}
          />
          <div className="relative">
            <Combobox.Input
              className={classNames(
                "outline-none w-full leading-5 text-gray-900 focus:ring-0",
                inputClassNames({
                  hideBorder: false,
                  issues: issue ? [issue] : [],
                })
              )}
              displayValue={(option: Option) => option && option.label}
              onChange={(event) => setQuery(event.target.value)}
            />
            <Combobox.Button
              className={classNames(
                "absolute inset-y-0 right-0 flex items-center pr-2"
              )}
            >
              <ChevronUpDownIcon
                className="h-5 w-5 text-gray-400"
                aria-hidden="true"
              />
            </Combobox.Button>
          </div>
          <DisplayIssues issues={issue ? [issue] : []} />
        </div>
        <Transition
          as={Fragment}
          leave="transition ease-in duration-100"
          leaveFrom="opacity-100"
          leaveTo="opacity-0"
          afterLeave={() => setQuery("")}
        >
          <Combobox.Options className="z-10 absolute mt-1 max-h-60 w-full overflow-auto rounded-md bg-white py-1 text-base shadow-lg ring-1 ring-black ring-opacity-5 focus:outline-none sm:text-sm">
            {query.length > 0 && open && (
              <Combobox.Option
                value={{ value: query, label: query }}
                className={({ active }) =>
                  classNames(
                    "relative cursor-default select-none py-2 pl-10 pr-4",
                    {
                      "bg-orange-500 text-white": active,
                      "text-gray-900": !active,
                    }
                  )
                }
              >
                Create "{query}"
              </Combobox.Option>
            )}
            {filteredOptions.map((option: Option) => (
              <Combobox.Option
                key={option.value}
                className={({ active }) =>
                  classNames(
                    "relative cursor-default select-none py-2 pl-10 pr-4",
                    {
                      "bg-orange-500 text-white": active,
                      "text-gray-900": !active,
                    }
                  )
                }
                value={option}
              >
                {({ selected, active }) => (
                  <>
                    <span
                      className={classNames("block truncate", {
                        "font-medium": selected,
                        "font-normal": !selected,
                      })}
                    >
                      {option.label}
                    </span>
                    {selected ? (
                      <span
                        className={classNames(
                          "absolute inset-y-0 left-0 flex items-center pl-3",
                          {
                            "text-white": active,
                            "text-orange-600": !active,
                          }
                        )}
                      >
                        <CheckIcon className="h-5 w-5" aria-hidden="true" />
                      </span>
                    ) : null}
                  </>
                )}
              </Combobox.Option>
            ))}
          </Combobox.Options>
        </Transition>
      </div>
    </Combobox>
  );
};
