/**
 * The role matrix's data model and the FHIR resources it produces.
 *
 * Kept free of React and of the FHIR client so the mapping from checkboxes to
 * rules can be tested directly. The expressions it writes match the policy
 * engine's actual variables: `%request.resource_type` and `%request.type`.
 */
import type {
  AccessPolicyV2,
  AccessPolicyV2Assignment,
  AccessPolicyV2Rule,
  Reference,
  code,
} from "@haste-health/fhir-types/r4/types";

export type CrudKey = "create" | "read" | "update" | "delete";

/** What the matrix grants for one resource type. */
export type Permission = Record<CrudKey, boolean>;

export type PermissionMap = Record<string, Permission>;

export type CrudPreset =
  | "custom"
  | "auditor"
  | "clinician"
  | "nurse"
  | "frontDesk"
  | "laboratory"
  | "oncology"
  | "intake";

export const PRESET_LABELS: Record<CrudPreset, string> = {
  custom: "Custom policy",
  auditor: "Auditor (read-only, all resources)",
  clinician: "Clinician (clinical write, patient read)",
  nurse: "Nurse",
  frontDesk: "Front desk / registration",
  laboratory: "Laboratory",
  oncology: "Oncology",
  intake: "Intake clerk",
};

export function emptyPermission(): Permission {
  return { create: false, read: false, update: false, delete: false };
}

export function hasAnyPermission(permission: Permission | undefined): boolean {
  return Boolean(
    permission &&
      (permission.create ||
        permission.read ||
        permission.update ||
        permission.delete),
  );
}

/**
 * The engine's interaction codes for a matrix cell.
 *
 * Read covers the three read interactions, and update covers both update and
 * patch, so the matrix cannot grant one without the other.
 */
export function interactionsFor(permission: Permission): string[] {
  const interactions: string[] = [];
  if (permission.create) interactions.push("create");
  if (permission.read) interactions.push("read", "vread", "search");
  if (permission.update) interactions.push("update", "patch");
  if (permission.delete) interactions.push("delete");
  return interactions;
}

/** The rule's `target`: only evaluate it for this type and these interactions. */
export function targetExpression(
  resourceType: string,
  permission: Permission,
): string | undefined {
  const interactions = interactionsFor(permission);
  if (interactions.length === 0) return undefined;
  const allowed = interactions.map((interaction) => `'${interaction}'`).join(" | ");
  return `%request.resource_type = '${resourceType}' and %request.type in (${allowed})`;
}

/** The matrix words for a description, rather than the engine's codes. */
export function describePermission(permission: Permission): string {
  const words: string[] = [];
  if (permission.create) words.push("create");
  if (permission.read) words.push("read");
  if (permission.update) words.push("update");
  if (permission.delete) words.push("delete");
  if (words.length === 0) return "no access";
  if (words.length === 1) return words[0]!;
  return `${words.slice(0, -1).join(", ")} and ${words[words.length - 1]}`;
}

/**
 * One flat rule per configured type.
 *
 * `target` gates the rule so a type the matrix did not select is
 * `undetermined` rather than denied. `condition: true` is essential: a rule
 * whose target matches but has no condition is rejected by the engine.
 */
export function buildRules(permissions: PermissionMap): AccessPolicyV2Rule[] {
  return Object.keys(permissions)
    .sort((a, b) => a.localeCompare(b))
    .flatMap((resourceType) => {
      const permission = permissions[resourceType];
      if (!permission || !hasAnyPermission(permission)) return [];

      const target = targetExpression(resourceType, permission);
      if (!target) return [];

      return [
        {
          name: `${resourceType} permissions`,
          description: `Grants ${describePermission(permission)} on ${resourceType}.`,
          effect: "permit",
          target: {
            expression: {
              language: "text/fhirpath",
              expression: target,
            },
          },
          condition: {
            expression: {
              language: "text/fhirpath",
              expression: "true",
            },
          },
        } as AccessPolicyV2Rule,
      ];
    });
}

export function configuredTypes(permissions: PermissionMap): string[] {
  return Object.keys(permissions)
    .filter((resourceType) => hasAnyPermission(permissions[resourceType]))
    .sort((a, b) => a.localeCompare(b));
}

export function buildPolicy(args: {
  name: string;
  description?: string;
  permissions: PermissionMap;
}): AccessPolicyV2 {
  const description = args.description?.trim();
  return {
    resourceType: "AccessPolicyV2",
    name: args.name.trim(),
    ...(description ? { description } : {}),
    // The hyphenated code from the AccessPolicyEngineV2 code system.
    engine: "rule-engine" as code,
    rule: buildRules(args.permissions),
  };
}

export function buildAssignment(
  policyReference: string,
  target: Reference,
): AccessPolicyV2Assignment {
  if (!target.reference) {
    throw new Error("The assignment target has no reference.");
  }
  return {
    resourceType: "AccessPolicyV2Assignment",
    accessPolicy: { reference: policyReference },
    link: { reference: target.reference },
  };
}

/** Types a clinician can write. Patient-facing data stays read-only. */
const CLINICIAN_WRITE = [
  "AllergyIntolerance",
  "CarePlan",
  "CareTeam",
  "Condition",
  "DiagnosticReport",
  "DocumentReference",
  "Encounter",
  "Goal",
  "Immunization",
  "MedicationAdministration",
  "MedicationDispense",
  "MedicationRequest",
  "NutritionOrder",
  "Observation",
  "Procedure",
  "QuestionnaireResponse",
  "ServiceRequest",
  "Specimen",
];

