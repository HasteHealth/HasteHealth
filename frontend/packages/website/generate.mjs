import fs from "node:fs";

import { sdTraversal } from "@haste-health/codegen";

const HASTE_HEALTH_URL =
  process.env.HASTE_HEALTH_URL ?? "http://localhost:3000";
const HASTE_HEALTH_TENANT = process.env.HASTE_HEALTH_TENANT ?? "my-health";
const HASTE_HEALTH_PROJECT = process.env.HASTE_HEALTH_PROJECT ?? "system";
const HASTE_HEALTH_USERNAME = process.env.HASTE_HEALTH_USERNAME;
const HASTE_HEALTH_PASSWORD = process.env.HASTE_HEALTH_PASSWORD;

if (!HASTE_HEALTH_USERNAME || !HASTE_HEALTH_PASSWORD) {
  throw new Error(
    "HASTE_HEALTH_USERNAME and HASTE_HEALTH_PASSWORD must be set (see .env.example). " +
      "Run with `pnpm node --env-file-if-exists=.env generate.mjs fhir`.",
  );
}

const fhirBaseUrl = `${HASTE_HEALTH_URL}/w/${HASTE_HEALTH_TENANT}/${HASTE_HEALTH_PROJECT}/api/v1/fhir/r4`;
const credentials = `${HASTE_HEALTH_USERNAME}:${HASTE_HEALTH_PASSWORD}`;
const authHeader = `Basic ${Buffer.from(credentials).toString("base64")}`;

async function fetchAllResources(resourceType) {
  const resources = [];
  let url = `${fhirBaseUrl}/${resourceType}?_count=10000`;

  while (url) {
    const response = await fetch(url, {
      headers: { Authorization: authHeader },
    });
    if (!response.ok) {
      throw new Error(
        `Failed to fetch ${resourceType} from ${url}: ${response.status} ${response.statusText}`,
      );
    }
    const bundle = await response.json();
    resources.push(...(bundle.entry ?? []).map((entry) => entry.resource));
    url = bundle.link?.find((link) => link.relation === "next")?.url;
  }

  return resources;
}

const mcpUrl = `${HASTE_HEALTH_URL}/w/${HASTE_HEALTH_TENANT}/${HASTE_HEALTH_PROJECT}/api/v1/mcp`;

async function fetchMcpTools() {
  const response = await fetch(mcpUrl, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      Authorization: authHeader,
    },
    body: JSON.stringify({ jsonrpc: "2.0", id: 1, method: "tools/list" }),
  });
  if (!response.ok) {
    throw new Error(
      `Failed to fetch MCP tools from ${mcpUrl}: ${response.status} ${response.statusText}`,
    );
  }
  const body = await response.json();
  if (body.error) {
    throw new Error(`MCP tools/list returned an error: ${body.error.message}`);
  }
  return body.result.tools;
}

async function generateMcpDocumentation() {
  const tools = await fetchMcpTools();
  fs.mkdirSync("./static/mcp", { recursive: true });
  fs.writeFileSync("./static/mcp/tools.json", JSON.stringify(tools, null, 2));
}

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

// JSON.stringify produces a double-quoted string with the same escaping
// rules YAML uses for double-quoted scalars, so it's a safe way to drop
// free-text (colons, quotes, newlines) into frontmatter.
function yamlString(v) {
  return JSON.stringify(v ?? "");
}

