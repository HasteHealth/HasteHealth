import React, { useEffect } from "react";
import { createRoot } from "react-dom/client";

import createHTTPClient from "@haste-health/client/lib/http";
import {
  Container,
  FHIRGenerativeSearchTable,
  Loading,
} from "@haste-health/components";

declare global {
  interface Window {
    API_URL: string;
    ACCESS_TOKEN: string;
    RESOURCE_TYPE: string;
    FHIR_VERSION: any;
  }
}

function SMARTSelector() {
  const client = createHTTPClient({
    url: window.API_URL,
    getAccessToken: async () => window.ACCESS_TOKEN,
  });

  const [wellKnown, setWellKnown] = React.useState<
    Record<string, string> | undefined
  >(undefined);

  let url = new URL(window.API_URL);

  let path = url.pathname;

  let host = url.origin;

  useEffect(() => {
    fetch(host + "/.well-known/openid-configuration" + path)
      .then((res) => res.json())
      .then((wellKnown: Record<string, string>) => {
        setWellKnown(wellKnown);
      });
  }, [setWellKnown]);

  return wellKnown === undefined ? (
    <div className="flex w-full h-full justify-center items-center">
      <Loading />
    </div>
  ) : (
    <Container title={`Select ${window.RESOURCE_TYPE}`} size="lg">
      <FHIRGenerativeSearchTable
        fhirVersion={window.FHIR_VERSION}
        client={client}
        resourceType={window.RESOURCE_TYPE as any}
        onRowClick={(row) => {
          const urlParams = new URLSearchParams(window.location.search);
          urlParams.set(
            `launch/${window.RESOURCE_TYPE.toLowerCase()}`,
            (row as Record<string, string>).id
          );

          window.location.replace(
            wellKnown.authorization_endpoint + "?" + urlParams.toString()
          );
        }}
      />
    </Container>
  );
}

createRoot(document.getElementById("root") as HTMLElement).render(
  <SMARTSelector />
);
