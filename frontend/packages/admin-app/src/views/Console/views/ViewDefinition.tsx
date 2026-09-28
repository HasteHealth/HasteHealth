/**
 * Views for a ViewDefinition: the projection, and the rows it produces.
 *
 * The SQL runner would render its own tab strip, which inside the panel's
 * tabs reads as two stacked rows. Instead it is shown one pane at a time and
 * its panes become siblings of the panel's own tabs.
 */
import { useAtomValue } from "jotai";
import React from "react";

import {
  VIEW_DEFINITION_PANES,
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

/** Key the panel state is stored under; see `paneState` on the registry. */
const PANE_KEY = "view";

function ViewPane({
  view,
  pane,
  readOnly,
  onPaneChange,
  onChange,
}: Readonly<{
  view: ViewDefinition;
  pane: number;
  readOnly: boolean;
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
      />
    </div>
  );
}

registerResourceViews("ViewDefinition", {
  views: ({ resource, readOnly, onChange, paneState, onPaneStateChange }) =>
    VIEW_DEFINITION_PANES.map((definition) => ({
      id: `view-${definition.id}`,
      title: definition.id === 0 ? "Projection" : definition.title,
      paneKey: PANE_KEY,
      pane: definition.id,
      content: (
        <ViewPane
          view={resource as ViewDefinition}
          pane={definition.id}
          readOnly={readOnly}
          onPaneChange={(next) => onPaneStateChange(PANE_KEY, next)}
          onChange={onChange}
        />
      ),
    })),
});
