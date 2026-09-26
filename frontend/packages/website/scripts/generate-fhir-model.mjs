// Generates the FHIR R4 model reference (docs/reference/fhir/model) from the
// artifacts the server embeds, so no running server is needed:
//
//   docs/reference/fhir/model/{resources,types}/<Name>.mdx  thin pages
//   src/fhir-model/<Name>.json                         page data
//   src/fhir-model/index.json                          index pages
//   static/fhir/R4/<Name>.json                              raw StructureDefinition
//
// Test results for each resource come from static/test-reports/support.json
// (see generate-test-support.mjs), so run that first when reports change.
//
// Usage: node scripts/generate-fhir-model.mjs

import fs from "node:fs";
import path from "node:path";

const ARTIFACTS = "../../../artifacts/r4";
const HL7 = `${ARTIFACTS}/hl7-core/definitions/hl7`;
const HASTE = `${ARTIFACTS}/hastehealth-core/definitions`;
const SUPPORT = "static/test-reports/support.json";

const DOCS = "docs/reference/fhir/model";
const DATA = "src/fhir-model";
const RAW = "static/fhir/R4";

const FHIR_TYPE_EXT =
  "http://hl7.org/fhir/StructureDefinition/structuredefinition-fhir-type";
const EXT = "http://hl7.org/fhir/StructureDefinition/structuredefinition-";

// ---------------------------------------------------------------------------
// Loading
// ---------------------------------------------------------------------------

function readResources(file) {
  const json = JSON.parse(fs.readFileSync(file, "utf8"));
  if (json.resourceType === "Bundle") {
    return (json.entry ?? []).map((entry) => entry.resource);
  }
  return Array.isArray(json) ? json : [json];
}

function jsonFiles(dir) {
  return fs.readdirSync(dir, { withFileTypes: true }).flatMap((entry) => {
    const file = path.join(dir, entry.name);
    if (entry.isDirectory()) {
      return entry.name === "operation-frontend-only" ? [] : jsonFiles(file);
    }
    return entry.name.endsWith(".json") ? [file] : [];
  });
}

/** The same resources the server embeds (backend/crates/artifacts). */
function loadArtifacts() {
  const hl7 = fs
    .readdirSync(HL7)
    .filter((name) => name.endsWith(".min.json"))
    .flatMap((name) => readResources(path.join(HL7, name)))
    .map((resource) => ({ resource, source: "hl7" }));
  const haste = jsonFiles(HASTE).flatMap((file) => {
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
      const ext = (name) =>
        sd.extension?.find((e) => e.url === EXT + name) ?? {};
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
  const support = JSON.parse(fs.readFileSync(SUPPORT, "utf8"));
  return new Map(support.resources.map((r) => [r.resourceType, r]));
}

// ---------------------------------------------------------------------------
// Transforming
// ---------------------------------------------------------------------------

const COLLECTION = { resource: "resources" };
const pageHref = (name, kind) =>
  `/docs/reference/fhir/model/${COLLECTION[kind] ?? "types"}/${name}`;

/** `http://hl7.org/fhirpath/System.String` with a fhir-type extension is `string`. */
function typeCode(type) {
  const fhirType = type.extension?.find((e) => e.url === FHIR_TYPE_EXT);
  return fhirType?.valueUrl ?? type.code.replace("http://hl7.org/fhirpath/", "");
}

const lastSegment = (url) => url.split("|")[0].split("/").pop();

function elementTypes(element, pages) {
  return (element.type ?? []).map((type) => {
    const code = typeCode(type);
    const targets = (type.targetProfile ?? []).map(lastSegment);
    return {
      code,
      href: pages.get(code),
      ...(targets.length > 0 && {
        targets: targets.map((name) => ({ name, href: pages.get(name) })),
      }),
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
    ...(element.base &&
      element.base.path.split(".")[0] !== type && { inherited: true }),
    ...(binding(element) && { binding: binding(element) }),
  }));
}

/**
 * Parameters shared by several resources describe each one on its own line
 * ("* [Patient](patient.html): A patient identifier") and join their
 * expressions with `|`. Keep only the part for this resource.
 */
function descriptionFor(description, name) {
  const line = description
    .split(/\r?\n/)
    .find((l) => l.startsWith(`* [${name}](`));
  return line ? line.replace(/^\* \[[^\]]+\]\([^)]*\):\s*/, "") : description;
}

