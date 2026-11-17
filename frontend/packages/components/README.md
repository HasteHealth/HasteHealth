# @haste-health/components

React component library for building healthcare apps on Haste Health: base UI (buttons, inputs, tables, modals, tabs, code editors), application layout, and FHIR-aware components that render forms, search tables and operation panels straight from FHIR StructureDefinitions and OperationDefinitions. Styled with Tailwind CSS and Headless UI. The Haste Health admin app is built from it, and every component is documented live in [Storybook](https://storybook.haste.health).

## Install

```bash
npm install @haste-health/components react react-dom
```

Peer dependencies: React 19 (`react`, `react-dom`) and, for the editor components, CodeMirror 6 (`@codemirror/state`, `@codemirror/view`, `@codemirror/autocomplete`, `@codemirror/theme-one-dark`). Import the stylesheet once in your app:

```ts
import "@haste-health/components/dist/index.css";
```

## Usage

```tsx
import { useState } from "react";
import createHTTPClient from "@haste-health/client/http";
import { Button, FHIRGenerativeForm } from "@haste-health/components";
import type {
  Patient,
  StructureDefinition,
} from "@haste-health/fhir-types/r4/types";
import { R4 } from "@haste-health/fhir-types/versions";

const client = createHTTPClient({
  url: "http://localhost:3000/w/my-tenant/my-project/api/v1/fhir",
  getAccessToken: async () => token,
});

// patientSD is the Patient StructureDefinition (or a profile of it).
export function PatientForm({ patientSD }: { patientSD: StructureDefinition }) {
  const [patient, setPatient] = useState<Patient>({ resourceType: "Patient" });

  return (
    <>
      <FHIRGenerativeForm
        fhirVersion={R4}
        client={client}
        structureDefinition={patientSD}
        value={patient}
        setValue={(setter) => setPatient(setter(patient) as Patient)}
      />
      <Button
        buttonType="primary"
        onClick={() => client.create({}, R4, patient)}
      >
        Save
      </Button>
    </>
  );
}
```

`FHIRGenerativeForm` renders an editor for every element of the StructureDefinition and applies edits as JSON patches, so `setValue` receives a function from the old resource to the new one. `client` is handed to the element editors that talk to the server, such as the reference picker and the coding and identifier editors.

## Theming

Every visual value the library uses is a CSS custom property, so a host app can
re-theme the components without forking them. Override the variables on
`:root`, or on a wrapper element to scope a theme to part of the page.

### Brand and primary colour

The `--hh-color-brand-*` scale is the primary colour API. It is a set of RGB
channels, so Tailwind's alpha modifiers keep working. Teal is the default.

```css
:root {
  --hh-color-brand-500: 20 184 166;
  --hh-color-brand-600: 13 148 136;
  --hh-color-brand-700: 15 118 110;
}
```

`--ci-signal` follows `--hh-color-brand-600` (and its dark-theme value), so an
existing brand override also re-colours primary buttons, selected rows and
focus rings. `--ci-signal` remains directly overridable when a host wants the
interactive colour to differ from the brand.

The default is the original Haste Health light palette: white surfaces, slate
ink and the teal brand. The **Ember** accent (`data-accent="ember"`) switches
both the brand to orange-red and the light surfaces to the warm paper palette.
Every component picks it up through `--ci-signal` and the surface tokens.

```html
<html lang="en" data-theme="light" data-accent="ember">
```

### Semantic tokens

The default light theme keeps the original white/slate surfaces with the teal
brand. Override any of these to change the surfaces and ink independently of the
brand:

```css
:root {
  --ci-bg: #f1f5f9;
  --ci-surface: #ffffff;
  --ci-surface-raised: #ffffff;
  --ci-surface-sunken: #f8fafc;
  --ci-ink: #0f172a;
  --ci-ink-muted: #475569;
  --ci-ink-subtle: #64748b;
  --ci-line: #e2e8f0;
  --ci-line-strong: #cbd5e1;
  --ci-signal: rgb(var(--hh-color-brand-600));
  --ci-signal-ink: #0b6b62;
  --ci-signal-soft: rgb(13 148 136 / 12%);
  --ci-signal-hover: #0a7a70;
  --ci-on-signal: #ffffff;
}
```

The Ember accent adds the paper surfaces on top of the orange-red brand:

```css
html[data-accent="ember"] {
  --ci-bg: #f1eee7;
  --ci-surface: #faf8f4;
  --ci-surface-raised: #ffffff;
  --ci-surface-sunken: #f2eee6;
  --ci-ink: #0e1013;
  --ci-ink-muted: #4a5260;
  --ci-ink-subtle: #5b6472;
  --ci-line: rgb(14 16 19 / 22%);
  --ci-line-strong: rgb(14 16 19 / 42%);
}
```

Dark mode is selected with `html[data-theme="dark"]`; it uses the dark neutral
surfaces and does not turn on the paper palette. The admin app's
`ThemeProvider` sets the theme and accent, remembers both choices and applies
them before first paint. If you set `--ci-signal` on a wrapper rather than at
the theme root, set `--ci-signal-ink`, `--ci-signal-soft`, `--ci-signal-hover`
and `--ci-on-signal` on the same wrapper as well; those values are derived at
the theme root.

The main semantic tokens are `--ci-bg`, `--ci-surface`, `--ci-surface-raised`,
`--ci-surface-sunken`, `--ci-ink`, `--ci-ink-muted`, `--ci-ink-subtle`,
`--ci-line`, `--ci-line-strong`, `--ci-signal`, `--ci-signal-ink`,
`--ci-signal-soft`, `--ci-signal-hover` and `--ci-on-signal`.

## Components

| Group           | Exports                                                                                                                                                                                                                                                                                                                                                                                                       |
| --------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Base            | `Button`, `Input`, `Select`, `InputContainer`, `Table`, `Tabs`, `Tag`, `Modal`, `Menu`, `DropDownMenu`, `BreadCrumbs`, `Loading`, `Add`, `Toaster` (`notify`, `success`, `error`, `loading`, `promise`), `CodeMirror`, `JSONTextEditor`, `MergeViewer`                                                                                                                                                        |
| Layout          | `Shell`, `Navigation`, `ProfileDropdown`, `SideBar`                                                                                                                                                                                                                                                                                                                                                           |
| FHIR generative | `FHIRGenerativeForm` (form from a StructureDefinition), `FHIRGenerativeSearchTable` (paged search results for a resource type)                                                                                                                                                                                                                                                                                |
| FHIR search     | `FHIRSearchInput` (search bar with completions for parameters, modifiers, prefixes and chains), `FHIRResultsTable`, `useSearch`, and the `parseSearch`, `fhirSearchCompletions` and `SearchMetadata` helpers behind them                                                                                                                                                                                      |
| FHIR operations | `OperationInvocationPanel`, `OperationParametersEditor`, `OperationParametersView`, driven by an OperationDefinition                                                                                                                                                                                                                                                                                          |
| FHIR resources  | `JSONResourceEditor`, `OutcomePanel`, `FHIROperationOutcomeDisplay`, `FHIRQuestionnaireRenderer`                                                                                                                                                                                                                                                                                                              |
| FHIR data types | Editable components for every R4 primitive (`FHIRStringEditable`, `FHIRDateEditable`, `FHIRCodeEditable`, ...) and for Address, Annotation, Attachment, CodeableConcept, Coding, ContactDetail, ContactPoint, Expression, HumanName, Identifier, Money, Period, Range, Ratio, Reference, SampledData, Signature, SimpleQuantity and Timing, plus read-only variants of the common ones and `FHIRMetaReadOnly` |
| SQL on FHIR     | `ViewDefinitionSqlRunner`: runs a ViewDefinition against the server and exports CSV, JSON or NDJSON                                                                                                                                                                                                                                                                                                           |
| Haste Health    | `HasteHealthProvider` and `useHasteHealth` (OAuth 2.0 login against a Haste Health tenant and project, with a ready-made client), `Login`, `EmailForm`, `PasswordResetForm`, `TenantSelect`, `ScopeVerifyForm`, `Feedback`, `Container`                                                                                                                                                                       |

## Links

- [Storybook](https://storybook.haste.health), interactive examples for every component
- [API reference](https://typescript.haste.health/modules/_haste-health_components.html)
- [Source](https://github.com/HasteHealth/HasteHealth/tree/main/frontend/packages/components)
- [React components guide](https://haste.health/docs/api/sdks/react)
- Related: [`@haste-health/client`](https://www.npmjs.com/package/@haste-health/client), [`@haste-health/fhir-types`](https://www.npmjs.com/package/@haste-health/fhir-types), [`@haste-health/generated-ops`](https://www.npmjs.com/package/@haste-health/generated-ops)

Part of [Haste Health](https://haste.health), licensed under [Apache-2.0](https://github.com/HasteHealth/HasteHealth/blob/main/LICENSE).
