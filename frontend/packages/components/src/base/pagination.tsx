import { ChevronLeftIcon, ChevronRightIcon } from "@heroicons/react/24/outline";
import classNames from "classnames";
import React, { ButtonHTMLAttributes, DetailedHTMLProps } from "react";

export interface PaginationProps {
  totalPages: number;
  currentPage: number;
  onPagination: (page: number) => void;
}

// Show 5 pages at a time with the current page in the middle
function paginationWindow(
  totalPages: number,
  currentPage: number,
  window: number = 5,
) {
  const half = Math.floor(window / 2);
  const start = Math.max(1, currentPage - half);
  const end = Math.min(totalPages, start + window - 1);

  // Array of pages
  const pages = Array.from({ length: end - start + 1 }, (_, i) => start + i);

  return pages;
}

interface PaginationItemProps
  extends DetailedHTMLProps<
    ButtonHTMLAttributes<HTMLButtonElement>,
    HTMLButtonElement
  > {
  children: React.ReactNode;
  active?: boolean;
  disabled?: boolean;
  label?: string;
}

const PaginationItem = ({
  active,
  disabled,
  label,
  children,
  ...buttonProps
}: PaginationItemProps) => {
  return (
    <li>
      <button
        {...buttonProps}
        type="button"
        aria-current={active ? "page" : undefined}
        aria-label={label}
        disabled={disabled}
        className={classNames(
          "flex h-6 items-center justify-center gap-1 px-2 leading-tight",
          {
            "pointer-events-none opacity-40": disabled,
            "cursor-pointer": !disabled,
            "text-brand-600 border border-slate-300 bg-brand-50 hover:bg-brand-100 hover:text-brand-700":
              active,

            "text-slate-500  border border-slate-300 hover:bg-slate-100 hover:text-slate-700 ":
              !active,
          },
        )}
      >
        {children}
      </button>
    </li>
  );
};

export const Pagination = (props: PaginationProps) => {
  const window = paginationWindow(props.totalPages, props.currentPage);
  const onFirstPage = props.currentPage <= 1;
  const onLastPage = props.currentPage >= props.totalPages;

  return (
    <nav aria-label="Pagination">
      <ul className="inline-flex items-center -space-x-px text-xs h-10 ">
        <PaginationItem
          disabled={onFirstPage}
          label="Previous page"
          onClick={() => {
            props.onPagination(Math.max(props.currentPage - 1, 1));
          }}
        >
          <ChevronLeftIcon className="h-3.5 w-3.5" aria-hidden="true" />
          Previous
        </PaginationItem>
        {window.map((page) => {
          return (
            <PaginationItem
              key={page}
              active={page === props.currentPage}
              label={`Page ${page}`}
              onClick={() => {
                props.onPagination(page);
              }}
            >
              {page}
            </PaginationItem>
          );
        })}
        <PaginationItem
          disabled={onLastPage}
          label="Next page"
          onClick={() => {
            props.onPagination(
              Math.min(props.currentPage + 1, props.totalPages),
            );
          }}
        >
          Next
          <ChevronRightIcon className="h-3.5 w-3.5" aria-hidden="true" />
        </PaginationItem>
      </ul>
    </nav>
  );
};
