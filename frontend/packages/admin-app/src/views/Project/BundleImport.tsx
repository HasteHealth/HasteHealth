import { ArrowUpTrayIcon } from "@heroicons/react/24/outline";
import { useAtomValue } from "jotai";
import React, { useState } from "react";
import { generatePath, useNavigate } from "react-router-dom";

import { Button, Input, Toaster } from "@haste-health/components";
import { Bundle } from "@haste-health/fhir-types/r4/types";
import { R4 } from "@haste-health/fhir-types/versions";

import { getClient } from "../../db/client";
import { getErrorMessage } from "../../utilities";

const getData = (file: File): Promise<string> => {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.readAsDataURL(file);

    reader.onload = () => {
      const data = reader.result;
      if (typeof data === "string") {
        resolve(data.split(",")[1]);
      } else {
        reject("FileReader result was not a string");
      }
    };
    reader.onerror = (error) => reject(error);
  });
};

export default function BatchImportView() {
  const navigate = useNavigate();
  const client = useAtomValue(getClient);
  const [bundle, setBundle] = useState<Bundle>();
  const [issues, setIssues] = useState<string[]>([]);
  const [fileName, setFileName] = useState<string | undefined>(undefined);

  return (
    <div className="flex w-full flex-col gap-6 text-slate-700">
      <header className="rounded-lg border border-slate-200 bg-white p-5 shadow-sm">
        <h1 className="text-2xl font-semibold text-slate-900">Bundle Import</h1>
        <p className="mt-1 max-w-3xl text-sm text-slate-500">
          Upload a FHIR Bundle and process it as either a batch or transaction.
          This is useful for bulk loading resources and testing import
          pipelines.
        </p>
      </header>

      <section className="w-full max-w-3xl rounded-lg border border-slate-200 bg-white p-5 shadow-sm">
        <div className="mb-4 flex items-start gap-3">
          <div className="rounded-md bg-brand-50 p-2 text-brand-700">
            <ArrowUpTrayIcon className="h-5 w-5" />
          </div>
          <div>
            <h2 className="text-base font-semibold text-slate-900">
              Select Bundle File
            </h2>
            <p className="text-sm text-slate-500">
              Accepted: JSON FHIR Bundle with{" "}
              <span className="font-medium">type</span> set to{" "}
              <span className="font-medium">batch</span> or{" "}
              <span className="font-medium">transaction</span>.
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

            getData(file).then((data) => {
              try {
                const json = JSON.parse(atob(data));
                if (
                  json.resourceType !== "Bundle" ||
                  (json.type !== "batch" && json.type !== "transaction")
                ) {
                  throw new Error("File is not a batch or transaction Bundle");
                }
                setBundle(json);
              } catch (e) {
                setIssues([`${e}`]);
              }
            });
          }}
        />

        <div className="mt-3 rounded-md border border-slate-200 bg-slate-50 px-3 py-2 text-sm text-slate-600">
          <span className="font-medium text-slate-700">Selected:</span>{" "}
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
