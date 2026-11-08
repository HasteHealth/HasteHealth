# @haste-health/lang-fp

FHIRPath language support for [CodeMirror 6](https://codemirror.net/): a [Lezer](https://lezer.codemirror.net/) grammar that gives an editor syntax highlighting, bracket-based indentation and folding, and `//` line comments for FHIRPath expressions. [`@haste-health/components`](https://www.npmjs.com/package/@haste-health/components) uses it in its `FHIRExpressionEditable` editor for FHIR `Expression` elements.

## Install

```bash
npm install @haste-health/lang-fp codemirror
```

The package depends on `@codemirror/language` and the Lezer runtime itself; `codemirror` (or `@codemirror/view` and `@codemirror/state`) is what you build the editor with. Both ESM and CommonJS builds are published.

## Usage

```ts
import { EditorView, minimalSetup } from "codemirror";
import fhirpath from "@haste-health/lang-fp";

new EditorView({
  doc: "Patient.name.where(use = 'official').given.first()",
  extensions: [minimalSetup, fhirpath()],
  parent: document.body,
});
```

The default export returns a `LanguageSupport` ready to drop into `extensions`. The underlying `LRLanguage` is exported as `FPLanguage` for composing with your own extensions or completion sources.

Highlighted tokens: identifiers and function calls, `$this`, `%variables`, numbers, `true`/`false`, single-quoted strings, the operators `= != * / + -`, parentheses and line comments. The grammar is a lightweight token grammar for editing, not a full FHIRPath parser. Evaluate expressions with [`@haste-health/fhirpath`](https://www.npmjs.com/package/@haste-health/fhirpath).

## Links

- [API reference](https://typescript.haste.health/modules/_haste-health_lang-fp.html)
- [Source](https://github.com/HasteHealth/HasteHealth/tree/main/frontend/packages/lang-fp-codemirror)
- [CodeMirror language packages](https://codemirror.net/examples/lang-package/), the template this package follows
- [FHIRPath specification](https://hl7.org/fhirpath/)

Part of [Haste Health](https://haste.health). This package is published under the MIT license, as its `package.json` states.
