export const CUSTOM_CLAIMS = {
  RESOURCE_TYPE: <const>"https://haste.health/resourceType",
  RESOURCE_ID: <const>"https://haste.health/resourceId",
  ACCESS_POLICY_VERSION_IDS: <const>(
    "https://haste.health/accessPolicyVersionIds"
  ),
  TENANT: <const>"https://haste.health/tenant",
  ROLE: <const>"https://haste.health/user_role",
};

export type ALGORITHMS_ALLOWED = (typeof ALGORITHMS)[keyof typeof ALGORITHMS];

export const ALGORITHMS = <const>{
  RS256: "RS256",
  RS384: "RS384",
};
