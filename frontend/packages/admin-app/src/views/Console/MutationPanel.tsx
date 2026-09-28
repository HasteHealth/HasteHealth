import { ExclamationTriangleIcon } from "@heroicons/react/24/outline";
import { useAtomValue } from "jotai";
import React, { useEffect, useMemo, useState } from "react";
import { useNavigate } from "react-router";

import {
  Button,
  JSONTextEditor,
  Outcome,
  OutcomePanel,
  Tabs,
  toOutcome,
} from "@haste-health/components";
import { Resource, id } from "@haste-health/fhir-types/r4/types";
import { R4, ResourceType } from "@haste-health/fhir-types/versions";

import { getClient } from "../../db/client";
import { getStructures } from "../../db/structures";
import { templateFor } from "../../db/template";
import {
  Command,
  VERBS_WITH_BODY,
  describeRequest,
  isConditional,
  mutationWarning,
} from "../../query/model";
import { customViews, hasCustomViews } from "./registry";
import { GeneratedForm } from "./GeneratedForm";
import { useViewTabs } from "./useViewTabs";
import { typeTemplate } from "./views";

type Client = ReturnType<typeof import("../../db/client").createAdminAppClient>;

/** Sends `command`, returning whatever the server sent back. */
async function execute(
  client: Client,
  command: Command,
  body: string,
): Promise<unknown> {
  const { verb, target } = command;
  const parsed = VERBS_WITH_BODY.includes(verb)
    ? (JSON.parse(body) as Resource)
    : undefined;

  switch (verb) {
    case "POST":
      if (target.level !== "type") {
        throw new Error("Creating is only supported at the type level.");
      }
      return client.create({}, R4, {
        ...(parsed as Resource),
        resourceType: target.resourceType,
      } as Resource);

    case "PUT":
      // A search makes the update conditional; otherwise it addresses one id.
      if (target.level === "instance") {
        return client.update(
          {},
          R4,
          target.resourceType as ResourceType<R4>,
          target.id as id,
          parsed as never,
        );
      }
      if (target.level === "type") {
        return client.conditionalUpdate(
          {},
          R4,
          target.resourceType as ResourceType<R4>,
          target.search ?? "",
          parsed as never,
        );
      }
      throw new Error("Updating is not supported at the system level.");

    case "PATCH":
      if (target.level !== "instance") {
        throw new Error("Patching is only supported on one resource.");
      }
      return client.patch(
        {},
        R4,
        target.resourceType as ResourceType<R4>,
        target.id as id,
        JSON.parse(body),
      );

    case "DELETE":
      switch (target.level) {
        case "instance":
          return client.delete_instance(
            {},
            R4,
            target.resourceType as ResourceType<R4>,
            target.id as id,
          );
        case "type":
          return client.delete_type(
            {},
            R4,
            target.resourceType as ResourceType<R4>,
            target.search,
          );
        case "system":
          return client.delete_system({}, R4, target.search);
      }
    // eslint-disable-next-line no-fallthrough
    default:
      throw new Error(`${verb} is not supported here.`);
  }
}

export interface MutationPanelProps {
  command: Command;
  /** Called after a write lands, so the console can show the result. */
  onDone: (result: unknown) => void;
  /** Leaves the editor without sending anything. */
  onCancel: () => void;
}

/** What the send button reads: sending, confirming, or the request itself. */
function sendLabel({
  running,
  confirming,
  command,
  request,
}: {
  running: boolean;
  confirming: boolean;
  command: Command;
  request: { method: string; path: string };
}): string {
  if (running) return "Sending…";
  if (confirming) return `Yes, send ${command.verb}`;
  return `Send ${request.method} ${request.path}`;
}

/**
 * Runs a write.
 *
 * A mutation is deliberately not something the URL can trigger: it is staged
 * here, shown with what it will do, and sent only when confirmed. Anything
 * that acts on a search rather than one id asks twice, because the blast
 * radius is however many resources happen to match.
 */
