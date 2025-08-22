import {
  LRLanguage,
  LanguageSupport,
  delimitedIndent,
  foldInside,
  foldNodeProp,
  indentNodeProp,
} from "@codemirror/language";
import { styleTags, tags as t } from "@lezer/highlight";

import { parser } from "./syntax.grammar.js";

export const FPLanguage = LRLanguage.define({
  parser: parser.configure({
    props: [
      indentNodeProp.add({
        Application: delimitedIndent({ closing: ")", align: false }),
      }),
      foldNodeProp.add({
        Application: foldInside,
      }),
      styleTags({
        Operators: t.arithmeticOperator,
        FunctionCall: t.function(t.propertyName),
        This: t.self,
        Variable: t.definition(t.variableName),
        Number: t.number,
        Identifier: t.name,
        Boolean: t.bool,
        String: t.string,
        LineComment: t.lineComment,
        "( )": t.paren,
      }),
    ],
  }),
  languageData: {
    commentTokens: { line: "//" },
  },
});

export default function () {
  return new LanguageSupport(FPLanguage);
}
