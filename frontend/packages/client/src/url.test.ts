import { expect, test } from "@jest/globals";

import parseFHIRSearch, {
  escapeParameter,
  splitParameter,
  unescapeParameter,
} from "./url.js";

test("Test resource level", () => {
  expect(parseFHIRSearch("Patient?name:text=bob")).toEqual([
    { name: "name", modifier: "text", value: ["bob"] },
  ]);
});

test("Test System level", () => {
  expect(
    parseFHIRSearch("Patient?name:text=bob&lastUpdated:not-in=1980-01-01"),
  ).toEqual([
    { name: "name", modifier: "text", value: ["bob"] },
    {
      name: "lastUpdated",
      modifier: "not-in",
      value: ["1980-01-01"],
    },
  ]);
});

test("TEST ESCAPING", () => {
  expect(escapeParameter("test|123")).toEqual("test\\|123");
  expect(unescapeParameter(escapeParameter("test|123"))).toEqual("test|123");
  expect(escapeParameter("test\\123")).toEqual("test\\\\123");
  expect(unescapeParameter(escapeParameter("test\\123"))).toEqual("test\\123");
  expect(escapeParameter("test$123")).toEqual("test\\$123");
  expect(escapeParameter("test,123")).toEqual("test\\,123");
});

test("TEST Parameter Split", () => {
  expect(splitParameter("test\\|123|456", "|")).toEqual(["test|123", "456"]);
  expect(splitParameter("|test\\|123|456|", "|")).toEqual([
    "",
    "test|123",
    "456",
    "",
  ]);

  expect(splitParameter("|123|test\\|123|456", "|")).toEqual([
    "",
    "123",
    "test|123",
    "456",
  ]);
});
