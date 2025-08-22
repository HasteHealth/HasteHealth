import { expect, test } from "@jest/globals";

import FPLexer from "./lexer.js";

test("QLexer Test", async () => {
  expect(FPLexer.tokenize("45.5 + 32.3")).toMatchSnapshot();
});
