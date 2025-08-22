import { uri } from "@oxidized-health/fhir-types/r4/types";
import { FHIR_VERSION, R4 } from "@oxidized-health/fhir-types/versions";
import { IMetaValue } from "@oxidized-health/meta-value/interface";
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
  variables?: Record<string, unknown> | ((v: string) => Promise<unknown>);
};

function flatten<T>(arr: T[][]): T[] {
  return arr.reduce((acc, v) => [...acc, ...v], []);
}

async function getVariableValue(
  variableId: string,
  options: Options
): Promise<IMetaValue<unknown>[]> {
  let value;
  if (options?.variables instanceof Function) {
    value = await options.variables(variableId);
  } else if (options?.variables instanceof Object) {
    value = options.variables[variableId];
  }

  return metaUtils.flatten(
    metaValueV2.metaValue({ fhirVersion: options.fhirVersion }, value)
  );
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
        await metaValueV2.metaValue(
          { fhirVersion: options.fhirVersion, type: "boolean" as uri },
          (await _evaluate(ast.parameters[0], context, options)).length > 0
        )
      );
    }
    return metaUtils.flatten(
      await metaValueV2.metaValue(
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
  async all(ast, context, options) {
    return metaUtils.flatten(
      await metaValueV2.metaValue(
        { fhirVersion: options.fhirVersion, type: "boolean" as uri },
        flatten(
          await Promise.all(
            context.map((v) => _evaluate(ast.parameters[0], [v], options))
          )
        )
          .map((v) => v.getValue())
          .reduce((acc, v) => acc && v, true)
      )
    );
  },
  async allTrue(ast, context, options) {
    return metaUtils.flatten(
      await metaValueV2.metaValue(
        { fhirVersion: options.fhirVersion, type: "boolean" as uri },
        context.reduce((acc, v) => v.getValue() === true && acc, true)
      )
    );
  },
  async anyTrue(ast, context, options) {
    return metaUtils.flatten(
      await metaValueV2.metaValue(
        { fhirVersion: options.fhirVersion, type: "boolean" as uri },
        context.reduce((acc, v) => v.getValue() === true || acc, false)
      )
    );
  },
  async allFalse(ast, context, options) {
    return metaUtils.flatten(
      await metaValueV2.metaValue(
        { fhirVersion: options.fhirVersion, type: "boolean" as uri },
        context.reduce((acc, v) => v.getValue() === false && acc, true)
      )
    );
  },
  async anyFalse(ast, context, options) {
    return metaUtils.flatten(
      await metaValueV2.metaValue(
        { fhirVersion: options.fhirVersion, type: "boolean" as uri },
        context.reduce((acc, v) => v.getValue() === false || acc, false)
      )
    );
  },
  async subsetOf(ast, context, options) {
    const otherSet = await _evaluate(ast.parameters[0], context, options);
    return metaUtils.flatten(
      await metaValueV2.metaValue(
        { fhirVersion: options.fhirVersion, type: "boolean" as uri },
        context.reduce(
          (acc, v1) =>
            acc &&
            otherSet.find((v2) => {
              return equalityCheck([v1], [v2]);
            }) !== undefined,
          true
        )
      )
    );
  },
  // Conceptionally this is the opposite of subsetOf.
  async supersetOf(ast, context, options) {
    const otherSet = await _evaluate(ast.parameters[0], context, options);
    return metaUtils.flatten(
      await metaValueV2.metaValue(
        { fhirVersion: options.fhirVersion, type: "boolean" as uri },
        otherSet.reduce(
          (acc, v1) =>
            acc &&
            context.find((v2) => {
              return equalityCheck([v1], [v2]);
            }) !== undefined,
          true
        )
      )
    );
  },
  async count(_ast, context, options) {
    return metaUtils.flatten(
      await metaValueV2.metaValue(
        { fhirVersion: options.fhirVersion, type: "integer" as uri },
        context.length
      )
    );
  },
  async distinct(_ast, context) {
    const map = context
      .map((v: IMetaValue<unknown>): [string, IMetaValue<unknown>] => [
        JSON.stringify(v.getValue()),
        v,
      ])
      .reduce(
        (
          m: { [key: string]: IMetaValue<unknown> },
          [k, v]: [string, IMetaValue<unknown>]
        ) => {
          m[k] = v;
          return m;
        },
        {}
      );
    return Object.values(map);
  },
  async isDistinct(ast, context, options) {
    const distinct = await fp_functions.distinct(
      { type: "function", functionName: "distinct", parameters: [] },
      context,
      options
    );
    return metaUtils.flatten(
      await metaValueV2.metaValue(
        { fhirVersion: options.fhirVersion, type: "boolean" as uri },
        context.length === distinct.length
      )
    );
  },
  // [FILTER FUNCTIONS]
  async where(ast, context, options) {
    const criteria = ast.parameters[0];
    const result: IMetaValue<unknown>[] = [];
    for (const v of context) {
      const evaluation = await _evaluate(criteria, [v], options);
      assert(
        evaluation.length === 1,
        "where clause evaluation must evaluate to a single boolean."
      );
      if (!typeChecking("boolean", evaluation)) {
        throw new Error("Where clause criteria must evaluate to a boolean");
      }
      if (evaluation[0].getValue()) {
        result.push(v);
      }
    }
    return result;
  },
  /**
   * Returns a collection with all immediate child nodes of all items in the input collection.
   * Note that the ordering of the children is undefined and using functions like first() on the result may return different results on different platforms.
   */
  async children(_ast, context) {
    const children = flatten(
      await Promise.all(
        context.map(async (node) => {
          const children = flatten(
            await Promise.all(
              node.keys().map(async (k) => metaUtils.flatten(node.descend(k)))
            )
          ).filter((v) => v !== undefined);

          return children;
        })
      )
    );

    return children;
  },
  /**
   * Returns a collection with all descendant nodes of all items in the input collection.
   */
  async descendants(_ast, context, options) {
    return _evaluate(compileAST("repeat(children())"), context, options);
  },
  async select(ast, context, options) {
    const selection = ast.parameters[0];
    return flatten(
      await Promise.all(
        context.map((v) => {
          return _evaluate(selection, [v], options);
        })
      )
    );
  },
  async repeat(ast, context, options) {
    const projection = ast.parameters[0];
    let endResult: IMetaValue<unknown>[] = [];
    let cur = context;
    while (cur.length !== 0) {
      cur = await _evaluate(projection, cur, options);
      endResult = [...endResult, ...cur];
    }
    return endResult;
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
      return getVariableValue(ast.value, options);
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

/**
 * Check for membership of value in collection.
 * Used for both in and contains (reversed).
 * @param options FP Options
 * @param collection Collection to check for membership
 * @param value Value to check if exists in collection
 * @returns MetaValue boolean value.
 */
function membershipCheck(
  options: Options,
  collection: IMetaValue<unknown>[],
  value: IMetaValue<unknown>[]
): IMetaValue<boolean>[] {
  if (value.length !== 1) {
    throw new Error("Membership check value must be a single value");
  }

  for (const colValue of collection) {
    if (equalityCheck(value, [colValue])) {
      return metaUtils.flatten(
        metaValueV2.metaValue(
          { fhirVersion: options.fhirVersion, type: "boolean" as uri },
          true
        )
      );
    }
  }
  return metaUtils.flatten(
    metaValueV2.metaValue(
      { fhirVersion: options.fhirVersion, type: "boolean" as uri },
      false
    )
  );
}

const fp_operations: Record<
  string,
  (
    ast: OperationAST,
    context: IMetaValue<unknown>[],
    options: Options
  ) => Promise<IMetaValue<unknown>[]>
> = {
  contains: op_prevaled(async (left, right, options) => {
    return membershipCheck(options, left, right);
  }),
  in: op_prevaled(async (left, right, options) => {
    return membershipCheck(options, right, left);
  }),
  "+": op_prevaled(async (left, right, options) => {
    if (typeChecking("number", left) && typeChecking("number", right)) {
      return metaUtils.flatten(
        await metaValueV2.metaValue(
          { fhirVersion: options.fhirVersion, type: "decimal" as uri },
          left[0].getValue() + right[0].getValue()
        )
      );
    } else if (typeChecking("string", left) && typeChecking("string", right)) {
      return metaUtils.flatten(
        await metaValueV2.metaValue(
          { fhirVersion: options.fhirVersion, type: "string" as uri },
          left[0].getValue() + right[0].getValue()
        )
      );
    } else {
      throw new InvalidOperandError([left[0], right[0]], "+");
    }
  }),
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
  "-": op_prevaled(async (left, right, options) => {
    if (typeChecking("number", left) && typeChecking("number", right)) {
      return metaUtils.flatten(
        await metaValueV2.metaValue(
          { fhirVersion: options.fhirVersion, type: "decimal" as uri },
          left[0].getValue() - right[0].getValue()
        )
      );
    } else {
      throw new InvalidOperandError([left[0], right[0]], "-");
    }
  }),
  "*": op_prevaled(async (left, right, options) => {
    if (typeChecking("number", left) && typeChecking("number", right)) {
      return metaUtils.flatten(
        await metaValueV2.metaValue(
          { fhirVersion: options.fhirVersion, type: "decimal" as uri },
          left[0].getValue() * right[0].getValue()
        )
      );
    } else {
      throw new InvalidOperandError([left[0], right[0]], "*");
    }
  }),
  "/": op_prevaled(async (left, right, options) => {
    if (typeChecking("number", left) && typeChecking("number", right)) {
      return metaUtils.flatten(
        await metaValueV2.metaValue(
          { fhirVersion: options.fhirVersion, type: "decimal" as uri },
          left[0].getValue() / right[0].getValue()
        )
      );
    } else {
      throw new InvalidOperandError([left[0], right[0]], "/");
    }
  }),
  "|": op_prevaled(async (left, right) => {
    return left.concat(right);
  }),
  and: op_prevaled(async (left, right, options) => {
    if (typeChecking("boolean", left) && typeChecking("boolean", right)) {
      return metaUtils.flatten(
        await metaValueV2.metaValue(
          { fhirVersion: options.fhirVersion, type: "boolean" as uri },
          left[0].getValue() && right[0].getValue()
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
          left[0].getValue() || right[0].getValue()
        )
      );
    }
    throw new InvalidOperandError([left[0], right[0]], "/");
  }),
  "=": op_prevaled(async (left, right, options) =>
    metaUtils.flatten(
      await metaValueV2.metaValue(
        { fhirVersion: options.fhirVersion, type: "boolean" as uri },
        equalityCheck(left, right)
      )
    )
  ),
  "!=": op_prevaled(async (left, right, options) => {
    const equality = equalityCheck(left, right);
    return metaUtils.flatten(
      await metaValueV2.metaValue(
        { fhirVersion: options.fhirVersion, type: "boolean" as uri },
        equality === false
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

/**
 * Because Primitive types could have undefined on .value do quick check here to confirm existence.
 */
function nonNullable(v: unknown): v is NonNullable<unknown> {
  return v !== undefined && v !== null;
}

const cachedAST: Record<string, ExpressionAST> = {};

function compileAST(expression: string): ExpressionAST {
  if (!cachedAST[expression]) {
    const ast = parse(expression);
    cachedAST[expression] = ast;
  }

  return cachedAST[expression];
}

export async function evaluateWithMeta(
  expression: string,
  ctx: unknown,
  options: Options & { type?: uri } = { fhirVersion: R4 }
): Promise<IMetaValue<NonNullable<unknown>>[]> {
  const ast = compileAST(expression);
  const context = metaUtils.flatten(
    metaValueV2.metaValue(
      { fhirVersion: options.fhirVersion, type: options.type },
      ctx
    )
  );

  return (await _evaluate(ast, context, options)).filter(
    (v: IMetaValue<unknown>): v is IMetaValue<NonNullable<unknown>> =>
      nonNullable(v.getValue())
  );
}

export async function evaluate(
  expression: string,
  ctx: unknown,
  options?: Options & { type?: uri }
): Promise<NonNullable<unknown>[]> {
  return (await evaluateWithMeta(expression, ctx, options)).map((v) =>
    v.getValue()
  );
}
