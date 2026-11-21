/**
 * Tab selection for a panel showing {@link ResourceView}s.
 *
 * A view may have sub-panes rendered as sibling tabs (a projection's editor
 * and its results). Selecting such a tab tells the view which pane to show,
 * and a view changing pane itself moves the tab strip. The state a view keeps
 * across those tabs (the results a run produced) is held here as well, since
 * the tab that produced it is unmounted once another is selected. Both panels
 * need that, so it lives here.
 */
import { useState } from "react";

import { ResourceView, ViewHostState } from "./registry";

/** How a panel builds its tabs, given the state the host holds for them. */
export type BuildTabs = (host: ViewHostState) => ResourceView[];

export interface ViewTabs {
  tabs: ResourceView[];
  selectedTab: number;
  /** Pass to `Tabs` as `onTab`. */
  onTab: (tab: { id: number | string }) => void;
}

/**
 * @param scope What the panel is showing, e.g. the resource's path. The view
 *   state is dropped when it changes, so one resource's results never show
 *   under another.
 */
export function useViewTabs(build: BuildTabs, scope?: string): ViewTabs {
  const [selectedTab, setSelectedTab] = useState(0);
  const [paneState, setPaneState] = useState<Record<string, number>>({});
  const [viewState, setViewState] = useState<Record<string, unknown>>({});
  const [viewStateScope, setViewStateScope] = useState(scope);
  if (scope !== viewStateScope) {
    setViewStateScope(scope);
    setViewState({});
  }
  // Where to look up a pane's tab position. Assigned below, before any
  // callback can run.
  let tabs: ResourceView[] = [];

  const showPane = (key: string, pane: number) => {
    setPaneState((current) => ({ ...current, [key]: pane }));
    const index = tabs.findIndex((t) => t.paneKey === key && t.pane === pane);
    if (index !== -1) setSelectedTab(index);
  };

  const onViewStateChange = (
    key: string,
    update: (current: unknown) => unknown,
  ) => {
    setViewState((current) => ({ ...current, [key]: update(current[key]) }));
  };

  tabs = build({
    paneState,
    onPaneStateChange: showPane,
    viewState,
    onViewStateChange,
  });

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
