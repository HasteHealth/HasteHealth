import { uri } from "@oxidized-health/fhir-types/r4/types";
import { FHIR_VERSION } from "@oxidized-health/fhir-types/versions";
import { IMetaValue } from "@oxidized-health/meta-value/interface";
import spoof, { SpoofMetaValueV2 } from "@oxidized-health/meta-value/spoof";
import * as metaUtils from "@oxidized-health/meta-value/utilities";
import * as metaValueV2 from "@oxidized-health/meta-value/v2";

import parse from "./parserv2/index.js";
import {
  ExpressionAST,
  FunctionAST,
  IndexedAST,
  InvocationAST,
  OperationAST,
  SingularExpression,
  TermAST,
} from "./parserv2/types.js";

export type Options = {
  fhirVersion?: FHIR_VERSION;
};

function flatten<T>(arr: T[][]): T[] {
  return arr.reduce((acc, v) => [...acc, ...v], []);
}

function assert(assertion: boolean, message?: string) {
  if (!assertion) {
    throw new Error(message || "Assertion failed");
  }
}

// Problem is you can't distinguish an identifier vs typeidentifier so just check to confirm singular and only identifier present.
function expressionToTypeIdentifier(ast: ExpressionAST): string | undefined {
  switch (ast.type) {
    case "expression": {
      if (ast.expression[0].type === "identifier") {
        return ast.expression[0].value;
      }
      return undefined;
    }
    default: {
      return undefined;
    }
  }
}

const fp_functions: Record<
  string,
  (
    ast: FunctionAST,
    context: IMetaValue<unknown>[],
    options: Options
  ) => Promise<IMetaValue<unknown>[]>
> = {
  // [EXISTENCE FUNCTIONS]
  // Returns true if the input collection is empty ({ }) and false otherwise.
  async exists(ast, context, options) {
    if (ast.parameters.length === 1) {
      return metaUtils.flatten(
        metaValueV2.metaValue(
          { fhirVersion: options.fhirVersion, type: "boolean" as uri },
          (await _evaluate(ast.parameters[0], context, options)).length > 0
        )
      );
    }
    return metaUtils.flatten(
      metaValueV2.metaValue(
        { fhirVersion: options.fhirVersion, type: "boolean" as uri },
        context.length > 0
      )
    );
  },
  // exists([criteria : expression]) : Boolean
  async empty(ast, context, options) {
    return metaUtils.flatten(
      await metaValueV2.metaValue(
        { fhirVersion: options.fhirVersion, type: "boolean" as uri },
        context.length === 0
      )
    );
  },
  // [FILTER FUNCTIONS]
  async where(ast, context) {
    return context;
  },
  // Type Functions
  async ofType(ast, context) {
    const parameters = ast.parameters;
    const typeIdentifier = expressionToTypeIdentifier(parameters[0]);
    if (!typeIdentifier) throw new Error("Invalid Type identifier");
    return filterByType(typeIdentifier, context);
  },
  async as(ast, context) {
    assert(
      context.length <= 1,
      "as function must have a length of 1 for context."
    );
    const parameters = ast.parameters;
    const typeIdentifier = expressionToTypeIdentifier(parameters[0]);
    if (!typeIdentifier) throw new Error("Invalid Type identifier");
    return filterByType(typeIdentifier, context);
  },
  async resolve(ast, context) {
    // console.warn("not supporting resolve just returning item");
    return context;
  },

  async replace(ast, context, options) {
    if (ast.parameters.length !== 2) {
      throw new Error("Replace function must have two arguments");
    }
    if (context.length !== 1) {
      throw new Error("Replace function must have one context");
    }
    if (!typeChecking("string", context)) {
      throw new Error("Replace function arguments must be strings");
    }
    const value = context[0].getValue();

    const [findAST, replaceAST] = ast.parameters;
    const findEval = await _evaluate(findAST, context, options);
    const replaceEval = await _evaluate(replaceAST, context, options);
    if (
      findEval.length !== 1 ||
      replaceEval.length !== 1 ||
      !typeChecking("string", findEval) ||
      !typeChecking("string", replaceEval)
    ) {
      throw new Error("Replace function arguments must be strings");
    }
    const find = findEval[0].getValue();
    const replace = replaceEval[0].getValue();

    return metaUtils.flatten(
      await metaValueV2.metaValue(
        context[0].meta(),
        value.replace(find, replace)
      )
    );
  },

  async type(ast, context, options) {
    return flatten(
      context.map((c) => {
        const meta = c.meta();
        return metaUtils.flatten(
          metaValueV2.metaValue(
            { fhirVersion: options.fhirVersion },
            //  { name: string, type: TypeSpecifier, isOneBased: Boolean }
            {
              name: c.location()?.pop(),
              type: meta?.type,
            }
          )
        );
      })
    );
  },
};

