import { useAtomValue } from "jotai";
import { useCallback, useEffect, useState } from "react";
import { Link, generatePath } from "react-router";

import {
  Button,
  FHIRReferenceEditable,
  Loading,
  Tag,
  Toaster,
} from "@haste-health/components";
import {
  AccessPolicyV2,
  AccessPolicyV2Assignment,
  Bundle,
  Reference,
  Resource,
  ResourceType,
  code,
  id,
  uri,
} from "@haste-health/fhir-types/r4/types";
import { R4 } from "@haste-health/fhir-types/versions";

import { getClient } from "../../db/client";
import { getErrorMessage } from "../../utilities";

// How long to wait for a write to appear in search results, which are
// indexed asynchronously.
const INDEX_POLL_INTERVAL_MS = 500;
const INDEX_WAIT_TIMEOUT_MS = 15_000;

/** Resource types an access policy can be assigned to. */
const ASSIGNABLE_TYPES: ResourceType[] = [
  "Membership",
  "ClientApplication",
  "OperationDefinition",
];

const TYPE_COLORS: Record<string, "blue" | "purple" | "indigo" | "gray"> = {
  Membership: "blue",
  ClientApplication: "purple",
  OperationDefinition: "indigo",
};

function splitReference(reference: Reference | undefined) {
  const [resourceType, resourceId] = reference?.reference?.split("/") ?? [];
  return { resourceType, resourceId };
}

/** A readable name for the resource a policy is assigned to. */
function describe(resource: Resource | undefined): string | undefined {
  switch (resource?.resourceType) {
    case "ClientApplication":
    case "OperationDefinition":
      return resource.name;
    case "Membership":
      return (
        resource.user?.display ??
        resource.user?.reference ??
        resource.link?.reference
      );
    default:
      return undefined;
  }
}

function AssignmentType({ reference }: Readonly<{ reference: Reference }>) {
  const { resourceType } = splitReference(reference);
  return (
    <Tag color={TYPE_COLORS[resourceType] ?? "gray"}>
      {resourceType ?? "Unknown"}
    </Tag>
  );
}

function AssignmentTarget({
  reference,
  resolved,
}: Readonly<{
  reference: Reference;
  resolved: Record<string, Resource | undefined>;
}>) {
  const { resourceType, resourceId } = splitReference(reference);
  const target = reference.reference
    ? resolved[reference.reference]
    : undefined;
  const name = describe(target);

  return (
    <div className="flex min-w-0 flex-col gap-0.5">
        {resourceType && resourceId ? (
          <Link
            className="truncate font-mono text-xs font-medium text-[var(--ci-signal)] hover:underline"
            to={generatePath("/r/:resourceType/:id", {
              resourceType,
              id: resourceId,
            })}
            title={name ?? resourceId}
          >
            {name ?? resourceId}
          </Link>
        ) : (
          <span className="truncate font-medium">{reference.reference}</span>
        )}
        {name && (
          <span
            className="truncate font-mono text-[11px] text-[var(--ci-muted)]"
            title={reference.reference}
          >
            {reference.reference}
          </span>
        )}
        {reference.reference &&
          reference.reference in resolved &&
          target === undefined && (
            <span className="text-xs text-red-600">Not found</span>
          )}
    </div>
  );
}

function AddAssignment({
  onAdd,
}: Readonly<{
  onAdd: (link: Reference) => Promise<unknown>;
}>) {
  const client = useAtomValue(getClient);
  const [link, setLink] = useState<Reference | undefined>();

  return (
    <div className="flex flex-wrap items-end gap-2 rounded-none border border-[var(--ci-line)] bg-[var(--ci-surface)] p-3">
      <div className="flex-1 min-w-48">
        <FHIRReferenceEditable
          label="Assign to (Membership, ClientApplication or OperationDefinition)"
          resourceTypesAllowed={ASSIGNABLE_TYPES}
          fhirVersion={R4}
          client={client}
          value={link}
          onChange={setLink}
        />
      </div>
      <Button
        buttonType="primary"
        buttonSize="small"
        disabled={!link?.reference}
        onClick={(e) => {
          e.preventDefault();
          if (!link) return;
          onAdd(link)
            .then(() => setLink(undefined))
            .catch(() => undefined);
        }}
      >
        Add assignment
      </Button>
    </div>
  );
}

