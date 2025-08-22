export const CUSTOM_CLAIMS = {
  RESOURCE_TYPE: <const>"https://oxidized-health.app/resourceType",
  RESOURCE_ID: <const>"https://oxidized-health.app/resourceId",
  ACCESS_POLICY_VERSION_IDS: <const>(
    "https://oxidized-health.app/accessPolicyVersionIds"
  ),
  TENANT: <const>"https://oxidized-health.app/tenant",
  ROLE: <const>"https://oxidized-health.app/role",
};

export type ALGORITHMS_ALLOWED = (typeof ALGORITHMS)[keyof typeof ALGORITHMS];

export const ALGORITHMS = <const>{
  RS256: "RS256",
  RS384: "RS384",
};
