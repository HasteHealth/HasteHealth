declare global {
  interface Window {
    VITE_FHIR_BASE_URL: string | undefined;
    VITE_CLIENT_ID: string | undefined;
  }
}

export const VITE_FHIR_BASE_URL =
  window.VITE_FHIR_BASE_URL ?? import.meta.env.VITE_FHIR_BASE_URL;

export const VITE_CLIENT_ID =
  window.VITE_CLIENT_ID ?? import.meta.env.VITE_CLIENT_ID ?? "admin-app";
