import * as fhirpath from "@oxidized-health/fhirpath";

const EXPRESSION_REGEX = /{{([^}]*)}}/;

export default async function execute(
  xfhirQuery: string,
  fpOptions: Parameters<typeof fhirpath.evaluate>[2]
): Promise<string> {
  let output = xfhirQuery;
  while (EXPRESSION_REGEX.test(output)) {
    const result = EXPRESSION_REGEX.exec(output);
    const expression = result?.[1];
    const match = result?.[0];
    if (!match || !expression) {
      throw new Error("Invalid expression");
    }

    const evaluation = (
      await fhirpath.evaluate(expression, undefined, fpOptions)
    )
      .map((e) => e.toString())
      .join(",");
    output = output.replace(match, evaluation);
  }

  return output;
}