async function evaluateInvocation(
  ast: InvocationAST,
  context: IMetaValue<unknown>[],
  options: Options
): Promise<IMetaValue<unknown>[]> {
  switch (ast.type) {
    case "index":
      throw new Error("Not implemented");
    case "total":
      throw new Error("Not implemented");
    case "this":
      return context;
    case "identifier": {
      const res = (
        await Promise.all(
          context.map(async (v) => metaUtils.flatten(v.descend(ast.value)))
        )
      ).flat();

      return res;
    }
    case "function": {
      const fp_func = fp_functions[ast.functionName];
      if (!fp_func)
        throw new Error("Unknown function '" + ast.functionName + "'");
      return fp_func(ast, context, options);
    }
    default: {
      // @ts-ignore
      throw new Error("Unknown invocation type: '" + ast.type + "'");
    }
  }
}

async function evaluateTerm(
  ast: TermAST,
  context: IMetaValue<unknown>[],
  options: Options
): Promise<IMetaValue<unknown>[]> {
  switch (ast.type) {
    case "function": {
      const fp_func = fp_functions[ast.functionName];
      if (!fp_func)
        throw new Error("Unknown function '" + ast.functionName + "'");
      return fp_func(ast, context, options);
    }
    case "expression":
    case "operation": {
      return _evaluate(ast, context, options);
    }

    case "literal": {
      return metaUtils.flatten(
        await metaValueV2.metaValue(
          { fhirVersion: options.fhirVersion },
          ast.value
        )
      );
    }
    case "variable": {
      throw new Error("Not implemented");
    }
    case "identifier": {
      // Special handling for type identifier at start for example Patient.name
      const typeIdentifier = ast.value;
      const result = filterByType(typeIdentifier, context);
      if (result.length > 0) return result;
      else {
        return evaluateInvocation(ast, context, options);
      }
    }
    case "index":
    case "this":
    case "total": {
      return evaluateInvocation(ast, context, options);
    }
    default: {
      // @ts-ignore
      throw new Error("Unknown term type: '" + ast.type + "'");
    }
  }
}

async function evaluateProperty(
  ast: InvocationAST | IndexedAST,
  context: IMetaValue<unknown>[],
  options: Options
): Promise<IMetaValue<unknown>[]> {
  switch (ast.type) {
    case "identifier":
    case "this":
    case "total":
    case "function":
    case "index": {
      return evaluateInvocation(ast, context, options);
    }
    case "indexed": {
      const indexed = await _evaluate(ast.index, context, options);
      if (indexed.length !== 1)
        throw new Error("Indexing requires a single value");
      if (!typeChecking("number", indexed))
        throw new Error("Indexing requires a number");
      const value = context[indexed[0].getValue()];
      return value !== undefined ? [value] : [];
    }
    default: {
      // @ts-ignore
      throw new Error("Unknown term type: '" + ast.type + "'");
    }
  }
}

async function evaluateSingular(
  ast: SingularExpression,
  context: IMetaValue<unknown>[],
  options: Options
): Promise<IMetaValue<unknown>[]> {
  let start = await evaluateTerm(ast.expression[0], context, options);

  for (const next of ast.expression.slice(1)) {
    start = await evaluateProperty(
      next as InvocationAST | IndexedAST,
      start,
      options
    );
  }

  return start;
}

type OperatorType<op_type> = op_type extends "number"
  ? number
  : op_type extends "string"
  ? string
  : op_type extends "boolean"
  ? boolean
  : unknown;

type ValidOperandType = "number" | "string" | "boolean";

function typeChecking<T extends ValidOperandType>(
  typeChecking: T,
  args: IMetaValue<unknown>[]
): args is IMetaValue<OperatorType<T>>[] {
  return args.reduce((acc: boolean, v: IMetaValue<unknown>) => {
    if (typeof v.getValue() !== typeChecking) return false;
    return acc;
  }, true) as boolean;
}

export class InvalidOperandError extends Error {
  private args: unknown[];
  private operator: string;
  constructor(args: unknown[], operator: string) {
    super(
      `Invalid operands for operator: '${operator}' Found types '${typeof args[0]}' and '${typeof args[1]}'`
    );
    this.args = args;
    this.operator = operator;
  }
}

type EvaledOperation = (
  left: IMetaValue<unknown>[],
  right: IMetaValue<unknown>[],
  options: Options
) => Promise<IMetaValue<unknown>[]>;

function op_prevaled(
  operation_function: EvaledOperation
): (
  ast: OperationAST,
  context: IMetaValue<unknown>[],
  options: Options
) => Promise<IMetaValue<unknown>[]> {
  return async (ast, context, options) => {
    const left = await _evaluate(ast.left, context, options);
    const right = await _evaluate(ast.right, context, options);

    return operation_function(left, right, options);
  };
}

