import { Dialog, Transition } from "@headlessui/react";
import classNames from "classnames";
import { atom, useAtom, useAtomValue } from "jotai";
import {
  CSSProperties,
  ComponentType,
  Fragment,
  useEffect,
  useMemo,
  useState,
} from "react";
import { generatePath, useNavigate } from "react-router-dom";
import {
  FixedSizeListProps,
  FixedSizeList as _FixedSizeList,
} from "react-window";

import {
  CapabilityStatementRestResource,
  ResourceType,
} from "@oxidized-health/fhir-types/r4/types";

import { getCapabilities } from "../db/capabilities";
import Modal from "./Modal";

const List = _FixedSizeList as ComponentType<FixedSizeListProps>;

export const openSearchModalAtom = atom(false);

export const currentIndex = atom(0);

function SearchResultItem({
  data,
  index,
  style,
}: Readonly<{
  data: {
    data: CapabilityStatementRestResource[];
    activeIndex: number;
    setActiveIndex: (v: number) => void;
    onSelect: () => void;
  };
  index: number;
  style: CSSProperties;
}>) {
  const resource = data.data[index];
  return (
    <div
      key={resource.type}
      onClick={() => {
        data.onSelect();
      }}
      style={style}
      onMouseEnter={() => data.setActiveIndex(index)}
      className={classNames("group cursor-pointer px-2 py-1 rounded ", {
        "bg-gray-50": index === data.activeIndex,
      })}
    >
      <div>
        <span className="text-sm group-hover:text-slate-700 font-weight">
          {resource.type}
        </span>
      </div>
      <div>
        <span className="text-xs text-slate-400 group-hover:text-slate-500">
          {resource.profile}
        </span>
      </div>
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

  useEffect(() => {
    if (openModal && inputSearch) {
      inputSearch.focus();
    }
  }, [openModal, inputSearch]);

  const searchResults = useMemo(() => {
    return capabilities?.rest?.[0].resource?.filter((r) => {
      if (
        props.resourceTypeFilter &&
        !props.resourceTypeFilter.includes(r.type as ResourceType)
      ) {
        return false;
      }

      return r.type.toLowerCase().includes(search.toLowerCase());
      // ||
      // r.profile?.toLowerCase().includes(search.toLowerCase())
    });
  }, [search, capabilities]);

  const onSelect = useMemo(() => {
    return () => {
      if (searchResults?.[searchIndex]?.type) {
        navigate(
          generatePath("/resources/:resourceType", {
            resourceType: searchResults?.[searchIndex]?.type,
          })
        );
        setOpenModal(false);
        setSearch("");
        return;
      }
    };
  }, [searchIndex, searchResults, setOpenModal, setSearch]);

  const navigate = useNavigate();

  useEffect(() => {
    const keyboardSearch = (e: KeyboardEvent) => {
      if (e.key === "k" && (e.metaKey || e.ctrlKey)) {
        e.preventDefault();
        setOpenModal((open) => !open);
      }

      if (openModal) {
        if (e.key === "ArrowDown") {
          setSearchIndex((v) =>
            Math.min(v + 1, searchResults ? searchResults.length - 1 : 0)
          );
          return;
        }
        if (e.key === "ArrowUp") {
          setSearchIndex((v) => Math.max(v - 1, 0));
          return;
        }
        if (e.key === "Enter") {
          onSelect();
          return;
        }
      }
    };
    window.addEventListener("keydown", keyboardSearch);
    return () => {
      window.removeEventListener("keydown", keyboardSearch);
    };
  }, [openModal, searchResults, searchIndex, setOpenModal]);

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
      <div className="w-full " />
      <div className="text-slate-600 space-y-2 px-2 py-2 max-h-64 overflow-y-auto">
        <List
          height={200}
          itemData={{
            data: searchResults || [],
            activeIndex: searchIndex,
            setActiveIndex: setSearchIndex,
            onSelect,
          }}
          itemCount={searchResults ? searchResults.length : 0}
          itemSize={55}
          width={"100%"}
        >
          {SearchResultItem}
        </List>
      </div>
    </Modal>
  );
}

export default SearchModal;
