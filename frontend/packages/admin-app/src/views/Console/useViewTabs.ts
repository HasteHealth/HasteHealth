/**
 * Tab selection for a panel showing {@link ResourceView}s.
 *
 * A view may have sub-panes rendered as sibling tabs (a projection's editor
 * and its results). Selecting such a tab tells the view which pane to show,
 * and a view changing pane itself moves the tab strip. Both panels need that,
 * so it lives here.
 */
import { useState } from "react";

import { ResourceView } from "./registry";

/** How a panel builds its tabs, given the pane each view should show. */
export type BuildTabs = (
  paneState: Record<string, number>,
  showPane: (key: string, pane: number) => void,
) => ResourceView[];

export interface ViewTabs {
  tabs: ResourceView[];
  selectedTab: number;
  /** Pass to `Tabs` as `onTab`. */
  onTab: (tab: { id: number | string }) => void;
}

export function useViewTabs(build: BuildTabs): ViewTabs {
  const [selectedTab, setSelectedTab] = useState(0);
  const [paneState, setPaneState] = useState<Record<string, number>>({});
  // Where to look up a pane's tab position. Assigned below, before any
  // callback can run.
  let tabs: ResourceView[] = [];

  const showPane = (key: string, pane: number) => {
    setPaneState((current) => ({ ...current, [key]: pane }));
    const index = tabs.findIndex((t) => t.paneKey === key && t.pane === pane);
    if (index !== -1) setSelectedTab(index);
  };

  tabs = build(paneState, showPane);

  const onTab = (tab: { id: number | string }) => {
    const index = tabs.findIndex((t) => t.id === String(tab.id));
    if (index === -1) return;
    setSelectedTab(index);
    const { paneKey, pane } = tabs[index];
    if (paneKey !== undefined && pane !== undefined) {
      setPaneState((current) => ({ ...current, [paneKey]: pane }));
    }
  };

  return { tabs, selectedTab, onTab };
}
