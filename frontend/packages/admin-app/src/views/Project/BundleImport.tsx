import { ArrowUpTrayIcon } from "@heroicons/react/24/outline";
import { useAtomValue } from "jotai";
import React, { useState } from "react";
import { generatePath, useNavigate } from "react-router";

import { Button, Input, Toaster } from "@haste-health/components";
import { Bundle } from "@haste-health/fhir-types/r4/types";
import { R4 } from "@haste-health/fhir-types/versions";

import { getClient } from "../../db/client";
import { getErrorMessage, readAsBase64 } from "../../utilities";

export default function BatchImportView() {
  const navigate = useNavigate();
  const client = useAtomValue(getClient);
  const [bundle, setBundle] = useState<Bundle>();
  const [issues, setIssues] = useState<string[]>([]);
  const [fileName, setFileName] = useState<string | undefined>(undefined);

  return (
    <div className="flex w-full flex-col gap-6 text-[var(--ci-ink)]">
      <header className="rounded-none border border-[var(--ci-line)] bg-[var(--ci-surface)] p-5 shadow-sm">
        <h1 className="text-xl font-mono font-semibold text-[var(--ci-ink)]">Bundle Import</h1>
        <p className="mt-1 max-w-3xl text-xs text-[var(--ci-muted)]">
          Upload a FHIR Bundle and process it as either a batch or transaction.
          This is useful for bulk loading resources and testing import
          pipelines.
        </p>
      </header>

      <section className="w-full max-w-3xl rounded-none border border-[var(--ci-line)] bg-[var(--ci-surface)] p-5 shadow-sm">
        <div className="mb-4 flex items-start gap-3">
          <div className="rounded-none border border-[var(--ci-line)] bg-[var(--ci-surface-raised)] p-2 text-[var(--ci-signal)]">
            <ArrowUpTrayIcon className="h-5 w-5" />
          </div>
          <div>
            <h2 className="text-sm font-mono font-semibold uppercase tracking-wider text-[var(--ci-ink)]">
              Select Bundle File
            </h2>
            <p className="text-xs text-[var(--ci-muted)]">
              Accepted: JSON FHIR Bundle with{" "}
              <span className="font-mono text-[var(--ci-ink)]">type</span> set to{" "}
              <span className="font-mono text-[var(--ci-ink)]">batch</span> or{" "}
              <span className="font-mono text-[var(--ci-ink)]">transaction</span>.
            </p>
          </div>
        </div>

        <Input
          label="Bundle file"
          issues={issues}
          type="file"
          onChange={(e) => {
            const file = e.target?.files?.[0];

            setIssues([]);
            setBundle(undefined);
            setFileName(undefined);

            if (!file) return;
            setFileName(file.name);

            readAsBase64(file).then(
              (data) => {
                try {
                  const json = JSON.parse(atob(data));
                  if (
                    json.resourceType !== "Bundle" ||
                    (json.type !== "batch" && json.type !== "transaction")
                  ) {
                    throw new Error(
                      "File is not a batch or transaction Bundle",
                    );
                  }
                  setBundle(json);
                } catch (e) {
                  setIssues([`${e}`]);
                }
              },
              (e) => setIssues([`${e}`]),
            );
          }}
        />

        <div className="mt-3 rounded-none border border-[var(--ci-line)] bg-[var(--ci-surface-raised)] px-3 py-2 text-xs font-mono text-[var(--ci-ink)]">
          <span className="text-[var(--ci-muted)]">Selected:</span>{" "}
          {fileName ?? "No file selected"}
        </div>

        <div className="mt-4 flex justify-end">
          <Button
            disabled={bundle === undefined}
            buttonSize="medium"
            onClick={() => {
              if (bundle) {
                const batchPromise =
                  bundle.type === "transaction"
                    ? client.transaction({}, R4, bundle)
                    : client.batch({}, R4, bundle);
                Toaster.promise(batchPromise, {
                  loading: "Uploading Bundle",
                  success: () => "Bundle was uploaded",
                  error: (error) => {
                    return getErrorMessage(error);
                  },
                }).then(() => {
                  navigate(generatePath("/", {}));
                });
              }
            }}
          >
            Import Bundle
          </Button>
        </div>
      </section>
    </div>
  );
}
