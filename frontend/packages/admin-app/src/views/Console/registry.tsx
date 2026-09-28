/**
 * Which views a resource type gets in the console.
 *
 * Every resource has the generic views (JSON, Elements, History). A type can
 * add its own - a ViewDefinition's projection, an OperationDefinition's code -
 * by registering a provider, and the panels read this registry instead of
 * switching on the type.
 *
 * To add a custom view see `views/index.ts`.
 */
import React from "react";

import { Resource } from "@haste-health/fhir-types/r4/types";

/** A view as the console's tab strip takes it. */
export interface ResourceView {
  /** Stable across renders; the tab strip keys and selects on it. */
  id: string;
  title: string;
  content: React.ReactNode;
  /**
   * Selecting this view sets `paneState[paneKey]` to `pane`. A view with
   * sub-panes uses it so its panes can be tabs without the host knowing what
   * they mean.
   */
  paneKey?: string;
  pane?: number;
}

/** What a view knows about the resource it is showing. */
export interface ResourceViewContext {
  /** The resource as edited; views render this, not the fetched copy. */
  resource: Resource;
  resourceType: string;
  /** Absent until the resource has been created. */
  resourceId?: string;
  /** The draft differs from what the server last returned. */
  dirty: boolean;
  /** A specific version is being shown, so nothing can be edited. */
  readOnly: boolean;
  /** False while the resource is still being created. */
  saved: boolean;
  /** Replaces the draft. Views edit through this; they never write. */
  onChange: (resource: Resource) => void;
  /** Sub-pane state the host holds for views that have panes, by key. */
  paneState: Record<string, number>;
  onPaneStateChange: (key: string, pane: number) => void;
}

export interface ResourceViewProvider {
  views: (context: ResourceViewContext) => ResourceView[];
  /** Drop the generic views; for a type whose own view is the whole story. */
  replacesGeneric?: boolean;
}

const PROVIDERS = new Map<string, ResourceViewProvider>();

export function registerResourceViews(
  resourceType: string,
  provider: ResourceViewProvider,
): void {
  PROVIDERS.set(resourceType, provider);
}

/** Whether `resourceType` has custom views, for a caller choosing a layout. */
export function hasCustomViews(resourceType: string | undefined): boolean {
  return resourceType !== undefined && PROVIDERS.has(resourceType);
}

/** Just the custom views, for a host that supplies no generic ones. */
export function customViews(context: ResourceViewContext): ResourceView[] {
  return PROVIDERS.get(context.resourceType)?.views(context) ?? [];
}

/** The views to show: custom ones first, then `generic` unless replaced. */
export function resourceViews(
  context: ResourceViewContext,
  generic: ResourceView[],
): ResourceView[] {
  const provider = PROVIDERS.get(context.resourceType);
  if (!provider) return generic;
  const custom = provider.views(context);
  return provider.replacesGeneric ? custom : [...custom, ...generic];
}
