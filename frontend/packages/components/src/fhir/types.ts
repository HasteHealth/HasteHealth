import createHTTPClient from "@haste-health/client/http";
import { FHIR_VERSION } from "@haste-health/fhir-types/versions";

type HTTPClient = ReturnType<typeof createHTTPClient>;

export type EditableProps<T> = {
  /**
   * The value of the input.
   */
  value?: T;
  /**
   * Issues
   */
  issue?: string;
  /**
   * Call back triggered when input changes.
   */
  onChange?: (value: T | undefined) => void;
  /**
   * Label string.
   */
  required?: boolean;
  label?: string;
};

export type ClientProps = {
  fhirVersion: FHIR_VERSION;
  client: HTTPClient;
};
