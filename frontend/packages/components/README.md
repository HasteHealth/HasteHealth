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
