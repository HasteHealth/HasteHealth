import { expect, test } from "@jest/globals";

import { uri } from "@haste-health/fhir-types/r4/types";
import { R4 } from "@haste-health/fhir-types/versions";

import { flatten } from "../utilities";
import metaValue from "./spoof";

test("Simple Type test1", async () => {
  let root = await metaValue(R4, "Patient" as uri);
  let value = root?.descend("name");
  expect(value?.meta()?.type).toEqual("HumanName");
  value = value?.descend(0)?.descend("given");
  expect(value?.meta()?.type).toEqual("string");
  value = value?.descend(0)?.descend("id");
  expect(value?.meta()?.type).toEqual("http://hl7.org/fhirpath/System.String");
});
