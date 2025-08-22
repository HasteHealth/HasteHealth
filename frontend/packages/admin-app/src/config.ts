declare global {
  interface Window {
    REACT_APP_FHIR_BASE_URL: string | undefined;
    REACT_APP_CLIENT_ID: string | undefined;
  }
}

export const REACT_APP_FHIR_BASE_URL =
  window.REACT_APP_FHIR_BASE_URL ?? process.env.REACT_APP_FHIR_BASE_URL;

export const REACT_APP_CLIENT_ID =
  window.REACT_APP_CLIENT_ID ?? process.env.REACT_APP_CLIENT_ID ?? "admin-app";
