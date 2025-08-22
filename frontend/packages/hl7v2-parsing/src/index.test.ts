import { test, expect } from "@jest/globals";
import parseHl7V2Message from "./index";
import fs from "node:fs";

test("ADT Message Parsing", () => {
  const a011 = fs.readFileSync("./test-data/adt-a01-1.txt", "utf8");
  const a012 = fs.readFileSync("./test-data/adt-a01-2.txt", "utf8");
  expect(parseHl7V2Message(a011)).toMatchSnapshot();
  expect(parseHl7V2Message(a012)).toMatchSnapshot();
});
