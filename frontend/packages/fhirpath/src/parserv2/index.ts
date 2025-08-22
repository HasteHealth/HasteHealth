import { EmbeddedActionsParser, ParserMethod, TokenType } from "chevrotain";

import FPLexer, {
  ADDITIVE_OPERATION,
  AND_OPERATION,
  BOOLEAN,
  COMMA,
  DATE,
  DATETIME,
  DATETIME_PRECISION,
  DELIMINATED_IDENTIFIER,
  DOT,
  EQUALITY_OPERATION,
  IDENTIFIER,
  IMPLIES_OPERATION,
  INDEX,
  LEFT_BRACKET,
  LEFT_PARAN,
  MEMBERSHIP_OPERATION,
  MULTIPLICATIVE_OPERATION,
  NULL,
  NUMBER,
  OR_OPERATION,
  PERCENTAGE,
  PLURAL_DATETIME_PRECISION,
  RIGHT_BRACKET,
  RIGHT_PARAN,
  STRING,
  THIS,
  TIME,
  TOTAL,
  TYPE_OPERATION,
  UNION_OPERATION,
  allTokens,
} from "./lexer.js";
import {
  ExpressionAST,
  FunctionAST,
  IdentifierAST,
  IndexedAST,
  InvocationAST,
  LiteralAST,
  OperationAST,
  SingularExpression,
  TermAST,
  VariableAST,
} from "./types.js";

class FPParser extends EmbeddedActionsParser {
  constructor() {
    super(allTokens);
    this.performSelfAnalysis();
  }
  public expression = this.RULE("expression", (): ExpressionAST => {
    const output = this.SUBRULE(this.impliesOperation);
    return output;
  });

  private _expression = this.RULE("_expression", (): SingularExpression => {
    const term = this.SUBRULE(this.term);
    const children = this.SUBRULE(this.expressionInner) ?? [];
    // Grammar recording requires this check.
    // @ts-ignore
    if (Array.isArray(children)) children.unshift(term);

    return {
      type: "expression",
      expression: children as SingularExpression["expression"],
    };
  });

  private expressionInner = this.RULE(
    "expressionInner",
    (): Array<InvocationAST | IndexedAST> => {
      const properties: Array<InvocationAST | IndexedAST> = [];
      this.MANY(() =>
        this.OR([
          {
            // DOT OPERATION
            ALT: () => {
              const dotRes = this.SUBRULE(this.dotAccess);
              properties.push(dotRes);
            },
          },
          {
            ALT: () => {
              const indexForm = this.SUBRULE(this.indexAccess);
              properties.push(indexForm);
            },
          },
        ]),
      );

      return properties;
    },
  );

  private dotAccess = this.RULE("invocationExpression", (): InvocationAST => {
    this.CONSUME(DOT);
    const invocation = this.SUBRULE(this.invocation);
    return invocation;
  });

  private indexAccess = this.RULE("indexExpression", (): IndexedAST => {
    this.CONSUME(LEFT_BRACKET);
    const index = this.SUBRULE(this.expression);
    this.CONSUME(RIGHT_BRACKET);

    return { type: "indexed", index };
  });

  private operation<T extends ExpressionAST>(
    operatorToken: TokenType,
    nested: ParserMethod<[], T>,
  ): ExpressionAST {
    let operationAST: ExpressionAST = this.SUBRULE1(nested);

    this.MANY(() => {
      const operator = this.CONSUME2(operatorToken);
      const rightSide = this.SUBRULE2(nested);

      operationAST = {
        type: "operation",
        operator: operator.image as OperationAST["operator"],
        left: operationAST,
        right: rightSide,
      };
    });

    return operationAST;
  }

  private impliesOperation = this.RULE(
    "impliesOperation",
    (): ExpressionAST => {
      return this.operation(IMPLIES_OPERATION, this.orOperation);
    },
  );

  private orOperation = this.RULE("orOperation", (): ExpressionAST => {
    return this.operation(OR_OPERATION, this.andOperation);
  });

