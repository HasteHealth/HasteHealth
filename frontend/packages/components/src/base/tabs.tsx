import { Tab, TabGroup, TabList, TabPanel, TabPanels } from "@headlessui/react";
import classNames from "classnames";
import React from "react";

export interface Tab {
  id: number | string;
  title: string;
  content: React.ReactNode;
}

export interface TabsProps {
  selectedTab?: number;
  onTab?: (tab: Tab) => void;
  tabs: Tab[];
  rightSide?: React.ReactNode;
}

export const Tabs = ({ tabs, rightSide }: TabsProps) => {
  return (
    <div className="flex flex-1 flex-col px-2 sm:px-0 overflow-auto">
      <TabGroup className="flex flex-1 flex-col">
        <TabList className="sticky top-0 z-10 bg-white flex space-x-4 border-b items-center">
          <div className="flex space-x-4 flex-grow">
            {tabs.map((tab) => (
              <Tab
                key={tab.id}
                className={({ selected }) =>
                  classNames(
                    "py-2.5 text-sm font-medium leading-5 border-b-2 outline-none",
                    selected
                      ? "text-orange-600  border-orange-600"
                      : "text-slate-600 hover:text-slate-700 border-transparent"
                  )
                }
              >
                {tab.title}
              </Tab>
            ))}
          </div>
          {rightSide}
        </TabList>
        <TabPanels className="mt-2 flex flex-grow flex-col overflow-auto">
          {tabs.map((tab) => (
            <TabPanel className="flex flex-grow flex-col" key={tab.id}>
              {tab.content}
            </TabPanel>
          ))}
        </TabPanels>
      </TabGroup>
    </div>
  );
};