function expressionFor(expression, name) {
  if (!expression) return undefined;
  const parts = expression.split(/\s\|\s/).map((p) => p.trim());
  const own = parts.filter(
    (p) => p.startsWith(`${name}.`) || p.startsWith(`(${name}.`),
  );
  return (own.length > 0 ? own : parts).join(" | ");
}

/** The worst result: fail, then warn (a known failure), then pass. */
function combine(results) {
  if (results.includes("fail")) return "fail";
  if (results.length === 0 || results.includes("not-run")) return "not-run";
  return results.includes("warn") ? "warn" : "pass";
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
      ...(paramCoverage(support, p.code) && {
        coverage: paramCoverage(support, p.code),
      }),
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

// ---------------------------------------------------------------------------
// Writing
// ---------------------------------------------------------------------------

// JSON.stringify produces a double-quoted string with the same escaping
// rules YAML uses for double-quoted scalars.
const yamlString = (v) => JSON.stringify(v ?? "");

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
  const sections = isResource
    ? `
## Search parameters

<SearchParameters data={data} />
`
    : "";

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

{/* Generated by scripts/generate-fhir-model.mjs. Do not edit. */}

import { ModelHeader, ElementTree, SearchParameters } from '@site/src/components/FhirModel';
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
${sections}`;
}

function writeFile(file, content) {
  fs.mkdirSync(path.dirname(file), { recursive: true });
  fs.writeFileSync(file, content);
}

/** Removes pages from a previous run whose type no longer exists. */
function removeStale(dir, extension, keep) {
  if (!fs.existsSync(dir)) return;
  for (const file of fs.readdirSync(dir)) {
    const name = file.slice(0, -extension.length);
    if (file.endsWith(extension) && name !== "index" && !keep.has(name)) {
      fs.rmSync(path.join(dir, file));
    }
  }
}

function main() {
  const artifacts = loadArtifacts();
  const metadata = loadHl7Metadata();
  const support = loadSupport();

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
    structureDefinitions.map(({ resource: sd }) => [
      sd.name,
      pageHref(sd.name, sd.kind),
    ]),
  );

  const common = parameters
    .filter((p) => p.base.includes("Resource") || p.base.includes("DomainResource"))
    .filter((p) => p.expression)
    .map((p) => ({ code: p.code, type: p.type, description: p.description ?? "" }))
    .sort((a, b) => a.code.localeCompare(b.code));

  const index = [];

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
      }),
    };

    writeFile(`${DATA}/${sd.name}.json`, JSON.stringify(data));
    writeFile(
      `${DOCS}/${isResource ? "resources" : "types"}/${sd.name}.mdx`,
      page(sd, data),
    );
    writeFile(`${RAW}/${sd.name}.json`, JSON.stringify(sd, null, 2));

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
        ...(resourceSupport && { overall: resourceSupport.overall }),
      }),
    });
  }

  index.sort((a, b) => a.name.localeCompare(b.name));
  writeFile(`${DATA}/index.json`, JSON.stringify(index));

  const names = new Set(index.map((entry) => entry.name));
  removeStale(`${DOCS}/resources`, ".mdx", names);
  removeStale(`${DOCS}/types`, ".mdx", names);
  removeStale(DATA, ".json", names);
  removeStale(RAW, ".json", names);

  const resources = index.filter((e) => e.kind === "resource").length;
  console.log(
    `Wrote ${resources} resources and ${index.length - resources} data types.`,
  );
}

main();
