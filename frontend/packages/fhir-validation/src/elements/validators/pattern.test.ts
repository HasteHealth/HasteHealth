import { expect, test } from "@jest/globals";

import { conformsToPattern } from "./pattern.js";

test("pattern validation", () => {
  // Test number primitives
  expect(conformsToPattern(1, 1)).toBeTruthy();
  expect(conformsToPattern(1, 2)).toBeFalsy();

  // Test string primitives
  expect(conformsToPattern("1", "1")).toBeTruthy();
  expect(conformsToPattern("1", "2")).toBeFalsy();

  // Test complex with different values on keys
  expect(conformsToPattern({ a: 1 }, { a: 1 })).toBeTruthy();
  expect(conformsToPattern({ a: 1 }, { a: 2 })).toBeFalsy();

  // Test complex with additional keys
  expect(conformsToPattern({ a: 1 }, { a: 1, b: 2 })).toBeTruthy();
  expect(conformsToPattern({ a: 1, b: 2 }, { a: 1 })).toBeFalsy();

  // Test nested logic
  expect(conformsToPattern({ a: { a: 1, b: 2 } }, { a: {} })).toBeFalsy();
  expect(
    conformsToPattern({ a: { a: 1, b: 2 } }, { a: { a: 1, b: 2 } }),
  ).toBeTruthy();
  // Test nested additional keys
  expect(
    conformsToPattern({ a: { a: 1, b: 2 } }, { a: { a: 1, b: 2, c: 3 } }),
  ).toBeTruthy();
  expect(conformsToPattern({ a: { a: 1, b: 2 } }, { a: { a: 1 } })).toBeFalsy();

  expect(
    conformsToPattern(
      { coding: [{ system: "http://loinc.org", code: "8480-6" }] },
      {
        coding: [
          {
            system: "http://loinc.org",
            code: "8462-4",
            display: "Diastolic blood pressure",
          },
        ],
        text: "Diastolic blood pressure",
      },
    ),
  ).toBeFalsy();
});
