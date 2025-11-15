export const CUSTOM_CLAIMS = {
  RESOURCE_TYPE: <const>"https://haste-health.app/resourceType",
  RESOURCE_ID: <const>"https://haste-health.app/resourceId",
  ACCESS_POLICY_VERSION_IDS: <const>(
    "https://haste-health.app/accessPolicyVersionIds"
  ),
  TENANT: <const>"https://haste-health.app/tenant",
  ROLE: <const>"https://haste-health.app/role",
};

export type ALGORITHMS_ALLOWED = (typeof ALGORITHMS)[keyof typeof ALGORITHMS];

export const ALGORITHMS = <const>{
  RS256: "RS256",
  RS384: "RS384",
};
