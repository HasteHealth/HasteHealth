import type { Meta, StoryObj } from "@storybook/react";

import { ClientApplication } from "@haste-health/fhir-types/lib/generated/r4/types";

import { ScopeVerifyForm } from "./ScopeVerify";

// More on how to set up stories at: https://storybook.js.org/docs/react/writing-stories/introduction#default-export
const meta = {
  title: "HasteHealth/ScopeVerify",
  component: ScopeVerifyForm,
  parameters: {
    // Optional parameter to center the component in the Canvas. More info: https://storybook.js.org/docs/react/configure/story-layout
    layout: "fullscreen",
  },
  // This component will have an automatically generated Autodocs entry: https://storybook.js.org/docs/react/writing-docs/autodocs
  tags: ["autodocs"],
} satisfies Meta<typeof ScopeVerifyForm>;

export default meta;

type Story = StoryObj<typeof meta>;

export const Primary: Story = {
  args: {
    client: {
      name: "Admin App",
      logoUri:
        "https://www.gravatar.com/avatar/d23aeb14e1da44674f5e9036abdc9305?s=160&d=retro&r=g",
    } as ClientApplication,
    title: "HasteHealth",
    postURL: "#",
    authorizeParameters: {
      client_id: "",
      response_type: "",
      state: "",
      code_challenge: "",
      code_challenge_method: "",
      scope: [
        "launch/patient",
        "openid",
        "fhirUser",
        "offline_access",
        "patient/Medication.rs",
        "patient/AllergyIntolerance.rs",
        "patient/CarePlan.rs",
        "patient/CareTeam.rs",
        "patient/Condition.rs",
        "patient/Device.rs",
        "patient/DiagnosticReport.rs",
        "patient/DocumentReference.rs",
        "patient/Encounter.rs",
        "patient/Goal.rs",
        "patient/Immunization.rs",
        "patient/Location.rs",
        "patient/MedicationRequest.rs",
        "patient/Observation.rs",
        "patient/Organization.rs",
        "patient/Patient.rs",
        "patient/Practitioner.rs",
        "patient/Procedure.rs",
        "patient/Provenance.rs",
        "patient/PractitionerRole.rs",
      ].join(" "),
    },
  },
};

export const NoImage: Story = {
  args: {
    client: {
      name: "Admin App",
    } as ClientApplication,
    title: "HasteHealth",
    postURL: "#",
    authorizeParameters: {
      client_id: "",
      response_type: "",
      state: "",
      code_challenge: "",
      code_challenge_method: "",
      scope: [
        "launch/patient",
        "openid",
        "fhirUser",
        "offline_access",
        "patient/Medication.rs",
        "patient/AllergyIntolerance.rs",
        "patient/CarePlan.rs",
        "patient/CareTeam.rs",
        "patient/Condition.rs",
        "patient/Device.rs",
        "patient/DiagnosticReport.rs",
        "patient/DocumentReference.rs",
        "patient/Encounter.rs",
        "patient/Goal.rs",
        "patient/Immunization.rs",
        "patient/Location.rs",
        "patient/MedicationRequest.rs",
        "patient/Observation.rs",
        "patient/Organization.rs",
        "patient/Patient.rs",
        "patient/Practitioner.rs",
        "patient/Procedure.rs",
        "patient/Provenance.rs",
        "patient/PractitionerRole.rs",
      ].join(" "),
    },
  },
};
