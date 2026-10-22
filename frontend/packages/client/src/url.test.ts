import { expect, test } from "@jest/globals";

import parseFHIRSearch, {
  escapeParameter,
  parameterValues,
  splitParameter,
  splitUnescaped,
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

test("TEST escaped separators", () => {
  expect(splitUnescaped("a\\,b,c", ",")).toEqual(["a\\,b", "c"]);
  expect(splitUnescaped("a\\\\,b", ",")).toEqual(["a\\\\", "b"]);
  expect(unescapeParameter("a\\\\,b")).toEqual("a\\,b");
  expect(unescapeParameter("a\\,b\\|c\\$d")).toEqual("a,b|c$d");
  expect(unescapeParameter("a\\xb")).toEqual("a\\xb");
});

test("TEST query comma escaping keeps the pieces as written", () => {
  expect(parseFHIRSearch("Patient?name=Smith\\,Jones")[0].value).toEqual([
    "Smith\\,Jones",
  ]);
  expect(parseFHIRSearch("Patient?name=Smith,Jones")[0].value).toEqual([
    "Smith",
    "Jones",
  ]);
  expect(parseFHIRSearch("Patient?identifier=a\\|b|1,c")[0].value).toEqual([
    "a\\|b|1",
    "c",
  ]);
  expect(parseFHIRSearch("Patient?name=a\\\\,b")[0].value).toEqual([
    "a\\\\",
    "b",
  ]);
});

test("TEST accessors resolve the escapes", () => {
  const [name] = parseFHIRSearch("Patient?name=Smith\\,Jones,a\\\\b");
  expect(parameterValues(name)).toEqual(["Smith,Jones", "a\\b"]);
  const [identifier] = parseFHIRSearch("Patient?identifier=http://a\\|b|1,c");
  expect(identifier.value.map((v) => splitParameter(v, "|"))).toEqual([
    ["http://a|b", "1"],
    ["c"],
  ]);
});