const CLINICIAN_READ = [
  "Coverage",
  "Device",
  "HealthcareService",
  "Location",
  "Medication",
  "Organization",
  "Patient",
  "Practitioner",
  "PractitionerRole",
  "RelatedPerson",
];

const INTAKE: Record<string, Partial<Permission>> = {
  Patient: { create: true, read: true, update: true },
  RelatedPerson: { create: true, read: true, update: true },
  Coverage: { create: true, read: true, update: true },
  Encounter: { create: true, read: true },
  Organization: { read: true },
  Practitioner: { read: true },
  PractitionerRole: { read: true },
  Location: { read: true },
};

/** Front desk: registration, scheduling and demographics; no clinical write. */
const FRONT_DESK: Record<string, Partial<Permission>> = {
  Patient: { create: true, read: true, update: true },
  RelatedPerson: { create: true, read: true, update: true },
  Coverage: { create: true, read: true, update: true },
  Encounter: { create: true, read: true, update: true },
  Appointment: { create: true, read: true, update: true },
  AppointmentResponse: { create: true, read: true, update: true },
  Schedule: { create: true, read: true, update: true },
  Slot: { create: true, read: true, update: true },
  Organization: { read: true },
  Practitioner: { read: true },
  PractitionerRole: { read: true },
  Location: { read: true },
  HealthcareService: { read: true },
};

/** Nursing: charting, medication administration, care plans and tasks. */
const NURSE_WRITE = [
  "AllergyIntolerance",
  "CarePlan",
  "CareTeam",
  "Condition",
  "Encounter",
  "Goal",
  "MedicationAdministration",
  "Observation",
  "Procedure",
  "QuestionnaireResponse",
  "ServiceRequest",
  "Specimen",
  "Task",
];

const NURSE_READ = [
  "Coverage",
  "Device",
  "DiagnosticReport",
  "DocumentReference",
  "Immunization",
  "Location",
  "Medication",
  "MedicationDispense",
  "MedicationRequest",
  "Organization",
  "Patient",
  "Practitioner",
  "PractitionerRole",
  "RelatedPerson",
];

/** Laboratory: specimens, results and reports; orders stay read-only. */
const LAB_WRITE = [
  "DiagnosticReport",
  "MolecularSequence",
  "Observation",
  "Specimen",
];

const LAB_READ = [
  "Device",
  "Location",
  "Medication",
  "Organization",
  "Patient",
  "Practitioner",
  "PractitionerRole",
  "QuestionnaireResponse",
  "ServiceRequest",
  "Substance",
];

/** Oncology: diagnosis, treatment, pathology and research context. */
const ONCOLOGY_WRITE = [
  "CarePlan",
  "CareTeam",
  "Condition",
  "DiagnosticReport",
  "FamilyMemberHistory",
  "Goal",
  "MedicationAdministration",
  "MedicationRequest",
  "MolecularSequence",
  "Observation",
  "Procedure",
  "QuestionnaireResponse",
  "RiskAssessment",
  "ServiceRequest",
  "Specimen",
];

const ONCOLOGY_READ = [
  "Coverage",
  "Device",
  "DocumentReference",
  "Immunization",
  "Location",
  "Medication",
  "Organization",
  "Patient",
  "Practitioner",
  "PractitionerRole",
  "RelatedPerson",
  "ResearchStudy",
  "ResearchSubject",
];

/**
 * A fresh matrix for a preset. Every type the server exposes is a row, so the
 * result contains a permission object for each one; unlisted types get none.
 */
export function applyPreset(
  preset: CrudPreset,
  resourceTypes: string[],
): PermissionMap {
  const permissions: PermissionMap = {};
  for (const resourceType of resourceTypes) {
    permissions[resourceType] = emptyPermission();
  }

  const apply = (types: string[], granted: Partial<Permission>) => {
    for (const resourceType of types) {
      const permission = permissions[resourceType];
      if (permission) Object.assign(permission, granted);
    }
  };

  if (preset === "auditor") {
    for (const resourceType of resourceTypes) {
      permissions[resourceType]!.read = true;
    }
    return permissions;
  }

  if (preset === "clinician") {
    for (const resourceType of CLINICIAN_WRITE) {
      const permission = permissions[resourceType];
      if (!permission) continue;
      permission.create = true;
      permission.read = true;
      permission.update = true;
    }
    for (const resourceType of CLINICIAN_READ) {
      const permission = permissions[resourceType];
      if (permission) permission.read = true;
    }
    return permissions;
  }

  if (preset === "frontDesk") {
    for (const [resourceType, granted] of Object.entries(FRONT_DESK)) {
      const permission = permissions[resourceType];
      if (permission) Object.assign(permission, granted);
    }
    return permissions;
  }

  if (preset === "nurse") {
    apply(NURSE_WRITE, { create: true, read: true, update: true });
    apply(NURSE_READ, { read: true });
    return permissions;
  }

  if (preset === "laboratory") {
    apply(LAB_WRITE, { create: true, read: true, update: true });
    apply(LAB_READ, { read: true });
    return permissions;
  }

  if (preset === "oncology") {
    apply(ONCOLOGY_WRITE, { create: true, read: true, update: true });
    apply(ONCOLOGY_READ, { read: true });
    return permissions;
  }

  if (preset === "intake") {
    for (const [resourceType, granted] of Object.entries(INTAKE)) {
      const permission = permissions[resourceType];
      if (!permission) continue;
      Object.assign(permission, granted);
    }
    return permissions;
  }

  return permissions;
}