  private andOperation = this.RULE("andOperation", (): ExpressionAST => {
    return this.operation(AND_OPERATION, this.membershipOperation);
  });

  private membershipOperation = this.RULE(
    "membershipOperation",
    (): ExpressionAST => {
      return this.operation(MEMBERSHIP_OPERATION, this.equalityOperation);
    },
  );

  private equalityOperation = this.RULE(
    "equalityOperation",
    (): ExpressionAST => {
      return this.operation(EQUALITY_OPERATION, this.unionOperation);
    },
  );

  private unionOperation = this.RULE("unionOperation", (): ExpressionAST => {
    return this.operation(UNION_OPERATION, this.typeOperation);
  });

  private typeOperation = this.RULE("typeOperation", (): ExpressionAST => {
    return this.operation(TYPE_OPERATION, this.additiveOperation);
  });

  private additiveOperation = this.RULE(
    "additiveOperation",
    (): ExpressionAST => {
      return this.operation(ADDITIVE_OPERATION, this.multiplicativeOperation);
    },
  );

  private multiplicativeOperation = this.RULE(
    "multiplicativeOperation",
    (): ExpressionAST => {
      return this.operation(MULTIPLICATIVE_OPERATION, this._expression);
    },
  );

  private term = this.RULE("term", (): TermAST => {
    return this.OR([
      { ALT: (): TermAST => this.SUBRULE(this.literal) },
      { ALT: (): TermAST => this.SUBRULE(this.invocation) },
      {
        ALT: (): TermAST => this.SUBRULE(this.externalConstant),
      },
      {
        ALT: (): TermAST => {
          this.CONSUME(LEFT_PARAN);
          const expression = this.SUBRULE(this.expression);
          this.CONSUME(RIGHT_PARAN);

          return expression;
        },
      },
    ]);
  });

  private literal = this.RULE("literal", (): LiteralAST => {
    return this.OR([
      {
        ALT: (): LiteralAST => {
          this.CONSUME(NULL);
          return {
            type: "literal",
            literalType: "null",
            value: null,
          };
        },
      },
      {
        ALT: (): LiteralAST => {
          const boolToken = this.CONSUME(BOOLEAN);
          return {
            type: "literal",
            literalType: "boolean",
            value: boolToken.image === "true",
          };
        },
      },
      {
        ALT: (): LiteralAST => {
          const string = this.CONSUME(STRING);
          return {
            type: "literal",
            literalType: "string",
            // Remove the parans
            value: string.image.slice(1, -1),
          };
        },
      },
      {
        ALT: (): LiteralAST => {
          const numberToken = this.CONSUME(NUMBER);
          return {
            type: "literal",
            literalType: "number",
            value: parseFloat(numberToken.image),
          };
        },
      },
      {
        ALT: (): LiteralAST => {
          const datetime = this.CONSUME(DATETIME);
          return {
            type: "literal",
            literalType: "datetime",
            // Remove the @
            value: datetime.image.slice(1),
          };
        },
      },
      {
        ALT: (): LiteralAST => {
          const date = this.CONSUME(DATE);
          return {
            type: "literal",
            literalType: "date",
            // Remove the @
            value: date.image.slice(1),
          };
        },
      },
      {
        ALT: (): LiteralAST => {
          const time = this.CONSUME(TIME);
          return {
            type: "literal",
            literalType: "time",
            // Remove the @T
            value: time.image.slice(2),
          };
        },
      },
      // TODO Quantity
    ]);
  });