export function MutationPanel({
  command,
  onDone,
  onCancel,
}: Readonly<MutationPanelProps>) {
  const client = useAtomValue(getClient);
  const navigate = useNavigate();
  const structures = useAtomValue(getStructures);
  const needsBody = VERBS_WITH_BODY.includes(command.verb);
  const [body, setBody] = useState("");

  // The starting body comes from the type's own StructureDefinition, so a
  // create begins with the elements the server actually requires rather than
  // with an empty object.
  useEffect(() => {
    if (!needsBody) {
      setBody("");
      return undefined;
    }
    if (command.verb === "PATCH") {
      setBody(
        JSON.stringify(
          [{ op: "replace", path: "/status", value: "" }],
          null,
          2,
        ),
      );
      return undefined;
    }
    const resourceType =
      command.target.level === "system"
        ? "Bundle"
        : command.target.resourceType;
    let cancelled = false;

    // Replacing a resource starts from what is there now; anything else
    // starts from the type's required elements.
    const starting =
      command.verb === "PUT" && command.target.level === "instance"
        ? client
            .read(
              {},
              R4,
              resourceType as ResourceType<R4>,
              command.target.id as id,
            )
            .catch(() => undefined)
        : Promise.resolve(undefined);

    starting.then(async (existing) => {
      if (cancelled) return;
      if (existing) {
        setBody(JSON.stringify(existing, null, 2));
        return;
      }
      // A type with its own editor starts from a template that editor can
      // show, rather than from the bare required elements.
      const specialized = typeTemplate(resourceType);
      if (specialized) {
        setBody(JSON.stringify(specialized, null, 2));
        return;
      }
      const structure = await structures?.get(resourceType);
      if (cancelled) return;
      setBody(JSON.stringify(templateFor(resourceType, structure), null, 2));
    });

    return () => {
      cancelled = true;
    };
  }, [structures, needsBody, command, client]);
  const [confirming, setConfirming] = useState(false);
  const [running, setRunning] = useState(false);
  const [outcome, setOutcome] = useState<Outcome>();

  // A type with its own editor gets it here too, so an operation's code or a
  // view's projection can be written before the resource exists. The editors
  // work on a resource, the request body is text, so it is parsed for them
  // and written back as text.
  const bodyResource = useMemo(() => {
    if (!needsBody || command.verb === "PATCH") return undefined;
    try {
      return JSON.parse(body) as Resource;
    } catch {
      // Mid-edit the JSON is often invalid; the JSON tab is where that gets
      // reported, so the specialized tabs simply have nothing to show.
      return undefined;
    }
  }, [body, needsBody, command.verb]);

  const {
    tabs: bodyTabs,
    selectedTab,
    onTab,
  } = useViewTabs((paneState, showPane) => [
    ...(bodyResource && hasCustomViews(bodyResource.resourceType)
      ? customViews({
          resource: bodyResource,
          resourceType: bodyResource.resourceType as string,
          // Nothing is saved yet, so there is nothing to diff against,
          // invoke, or look up logs for.
          saved: false,
          dirty: false,
          readOnly: false,
          onChange: (next) => setBody(JSON.stringify(next, null, 2)),
          paneState,
          onPaneStateChange: showPane,
        })
      : []),
    {
      id: "json",
      title: "JSON",
      content: (
        <JSONTextEditor value={body} onChange={setBody} hint="Request body" />
      ),
    },
    // A create has no resource to read yet, so the form is the only place
    // the body can be written without knowing the type's shape. It needs the
    // body to parse, which it does not while the JSON tab is mid-edit.
    ...(bodyResource?.resourceType
      ? [
          {
            id: "generated-form",
            title: "Form",
            content: (
              <GeneratedForm
                resource={bodyResource}
                resourceType={bodyResource.resourceType as string}
                // The form edits a resource, the body is text, so it is
                // written back as text. `bodyResource` is what parsed, so
                // applying to it avoids re-parsing text that may not.
                onChange={(update) =>
                  setBody(
                    JSON.stringify(
                      typeof update === "function"
                        ? update(bodyResource)
                        : update,
                      null,
                      2,
                    ),
                  )
                }
              />
            ),
          },
        ]
      : []),
  ]);

  const request = useMemo(() => describeRequest(command), [command]);
  const warning = mutationWarning(command);
  const conditional = isConditional(command);

  const run = () => {
    setRunning(true);
    setOutcome(undefined);
    execute(client, command, body)
      .then((result) => {
        setConfirming(false);
        const saved = result as Resource | undefined;
        // A create or update lands on the resource it produced; a delete has
        // nothing to show, so fall back to the caller.
        if (saved?.id && saved.resourceType) {
          navigate(`/r/${saved.resourceType}/${saved.id}`);
        } else {
          onDone(result);
        }
      })
      .catch((error) => setOutcome(toOutcome(error)))
      .finally(() => setRunning(false));
  };

  return (
    <div className="flex min-h-0 flex-1 flex-col gap-3">
      <div
        className={`rounded-lg border p-3 ${
          conditional
            ? "border-red-300 bg-red-50"
            : "border-amber-300 bg-amber-50"
        }`}
      >
        <div className="flex items-start gap-2">
          <ExclamationTriangleIcon
            className={`mt-0.5 h-5 w-5 shrink-0 ${
              conditional ? "text-red-600" : "text-amber-600"
            }`}
          />
          <div className="min-w-0 flex-1">
            <p
              className={`text-sm font-medium ${
                conditional ? "text-red-900" : "text-amber-900"
              }`}
            >
              {warning}
            </p>
            <p className="mt-0.5 font-mono text-xs text-slate-600">
              {request.method} {request.path}
            </p>
            {conditional && (
              <p className="mt-1 text-xs text-red-700">
                This acts on every resource the search matches, which may be
                more than you expect. Run the same path as a GET first to see
                what it would hit.
              </p>
            )}
          </div>
          {/* One button sends, and it names the request it will send. A
              conditional write swaps it for an explicit confirmation, since
              it acts on however many resources happen to match. */}
          <div className="flex shrink-0 gap-2">
            <Button
              buttonSize="small"
              buttonType="secondary"
              disabled={running}
              onClick={confirming ? () => setConfirming(false) : onCancel}
            >
              Cancel
            </Button>
            <Button
              buttonSize="small"
              disabled={running}
              onClick={() =>
                conditional && !confirming ? setConfirming(true) : run()
              }
            >
              <span className="block max-w-80 truncate">
                {sendLabel({ running, confirming, command, request })}
              </span>
            </Button>
          </div>
        </div>
      </div>

      {outcome && <OutcomePanel outcome={outcome} request={request} />}

      {needsBody && (
        <div className="flex min-h-0 flex-1 flex-col rounded-lg border border-slate-200 bg-white p-3">
          {bodyTabs.length > 1 ? (
            <Tabs selectedTab={selectedTab} onTab={onTab} tabs={bodyTabs} />
          ) : (
            <JSONTextEditor
              value={body}
              onChange={setBody}
              hint={
                command.verb === "PATCH"
                  ? "JSON Patch document: an array of operations."
                  : "Request body"
              }
            />
          )}
        </div>
      )}
    </div>
  );
}
