/**
 * Views for a ViewDefinition: the projection, and the rows it produces.
 *
 * The SQL runner would render its own tab strip, which inside the panel's
 * tabs reads as two stacked rows. Instead it is shown one pane at a time and
 * its panes become siblings of the panel's own tabs. That mounts a runner per
 * tab, so the runner's state (the run's options and its results) is held by
 * the host and handed to each of them; otherwise a run's results would vanish
 * with the editor pane that produced them.
 */
import { useAtomValue } from "jotai";
import React from "react";

import {
  VIEW_DEFINITION_PANES,
  ViewDefinitionRunnerState,
  ViewDefinitionSqlRunner,
} from "@haste-health/components";
import {
  Resource,
  ViewDefinition,
  instant,
} from "@haste-health/fhir-types/r4/types";
import { R4 } from "@haste-health/fhir-types/versions";

import { getClient } from "../../../db/client";
import { registerResourceViews } from "../registry";
import { JSON_EXTENSIONS, PanelHint } from "./shared";

/** Far enough back that a new view still returns rows. */
const SINCE = "1980-01-01T00:00:00Z" as instant;

/** Key the pane shown is stored under; see `paneState` on the registry. */
const PANE_KEY = "view";

/** Key the runner's state is stored under; see `viewState` on the registry. */
const STATE_KEY = "view-runner";

/** How the runner reports a change to the state the host holds. */
type RunnerStateChange = (
  update: (
    current: ViewDefinitionRunnerState | undefined,
  ) => ViewDefinitionRunnerState,
) => void;

function ViewPane({
  view,
  pane,
  readOnly,
  state,
  onStateChange,
  onPaneChange,
  onChange,
}: Readonly<{
  view: ViewDefinition;
  pane: number;
  readOnly: boolean;
  state: ViewDefinitionRunnerState | undefined;
  onStateChange: RunnerStateChange;
  onPaneChange: (pane: number) => void;
  onChange: (resource: Resource) => void;
}>) {
  const client = useAtomValue(getClient);

  return (
    <div className="flex min-h-0 flex-1 flex-col">
      {pane === 0 && (
        <PanelHint>
          The columns this view projects. Run it to see the rows.
        </PanelHint>
      )}
      <ViewDefinitionSqlRunner
        client={client}
        viewDefinition={view}
        setViewDefinition={
          readOnly ? () => {} : (next) => onChange(next as Resource)
        }
        editorExtensions={JSON_EXTENSIONS}
        defaultPageSize={10}
        fhirVersion={R4}
        since={SINCE}
        activePane={pane}
        onActivePaneChange={onPaneChange}
        state={state}
        onStateChange={onStateChange}
      />
    </div>
  );
}

registerResourceViews("ViewDefinition", {
  views: ({
    resource,
    readOnly,
    onChange,
    onPaneStateChange,
    viewState,
    onViewStateChange,
  }) => {
    const state = viewState[STATE_KEY] as ViewDefinitionRunnerState | undefined;
    const onStateChange: RunnerStateChange = (update) =>
      onViewStateChange(STATE_KEY, (current) =>
        update(current as ViewDefinitionRunnerState | undefined),
      );

    return VIEW_DEFINITION_PANES.map((definition) => ({
      id: `view-${definition.id}`,
      title: definition.id === 0 ? "Projection" : definition.title,
      paneKey: PANE_KEY,
      pane: definition.id,
      content: (
        <ViewPane
          view={resource as ViewDefinition}
          pane={definition.id}
          readOnly={readOnly}
          state={state}
          onStateChange={onStateChange}
          onPaneChange={(next) => onPaneStateChange(PANE_KEY, next)}
          onChange={onChange}
        />
      ),
    }));
  },
});
