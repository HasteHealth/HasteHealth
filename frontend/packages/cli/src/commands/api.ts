import { confirm } from "@inquirer/prompts";
import { Command } from "commander";

import * as r4Sets from "@oxidized-health/fhir-types/r4/sets";
import {
  Bundle,
  Parameters,
  Resource,
  ResourceType,
  code,
  id,
} from "@oxidized-health/fhir-types/r4/types";
import * as r4bSets from "@oxidized-health/fhir-types/r4b/sets";
import { FHIR_VERSION, R4, R4B } from "@oxidized-health/fhir-types/versions";

import { createClient } from "../client.js";
import { CONFIG_LOCATION } from "../config.js";
import logger from "../logger.js";
import { conversion, dataCommand } from "../utilities.js";

function validateResourceType(
  fhirVersion: FHIR_VERSION,
  resourceType: string
): resourceType is ResourceType {
  switch (fhirVersion) {
    case R4: {
      return r4Sets.resourceTypes.has(resourceType);
    }
    case R4B: {
      return r4bSets.resourceTypes.has(resourceType);
    }
  }
}

function isBundle(value: unknown): value is Bundle {
  return (value as Record<string, unknown>)?.resourceType === "Bundle";
}

function isParameters(value: unknown): value is Parameters {
  return (value as Record<string, unknown>)?.resourceType === "Parameters";
}

