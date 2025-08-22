import { expect, test } from "@jest/globals";

import execute from "./index.js";

test("X-FHIR-QUERY testing", async () => {
  const result = await execute("test={{%person.name}}", {
    variables: { person: { name: "John Doe" } },
  });
  expect(result).toBe("test=John Doe");
});
