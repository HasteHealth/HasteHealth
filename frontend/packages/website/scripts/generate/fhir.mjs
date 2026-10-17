// The FHIR R4 reference (docs/reference/fhir): a page per resource and data
// type, from the StructureDefinitions the server embeds, and a page for the
// system level, from what the server declares. No running server is needed:
// the server step has already snapshotted it.
//
//   docs/reference/fhir/model/{resources,types}/<Name>.mdx  thin pages
//   docs/reference/fhir/system.mdx
//   src/fhir-model/<Name>.json, system.json, index.json     page data
//   static/fhir/R4/<Name>.json                              raw StructureDefinition
//
// Inputs: artifacts/r4 (structure definitions and search parameters), the
// conformance report (test-support step) and the server snapshot (server
// step): its CapabilityStatement for the interactions, its OperationDefinitions
// for the operations each resource type and the system level offer.

import fs from "node:fs";
import path from "node:path";

import {
  ARTIFACTS,
  METADATA,
  OPERATION_DEFINITIONS,
  SUPPORT,
  combine,
  jsonFiles,
  readJson,
  readResources,
  relative,
  removeStale,
  site,
  writeFile,
  writeJson,
} from "./lib.mjs";

const HL7 = path.join(ARTIFACTS, "hl7-core/definitions/hl7");
const HASTE = path.join(ARTIFACTS, "hastehealth-core/definitions");

const DOCS = site("docs/reference/fhir");
const DATA = site("src/fhir-model");
const RAW = site("static/fhir/R4");

const FHIR_TYPE_EXT =
  "http://hl7.org/fhir/StructureDefinition/structuredefinition-fhir-type";
const EXT = "http://hl7.org/fhir/StructureDefinition/structuredefinition-";

// ---------------------------------------------------------------------------
// Loading
// ---------------------------------------------------------------------------

/** The same resources the server embeds (backend/crates/artifacts). */
function loadArtifacts() {
  const hl7 = fs
    .readdirSync(HL7)
    .filter((name) => name.endsWith(".min.json"))
    .sort()
    .flatMap((name) => readResources(path.join(HL7, name)))
    .map((resource) => ({ resource, source: "hl7" }));
  const haste = jsonFiles(HASTE, { skip: ["operation-frontend-only"] }).flatMap((file) => {
    const source = file.includes("/sql-on-fhir/") ? "sql-on-fhir" : "haste-health";
    return readResources(file).map((resource) => ({ resource, source }));
  });
  return [...hl7, ...haste];
}

/**
 * The .min.json bundles drop StructureDefinition extensions, so the category,
 * maturity and status come from the upstream bundles.
 */
function loadHl7Metadata() {
  const metadata = new Map();
  for (const file of ["profiles-resources.json", "profiles-types.json"]) {
    for (const sd of readResources(path.join(HL7, file))) {
      if (sd.resourceType !== "StructureDefinition") continue;
      const ext = (name) => sd.extension?.find((e) => e.url === EXT + name) ?? {};
      metadata.set(sd.url, {
        category: ext("category").valueString?.replaceAll("&amp;", "&"),
        status: ext("standards-status").valueCode,
        maturity: ext("fmm").valueInteger,
      });
    }
  }
  return metadata;
}

function loadSupport() {
  if (!fs.existsSync(SUPPORT)) return new Map();
  return new Map(readJson(SUPPORT).resources.map((r) => [r.resourceType, r]));
}

/** The server snapshot: its CapabilityStatement and the OperationDefinitions it serves. */
function loadServer() {
  if (!fs.existsSync(METADATA)) {
    throw new Error(
      `No server snapshot at ${relative(METADATA)}; run \`pnpm generate server\` first.`,
    );
  }
  const capabilities = readJson(METADATA);
  const rest = capabilities.rest?.find((r) => r.mode === "server") ?? capabilities.rest?.[0] ?? {};
  const definitions = jsonFiles(OPERATION_DEFINITIONS)
    .map(readJson)
    .filter((resource) => resource.resourceType === "OperationDefinition" && resource.code);
  return { capabilities, rest, definitions };
}

