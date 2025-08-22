export type FunctionAST = {
  type: "function";
  functionName: string;
  parameters: ExpressionAST[];
};

export type VariableAST = { type: "variable"; value: string };

export type IdentifierAST = { type: "identifier"; value: string };

export type ThisAST = { type: "this" };
export type IndexAST = { type: "index" };
export type TotalAST = { type: "total" };

export type InvocationAST =
  | FunctionAST
  | IdentifierAST
  | ThisAST
  | IndexAST
  | TotalAST;

export type IndexedAST = {
  type: "indexed";
  index: ExpressionAST;
};

export type NullAST = { type: "literal"; literalType: "null"; value: null };
export type BooleanAST = {
  type: "literal";
  literalType: "boolean";
  value: boolean;
};
export type NumberAST = {
  type: "literal";
  literalType: "number";
  value: number;
};
export type StringAST = {
  type: "literal";
  literalType: "string";
  value: string;
};
export type DateAST = { type: "literal"; literalType: "date"; value: string };
export type DateTimeAST = {
  type: "literal";
  literalType: "datetime";
  value: string;
};
export type TimeAST = { type: "literal"; literalType: "time"; value: string };

export type LiteralAST =
  | NullAST
  | BooleanAST
  | NumberAST
  | StringAST
  | DateAST
  | DateTimeAST
  | TimeAST;

export type TermAST = ExpressionAST | InvocationAST | LiteralAST | VariableAST;

export type OperationAST = {
  type: "operation";
  operator:
    | "*"
    | "/"
    | "div"
    | "mod"
    | "+"
    | "-"
    | "&"
    | "is"
    | "as"
    | "|"
    | "<="
    | "<"
    | ">"
    | ">="
    | "="
    | "~"
    | "!="
    | "!~"
    | "in"
    | "contains"
    | "and"
    | "or"
    | "xor"
    | "implies";
  left: ExpressionAST;
  right: ExpressionAST;
};

export type SingularExpression = {
  type: "expression";
  expression: [TermAST, ...Array<InvocationAST | IndexedAST>];
};

export type ExpressionAST = SingularExpression | OperationAST;