  private identifier = this.RULE("identifier", (): IdentifierAST => {
    return this.OR([
      {
        ALT: (): IdentifierAST => {
          const identifier = this.CONSUME(IDENTIFIER);
          return { type: "identifier", value: identifier.image };
        },
      },
      {
        ALT: (): IdentifierAST => {
          const deliminatedIdentifier = this.CONSUME(DELIMINATED_IDENTIFIER);
          return {
            type: "identifier",
            value: deliminatedIdentifier.image.slice(1, -1),
          };
        },
      },
      // Handle keywords being used as identifier.
      {
        ALT: (): IdentifierAST => {
          const impliesOperator = this.CONSUME2(IMPLIES_OPERATION);
          return {
            type: "identifier",
            value: impliesOperator.image,
          };
        },
      },
      {
        ALT: (): IdentifierAST => {
          const orOperator = this.CONSUME2(OR_OPERATION);
          return {
            type: "identifier",
            value: orOperator.image,
          };
        },
      },
      {
        ALT: (): IdentifierAST => {
          const andOperator = this.CONSUME2(AND_OPERATION);
          return {
            type: "identifier",
            value: andOperator.image,
          };
        },
      },
      {
        ALT: (): IdentifierAST => {
          const memOperator = this.CONSUME2(MEMBERSHIP_OPERATION);
          return {
            type: "identifier",
            value: memOperator.image,
          };
        },
      },
      {
        ALT: (): IdentifierAST => {
          const typeOperator = this.CONSUME2(TYPE_OPERATION);
          return {
            type: "identifier",
            value: typeOperator.image,
          };
        },
      },
      {
        ALT: (): IdentifierAST => {
          const multOperator = this.CONSUME2(MULTIPLICATIVE_OPERATION);

          return {
            type: "identifier",
            value: multOperator.image,
          };
        },
      },
      {
        ALT: (): IdentifierAST => {
          const multOperator = this.CONSUME2(DATETIME_PRECISION);

          return {
            type: "identifier",
            value: multOperator.image,
          };
        },
      },
      {
        ALT: (): IdentifierAST => {
          const multOperator = this.CONSUME2(PLURAL_DATETIME_PRECISION);

          return {
            type: "identifier",
            value: multOperator.image,
          };
        },
      },
    ]);
  });

  private invocation = this.RULE("invocation", (): InvocationAST => {
    return this.OR([
      { ALT: (): InvocationAST => this.SUBRULE(this._function) },
      { ALT: (): InvocationAST => this.SUBRULE(this.identifier) },
      {
        ALT: (): InvocationAST => {
          this.CONSUME(THIS);
          return { type: "this" };
        },
      },
      {
        ALT: (): InvocationAST => {
          this.CONSUME(INDEX);
          return { type: "index" };
        },
      },
      {
        ALT: (): InvocationAST => {
          this.CONSUME(TOTAL);
          return { type: "total" };
        },
      },
    ]);
  });

  private externalConstant = this.RULE("externalConstant", (): VariableAST => {
    this.CONSUME(PERCENTAGE);
    return this.OR([
      {
        ALT: (): VariableAST => {
          const string = this.CONSUME(STRING);
          return {
            type: "variable",
            value: string.image.slice(1, -1),
          };
        },
      },
      {
        ALT: (): VariableAST => {
          const identifier = this.CONSUME(IDENTIFIER);
          return { type: "variable", value: identifier.image };
        },
      },
    ]);
  });

  private _function = this.RULE("function", (): FunctionAST => {
    const functionName = this.SUBRULE(this.identifier);
    this.CONSUME(LEFT_PARAN);
    const parameters: ExpressionAST[] = [];
    this.MANY_SEP({
      SEP: COMMA,
      DEF: () => {
        const expression = this.SUBRULE(this.expression);
        parameters.push(expression);
      },
    });
    this.CONSUME(RIGHT_PARAN);

    return {
      type: "function",
      functionName: functionName.value,
      parameters,
    };
  });
}

// reuse the same parser instance.
const parser = new FPParser();
export default function parse(expression: string): ExpressionAST {
  const lexResult = FPLexer.tokenize(expression);
  // setting a new input will RESET the parser instance's state.
  parser.input = lexResult.tokens;

  // any top level rule may be used as an entry point
  const ast = parser.expression();

  if (parser.errors.length > 0) {
    console.log(JSON.stringify(lexResult, undefined, 2));
    console.error(ast, parser.errors);
    throw new Error(`Failed to parse expression '${expression}'`);
  }

  return ast;
}