// ---------------------------------------------------------------------------
// Transforming
// ---------------------------------------------------------------------------

const COLLECTION = { resource: "resources" };
const pageHref = (name, kind) =>
  `/docs/reference/fhir/model/${COLLECTION[kind] ?? "types"}/${name}`;
const SYSTEM_HREF = "/docs/reference/fhir/system";

/** `http://hl7.org/fhirpath/System.String` with a fhir-type extension is `string`. */
function typeCode(type) {
  const fhirType = type.extension?.find((e) => e.url === FHIR_TYPE_EXT);
  return fhirType?.valueUrl ?? type.code.replace("http://hl7.org/fhirpath/", "");
}

const lastSegment = (url) => url.split("|")[0].split("/").pop();

const targetLinks = (profiles, pages) =>
  profiles.map(lastSegment).map((name) => ({ name, href: pages.get(name) }));

function elementTypes(element, pages) {
  return (element.type ?? []).map((type) => {
    const code = typeCode(type);
    const targets = type.targetProfile ?? [];
    return {
      code,
      href: pages.get(code),
      ...(targets.length > 0 && { targets: targetLinks(targets, pages) }),
    };
  });
}

function binding(element) {
  if (!element.binding?.valueSet) return undefined;
  return {
    strength: element.binding.strength,
    valueSet: element.binding.valueSet.split("|")[0],
  };
}

function elements(sd, pages) {
  const type = sd.type ?? sd.name;
  return sd.snapshot.element.slice(1).map((element) => ({
    path: element.path,
    min: element.min ?? 0,
    max: element.max ?? "1",
    types: elementTypes(element, pages),
    short: element.short ?? "",
    ...(element.contentReference && {
      contentReference: element.contentReference.replace(/^#/, ""),
    }),
    ...(element.isModifier && { modifier: true }),
    ...(element.isSummary && { summary: true }),
    ...(element.base && element.base.path.split(".")[0] !== type && { inherited: true }),
    ...(binding(element) && { binding: binding(element) }),
  }));
}

/**
 * Parameters shared by several resources describe each one on its own line
 * ("* [Patient](patient.html): A patient identifier") and join their
 * expressions with `|`. Keep only the part for this resource.
 */
function descriptionFor(description, name) {
  const line = description.split(/\r?\n/).find((l) => l.startsWith(`* [${name}](`));
  return line ? line.replace(/^\* \[[^\]]+\]\([^)]*\):\s*/, "") : description;
}

function expressionFor(expression, name) {
  if (!expression) return undefined;
  const parts = expression.split(/\s\|\s/).map((p) => p.trim());
  const own = parts.filter((p) => p.startsWith(`${name}.`) || p.startsWith(`(${name}.`));
  return (own.length > 0 ? own : parts).join(" | ");
}

function paramCoverage(support, code) {
  const groups = support?.groups.filter(
    (g) => g.group === "search" && g.searchParameterCode === code,
  );
  if (!groups?.length) return undefined;
  return {
    postgres: combine(groups.map((g) => g.results.postgres)),
    elasticsearch: combine(groups.map((g) => g.results.elasticsearch)),
  };
}

function searchParameters(name, parameters, support) {
  return parameters
    .filter((p) => p.base.includes(name))
    .map((p) => ({
      code: p.code,
      type: p.type,
      description: descriptionFor(p.description ?? "", name),
      expression: expressionFor(p.expression, name),
      ...(paramCoverage(support, p.code) && { coverage: paramCoverage(support, p.code) }),
    }))
    .sort((a, b) => a.code.localeCompare(b.code));
}

const INTERACTIONS = ["create", "read", "update", "patch", "delete"];

