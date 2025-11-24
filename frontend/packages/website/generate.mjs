import fs from "node:fs";
import { fileURLToPath } from "url";

import { loadArtifacts } from "@haste-health/artifacts";
import { R4 } from "@haste-health/fhir-types/versions";

const r4Artifacts = ["StructureDefinition", "SearchParameter"]
  .map((resourceType) =>
    loadArtifacts({
      loadDevelopmentPackages: true,
      resourceType: resourceType,
      silence: false,
      fhirVersion: R4,
      currentDirectory: fileURLToPath(import.meta.url),
    })
  )
  .flat();

function escapeCharacters(v) {
  return v
    ?.replaceAll("|", "/")
    .replace(/(\r\n|\n|\r)/gm, "")
    .replaceAll("{", "\\{")
    .replaceAll("}", "\\}")
    .replaceAll("`", "\\`")
    .replaceAll(">", "\\>")
    .replaceAll("<", "\\<");
}

function metaProperties(sd) {
  return `
|Property|Value|
|---|---|
|Publisher|${sd.publisher ?? ""}|
|Name|${sd.name ?? ""}|
|URL|${sd.url ?? ""}|
|Status|${sd.status ?? ""}|
|Description|${sd.description ?? ""}|
|Abstract|${sd.abstract ?? ""}|`;
}

async function processStructureDefinition(artifacts, structureDefinition) {
  const parameters = artifacts
    .filter((r) => r.resourceType === "SearchParameter")
    .filter(
      (r) =>
        r.base.includes(structureDefinition.name) ||
        r.base.includes("Resource") ||
        r.base.includes("DomainResource")
    );

  let doc = `---
id: ${structureDefinition.id}
title: ${structureDefinition.name}
tags:
  - fhir
  - Fast Healthcare Interoperability Resources
  - hl7
  - healthcare it
  - interoperability
---

import TabItem from "@theme/TabItem";
import Tabs from "@theme/Tabs";

# ${structureDefinition.name}\n
${structureDefinition.snapshot?.element[0]?.definition ?? ""}

<head>
  <meta name="keywords" content="fhir, hl7, interoperability, healthcare" />
  <script type="application/ld+json">
    {JSON.stringify({
      '@context': 'https://schema.org/',
      '@type': 'Organization',
      name: 'IGUHealth',
      url: 'https://iguhealth.app',
      logo: 'https://iguhealth.app/img/logo.svg',
    })}
  </script>
</head>

${metaProperties(structureDefinition)}\n
  `;
  doc = `${doc} ## Structure\n
   | Path | Cardinality | Type | Description
  | ---- | ----------- | ---- | -------  \n`;
  for (const element of structureDefinition.snapshot?.element || []) {
    const path = element.path;
    const min = element.min;
    const max = element.max;
    const type = element.type?.[0]?.code;
    const description = escapeCharacters(element.definition);
    doc = `${doc} | ${path} | ${min}..${max} | ${
      type ? type : structureDefinition.name
    } | ${description} \n`;
  }

  doc = `${doc}\n`;

  doc = `${doc} ## Search Parameters\n
   | Name | Type | Description  | Expression 
    | ---- | ---- | ------- | ------  \n`;
  for (const parameter of parameters) {
    const name = parameter.name;
    const type = parameter.type;

    const description = escapeCharacters(parameter.description || "");

    const expression = escapeCharacters(parameter.expression || "");

    doc = `${doc} | ${name} | ${type} | ${description} | ${expression}  \n`;
  }
  doc = `${doc}\n\n`;

  doc = `${doc}`;

  return doc;
}

async function generateFHIRDocumentation() {
  const r4StructureDefinitions = r4Artifacts
    .filter((r) => r.resourceType === "StructureDefinition")
    .filter((sd) => sd.derivation !== "constraint")
    .filter((r) => r.kind === "resource");

  for (const structureDefinition of r4StructureDefinitions) {
    const pathName = `./docs/API/FHIR/${structureDefinition.name}.mdx`;
    const content = await processStructureDefinition(
      r4Artifacts,
      structureDefinition
    );
    fs.writeFileSync(pathName, content);
  }
}

switch (process.argv[2]) {
  case "fhir": {
    await generateFHIRDocumentation();
    break;
  }
  default: {
    throw new Error(
      "Invalid argument. Please provide either 'npm' or 'fhir' as an argument."
    );
  }
}
