import { Command } from "commander";

import { code } from "@haste-health/fhir-types/lib/generated/r4/types";
import {
  AllResourceTypes,
  R4,
  R4B,
  Resource,
} from "@haste-health/fhir-types/versions";

import { createClient } from "../client.js";
import { CONFIG_LOCATION } from "../config.js";
import logger from "../logger.js";
import { conversion, dataCommand } from "../utilities.js";

export function validate(command: Command) {
  dataCommand.command(
    command
      .command("validate")
      .argument("<fhirVersion>", "FHIR Version")
      .action(async (userFHIRVersion, options) => {
        const resource = dataCommand.readData(options) as Resource<
          R4,
          AllResourceTypes
        >;
        const FHIRVersion = conversion.asFHIRType(userFHIRVersion);
        const client = await createClient(CONFIG_LOCATION);

        switch (FHIRVersion) {
          case R4: {
            const result = await client.invoke_type(
              "validate" as code,
              {},
              FHIRVersion,
              resource.resourceType,
              {
                resourceType: "Parameters",
                parameter: [{ name: "resource", resource }],
              }
            );

            logger.warn(result);
            return;
          }
          case R4B:
          default: {
            throw new Error("Unsupported FHIR Version");
          }
        }
      })
  );
}
