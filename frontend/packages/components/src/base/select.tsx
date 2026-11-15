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
  id?: string;
  ariaLabel?: string;
  open?: boolean;
  required?: boolean;
}

export const Select = ({
  value,
  options,
  label,
  id,
  ariaLabel,
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
            .includes(query.toLowerCase().replace(/\s+/g, "")),
        );

  return (
    <Combobox
      value={
        value
          ? options.find((o) => o.value === value) || {
              value,
              label: value,
            }
          : undefined
      }
      onChange={onChange}
      nullable
    >
      <div className="relative">
        <div className="flex flex-col grow gap-0.5">
          <Label
            required={required}
            label={label}
            className={classNames(
              "block text-xs font-mono uppercase tracking-wider text-[var(--ci-muted)]",
            )}
          />
          <div className="relative">
            <Combobox.Input
              id={id}
              aria-label={ariaLabel}
              className={classNames(
                "outline-none w-full leading-5 text-[var(--ci-ink)] bg-transparent font-mono text-xs focus:ring-0 pr-8",
                inputClassNames({
                  hideBorder: false,
                  issues: issue ? [issue] : [],
                }),
              )}
              displayValue={(option: Option) => option && option.label}
              onChange={(event) => setQuery(event.target.value)}
            />
            <Combobox.Button
              aria-label="Open options"
              className={classNames(
                "absolute inset-y-0 right-0 flex cursor-pointer items-center px-2",
              )}
            >
              <ChevronUpDownIcon
                className="h-4 w-4 text-[var(--ci-muted)]"
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
          <Combobox.Options className="z-30 absolute mt-1 max-h-60 w-full overflow-auto rounded-none border border-[var(--ci-line)] bg-[var(--ci-surface)] py-1 font-mono text-xs shadow-2xl focus:outline-none">
            {query.length > 0 && open && (
              <Combobox.Option
                value={{ value: query, label: query }}
                className={({ active }) =>
                  classNames(
                    "relative cursor-default select-none py-1.5 pl-8 pr-4 font-mono text-xs transition-colors",
                    {
                      "bg-[var(--ci-signal)] text-[var(--ci-on-signal)] font-medium": active,
                      "text-[var(--ci-ink)] hover:bg-[var(--ci-surface-raised)]": !active,
                    },
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
                    "relative cursor-default select-none py-1.5 pl-8 pr-4 font-mono text-xs transition-colors",
                    {
                      "bg-[var(--ci-signal)] text-[var(--ci-on-signal)] font-medium": active,
                      "text-[var(--ci-ink)] hover:bg-[var(--ci-surface-raised)]": !active,
                    },
                  )
                }
                value={option}
              >
                {({ selected, active }) => (
                  <>
                    <span
                      className={classNames("block truncate", {
                        "font-semibold": selected,
                        "font-normal": !selected,
                      })}
                    >
                      {option.label}
                    </span>
                    {selected ? (
                      <span
                        className={classNames(
                          "absolute inset-y-0 left-0 flex items-center pl-2",
                          {
                            "text-white": active,
                            "text-[var(--ci-signal)]": !active,
                          },
                        )}
                      >
                        <CheckIcon className="h-4 w-4" aria-hidden="true" />
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
