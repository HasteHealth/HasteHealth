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

  return (
    <div className="w-full flex flex-col items-center text-slate-700">
      <div className="mt-[10%]">
        <Input
          label="Select Bundle to import (must be either a batch or transaction bundle)"
          issues={issues}
          type="file"
          onChange={(e) => {
            const file = e.target?.files?.[0];

            if (!file) return;
            getData(file).then((data) => {
              try {
                const json = JSON.parse(atob(data));
                if (
                  json.resourceType !== "Bundle" &&
                  (json.type !== "batch" || json.type !== "transaction")
                )
                  throw new Error("Not a transaction or batch bundle");
                setBundle(json);
              } catch (e) {
                setIssues([`${e}`]);
              }
            });
          }}
        />

        <div className="flex justify-end">
          <Button
            disabled={bundle === undefined}
            className="mt-2"
            buttonSize="medium"
            onClick={() => {
              if (bundle) {
                const batchPromise =
                  bundle.type === "transaction"
                    ? client.transaction({}, R4, bundle)
                    : client.batch({}, R4, bundle);
                Toaster.promise(batchPromise, {
                  loading: "Uploading Bundle",
                  success: () => `Bundle was uploaded`,
                  error: (error) => {
                    return getErrorMessage(error);
                  },
                }).then(() => {
                  navigate(generatePath("/", {}));
                });
              }
            }}
          >
            Import
          </Button>
        </div>
      </div>
    </div>
  );
}
