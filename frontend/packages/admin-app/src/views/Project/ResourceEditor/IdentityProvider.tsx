import { useAtomValue } from "jotai";
import { useEffect, useState } from "react";

import { Input } from "@oxidized-health/components";
import {
  IdentityProvider,
  ResourceType,
  id,
} from "@oxidized-health/fhir-types/r4/types";
import { R4 } from "@oxidized-health/fhir-types/versions";
import { OxidizedHealthIdpRegistrationInfo } from "@oxidized-health/generated-ops/lib/r4/ops";

import ResourceEditorComponent, {
  AdditionalContent,
} from "../../../components/ResourceEditor";
import { getClient } from "../../../db/client";

interface IdentityProviderEditorProps extends AdditionalContent {
  resource: IdentityProvider | undefined;
  onChange: NonNullable<AdditionalContent["onChange"]>;
}

function RegistrationInformation({ id }: Readonly<{ id: id }>) {
  const client = useAtomValue(getClient);
  const [registrationInformation, setRegistrationInformation] = useState<
    OxidizedHealthIdpRegistrationInfo.Output | undefined
  >();

  useEffect(() => {
    client
      .invoke_instance(
        OxidizedHealthIdpRegistrationInfo.Op,
        {},
        R4,
        "IdentityProvider",
        id,
        {}
      )
      .then((output) => {
        setRegistrationInformation(output);
      });
  }, [id]);

  return (
    <div className="space-y-4">
      <div>
        <div className="text-lg font-medium">Documentation</div>
        <p className="text-sm text-gray-500">
          For more information on how to register an external IDP read the
          following{" "}
          <a
            className="text-teal-500 hover:text-teal-600 cursor-pointer"
            href="https://oxidized-health.app/documentation/Getting%20Started/Local_Development"
          >
            documentation
          </a>
          .
        </p>
      </div>
      <div className="space-y-2">
        <div className="text-lg font-medium">Information</div>
        <p className="text-sm text-gray-500">
          Registration information for registering an OIDC client with this
          identity provider.
        </p>
        <div className="space-y-1">
          {(registrationInformation?.information ?? []).map((info) => (
            <Input
              key={info.name}
              readOnly
              label={info.name}
              value={info.value}
            />
          ))}
        </div>
      </div>
    </div>
  );
}

export default function IdentityProviderView({
  id,
  resourceType,
  resource,
  actions,
  structureDefinition,
  onChange,
}: Readonly<IdentityProviderEditorProps>) {
  return (
    <ResourceEditorComponent
      id={id}
      actions={actions}
      structureDefinition={structureDefinition}
      resourceType={resourceType as ResourceType}
      resource={resource}
      onChange={onChange}
      rightTabs={[
        {
          id: "endpoints",
          title: "Registration Information",
          content: <RegistrationInformation id={id} />,
        },
      ]}
    />
  );
}