/**
 * Lists everything an access policy is assigned to (its
 * `AccessPolicyV2Assignment` resources, plus any deprecated
 * `AccessPolicyV2.target` entries) and lets assignments be added, changed and
 * removed.
 */
export default function AccessPolicyAssignments({
  policy,
  onChange,
}: Readonly<{
  policy: AccessPolicyV2 | undefined;
  onChange: React.Dispatch<React.SetStateAction<Resource | undefined>>;
}>) {
  const client = useAtomValue(getClient);
  const [loading, setLoading] = useState(true);
  // True while waiting for a write to show up in search results.
  const [syncing, setSyncing] = useState(false);
  const [assignments, setAssignments] = useState<AccessPolicyV2Assignment[]>(
    [],
  );
  // Resolved assignment targets keyed by reference (`Type/id`).
  const [resolved, setResolved] = useState<
    Record<string, Resource | undefined>
  >({});

  const policyReference = policy?.id ? `AccessPolicyV2/${policy.id}` : null;
  const legacyTargets = policy?.target ?? [];

  const search = useCallback(async () => {
    if (!policyReference) return [];
    const response = await client.search_type(
      {},
      R4,
      "AccessPolicyV2Assignment",
      [
        { name: "access-policy", value: [policyReference] },
        { name: "_count", value: [500] },
      ],
    );
    return response.resources;
  }, [client, policyReference]);

  useEffect(() => {
    let cancelled = false;
    setLoading(true);
    search()
      .then((results) => {
        if (!cancelled) setAssignments(results);
      })
      .catch((e) => Toaster.error(getErrorMessage(e)))
      .finally(() => {
        if (!cancelled) setLoading(false);
      });
    return () => {
      cancelled = true;
    };
  }, [search]);

  /**
   * Search indexing happens asynchronously after a write, so re-query until the
   * results reflect the change (`isIndexed`) or the timeout passes, then show
   * the latest results either way.
   */
  const refreshUntil = useCallback(
    async (isIndexed: (results: AccessPolicyV2Assignment[]) => boolean) => {
      setSyncing(true);
      try {
        const deadline = Date.now() + INDEX_WAIT_TIMEOUT_MS;
        let results = await search();
        while (!isIndexed(results) && Date.now() < deadline) {
          await new Promise((resolve) =>
            setTimeout(resolve, INDEX_POLL_INTERVAL_MS),
          );
          results = await search();
        }
        setAssignments(results);
      } catch (e) {
        Toaster.error(getErrorMessage(e));
      } finally {
        setSyncing(false);
      }
    },
    [search],
  );

  // Every reference shown, as a stable string so edits elsewhere in the policy
  // don't trigger new lookups.
  const referencesKey = JSON.stringify(
    Array.from(
      new Set(
        [...assignments.map((a) => a.link), ...legacyTargets.map((t) => t.link)]
          .map((r) => r.reference)
          .filter((r): r is string => !!r && r.includes("/")),
      ),
    ).sort((a, b) => a.localeCompare(b)),
  );

  useEffect(() => {
    const references: string[] = JSON.parse(referencesKey);
    if (references.length === 0) {
      setResolved({});
      return;
    }
    let cancelled = false;
    client
      .batch({}, R4, {
        resourceType: "Bundle",
        type: "batch" as code,
        entry: references.map((reference) => ({
          request: { method: "GET" as code, url: reference as uri },
        })),
      } as Bundle)
      .then((response) => {
        if (cancelled) return;
        setResolved(
          Object.fromEntries(
            references.map((reference, i) => [
              reference,
              response.entry?.[i]?.resource as Resource | undefined,
            ]),
          ),
        );
      })
      .catch((e) => console.error(e));
    return () => {
      cancelled = true;
    };
  }, [client, referencesKey]);

  const addAssignment = (link: Reference) => {
    if (!policyReference) return Promise.resolve();
    const promise = client
      .create({}, R4, {
        resourceType: "AccessPolicyV2Assignment",
        accessPolicy: { reference: policyReference },
        link,
      } as AccessPolicyV2Assignment)
      .then((created) =>
        refreshUntil((results) => results.some((a) => a.id === created.id)),
      );
    return Toaster.promise(promise, {
      loading: "Adding assignment",
      success: () => `Assigned to ${link.reference}`,
      error: (error) => getErrorMessage(error),
    });
  };

  const changeAssignment = (
    assignment: AccessPolicyV2Assignment,
    link: Reference,
  ) => {
    const promise = client
      .update({}, R4, "AccessPolicyV2Assignment", assignment.id as id, {
        ...assignment,
        link,
      })
      .then(() =>
        refreshUntil((results) =>
          results.some(
            (a) =>
              a.id === assignment.id && a.link.reference === link.reference,
          ),
        ),
      );
    Toaster.promise(promise, {
      loading: "Updating assignment",
      success: () => `Assignment now applies to ${link.reference}`,
      error: (error) => getErrorMessage(error),
    });
  };

  const removeAssignment = (assignment: AccessPolicyV2Assignment) => {
    const promise = client
      .delete_instance({}, R4, "AccessPolicyV2Assignment", assignment.id as id)
      .then(() =>
        refreshUntil((results) => results.every((a) => a.id !== assignment.id)),
      );
    Toaster.promise(promise, {
      loading: "Removing assignment",
      success: () => `Removed assignment to ${assignment.link.reference}`,
      error: (error) => getErrorMessage(error),
    });
  };

  // Moves a deprecated `target.link` to an assignment in one transaction: the
  // assignment is created and the link is removed from the saved policy
  // together, so the policy is never left with both or neither.
  const convertLegacyTarget = (index: number) => {
    const reference = legacyTargets[index]?.link.reference;
    if (!policyReference || !policy?.id || !reference) return;
    const policyId = policy.id;

    const withoutTarget = (p: AccessPolicyV2): AccessPolicyV2 => {
      const target = (p.target ?? []).filter(
        (t) => t.link.reference !== reference,
      );
      return { ...p, target: target.length > 0 ? target : undefined };
    };

    const promise = (async () => {
      // Start from the saved policy so unsaved edits in the editor are not
      // written as a side effect of the conversion.
      const saved = await client.read({}, R4, "AccessPolicyV2", policyId);
      if (!saved) throw new Error(`AccessPolicyV2/${policyId} not found`);

      const response = await client.transaction({}, R4, {
        resourceType: "Bundle",
        type: "transaction" as code,
        entry: [
          {
            request: {
              method: "POST" as code,
              url: "AccessPolicyV2Assignment" as uri,
            },
            resource: {
              resourceType: "AccessPolicyV2Assignment",
              accessPolicy: { reference: policyReference },
              link: { reference },
            },
          },
          {
            request: { method: "PUT" as code, url: policyReference as uri },
            resource: withoutTarget(saved),
          },
        ],
      } as Bundle);

      const created = response.entry
        ?.map((e) => e.resource)
        .find((r) => r?.resourceType === "AccessPolicyV2Assignment");
      return created?.id;
    })();

    Toaster.promise(promise, {
      loading: "Converting to assignment",
      success: () => `Converted ${reference} to an assignment`,
      error: (error) => getErrorMessage(error),
    })
      .then((createdId) => {
        // Mirror the saved change in the editor, keeping any unsaved edits.
        onChange((current) =>
          current?.resourceType === "AccessPolicyV2"
            ? withoutTarget(current)
            : current,
        );
        return refreshUntil((results) =>
          createdId
            ? results.some((a) => a.id === createdId)
            : results.some((a) => a.link.reference === reference),
        );
      })
      .catch(() => undefined);
  };

  if (!policyReference) {
    return (
      <div className="rounded-none border border-[var(--ci-line)] bg-[var(--ci-surface-raised)] px-3 py-2 font-mono text-xs text-[var(--ci-ink)]">
        Create the access policy first, then assign it here.
      </div>
    );
  }

  return (
    <div className="space-y-3">
      <div className="rounded-none border border-[var(--ci-line)] bg-[var(--ci-surface-raised)] px-3 py-2 font-mono text-xs text-[var(--ci-ink)]">
        This policy applies to the memberships, client applications and
        operations below. Each is an{" "}
        <span className="font-mono">{`AccessPolicyV2Assignment`}</span>; changes
        take effect the next time the user or client gets a token.
      </div>

      <AddAssignment onAdd={addAssignment} />

      {syncing && (
        <div className="flex items-center gap-2 text-xs font-mono text-[var(--ci-muted)]">
          <Loading className="w-4 h-4" />
          <span>Updating list…</span>
        </div>
      )}

      {loading ? (
        <div className="flex justify-center py-4">
          <Loading />
        </div>
      ) : (
        <div className="overflow-x-auto">
          <table className="w-full table-fixed border-collapse text-left font-mono text-xs text-[var(--ci-ink)]">
            <thead className="border-b border-[var(--ci-line)] bg-[var(--ci-surface-raised)] text-[11px] font-medium uppercase tracking-wider text-[var(--ci-muted)]">
              <tr>
                <th className="w-[20%] px-4 py-2">Type</th>
                <th className="w-[40%] px-4 py-2">Who</th>
                <th className="w-[20%] px-4 py-2">Change</th>
                <th className="w-[20%] px-4 py-2 text-right">Remove</th>
              </tr>
            </thead>
            <tbody>
              {assignments.length === 0 && legacyTargets.length === 0 && (
                <tr>
                  <td
                    colSpan={4}
                    className="px-4 py-6 text-center font-mono text-xs text-[var(--ci-muted)]"
                  >
                    Not assigned to anything yet.
                  </td>
                </tr>
              )}
              {assignments.map((assignment) => (
                <tr key={assignment.id} className="border-b border-[var(--ci-line)] hover:bg-[var(--ci-surface-raised)] transition-colors">
                  <td className="px-4 py-3 align-middle">
                    <AssignmentType reference={assignment.link} />
                  </td>
                  <td className="px-4 py-3 align-middle">
                    <AssignmentTarget
                      reference={assignment.link}
                      resolved={resolved}
                    />
                  </td>
                  <td className="px-4 py-3 align-middle">
                    <FHIRReferenceEditable
                      resourceTypesAllowed={ASSIGNABLE_TYPES}
                      fhirVersion={R4}
                      client={client}
                      value={assignment.link}
                      showValue={false}
                      searchLabel="Change"
                      searchAriaLabel={`Change the assignment target (currently ${assignment.link.reference})`}
                      onChange={(link) => {
                        if (
                          link?.reference &&
                          link.reference !== assignment.link.reference
                        ) {
                          changeAssignment(assignment, link);
                        }
                      }}
                    />
                  </td>
                  <td className="px-4 py-3 text-right align-middle">
                    <Button
                      buttonType="secondary"
                      buttonSize="small"
                      className="!text-red-600"
                      onClick={(e) => {
                        e.preventDefault();
                        removeAssignment(assignment);
                      }}
                    >
                      Remove
                    </Button>
                  </td>
                </tr>
              ))}
              {legacyTargets.map((target, index) => (
                <tr
                  key={`legacy-${target.link.reference}-${index}`}
                  className="border-b bg-yellow-50"
                >
                  <td className="px-4 py-2 align-middle">
                    <AssignmentType reference={target.link} />
                  </td>
                  <td className="px-4 py-2 align-middle">
                    <div className="flex min-w-0 items-center gap-2">
                      <AssignmentTarget
                        reference={target.link}
                        resolved={resolved}
                      />
                      <Tag color="yellow" className="shrink-0">
                        deprecated
                      </Tag>
                    </div>
                  </td>
                  <td className="px-4 py-2 font-mono text-xs text-[var(--ci-muted)]">
                    Set in <span className="font-mono">target.link</span> on the
                    policy.
                  </td>
                  <td className="px-4 py-2 text-right">
                    <Button
                      buttonType="secondary"
                      buttonSize="small"
                      onClick={(e) => {
                        e.preventDefault();
                        convertLegacyTarget(index);
                      }}
                    >
                      Convert to assignment
                    </Button>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
    </div>
  );
}
