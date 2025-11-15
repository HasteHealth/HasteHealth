# FHIRPath

## API

### evaluate

Returns a JS object/primitive based on the evaluation.

```typescript
evaluate(
  expression: string,
  ctx: unknown,
  options?: Options
): NonNullable<unknown>[]
```

### evaluateWithMeta

Returns a @haste-health/meta-value singular value which contains data along with metadata about the evaluation.

```typescript
function evaluateWithMeta(
  expression: string,
  ctx: unknown,
  options?: Options
): MetaValueSingular<NonNullable<unknown>>[];
```

### Arguments

| name               | type                                                             | description                                                                                |
| ------------------ | ---------------------------------------------------------------- | ------------------------------------------------------------------------------------------ |
| expression         | string                                                           | FHIRPath expression to evaluate.                                                           |
| ctx                | unknown                                                          | The context which the fhirpath is being evaluated from (sets $this context).               |
| options            | Object                                                           | Includes variables and metadata deriviation functions.                                     |
| options.variables  | Object \| Function                                               | If an object is Record\<variableName, variableValue\> else (variableName)=> variableValue. |
| options.meta       | Object                                                           | MetaInformation used to zip metadata with value. Used in type functions and operators.     |
| options.meta.type  | string                                                           | Root type for ctx.                                                                         |
| options.meta.getSD | (fhir_version: FHIRVersion, type: string) => StructureDefinition | Returns a StructureDefinition based on type passed in.                                     |

## Usage

```typescript
import * as fhirpath from "@haste-health/fhirpath";

// Default returns javascript object.
expect(fhirpath.evaluate("4 + 5", {})).toEqual([9]);
expect(fhirpath.evaluate("$this.test + 2 * 4", { test: 4 })).toEqual([12]);

// Evaluation with metavalue return (returns value in addition to meta information.)
expect(
  evaluateWithMeta(
    "$this.name",
    {
      resourceType: "Patient",
      name: [{ given: ["bob"], family: "jameson" }],
    },
    {
      meta: {
        type: "Patient",
        getSD: (fhirVersion, type: code) => {
          const foundSD = sds.find((sd) => sd.type === type);
          return foundSD;
        },
      },
    }
  ).map((v) => v.meta()?.type)
).toEqual(["HumanName"]);
```

## Operations supported

| Operation                 | Description                                                                                  | Supported |
| ------------------------- | -------------------------------------------------------------------------------------------- | --------- |
| + (addition)              | Concatenate strings or add values if numeric.                                                | true      |
| - (subtraction)           | Subtracts the right operand from the left operand.                                           | true      |
| as type specifier         | Returns left operand if value is of type specified.                                          | true      |
| is(type : type specifier) | Filters left operand based on type specifier.                                                | true      |
| \* (multiplication)       | Multiplies both arguments.                                                                   | true      |
| / (division)              | Divides the left operand by the right operand                                                | true      |
| \| (union collections)    | Merge the two collections into a single collection                                           | true      |
| = (Equals)                | Returns true if the left collection is equal to the right collection                         | true      |
| != (Not Equals)           | Converse operator of equals returns true if left collection is not equal to right collection | true      |

## Supported Functions

| Name       | Description                                                                                                   | Supported |
| ---------- | ------------------------------------------------------------------------------------------------------------- | --------- |
| empty      | Returns true if the input collection is empty ({ }) and false otherwise.                                      | true      |
| exists     | Returns true if the collection has any elements, and false otherwise.                                         | true      |
| all        | Returns true if for every element in the input collection, criteria evaluates to true.                        | true      |
| allTrue    | Takes a collection of Boolean values and returns true if all the items are true.                              | true      |
| anyTrue    | Takes a collection of Boolean values and returns true if any of the items are true.                           | true      |
| allFalse   | Takes a collection of Boolean values and returns true if all the items are false.                             | true      |
| anyFalse   | Takes a collection of Boolean values and returns true if any of the items are false.                          | true      |
| subsetOf   | Returns true if all items in the input collection are members of the collection passed in.                    | true      |
| supersetOf | Returns true if all items in the collection passed as the other argument are members of the input collection. | true      |
| count      | Returns the integer count of the number of items in the input collection.                                     | true      |
| distinct   | Returns a collection containing only the unique items in the input collection.                                | true      |
| isDistinct | Returns true if all the items in the input collection are distinct.                                           | true      |
| where      | Returns a collection containing only those elements in the input collection based on where expression.        | true      |
| select     | Evaluates the projection expression for each item in the input collection.                                    | true      |
| repeat     | A version of select that will repeat the projection and add it to the output collection.                      | true      |
| ofType     | Returns a collection that contains all items in the input collection that are of the given type.              | true      |
| as         | Returns filter same as ofType used for backwards compatibility.                                               | true      |
