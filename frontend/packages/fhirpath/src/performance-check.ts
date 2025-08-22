import { fileURLToPath } from "url";

import { loadArtifacts } from "@oxidized-health/artifacts";
import { R4 } from "@oxidized-health/fhir-types/versions";

import parseV2 from "./parserv2/index.js";

const searchParameters = loadArtifacts({
  fhirVersion: R4,
  resourceType: "SearchParameter",
  loadDevelopmentPackages: true,
  currentDirectory: fileURLToPath(import.meta.url),
}).filter((s) => s.expression);

console.time("v2");
for (let i = 0; i < 1000; i++) {
  for (const parameter of searchParameters) {
    parseV2(parameter.expression as string);
  }
}
console.timeEnd("v2");