function coverage(support) {
  if (!support) return undefined;
  const byGroup = (group) => {
    const groups = support.groups.filter((g) => g.group === group);
    if (groups.length === 0) return undefined;
    return {
      postgres: combine(groups.map((g) => g.results.postgres)),
      elasticsearch: combine(groups.map((g) => g.results.elasticsearch)),
    };
  };
  return {
    overall: support.overall,
    interactions: Object.fromEntries(
      INTERACTIONS.map((i) => [i, byGroup(i)]).filter(([, r]) => r),
    ),
  };
}

function hl7Link(sd, source) {
  if (source !== "hl7") return undefined;
  return sd.kind === "resource"
    ? `https://hl7.org/fhir/R4/${sd.name.toLowerCase()}.html`
    : `https://hl7.org/fhir/R4/datatypes.html`;
}

// Operations, as the server defines them (haste_fhir_ops::defines): a
// definition applies at each level it flags, and to the resource types it
// lists, or to every type when it lists none.

const LEVELS = ["system", "type", "instance"];
const levelsOf = (definition) => LEVELS.filter((level) => definition[level] === true);

function definedOn(definition, type) {
  const types = definition.resource ?? [];
  return types.length === 0 || types.includes(type);
}

/** The HL7 core operations each have a page: operation-patient-everything.html. */
function hl7Operation(definition) {
  const match = /^http:\/\/hl7\.org\/fhir\/OperationDefinition\/([^|]+)$/.exec(
    definition.url ?? "",
  );
  return match ? `https://hl7.org/fhir/R4/operation-${match[1].toLowerCase()}.html` : undefined;
}

function parameter(part, pages) {
  return {
    name: part.name,
    use: part.use,
    min: part.min ?? 0,
    max: part.max ?? "1",
    ...(part.type && { type: part.type, href: pages.get(part.type) }),
    ...(part.targetProfile?.length > 0 && { targets: targetLinks(part.targetProfile, pages) }),
    ...(part.searchType && { searchType: part.searchType }),
    ...(part.documentation && { documentation: part.documentation }),
    ...(part.binding?.valueSet && {
      binding: { strength: part.binding.strength, valueSet: part.binding.valueSet.split("|")[0] },
    }),
    ...(part.part?.length > 0 && { parts: part.part.map((p) => parameter(p, pages)) }),
  };
}

function operation(definition, pages) {
  return {
    code: definition.code,
    id: definition.id,
    url: definition.url,
    title: definition.title ?? definition.name,
    levels: levelsOf(definition),
    resources: (definition.resource ?? []).map((name) => ({ name, href: pages.get(name) })),
    description: definition.description ?? "",
    affectsState: definition.affectsState === true,
    hl7: hl7Operation(definition),
    raw: `/fhir/R4/OperationDefinition/${definition.id}.json`,
    parameters: (definition.parameter ?? []).map((p) => parameter(p, pages)),
  };
}

const byCode = (a, b) => a.code.localeCompare(b.code);

/** The operations invoked on `type` or its instances. */
function operationsFor(type, definitions, pages) {
  return definitions
    .filter((d) => (d.type === true || d.instance === true) && definedOn(d, type))
    .map((d) => operation(d, pages))
    .sort(byCode);
}

function systemData(server, definitions, pages, resources) {
  const { capabilities, rest } = server;
  return {
    fhirVersion: capabilities.fhirVersion,
    formats: capabilities.format ?? [],
    cors: rest.security?.cors === true,
    interactions: (rest.interaction ?? []).map((i) => i.code),
    operations: definitions
      .filter((d) => d.system === true)
      .map((d) => operation(d, pages))
      .sort(byCode),
    resourceTypes: rest.resource?.length ?? 0,
    resources,
    raw: "/fhir/R4/metadata.json",
  };
}

// ---------------------------------------------------------------------------
// Writing
// ---------------------------------------------------------------------------

// JSON.stringify produces a double-quoted string with the same escaping
// rules YAML uses for double-quoted scalars.
const yamlString = (v) => JSON.stringify(v ?? "");

const GENERATED = "{/* Generated by scripts/generate/fhir.mjs. Do not edit. */}";

const KIND_LABEL = {
  resource: "resource",
  "complex-type": "data type",
  "primitive-type": "data type",
};

