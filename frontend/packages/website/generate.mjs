import fs from "node:fs";
import { fileURLToPath } from "url";

import { loadArtifacts } from "@haste-health/artifacts";
import { R4 } from "@haste-health/fhir-types/versions";
import { sdTraversal } from "@haste-health/codegen";

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

function generateProperties(sd) {
  return sdTraversal.traversalBottomUp(sd, (element, nestedElements) => {
    return `<BrowserOnly> {() => <StructureDefinitionDisplay sd="${sd.name}" />}</BrowserOnly>`;
  });
}

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

function escapeLinks(v) {
  return v
    .replaceAll("(", "%28")
    .replaceAll(")", "%29")
    .replaceAll("[", "%5B")
    .replaceAll("]", "%5D");
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
hide_table_of_contents: true
tags:
  - fhir
  - Fast Healthcare Interoperability Resources
  - hl7
  - healthcare it
  - interoperability
---

import TabItem from "@theme/TabItem";
import Tabs from "@theme/Tabs";
import StructureDefinitionDisplay from '@site/src/components/StructureDefinitionDisplay';
import BrowserOnly from '@docusaurus/BrowserOnly';

# ${structureDefinition.name}\n
${escapeLinks(structureDefinition.snapshot?.element[0]?.definition ?? "")}

<head>
  <meta name="keywords" content="fhir, hl7, interoperability, healthcare" />
  <script type="application/ld+json">
    {JSON.stringify({
      '@context': 'https://schema.org/',
      '@type': 'Organization',
      name: 'Haste Health',
      url: 'https://haste.health',
      logo: 'https://haste.health/img/logo.svg',
    })}
  </script>
</head>

  `;
  doc = `${doc}
  <div className="grid grid-cols-1 lg:grid-cols-3 gap-4">
  <div className="col-span-2">
  ## Structure
  ${generateProperties(structureDefinition)}
  </div>
  `;

  doc = `${doc}\n`;

  if (structureDefinition.kind === "resource") {
    doc = `${doc} 
  <div>
  ## Search Parameters\n<div class="space-y-4">`;
    for (const parameter of parameters) {
      const name = parameter.name;
      const type = parameter.type;

      const description = escapeCharacters(parameter.description || "");

      const expression = escapeCharacters(parameter.expression || "");

      doc = `${doc} 
    <div class="text-xs space-y-1">
        <div class="text-sm">
            <span class="font-semibold">${name}</span> <span> (${type})</span>
        </div>
        <div class="text-orange-900 line-clamp-3 truncate"> <span>${escapeLinks(
          description
        )}</span></div>
        ${
          expression
            ? `<div class="line-clamp-3 truncate">
              <code>${expression}</code>
            </div>`
            : ""
        }
    </div>
    \n
  `;
    }
    doc = `${doc} </div></div>`;
  }

  doc = `${doc} </div>`;

  return doc;
}

async function generateFHIRDocumentation() {
  const r4StructureDefinitions = r4Artifacts
    .filter((r) => r.resourceType === "StructureDefinition")
    .filter((sd) => sd.derivation !== "constraint")
    .filter((r) => r.kind === "resource");

  const r4DataTypes = r4Artifacts
    .filter((r) => r.resourceType === "StructureDefinition")
    .filter((sd) => sd.derivation !== "constraint")
    .filter((r) => r.kind === "complex-type" || r.kind === "primitive-type");

  for (const structureDefinition of r4StructureDefinitions) {
    const pathName = `./docs/Reference/FHIR/Model/Resources/${structureDefinition.name}.mdx`;
    const content = await processStructureDefinition(
      r4Artifacts,
      structureDefinition
    );
    fs.writeFileSync(pathName, content);
    fs.writeFileSync(
      "./static/fhir/R4/" + structureDefinition.name + ".json",
      JSON.stringify(structureDefinition, null, 2)
    );
  }

  for (const structureDefinition of r4DataTypes) {
    const pathName = `./docs/Reference/FHIR/Model/Types/${structureDefinition.name}.mdx`;
    const content = await processStructureDefinition(
      r4Artifacts,
      structureDefinition
    );
    fs.writeFileSync(pathName, content);
    fs.writeFileSync(
      "./static/fhir/R4/" + structureDefinition.name + ".json",
      JSON.stringify(structureDefinition, null, 2)
    );
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
