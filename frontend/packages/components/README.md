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

The theme is a set of CSS custom properties, defined in [`src/theme.css`](./src/theme.css) and shipped in `dist/index.css`. A host app re-themes the components by overriding them: on `:root` for the light theme, on `[data-theme="dark"]` for the dark one.

### Brand palette

`--hh-color-brand-50` to `--hh-color-brand-950` are the scale behind the `brand-*` utilities. The default is teal. Each value is a list of RGB channels rather than a colour, so that alpha modifiers such as `bg-brand-600/10` work. The palette is the same in both themes.

```css
:root {
  --hh-color-brand-500: 99 102 241;
  --hh-color-brand-600: 79 70 229;
  --hh-color-brand-700: 67 56 202;
}
```

### Colour roles

A role names what a colour is for, and has one value in each theme. The roles and their light values are the ones the Haste Health server uses for its sign-in pages. Each role is a property (`--hh-color-surface`) and a Tailwind colour of the same name (`bg-surface`, `text-ink-muted`, `border-line`).

| Role            | Light     | Dark             | For                                         |
| --------------- | --------- | ---------------- | ------------------------------------------- |
| `background`    | slate-100 | slate-900        | The page behind every surface               |
| `surface`       | white     | slate-800        | Cards, panels, headers, menus               |
| `surface-muted` | gray-50   | slate-700        | Inputs, hovered rows and buttons            |
| `ink`           | slate-900 | white            | Headings and input text                     |
| `ink-body`      | slate-700 | slate-300        | Running text and labels                     |
| `ink-muted`     | slate-500 | slate-400        | Secondary text                              |
| `ink-subtle`    | slate-400 | slate-500        | Hints and placeholders                      |
| `line`          | gray-200  | slate-700        | Borders and dividers                        |
| `line-strong`   | gray-300  | slate-600        | Input borders, and rules on `surface-muted` |
| `primary`       | brand-600 | brand-600        | Primary buttons, links, the selected item   |
| `primary-hover` | brand-500 | brand-500        | A hovered primary button                    |
| `primary-ring`  | brand-300 | brand-800        | Focus rings                                 |
| `primary-line`  | brand-200 | brand-900        | The border of a hovered or selected item    |
| `primary-soft`  | brand-50  | brand-600 at 15% | The fill of a hovered or selected item      |
| `on-primary`    | white     | white            | Text and icons on `primary`                 |
| `error`         | red-600   | red-400          | Error text                                  |
| `error-line`    | red-200   | red-900          | The border of an error message              |
| `error-soft`    | red-50    | red-500 at 15%   | The fill of an error message                |
| `warning`       | amber-600 | amber-400        | Warning text                                |

The `primary` roles are derived from the brand palette, so overriding the palette re-colours them too. A role can also be set directly:

```css
:root {
  --hh-color-background: #fafaf9;
}

[data-theme="dark"] {
  --hh-color-background: #0c0a09;
}
```

### Dark theme

Set `data-theme="dark"` on `<html>` to select the dark values. That also sets `color-scheme`, so scrollbars and native controls follow, and it is what Tailwind's `dark:` variant matches. The library never reads the operating system's setting; the host decides, as the admin app's `ThemeProvider` does from a saved preference. Set on an element other than `<html>`, the attribute makes that part of the page dark.

The components do not use the roles yet: they are styled with fixed palette colours, so `data-theme="dark"` does not give a dark interface today. The dark values are here for the components to move onto.

### In your own Tailwind build

Import the theme next to Tailwind to use the role utilities and the `dark:` variant in your own markup:

```css
@import "tailwindcss";
@import "@haste-health/components/src/theme.css";
```

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