/** `[text](url)` -> `text`, scanning rather than with a backtracking regex. */
function stripLinks(text) {
  let result = "";
  let index = 0;
  while (index < text.length) {
    const open = text.indexOf("[", index);
    const close = open === -1 ? -1 : text.indexOf("](", open);
    const end = close === -1 ? -1 : text.indexOf(")", close);
    if (end === -1) break;
    result += text.slice(index, open) + text.slice(open + 1, close);
    index = end + 1;
  }
  return result + text.slice(index);
}

function metaDescription(sd, definition) {
  const kind = KIND_LABEL[sd.kind];
  const raw = stripLinks(definition);
  const body = raw.replace(/\s+/g, " ").trim();
  const prefix = `${sd.name} (FHIR R4 ${kind}): `;
  const budget = 155 - prefix.length;
  const text = body.length > budget ? `${body.slice(0, budget - 1).trimEnd()}…` : body;
  return `${prefix}${text}`;
}

function page(sd, data) {
  const isResource = sd.kind === "resource";
  const kindTitle = isResource ? "Resource" : "Data Type";
  const description = metaDescription(sd, data.definition);
  const canonical = `https://haste.health${pageHref(sd.name, sd.kind)}`;
  const sections = [];
  if (isResource) {
    sections.push(`## Search parameters\n\n<SearchParameters data={data} />\n`);
  }
  if (data.operations?.length > 0) {
    sections.push(
      `## Operations\n\n<Operations operations={data.operations} type=${yamlString(sd.name)} />\n`,
    );
  }

  return `---
id: ${sd.name}
title: ${sd.name}
description: ${yamlString(description)}
hide_table_of_contents: true
tags:
  - fhir
  - Fast Healthcare Interoperability Resources
  - hl7
  - healthcare it
  - interoperability
  - ${sd.name}
---

${GENERATED}

import { ModelHeader, ElementTree, Operations, SearchParameters } from '@site/src/components/FhirModel';
import data from '@site/src/fhir-model/${sd.name}.json';

<head>
  <title>${sd.name} — FHIR R4 ${kindTitle} Reference | Haste Health</title>
  <meta name="keywords" content="${sd.name}, FHIR ${sd.name}, FHIR R4, HL7 FHIR, fhir, hl7, interoperability, healthcare, clinical data repository" />
  <link rel="canonical" href="${canonical}" />
  <script type="application/ld+json">
    {JSON.stringify({
      '@context': 'https://schema.org/',
      '@type': 'DefinedTerm',
      name: ${yamlString(sd.name)},
      description: ${yamlString(description)},
      url: ${yamlString(canonical)},
      inDefinedTermSet: {
        '@type': 'DefinedTermSet',
        name: 'FHIR R4 ${kindTitle} Reference',
        publisher: {
          '@type': 'Organization',
          name: 'Haste Health',
          url: 'https://haste.health',
        },
      },
    })}
  </script>
</head>

# ${sd.name}

<ModelHeader data={data} />

## Structure

<ElementTree data={data} />
${sections.map((section) => `\n${section}`).join("")}`;
}

function systemPage(data) {
  const description =
    "What the Haste Health FHIR R4 server offers at the system level: the interactions on the base URL, the system-level operations and the operations each resource type supports.";
  const canonical = `https://haste.health${SYSTEM_HREF}`;
  return `---
id: system
title: System-level API
sidebar_label: System
sidebar_position: 2
description: ${yamlString(description)}
hide_table_of_contents: true
tags:
  - fhir
  - hl7
  - interoperability
  - CapabilityStatement
---

${GENERATED}

import { OperationIndex, Operations, SystemHeader, SystemInteractions } from '@site/src/components/FhirModel';
import data from '@site/src/fhir-model/system.json';

<head>
  <title>System-level API — FHIR R4 Reference | Haste Health</title>
  <link rel="canonical" href="${canonical}" />
</head>

# System-level API

<SystemHeader data={data} />

## Interactions

<SystemInteractions data={data} />

## Operations

<Operations operations={data.operations} />

## Operations by resource type

<OperationIndex data={data} />
`;
}

