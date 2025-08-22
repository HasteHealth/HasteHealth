import { expect, test } from "@jest/globals";

import {
  ConceptMap,
  Patient,
  Practitioner,
} from "@oxidized-health/fhir-types/r4/types";
import { R4 } from "@oxidized-health/fhir-types/versions";

import { flatten } from "../utilities";
import { metaValue } from "./index";

test("Simple Type test1", async () => {
  const patient: Patient = {
    id: "123",
    resourceType: "Patient",
    identifier: [{ system: "mrn", value: "123" }],
    name: [{ given: ["bob"], _given: [{ id: "test" }] }],
    deceasedBoolean: true,
  } as Patient;

  let root = await metaValue({ fhirVersion: R4 }, patient, []);
  expect(root?.meta()?.type).toEqual("Patient");
  let value = root?.descend("name");
  expect(value?.meta()?.type).toEqual("HumanName");
  value = value?.descend(0)?.descend("given");
  expect(value?.meta()?.type).toEqual("string");
  expect(value?.descend(0)?.getValue()).toEqual("bob");
  value = value?.descend(0)?.descend("id");
  expect(value?.meta()?.type).toEqual("http://hl7.org/fhirpath/System.String");
  expect(value?.getValue()).toEqual("test");
});

test("Simple Type test2", async () => {
  const patient: Patient = {
    id: "123",
    resourceType: "Patient",
    identifier: [{ system: "mrn", value: "123" }],
    name: [{ given: ["bob"] }],
    deceasedBoolean: true,
  } as Patient;
  const myValue = await metaValue({ fhirVersion: R4 }, patient);
  expect(myValue?.descend("name")?.getValue()).toEqual([{ given: ["bob"] }]);
  expect(myValue?.descend("name")?.meta()?.type).toEqual("HumanName");
  expect(myValue?.descend("identifier")?.meta()?.type).toEqual("Identifier");
  expect(myValue?.descend("id")?.meta()?.type).toEqual(
    "http://hl7.org/fhirpath/System.String"
  );
  expect(myValue?.descend("deceased")?.getValue()).toEqual(true);
  expect(myValue?.descend("deceased")?.meta()?.type).toEqual("boolean");

  expect(
    flatten(myValue?.descend("identifier")).map(
      (v) => v.descend("system")?.meta()?.type
    )
  ).toEqual(["uri"]);
});

test("ConceptMap test", async () => {
  const cm: ConceptMap = {
    resourceType: "ConceptMap",
    status: "final",
    group: [
      {
        unmapped: {
          mode: "other-map",
          url: "test",
        },
        element: [
          {
            target: [
              {
                equivalence: "equal",
                dependsOn: [
                  { property: "system", value: "http://snomed.info/sct" },
                ],
                product: [
                  {
                    property: "code",
                    system: "http://snomed.info/sct",
                    value: "123",
                  },
                ],
              },
            ],
          },
        ],
      },
    ],
  } as ConceptMap;

  const myValue = await metaValue({ fhirVersion: R4 }, cm);

  //ConceptMap.group.element.target.product.property
  let cur = flatten(myValue?.descend("group"))[0];
  expect(cur?.meta()?.type).toEqual("BackboneElement");
  cur = flatten(cur.descend("element"))[0];
  expect(cur?.meta()?.type).toEqual("BackboneElement");
  cur = flatten(cur.descend("target"))[0];
  expect(cur?.meta()?.type).toEqual("BackboneElement");
  cur = flatten(cur.descend("product"))[0];
  expect(cur?.meta()?.type).toEqual("BackboneElement");
  cur = flatten(cur.descend("property"))[0];
  expect(cur?.meta()?.type).toEqual("uri");

  // Test unmapped
  cur = flatten(myValue?.descend("group"))[0];
  expect(cur?.meta()?.type).toEqual("BackboneElement");
  cur = flatten(cur.descend("unmapped"))[0];
  expect(cur?.meta()?.type).toEqual("BackboneElement");
  cur = flatten(cur.descend("url"))[0];
  expect(cur?.meta()?.type).toEqual("canonical");
});

test("Location test", async () => {
  const patient: Patient = {
    id: "123",
    resourceType: "Patient",
    identifier: [{ system: "mrn", value: "123" }],
    name: [{ given: ["bob", "frank"] }],
    deceasedBoolean: true,
  } as Patient;

  const myValue = await metaValue({ fhirVersion: R4 }, patient);
  let cur = flatten(await myValue?.descend("name"));
  cur = flatten(cur[0]?.descend("given"));
  expect(cur.map((v) => v.location())).toEqual([
    ["name", 0, "given", 0],
    ["name", 0, "given", 1],
  ]);

  cur = flatten(myValue?.descend("identifier"));
  cur = flatten(cur[0]?.descend("system"));
  expect(cur[0].location()).toEqual(["identifier", 0, "system"]);
});

test("typechoice", async () => {
  const practitioner: Practitioner = {
    extension: [{ url: "test", valueReference: { reference: "urn:oid:2" } }],
    resourceType: "Practitioner",
    name: [{ given: ["Bob"] }],
  } as Practitioner;
  const myValue = await metaValue({ fhirVersion: R4 }, practitioner);

  let cur = flatten(myValue?.descend("extension"));
  cur = flatten(cur[0].descend("value"));

  expect(cur[0].meta()?.type).toEqual("Reference");

  cur = flatten(myValue?.descend("extension"));
  cur = flatten(cur[0].descend("value"));
  expect(cur[0].meta()?.type).toEqual("Reference");
});

function getValue(loc: (string | number)[] = [], obj: any) {
  let cur: any = obj;
  for (const l of loc) {
    cur = cur[l];
  }
  return cur;
}

test("Location test primitive extensions", async () => {
  const practitioner = await metaValue(
    { fhirVersion: R4 },
    {
      id: "123",
      resourceType: "Patient",
      identifier: [{ system: "mrn", value: "123" }],
      deceasedBoolean: true,
      name: [
        {
          given: ["Bob"],
          _given: [
            { extension: [{ url: "https://given.com", valueString: "given" }] },
          ],
        },
      ],
      _deceasedBoolean: {
        extension: [{ url: "https://test.com", valueString: "boolean" }],
      },
    }
  );

  let cur = flatten(practitioner?.descend("deceased"));
  cur = flatten(cur[0].descend("extension"));
  cur = flatten(cur[0].descend("value"));
  expect(cur[0].location()).toEqual([
    "_deceasedBoolean",
    "extension",
    0,
    "valueString",
  ]);
  expect(cur[0].getValue()).toEqual("boolean");
  expect(getValue(cur[0].location(), practitioner?.getValue())).toEqual(
    "boolean"
  );

  cur = flatten(practitioner?.descend("name"));
  cur = flatten(cur[0].descend("given"));
  cur = flatten(cur[0].descend("extension"));
  cur = flatten(cur[0].descend("value"));

  expect(cur[0].location()).toEqual([
    "name",
    0,
    "_given",
    0,
    "extension",
    0,
    "valueString",
  ]);
  expect(cur[0].getValue()).toEqual("given");
  expect(getValue(cur[0].location(), practitioner?.getValue())).toEqual(
    "given"
  );
});
