/* eslint-disable @typescript-eslint/no-explicit-any */
import { Command } from "commander";
import fs from "node:fs";
import path from "node:path";

import { code } from "@oxidized-health/fhir-types/lib/generated/r4/types";
import { FHIR_VERSION, Resource } from "@oxidized-health/fhir-types/versions";
import * as ts from "@oxidized-health/testscript-runner";

import { createClient } from "../client.js";
import { CONFIG_LOCATION } from "../config.js";
import logger from "../logger.js";

function getAllFiles(location: string): string[] {
  if (fs.lstatSync(location).isFile()) {
    return [location];
  }
  const files: string[] = [];
  const filesInDirectory = fs.readdirSync(location);
  for (const file of filesInDirectory) {
    const absolute = path.join(location, file);
    if (fs.statSync(absolute).isDirectory()) {
      files.push(...getAllFiles(absolute));
    } else {
      files.push(absolute);
    }
  }
  return files;
}

async function executeTestScript<Version extends FHIR_VERSION>(
  fhirVersion: Version,
  testScript: Resource<Version, "TestScript">,
  timeout: number = 0
): Promise<Resource<Version, "TestReport">> {
  const client = await createClient(CONFIG_LOCATION);

  const report = await ts.run(logger, client, fhirVersion, testScript, timeout);

  return report;
}

export function testscriptCommands(command: Command) {
  command
    .command("run")
    .description("Run testscript files against server.")
    .requiredOption("-i, --input <input>")
    .option("-o --output <output>", "Where to output files")
    .option(
      "-e, --extension   <extension>",
      "Extensions to search for tests",
      ".testscript.json"
    )
    .option(
      "-t, --timeout   <timeout>",
      "wait time between operations",
      ".testscript.json"
    )
    .option("--fhir-version <fhirVersion>", "FHIR Version to use", "4.0")

    .action(async (options) => {
      let exitCode = 0;
      const files = getAllFiles(options.input)
        .flat()
        .filter((f) => f.endsWith(options.extension))
        .sort((a, b) => a.localeCompare(b));

      const output: Resource<FHIR_VERSION, "Bundle"> = {
        resourceType: "Bundle",
        type: "collection" as code,
        entry: [],
      };

      for (const file of files) {
        const fileContents = fs.readFileSync(file);
        const json = JSON.parse(fileContents.toString("utf8"));
        if (json.resourceType === "Bundle") {
          const bundle = json as Resource<FHIR_VERSION, "Bundle">;
          const testscripts =
            bundle.entry
              ?.map((e) => e.resource)
              .filter(
                (r): r is Resource<FHIR_VERSION, "TestScript"> =>
                  r?.resourceType === "TestScript"
              ) ?? [];

          for (const testScript of testscripts) {
            output.entry?.push({ resource: testScript as any });
            const report = await executeTestScript(
              options.fhirVersion,
              testScript,
              parseInt(options.timeout ?? "0")
            );
            output.entry?.push({ resource: report as any });
            if (report.result === "fail") {
              exitCode = 1;
            }
          }
        } else if (json.resourceType === "TestScript") {
          const testScript = json as Resource<FHIR_VERSION, "TestScript">;
          output.entry?.push({ resource: testScript as any });
          const report = await executeTestScript(
            options.fhirVersion,
            testScript,
            parseInt(options.timeout ?? "0")
          );
          output.entry?.push({ resource: report as any });
          if (report.result === "fail") {
            exitCode = 1;
          }
        }
      }

      if (options.output) {
        fs.writeFileSync(options.output, JSON.stringify(output, null, 2));
      }

      process.exit(exitCode);
    });
}
