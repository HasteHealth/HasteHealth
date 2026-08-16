import classNames from "classnames";
import { atom, useAtom, useAtomValue } from "jotai";
import { useEffect, useMemo, useState } from "react";
import { generatePath, useNavigate } from "react-router";

import { ResourceType } from "@haste-health/fhir-types/r4/types";

import { getCapabilities } from "../db/capabilities";
import Modal from "./Modal";

export const openSearchModalAtom = atom(false);

export const currentIndex = atom(0);

type SearchItem =
  | {
      kind: "page";
      id: string;
      label: string;
      category: string;
      path: string;
    }
  | {
      kind: "resource";
      id: string;
      label: string;
      category: string;
      resourceType: string;
      profile?: string;
    };

// Non-resource-type pages reachable from the sidebar, categorized the same
// way as the sidebar itself so they show up as their own group in search.
const STATIC_PAGES: Readonly<
  { id: string; label: string; category: string; path: string }[]
> = [
  { id: "dashboard", label: "Dashboard", category: "General", path: "/" },
  {
    id: "settings",
    label: "Settings",
    category: "General",
    path: "/settings",
  },
  {
    id: "all-resources",
    label: "All Resources",
    category: "Data",
    path: "/resources",
  },
  {
    id: "system-history",
    label: "Event History",
    category: "Monitoring",
    path: "/history/system",
  },
  {
    id: "indexing-errors",
    label: "Indexing Errors",
    category: "Monitoring",
    path: "/indexing-errors",
  },
  {
    id: "bundle-import",
    label: "Bundles",
    category: "Import",
    path: "/bundle-import",
  },
];

// Resource types that already have a dedicated sidebar shortcut - shown
// under that shortcut's label/category instead of the generic "Resources"
// bucket, so they don't show up twice.
const SIDEBAR_RESOURCE_TYPES: Readonly<
  Record<string, { label: string; category: string }>
> = {
  Patient: { label: "Patients", category: "Clinical" },
  Encounter: { label: "Encounters", category: "Clinical" },
  Observation: { label: "Observations", category: "Clinical" },
  Questionnaire: { label: "Questionnaires", category: "UI" },
  QuestionnaireResponse: {
    label: "Questionnaire Responses",
    category: "UI",
  },
  AuditEvent: { label: "Audit Events", category: "Monitoring" },
  Membership: { label: "Membership", category: "Security" },
  AccessPolicyV2: { label: "Access Policies", category: "Security" },
  ClientApplication: { label: "Client Applications", category: "Security" },
  OperationDefinition: {
    label: "Custom Operations",
    category: "Configuration",
  },
  Subscription: { label: "Subscriptions", category: "Configuration" },
  ViewDefinition: { label: "Projection", category: "Analytics" },
};

const CATEGORY_ORDER: readonly string[] = [
  "General",
  "Clinical",
  "UI",
  "Monitoring",
  "Security",
  "Import",
  "Configuration",
  "Analytics",
  "Data",
  "Resources",
];

function SearchResultItem({
  item,
  active,
  onSelect,
  onHover,
}: Readonly<{
  item: SearchItem;
  active: boolean;
  onSelect: () => void;
  onHover: () => void;
}>) {
  return (
    <div
      onClick={onSelect}
      onMouseEnter={onHover}
      className={classNames(
        "group cursor-pointer px-2 py-1.5 rounded",
        active ? "bg-gray-50" : "",
      )}
    >
      <span className="text-sm group-hover:text-slate-700 font-medium">
        {item.label}
      </span>
      {item.kind === "resource" && item.profile && (
        <div className="text-xs text-slate-400 group-hover:text-slate-500">
          {item.profile}
        </div>
      )}
    </div>
  );
}

type SearchModalProps = {
  resourceTypeFilter?: ResourceType[];
};

