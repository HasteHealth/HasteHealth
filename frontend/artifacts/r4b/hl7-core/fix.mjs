import fs from "node:fs";
import { fileURLToPath } from "node:url";
import path from "node:path";

const PATH = path.join(
  fileURLToPath(import.meta.url),
  "../definitions/valuesets.json",
);

const content = JSON.parse(fs.readFileSync(PATH, { encoding: "utf8" }));

function shouldAddPublisher(e) {
  return (
    ["ValueSet", "CodeSystem"].includes(e.resource.resourceType) &&
    e.resource.publisher === undefined
  );
}

function addPublisher(e) {
  return shouldAddPublisher(e)
    ? { ...e, resource: { ...e.resource, publisher: "HL7 FHIR" } }
    : e;
}

function shouldAddDescription(e) {
  return (
    ["ValueSet", "CodeSystem"].includes(e.resource.resourceType) &&
    e.resource.description === undefined
  );
}

function addDescription(e) {
  return shouldAddDescription(e)
    ? {
        ...e,
        resource: { ...e.resource, description: "placeholder description" },
      }
    : e;
}

content.entry = content.entry.map(addPublisher).map(addDescription);

fs.writeFileSync(PATH, JSON.stringify(content, null, 2));