export async function run() {
  const artifacts = loadArtifacts();
  const metadata = loadHl7Metadata();
  const support = loadSupport();
  const server = loadServer();

  const structureDefinitions = artifacts.filter(
    ({ resource }) =>
      resource.resourceType === "StructureDefinition" &&
      resource.derivation !== "constraint" &&
      ["resource", "complex-type", "primitive-type"].includes(resource.kind),
  );
  const parameters = artifacts
    .map(({ resource }) => resource)
    .filter((r) => r.resourceType === "SearchParameter");

  const pages = new Map(
    structureDefinitions.map(({ resource: sd }) => [sd.name, pageHref(sd.name, sd.kind)]),
  );

  const common = parameters
    .filter((p) => p.base.includes("Resource") || p.base.includes("DomainResource"))
    .filter((p) => p.expression)
    .map((p) => ({ code: p.code, type: p.type, description: p.description ?? "" }))
    .sort((a, b) => a.code.localeCompare(b.code));

  const index = [];
  const resourceOperations = [];

  for (const { resource: sd, source } of structureDefinitions) {
    const meta = metadata.get(sd.url) ?? {};
    const resourceSupport = support.get(sd.name);
    const isResource = sd.kind === "resource";
    const data = {
      name: sd.name,
      kind: sd.kind,
      url: sd.url,
      source,
      definition: sd.snapshot.element[0]?.definition ?? sd.description ?? "",
      ...meta,
      hl7: hl7Link(sd, source),
      raw: `/fhir/R4/${sd.name}.json`,
      elements: elements(sd, pages),
      ...(isResource && {
        searchParameters: searchParameters(sd.name, parameters, resourceSupport),
        commonParameters: common,
        coverage: coverage(resourceSupport),
        operations: operationsFor(sd.name, server.definitions, pages),
      }),
    };

    writeJson(`${DATA}/${sd.name}.json`, data);
    writeFile(`${DOCS}/model/${isResource ? "resources" : "types"}/${sd.name}.mdx`, page(sd, data));
    writeJson(`${RAW}/${sd.name}.json`, sd, { pretty: true });

    if (isResource && data.operations.length > 0) {
      resourceOperations.push({
        name: sd.name,
        href: pages.get(sd.name),
        operations: data.operations.map(({ code, levels }) => ({
          code,
          levels: levels.filter((level) => level !== "system"),
        })),
      });
    }

    index.push({
      name: sd.name,
      kind: sd.kind,
      source,
      href: pages.get(sd.name),
      short: sd.snapshot.element[0]?.short ?? "",
      ...(meta.category && { category: meta.category }),
      ...(meta.maturity !== undefined && { maturity: meta.maturity }),
      ...(meta.status && { status: meta.status }),
      ...(isResource && {
        searchParameters: data.searchParameters.length,
        operations: data.operations.length,
        ...(resourceSupport && { overall: resourceSupport.overall }),
      }),
    });
  }

  index.sort((a, b) => a.name.localeCompare(b.name));
  writeJson(`${DATA}/index.json`, index);

  const system = systemData(server, server.definitions, pages, resourceOperations);
  writeJson(`${DATA}/system.json`, system);
  writeFile(`${DOCS}/system.mdx`, systemPage(system));

  const names = new Set(index.map((entry) => entry.name));
  removeStale(`${DOCS}/model/resources`, ".mdx", new Set([...names, "index"]));
  removeStale(`${DOCS}/model/types`, ".mdx", new Set([...names, "index"]));
  removeStale(DATA, ".json", new Set([...names, "index", "system"]));
  removeStale(RAW, ".json", new Set([...names, "metadata"]));

  const resources = index.filter((e) => e.kind === "resource").length;
  return `Wrote ${resources} resources (${resourceOperations.length} with operations), ${index.length - resources} data types and the system page (${system.interactions.length} interactions, ${system.operations.length} operations)`;
}