function SearchModal(props: SearchModalProps) {
  const [inputSearch, setInputSearch] = useState<HTMLInputElement | null>(null);
  const capabilities = useAtomValue(getCapabilities);
  const [search, setSearch] = useState("");
  const [openModal, setOpenModal] = useAtom(openSearchModalAtom);
  const [searchIndex, setSearchIndex] = useAtom(currentIndex);
  const navigate = useNavigate();

  useEffect(() => {
    if (openModal && inputSearch) {
      inputSearch.focus();
    }
  }, [openModal, inputSearch]);

  const allItems = useMemo((): SearchItem[] => {
    const resourceItems: SearchItem[] = (capabilities?.rest?.[0].resource ?? [])
      .filter((r) => {
        if (
          props.resourceTypeFilter &&
          !props.resourceTypeFilter.includes(r.type as ResourceType)
        ) {
          return false;
        }
        return true;
      })
      .map((r) => {
        const shortcut = SIDEBAR_RESOURCE_TYPES[r.type];
        return {
          kind: "resource",
          id: `resource-${r.type}`,
          label: shortcut?.label ?? r.type,
          category: shortcut?.category ?? "Resources",
          resourceType: r.type,
          profile: r.profile,
        };
      });

    // Static pages don't apply to filtered (e.g. system-level) search
    // contexts, since those routes aren't mounted there.
    const pageItems: SearchItem[] = props.resourceTypeFilter
      ? []
      : STATIC_PAGES.map((page) => ({ kind: "page", ...page }));

    return [...pageItems, ...resourceItems];
  }, [capabilities, props.resourceTypeFilter]);

  const groupedResults = useMemo(() => {
    const query = search.toLowerCase();
    const matches = allItems.filter((item) => {
      if (item.label.toLowerCase().includes(query)) {
        return true;
      }
      // Sidebar-recategorized resource types (e.g. Patient -> "Patients")
      // should still match on their raw FHIR resource type name.
      return (
        item.kind === "resource" &&
        item.resourceType.toLowerCase().includes(query)
      );
    });

    const byCategory = new Map<string, SearchItem[]>();
    for (const item of matches) {
      const bucket = byCategory.get(item.category) ?? [];
      bucket.push(item);
      byCategory.set(item.category, bucket);
    }

    const orderedCategories = [
      ...CATEGORY_ORDER.filter((c) => byCategory.has(c)),
      ...Array.from(byCategory.keys()).filter(
        (c) => !CATEGORY_ORDER.includes(c),
      ),
    ];

    return orderedCategories.map((category) => ({
      category,
      items: byCategory.get(category) ?? [],
    }));
  }, [allItems, search]);

  const flatResults = useMemo(
    () => groupedResults.flatMap((group) => group.items),
    [groupedResults],
  );

  const goToItem = useMemo(() => {
    return (item?: SearchItem) => {
      if (!item) {
        return;
      }

      if (item.kind === "page") {
        navigate(generatePath(item.path, {}));
      } else {
        navigate(
          generatePath("/resources/:resourceType", {
            resourceType: item.resourceType,
          }),
        );
      }

      setOpenModal(false);
      setSearch("");
      setSearchIndex(0);
    };
  }, [navigate, setOpenModal, setSearchIndex]);

  useEffect(() => {
    const keyboardSearch = (e: KeyboardEvent) => {
      if (e.key === "k" && (e.metaKey || e.ctrlKey)) {
        e.preventDefault();
        setOpenModal((open) => !open);
      }

      if (openModal) {
        if (e.key === "ArrowDown") {
          e.preventDefault();
          setSearchIndex((v) => Math.min(v + 1, flatResults.length - 1));
          return;
        }
        if (e.key === "ArrowUp") {
          e.preventDefault();
          setSearchIndex((v) => Math.max(v - 1, 0));
          return;
        }
        if (e.key === "Enter") {
          goToItem(flatResults[searchIndex]);
          return;
        }
      }
    };
    window.addEventListener("keydown", keyboardSearch);
    return () => {
      window.removeEventListener("keydown", keyboardSearch);
    };
  }, [
    openModal,
    flatResults,
    searchIndex,
    setOpenModal,
    setSearchIndex,
    goToItem,
  ]);

  let renderedIndex = -1;

  return (
    <Modal open={openModal} setOpen={() => setOpenModal(false)}>
      <div className="flex flex-1 p-3 space-x-2 items-center focus:outline-none shadow-sm">
        <input
          ref={(ref) => {
            setInputSearch(ref);
          }}
          className="focus:outline-none text-left flex-1 text-slate-500 text-sm"
          placeholder="Search..."
          value={search}
          onChange={(e) => {
            setSearch(e.target.value);
            setSearchIndex(0);
          }}
        />
        <button
          onClick={() => {
            setOpenModal(false);
          }}
          className="shadow-sm cursor flex-none text-xs text-slate-400 p-1 border"
        >
          ESC
        </button>
      </div>
      <div className="w-full" />
      <div className="text-slate-600 px-2 py-2 max-h-96 overflow-y-auto">
        {flatResults.length === 0 && (
          <div className="px-2 py-4 text-sm text-slate-400">No results.</div>
        )}
        {groupedResults.map(
          (group) =>
            group.items.length > 0 && (
              <div key={group.category} className="mb-2">
                <div className="px-2 pt-2 pb-1 text-[11px] font-semibold uppercase tracking-wide text-slate-400">
                  {group.category}
                </div>
                <div className="space-y-0.5">
                  {group.items.map((item) => {
                    renderedIndex += 1;
                    const index = renderedIndex;
                    return (
                      <SearchResultItem
                        key={item.id}
                        item={item}
                        active={index === searchIndex}
                        onSelect={() => goToItem(item)}
                        onHover={() => setSearchIndex(index)}
                      />
                    );
                  })}
                </div>
              </div>
            ),
        )}
      </div>
    </Modal>
  );
}

export default SearchModal;
