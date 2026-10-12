/**
 * Starts a SMART EHR launch of an app for the patient (or encounter) in view.
 *
 * Haste Health plays the EHR: `ClientApplication/{id}/$launch` saves the
 * launch and returns the app's `launchUri` with `iss` and `launch` added,
 * which is opened in a new tab. The app then authorizes as the signed-in
 * user, in the same browser session.
 */
import { RocketLaunchIcon } from "@heroicons/react/24/outline";
import classNames from "classnames";
import { useAtomValue } from "jotai";
import React, { useState } from "react";

import {
  Button,
  Loading,
  Modal,
  Outcome,
  OutcomePanel,
  toOutcome,
} from "@haste-health/components";
import {
  ClientApplication,
  Encounter,
  Resource,
  id,
} from "@haste-health/fhir-types/r4/types";
import { R4 } from "@haste-health/fhir-types/versions";
import { HasteHealthSmartLaunch } from "@haste-health/generated-ops/lib/r4/ops";

import { getClient } from "../../db/client";
import { useRequest } from "../../hooks/useRequest";

/** The resource types a launch can be started from. */
export function canLaunchFrom(resourceType: string): boolean {
  return resourceType === "Patient" || resourceType === "Encounter";
}

/** What a launch is for: the patient, and the encounter when started from one. */
interface LaunchSubject {
  patient: string;
  encounter?: string;
}

/**
 * The patient (and encounter) `resource` stands for, or why it can't be
 * launched for: an Encounter needs a Patient as its subject.
 */
function launchSubject(resource: Resource): LaunchSubject | string {
  if (resource.resourceType === "Patient") {
    return { patient: `Patient/${resource.id}` };
  }
  const subject = (resource as Encounter).subject?.reference;
  if (!subject?.startsWith("Patient/")) {
    return "This encounter's subject is not a Patient, so there is no patient to launch for.";
  }
  return { patient: subject, encounter: `Encounter/${resource.id}` };
}

function LaunchDialog({
  resource,
  close,
}: Readonly<{ resource: Resource; close: () => void }>) {
  const client = useAtomValue(getClient);
  const subject = launchSubject(resource);
  const [selected, setSelected] = useState<string>();
  const [launching, setLaunching] = useState(false);
  const [outcome, setOutcome] = useState<Outcome>();

  // Only apps with somewhere to open them can be launched.
  const apps = useRequest<ClientApplication[]>(
    () =>
      client
        .search_type({}, R4, "ClientApplication", [
          { name: "_count", value: ["200"] },
          { name: "_sort", value: ["name"] },
        ])
        .then((response) =>
          (response.resources as ClientApplication[]).filter(
            (app) => app.launchUri,
          ),
        ),
    [client],
  );

  const launch = async (app: ClientApplication) => {
    if (typeof subject === "string") return;
    setLaunching(true);
    setOutcome(undefined);
    // Opened now, while the click still counts as the user's: a window opened
    // after the request returns would be blocked as a popup.
    const tab = window.open("", "_blank");
    try {
      const output = await client.invoke_instance(
        HasteHealthSmartLaunch.Op,
        {},
        R4,
        "ClientApplication",
        app.id as id,
        {
          patient: { reference: subject.patient },
          ...(subject.encounter
            ? { encounter: { reference: subject.encounter } }
            : {}),
        },
      );
      if (tab) {
        // The app must not be able to reach back into the admin app.
        tab.opener = null;
        tab.location.href = output.url;
      } else {
        window.open(output.url, "_blank", "noopener,noreferrer");
      }
      close();
    } catch (error) {
      tab?.close();
      setOutcome(toOutcome(error));
    } finally {
      setLaunching(false);
    }
  };

  if (typeof subject === "string") {
    return <p className="text-sm text-slate-600">{subject}</p>;
  }

  const chosen = apps.data?.find((app) => app.id === selected);

  return (
    <div className="space-y-3">
      <p className="text-sm text-slate-600">
        Opens the app in a new tab for{" "}
        <span className="font-mono">{subject.patient}</span>
        {subject.encounter && (
          <>
            {" "}
            in <span className="font-mono">{subject.encounter}</span>
          </>
        )}
        , signed in as you. The launch expires after five minutes.
      </p>

      {apps.loading ? (
        <div className="flex justify-center py-6">
          <Loading />
        </div>
      ) : apps.outcome ? (
        <OutcomePanel outcome={apps.outcome} />
      ) : (apps.data?.length ?? 0) === 0 ? (
        <p className="rounded-md border border-amber-200 bg-amber-50 px-3 py-2 text-sm text-amber-800">
          No client application has a launch URL. Set <code>launchUri</code> on
          a ClientApplication, and include <code>launch</code> in its scope, to
          launch it from here.
        </p>
      ) : (
        <ul className="max-h-72 divide-y divide-slate-200 overflow-auto rounded-md border border-slate-200">
          {apps.data?.map((app) => (
            <li key={app.id}>
              <button
                type="button"
                aria-pressed={selected === app.id}
                className={classNames(
                  "flex w-full flex-col items-start px-3 py-2 text-left",
                  {
                    "bg-brand-50": selected === app.id,
                    "hover:bg-slate-50": selected !== app.id,
                  },
                )}
                onClick={() => setSelected(app.id)}
              >
                <span className="text-sm font-medium text-slate-900">
                  {app.name}
                </span>
                <span className="font-mono text-xs text-slate-500">
                  {app.launchUri}
                </span>
              </button>
            </li>
          ))}
        </ul>
      )}

      {outcome && (
        <OutcomePanel
          outcome={outcome}
          request={{
            method: "POST",
            path: `/ClientApplication/${selected}/$launch`,
          }}
        />
      )}

      <div className="flex justify-end gap-2">
        <Button buttonType="secondary" buttonSize="small" onClick={close}>
          Cancel
        </Button>
        <Button
          buttonSize="small"
          disabled={!chosen || launching}
          onClick={() => chosen && launch(chosen)}
        >
          {launching ? "Launching…" : "Launch"}
        </Button>
      </div>
    </div>
  );
}

/** The header button that opens the launch dialog. */
export function LaunchAppButton({
  resource,
}: Readonly<{ resource: Resource }>) {
  return (
    <Modal
      modalTitle="Launch a SMART app"
      ModalContent={(setOpen) => (
        <LaunchDialog resource={resource} close={() => setOpen(false)} />
      )}
    >
      {(setOpen) => (
        <Button
          buttonSize="small"
          buttonType="secondary"
          onClick={() => setOpen(true)}
        >
          <span className="flex items-center">
            <RocketLaunchIcon className="mr-1 h-4 w-4" />
            Launch app
          </span>
        </Button>
      )}
    </Modal>
  );
}
