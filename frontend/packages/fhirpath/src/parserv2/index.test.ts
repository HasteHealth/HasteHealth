import { expect, test } from "@jest/globals";

import parse from "./index.js";

test("parserv2 primitive test", () => {
  expect(parse("45.5 + 2 * 5 = 27")).toMatchSnapshot();
  expect(parse("'hello'+'wolrd'")).toMatchSnapshot();
  expect(parse("@1980 + @T12:00")).toMatchSnapshot();
  expect(parse("@1980T12:00Z + @T12:00")).toMatchSnapshot();
});

test("Nested access ops", () => {
  expect(parse("$this.test != 45")).toMatchSnapshot();
});

test("Nested access", () => {
  expect(parse("$this.test")).toMatchSnapshot();
});

test("variables", () => {
  expect(parse("%hello.test + %myint_test")).toMatchSnapshot();
});

test("functions", () => {
  expect(parse("hello()")).toMatchSnapshot();
  expect(parse("hello(45)")).toMatchSnapshot();
  expect(parse("hello(45, 32)")).toMatchSnapshot();
  expect(parse("%deep.test(45,32)")).toMatchSnapshot();
});

test("paren", () => {
  expect(parse("($this.test)[$total]")).toMatchSnapshot();
});

test("parser v1 tests", () => {
  expect(parse("$this.test + $this.test2")).toMatchSnapshot();
  expect(parse("$this.test + $this.test2 - 14")).toMatchSnapshot();
  expect(
    parse(
      "45 < $this.test.that * $this.test2 / 14 | 'test' in 'hello' and x != 12 and 45 implies 23",
    ),
  ).toMatchSnapshot();

  // Deliniated identifiers testing
  expect(
    parse("test.`HELLO-WORLD IDENTIFIER`.deliniated.`strange-synt`"),
  ).toMatchSnapshot();

  // expect(parse("45days - 12years")).toMatchSnapshot();
  // Parameter testing
  expect(
    parse(
      "$this.test.where(value != 'test' and value / 11 = 12, $index != $total + %testing)",
    ),
  ).toMatchSnapshot();
  // Literal testing
  expect(parse("'testing'")).toMatchSnapshot();
  expect(
    parse(
      "true != false and @T04:00 = @T04:00 and @1980-01-01 != @1980-01-01T00:00:00Z = @1980-01-01T00:00:00-05:00",
    ),
  ).toMatchSnapshot();
  // Number testing
  expect(parse("45 - 55.23 + 0.23")).toMatchSnapshot();

  expect(parse("this[45 + 15 * 3].hello")).toMatchSnapshot();
  expect(parse("$this.test + 2 * 4")).toMatchSnapshot();

  expect(parse("true.anyTrue()")).toMatchSnapshot();
});

test("Union tests", () => {
  expect(parse("ValueSet.and.test")).toMatchSnapshot();

  expect(
    parse(
      "ValueSet.expansion.contains.code | ValueSet.compose.include.concept.code",
    ),
  ).toMatchSnapshot();
});

test("As tests", () => {
  expect(
    parse("Condition.abatement.as(Age) | Condition.abatement.as(Range)"),
  ).toMatchSnapshot();
});

test("Complex or statement", () => {
  expect(
    parse(
      "%request.parameters.where(name='patient').value = %user.resource.link.reference.replace('Patient/', '') or %request.parameters.where(name='patient').value = %user.resource.link.reference",
    ),
  ).toMatchSnapshot();
});
