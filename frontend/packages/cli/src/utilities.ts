import { Command } from "commander";
import fs from "node:fs";
import path from "node:path";

import { FHIR_VERSION, R4, R4B } from "@oxidized-health/fhir-types/versions";

type MutationOptions = { file?: string; data?: string };

export namespace conversion {
  export function asFHIRType(fhirType: string): FHIR_VERSION {
    switch (fhirType) {
      case "r4":
      case R4: {
        return R4;
      }
      case "r4b":
      case R4B: {
        return R4B;
      }
      default: {
        throw new Error(
          "invalid FHIR version must be 'r4' or 'r4b' or '4.3' or '4.0'"
        );
      }
    }
  }
}

export function getAllFiles(directory: string): string[] {
  const files: string[] = [];
  const filesInDirectory = fs.readdirSync(directory);
  for (const file of filesInDirectory) {
    const absolute = path.join(directory, file);
    if (fs.statSync(absolute).isDirectory()) {
      files.push(...getAllFiles(absolute));
    } else {
      files.push(absolute);
    }
  }
  return files;
}

export namespace dataCommand {
  export function readData(option: MutationOptions): unknown {
    if (option.file) {
      return JSON.parse(fs.readFileSync(option.file, "utf-8"));
    }

    if (option.data) {
      return JSON.parse(option.data);
    }
    throw new Error("No resource provided");
  }

  export function command(command: Command) {
    command
      .option("-f, --file <file>", "File for resource")
      .option("-d, --data <data>", "Data for resource");
    return command;
  }
}