function metaDescription(sd) {
  const kindLabel =
    sd.kind === "resource" ? "resource" : "data type";
  const raw = (sd.description ?? "")
    .replace(/(\r\n|\n|\r)/gm, " ")
    .replace(/\s+/g, " ")
    .trim();
  const fallback = `Reference documentation for the FHIR R4 ${sd.name} ${kindLabel}: structure, elements, and search parameters.`;
  const base = raw || fallback;
  const prefix = `${sd.name} (FHIR R4 ${kindLabel}): `;
  const budget = 155 - prefix.length;
  const body =
    base.length > budget ? `${base.slice(0, budget - 1).trimEnd()}…` : base;
  return `${prefix}${body}`;
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
        r.base.includes("DomainResource"),
    );

  const description = metaDescription(structureDefinition);
  const kindLabel =
    structureDefinition.kind === "resource" ? "Resource" : "Data Type";
  const collection =
    structureDefinition.kind === "resource" ? "resources" : "types";
  const canonicalUrl = `https://haste.health/docs/reference/fhir/model/${collection}/${structureDefinition.name}`;

  let doc = `---
id: ${structureDefinition.id}
title: ${structureDefinition.name}
description: ${yamlString(description)}
hide_table_of_contents: true
tags:
  - fhir
  - Fast Healthcare Interoperability Resources
  - hl7
  - healthcare it
  - interoperability
  - ${structureDefinition.name}
---

import TabItem from "@theme/TabItem";
import Tabs from "@theme/Tabs";
import StructureDefinitionDisplay from '@site/src/components/StructureDefinitionDisplay';
import BrowserOnly from '@docusaurus/BrowserOnly';

# ${structureDefinition.name}\n
${escapeLinks(structureDefinition.snapshot?.element[0]?.definition ?? "")}

<head>
  <title>${structureDefinition.name} — FHIR R4 ${kindLabel} Reference | Haste Health</title>
  <meta name="description" content="${description.replaceAll('"', "'")}" />
  <meta name="keywords" content="${structureDefinition.name}, FHIR ${structureDefinition.name}, FHIR R4, HL7 FHIR, fhir, hl7, interoperability, healthcare, clinical data repository" />
  <link rel="canonical" href="${canonicalUrl}" />
  <script type="application/ld+json">
    {JSON.stringify({
      '@context': 'https://schema.org/',
      '@type': 'DefinedTerm',
      name: '${structureDefinition.name}',
      description: ${yamlString(description)},
      url: '${canonicalUrl}',
      inDefinedTermSet: {
        '@type': 'DefinedTermSet',
        name: 'FHIR R4 ${kindLabel} Reference',
        publisher: {
          '@type': 'Organization',
          name: 'Haste Health',
          url: 'https://haste.health',
        },
      },
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
        <div class="text-brand-900 line-clamp-3 truncate"> <span>${escapeLinks(
          description,
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
  const r4Artifacts = (
    await Promise.all(
      ["StructureDefinition", "SearchParameter"].map(fetchAllResources),
    )
  ).flat();

  const r4StructureDefinitions = r4Artifacts
    .filter((r) => r.resourceType === "StructureDefinition")
    .filter((sd) => sd.derivation !== "constraint")
    .filter((r) => r.kind === "resource");

  const r4DataTypes = r4Artifacts
    .filter((r) => r.resourceType === "StructureDefinition")
    .filter((sd) => sd.derivation !== "constraint")
    .filter((r) => r.kind === "complex-type" || r.kind === "primitive-type");

  for (const structureDefinition of r4StructureDefinitions) {
    const pathName = `./docs/Reference/fhir/model/resources/${structureDefinition.name}.mdx`;
    const content = await processStructureDefinition(
      r4Artifacts,
      structureDefinition,
    );
    fs.writeFileSync(pathName, content);
    fs.writeFileSync(
      "./static/fhir/R4/" + structureDefinition.name + ".json",
      JSON.stringify(structureDefinition, null, 2),
    );
  }

  for (const structureDefinition of r4DataTypes) {
    const pathName = `./docs/Reference/fhir/model/types/${structureDefinition.name}.mdx`;
    const content = await processStructureDefinition(
      r4Artifacts,
      structureDefinition,
    );
    fs.writeFileSync(pathName, content);
    fs.writeFileSync(
      "./static/fhir/R4/" + structureDefinition.name + ".json",
      JSON.stringify(structureDefinition, null, 2),
    );
  }
}

switch (process.argv[2]) {
  case "fhir": {
    await generateFHIRDocumentation();
    break;
  }
  case "mcp": {
    await generateMcpDocumentation();
    break;
  }
  default: {
    throw new Error(
      "Invalid argument. Please provide either 'fhir' or 'mcp' as an argument.",
    );
  }
}
