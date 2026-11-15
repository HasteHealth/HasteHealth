import { ChevronLeftIcon, ChevronRightIcon } from "@heroicons/react/24/outline";
import classNames from "classnames";
import React, { DetailedHTMLProps, HTMLAttributes } from "react";

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
    HTMLAttributes<HTMLButtonElement>,
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
          "inline-flex h-8 min-w-8 cursor-pointer items-center justify-center gap-1 rounded-none border px-2.5 font-mono text-[11px] leading-none tracking-wider transition-colors",
          "disabled:cursor-not-allowed disabled:opacity-40",
          {
            "border-[var(--ci-signal)] bg-[var(--ci-signal)] font-semibold text-white":
              active,

            "border-[var(--ci-line)] bg-[var(--ci-surface)] text-[var(--ci-muted)] hover:bg-[var(--ci-surface-raised)] hover:text-[var(--ci-ink)]":
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
      <ul className="inline-flex items-center gap-1">
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
