import { expect, test } from "@jest/globals";

import { id } from "@haste-health/fhir-types/lib/generated/r4/types";
import { R4 } from "@haste-health/fhir-types/versions";

import { Loc, descend, pointer, toFHIRPath } from "./index.js";

test("Test fhirpath conversion", async () => {
  expect(
    toFHIRPath(
      descend(
        descend(
          descend(
            pointer(R4, "Observation", "test" as id),
            "valueCodeableConcept"
          ),
          "coding"
        ),
        0
      )
    )
  ).toEqual("$this.value.coding[0]");
});

test("4.0|Patient|invalid-us-core-patient/extension/0/extension/0/valueCoding", async () => {
  const fields = ["extension", 0, "extension", 0, "valueCoding"];
  const pt = fields.reduce((pt: Loc<any, any, any>, field: string | number) => {
    return descend(pt, field);
  }, pointer(R4, "Patient", "invalid-us-core-patient" as id));

  expect(toFHIRPath(pt)).toEqual("$this.extension[0].extension[0].value");
});