const equalityCheck = (
  left: IMetaValue<unknown>[],
  right: IMetaValue<unknown>[]
): boolean => {
  // TODO improve Deep Equals speed.
  if (left.length !== right.length) {
    return false;
  } else if (typeof left[0].getValue() === "object") {
    return (
      JSON.stringify(left[0].getValue()) === JSON.stringify(right[0].getValue())
    );
  }
  return left[0].getValue() === right[0].getValue();
};

function filterByType<T>(type: string, context: IMetaValue<T>[]) {
  return context.map((v) => v.asType(type)).filter((v) => v !== undefined);
}

const fp_operations: Record<
  string,
  (
    ast: OperationAST,
    context: IMetaValue<unknown>[],
    options: Options
  ) => Promise<IMetaValue<unknown>[]>
> = {
  as: async (ast, context, options) => {
    const left = await _evaluate(ast.left, context, options);
    if (left.length > 1)
      throw new Error(
        "The 'as' operator left hand operand must be equal to length 1"
      );
    const typeIdentifier = expressionToTypeIdentifier(ast.right);
    if (!typeIdentifier) throw new Error("Invalid Type identifier");
    return filterByType(typeIdentifier, left);
  },
  is: async (ast, context, options) => {
    const left = await _evaluate(ast.left, context, options);
    if (left.length > 1)
      throw new Error(
        "The 'is' operator left hand operand must be equal to length 1"
      );
    const typeIdentifier = expressionToTypeIdentifier(ast.right);
    if (!typeIdentifier) throw new Error("Invalid Type identifier");

    return (
      await Promise.all(
        left.map(async (c) => {
          return metaUtils.flatten(
            await metaValueV2.metaValue(
              { fhirVersion: options.fhirVersion, type: "boolean" as uri },
              c.isType(typeIdentifier)
            )
          );
        })
      )
    ).flat();
  },
  "|": op_prevaled(async (left, right) => {
    return left.concat(right);
  }),
  and: op_prevaled(async (left, right, options) => {
    if (typeChecking("boolean", left) && typeChecking("boolean", right)) {
      return metaUtils.flatten(
        await metaValueV2.metaValue(
          { fhirVersion: options.fhirVersion, type: "boolean" as uri },
          true
        )
      );
    }
    throw new InvalidOperandError([left[0], right[0]], "/");
  }),
  or: op_prevaled(async (left, right, options) => {
    if (typeChecking("boolean", left) && typeChecking("boolean", right)) {
      return metaUtils.flatten(
        await metaValueV2.metaValue(
          { fhirVersion: options.fhirVersion, type: "boolean" as uri },
          true
        )
      );
    }
    throw new InvalidOperandError([left[0], right[0]], "/");
  }),
  "=": op_prevaled(async (left, right, options) =>
    metaUtils.flatten(
      await metaValueV2.metaValue(
        { fhirVersion: options.fhirVersion, type: "boolean" as uri },
        true
      )
    )
  ),
  "!=": op_prevaled(async (left, right, options) => {
    return metaUtils.flatten(
      await metaValueV2.metaValue(
        { fhirVersion: options.fhirVersion, type: "boolean" as uri },
        true
      )
    );
  }),
};

async function evaluateOperation(
  ast: OperationAST,
  context: IMetaValue<unknown>[],
  options: Options
): Promise<IMetaValue<unknown>[]> {
  const operator = fp_operations[ast.operator];
  if (operator) return operator(ast, context, options);
  else throw new Error("Unsupported operator: '" + ast.operator + "'");
}

async function _evaluate(
  ast: ExpressionAST,
  context: IMetaValue<unknown>[],
  options: Options
): Promise<IMetaValue<unknown>[]> {
  switch (ast.type) {
    case "operation": {
      return evaluateOperation(ast, context, options);
    }
    case "expression": {
      return evaluateSingular(ast, context, options);
    }
    default: {
      // @ts-ignore
      throw new Error("Invalid AST Expression Node '" + ast.type + "'");
    }
  }
}

const cachedAST: Record<string, ExpressionAST> = {};

function compileAST(expression: string): ExpressionAST {
  if (!cachedAST[expression]) {
    const ast = parse(expression);
    cachedAST[expression] = ast;
  }

  return cachedAST[expression];
}

export default async function analyze(
  fhirVersion: FHIR_VERSION,
  type: uri,
  expression: string
): Promise<SpoofMetaValueV2<unknown>[]> {
  const ast = compileAST(expression);
  const value = spoof(fhirVersion, type);
  return _evaluate(ast, [value], {
    fhirVersion,
  }) as unknown as SpoofMetaValueV2<unknown>[];
}