export function apiCommands(command: Command) {
  command
    .command("capabilities")
    .argument("<fhirVersion>", "FHIR Version")
    .action(async (userFHIRVersion) => {
      const client = await createClient(CONFIG_LOCATION);
      const FHIRVersion = conversion.asFHIRType(userFHIRVersion);
      const capabilities = await client.capabilities({}, FHIRVersion);
      logger.info(capabilities);
    });

  dataCommand.command(
    command
      .command("create")
      .argument("<fhirVersion>", "FHIR Version")
      .action(async (userFHIRVersion, options) => {
        const resourceToSave = dataCommand.readData(options);
        const client = await createClient(CONFIG_LOCATION);
        const FHIRVersion = conversion.asFHIRType(userFHIRVersion);

        if (!resourceToSave) {
          throw new Error("No resource provided to save");
        }
        const resource = await client.create(
          {},
          FHIRVersion,
          resourceToSave as Resource
        );
        logger.info(resource);
      })
  );

  dataCommand.command(
    command
      .command("update")
      .argument("<fhirVersion>", "FHIR Version")
      .argument("<resourceType>", "Resource Type")
      .argument("<id>", "Resource ID")
      .action(async (userFHIRVersion, resourceType, id, options) => {
        const FHIRVersion = conversion.asFHIRType(userFHIRVersion);
        if (!validateResourceType(FHIRVersion, resourceType))
          throw new Error("Invalid resource type");

        const resourceToSave = dataCommand.readData(options);
        const client = await createClient(CONFIG_LOCATION);
        if (!resourceToSave) {
          throw new Error("No resource provided to save");
        }

        const resource = await client.update(
          {},
          FHIRVersion,
          resourceType,
          id,
          resourceToSave as Resource
        );
        logger.info(resource);
      })
  );

  dataCommand.command(
    command
      .command("batch")
      .argument("<fhirVersion>", "FHIR Version")
      .action(async (userFHIRVersion, options) => {
        const batchBundle = dataCommand.readData(options);
        const client = await createClient(CONFIG_LOCATION);
        const FHIRVersion = conversion.asFHIRType(userFHIRVersion);
        if (!batchBundle) {
          throw new Error("No resource provided to save");
        }
        if (!isBundle(batchBundle))
          throw new Error("invalid resource type must be a 'Bundle'.");
        const resource = await client.batch({}, FHIRVersion, batchBundle);
        logger.info(resource);
      })
  );

  dataCommand.command(
    command
      .command("transaction")
      .argument("<fhirVersion>", "FHIR Version")
      .action(async (userFHIRVersion, options) => {
        const transaction = dataCommand.readData(options);
        const client = await createClient(CONFIG_LOCATION);
        const FHIRVersion = conversion.asFHIRType(userFHIRVersion);
        if (!transaction) {
          throw new Error("No resource provided to save");
        }
        if (!isBundle(transaction))
          throw new Error("invalid resource type must be a 'Bundle'.");
        const resource = await client.transaction({}, FHIRVersion, transaction);
        logger.info(resource);
      })
  );

  dataCommand.command(
    command
      .command("patch")
      .argument("<fhirVersion>", "FHIR Version")
      .argument("<resourceType>", "Resource Type")
      .argument("<id>", "Resource ID")
      .action(async (userFHIRVersion, resourceType, id, options) => {
        const patches = dataCommand.readData(options);
        const client = await createClient(CONFIG_LOCATION);
        const FHIRVersion = conversion.asFHIRType(userFHIRVersion);

        if (!validateResourceType(FHIRVersion, resourceType))
          throw new Error("Invalid resource type");

        if (!patches) {
          throw new Error("No patches provided.");
        }

        const resource = await client.patch(
          {},
          FHIRVersion,
          resourceType,
          id,
          patches
        );
        logger.info(resource);
      })
  );

  command
    .command("delete")
    .argument("<fhirVersion>", "FHIR Version")
    .argument("<resourceType>", "Resource Type")
    .argument("<id>", "Resource ID")
    .action(async (userFHIRVersion, resourceType, id) => {
      const client = await createClient(CONFIG_LOCATION);
      const FHIRVersion = conversion.asFHIRType(userFHIRVersion);

      if (!validateResourceType(FHIRVersion, resourceType))
        throw new Error("Invalid resource type");

      const shouldDelete = await confirm({
        message: "Are you sure you want to delete this resource?",
      });

      if (shouldDelete) {
        await client.delete_instance({}, FHIRVersion, resourceType, id);
        logger.info("Resource has been deleted.");
      }
    });

  command
    .command("search_system")
    .argument("<fhirVersion>", "FHIR Version")
    .argument("[query]", "query")
    .action(async (userFHIRVersion, query: string | undefined) => {
      const client = await createClient(CONFIG_LOCATION);
      const FHIRVersion = conversion.asFHIRType(userFHIRVersion);

      const searchResponse = await client.search_system(
        {},
        FHIRVersion,
        query ?? ""
      );
      logger.info(searchResponse);
    });

  command
    .command("search_type")
    .argument("<fhirVersion>", "FHIR Version")
    .argument("<resourceType>", "Resource Type")
    .argument("[query]", "query")
    .action(
      async (
        userFHIRVersion,
        resourceType: string,
        query: string | undefined
      ) => {
        const client = await createClient(CONFIG_LOCATION);
        const FHIRVersion = conversion.asFHIRType(userFHIRVersion);

        if (!validateResourceType(FHIRVersion, resourceType))
          throw new Error("Invalid resource type");
        const searchResponse = await client.search_type(
          {},
          FHIRVersion,
          resourceType,
          query ?? ""
        );
        logger.info(searchResponse);
      }
    );
  command
    .command("read")
    .argument("<fhirVersion>", "FHIR Version")
    .argument("<resourceType>", "Resource Type")
    .argument("<id>", "Resource ID")
    .action(async (userFHIRVersion, resourceType, id) => {
      const client = await createClient(CONFIG_LOCATION);
      const FHIRVersion = conversion.asFHIRType(userFHIRVersion);

      if (!validateResourceType(FHIRVersion, resourceType))
        throw new Error("Invalid resource type");

      const resource = await client.read({}, FHIRVersion, resourceType, id);
      logger.info(resource);
    });
  command
    .command("vread")
    .argument("<fhirVersion>", "FHIR Version")
    .argument("<resourceType>", "Resource Type")
    .argument("<id>", "Resource ID")
    .argument("<versionId>", "Resource version ID")
    .action(async (userFHIRVersion, resourceType, id, versionId) => {
      const client = await createClient(CONFIG_LOCATION);
      const FHIRVersion = conversion.asFHIRType(userFHIRVersion);

      if (!validateResourceType(FHIRVersion, resourceType))
        throw new Error("Invalid resource type");

      const resourceVersion = await client.vread(
        {},
        FHIRVersion,
        resourceType,
        id,
        versionId
      );
      logger.info(resourceVersion);
    });

  command
    .command("history_system")
    .argument("<fhirVersion>", "FHIR Version")
    .argument("[query]", "query")
    .action(async (userFHIRVersion, query?: string) => {
      const client = await createClient(CONFIG_LOCATION);
      const FHIRVersion = conversion.asFHIRType(userFHIRVersion);

      const history = await client.history_system({}, FHIRVersion, query);
      logger.info(history);
    });

  command
    .command("history_type")
    .argument("<fhirVersion>", "FHIR Version")
    .argument("<resourceType>", "Resource Type")
    .argument("[query]", "query")
    .action(async (userFHIRVersion, resourceType, query?: string) => {
      const client = await createClient(CONFIG_LOCATION);
      const FHIRVersion = conversion.asFHIRType(userFHIRVersion);

      if (!validateResourceType(FHIRVersion, resourceType))
        throw new Error("Invalid resource type");
      const history = await client.history_type(
        {},
        FHIRVersion,
        resourceType,
        query
      );
      logger.info(history);
    });

  command
    .command("history_instance")
    .argument("<fhirVersion>", "FHIR Version")
    .argument("<resourceType>", "Resource Type")
    .argument("<id>", "Resource ID")
    .argument("[query]", "query")
    .action(async (userFHIRVersion, resourceType, id, query?: string) => {
      const client = await createClient(CONFIG_LOCATION);
      const FHIRVersion = conversion.asFHIRType(userFHIRVersion);

      if (!validateResourceType(FHIRVersion, resourceType))
        throw new Error("Invalid resource type");
      const history = await client.history_instance(
        {},
        FHIRVersion,
        resourceType,
        id,
        query
      );
      logger.info(history);
    });

  dataCommand.command(
    command
      .command("invoke_system")
      .argument("<fhirVersion>", "FHIR Version")
      .argument("<code>", "code")
      .action(async (userFHIRVersion, code: string, options) => {
        const client = await createClient(CONFIG_LOCATION);
        const FHIRVersion = conversion.asFHIRType(userFHIRVersion);
        const parameters = dataCommand.readData(options);
        if (!parameters) {
          throw new Error("No Parameters provided");
        }
        if (!isParameters(parameters)) throw new Error("invalid Parameters.");

        const output = await client.invoke_system(
          code as code,
          {},
          FHIRVersion,
          parameters
        );
        logger.info(output);
      })
  );

  dataCommand.command(
    command
      .command("invoke_type")
      .argument("<fhirVersion>", "FHIR Version")
      .argument("<resourceType>", "Resource Type")
      .argument("<code>", "code")
      .action(
        async (
          userFHIRVersion,
          resourceType: string,
          code: string,
          options
        ) => {
          const client = await createClient(CONFIG_LOCATION);
          const FHIRVersion = conversion.asFHIRType(userFHIRVersion);
          const parameters = dataCommand.readData(options);
          if (!parameters) {
            throw new Error("No Parameters provided");
          }
          if (!isParameters(parameters)) throw new Error("invalid Parameters.");
          if (!validateResourceType(FHIRVersion, resourceType))
            throw new Error("Invalid resource type");

          const output = await client.invoke_type(
            code as code,
            {},
            FHIRVersion,
            resourceType,
            parameters
          );
          logger.info(output);
        }
      )
  );
  dataCommand.command(
    command
      .command("invoke_instance")
      .argument("<fhirVersion>", "FHIR Version")
      .argument("<resourceType>", "Resource Type")
      .argument("<id>", "Resource ID")
      .argument("<code>", "code")
      .action(
        async (
          userFHIRVersion,
          resourceType: string,
          id: string,
          code: string,
          options
        ) => {
          const client = await createClient(CONFIG_LOCATION);
          const FHIRVersion = conversion.asFHIRType(userFHIRVersion);
          const parameters = dataCommand.readData(options);
          if (!parameters) {
            throw new Error("No Parameters provided");
          }
          if (!isParameters(parameters)) throw new Error("invalid Parameters.");
          if (!validateResourceType(FHIRVersion, resourceType))
            throw new Error("Invalid resource type");

          const output = await client.invoke_instance(
            code as code,
            {},
            FHIRVersion,
            resourceType,
            id as id,
            parameters
          );
          logger.info(output);
        }
      )
  );
}
