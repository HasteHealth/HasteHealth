/** Views for an AccessPolicyV2: what the policy is assigned to. */
import React from "react";

import { AccessPolicyV2, Resource } from "@haste-health/fhir-types/r4/types";

import AccessPolicyAssignments from "../../ResourceEditor/AccessPolicyAssignments";
import { registerResourceViews } from "../registry";

registerResourceViews("AccessPolicyV2", {
  // Assignments are separate resources pointing at this policy's id, so there
  // is nothing to show until it exists.
  views: ({ resource, saved, onChange }) =>
    saved
      ? [
          {
            id: "assignments",
            title: "Assignments",
            content: (
              <AccessPolicyAssignments
                policy={resource as AccessPolicyV2}
                // The panel hands views a plain setter; this component wants a
                // React state dispatch, so resolve an updater against the
                // resource we already hold.
                onChange={(next) =>
                  onChange(
                    (typeof next === "function"
                      ? (next as (p: Resource | undefined) => Resource)(
                          resource,
                        )
                      : next) as Resource,
                  )
                }
              />
            ),
          },
        ]
      : [],
});
