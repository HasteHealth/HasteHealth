/* eslint @typescript-eslint/no-explicit-any: 0 */
import React from "react";

import {
  Address,
  Annotation,
  Attachment,
  CodeableConcept,
  Coding,
  ContactDetail,
  ContactPoint,
  Expression,
  HumanName,
  Identifier,
  Meta,
  Period,
  Quantity,
  Range,
  Ratio,
  Reference,
  ResourceType,
  code,
  date,
  dateTime,
  decimal,
  id,
  instant,
  integer,
  uri,
  url,
} from "@haste-health/fhir-types/r4/types";

import * as ComplexTypes from "../complex";
import * as Primitives from "../primitives";
import { getElementDefinition } from "./helpers";
import { MetaProps } from "./types";
import { OID } from "../primitives/oid";
import { UUID } from "../primitives/uuid";

type TypeProps = MetaProps<any, any> & {
  label?: string;
};
type TypeComponent = React.FC<TypeProps>;

function withFieldShell(component: TypeComponent): TypeComponent {
  return function WrappedTypeComponent(props: TypeProps) {
    const Component = component;
    return (
      <div className="w-full">
        <Component {...props} />
      </div>
    );
  };
}

type SharedProps<T> = {
  label?: string;
  value?: T;
  onChange: (v: T | undefined) => void;
};

function deriveSharedProps<T>(props: TypeProps): SharedProps<T> {
  return {
    label: props.label,
    value: props.value as T | undefined,
    onChange: (v: T | undefined) => {
      props.onChange({
        op: "replace",
        path: props.pointer,
        value: v,
      });
    },
  };
}

const BaseTypeComponents: Record<string, TypeComponent> = {
  "http://hl7.org/fhirpath/System.String": (props) => (
    <Primitives.FHIRStringEditable
      disabled={true}
      {...deriveSharedProps<string>(props)}
    />
  ),
  id: (props) => (
    <Primitives.FHIRIdEditable {...deriveSharedProps<id>(props)} />
  ),
  string: (props) => (
    <Primitives.FHIRStringEditable {...deriveSharedProps<string>(props)} />
  ),
  boolean: (props) => (
    <Primitives.FHIRBooleanEditable {...deriveSharedProps<boolean>(props)} />
  ),
  url: (props) => (
    <Primitives.FHIRUrlEditable {...deriveSharedProps<url>(props)} />
  ),
  date: (props) => (
    <Primitives.FHIRDateEditable {...deriveSharedProps<date>(props)} />
  ),
  dateTime: (props) => (
    <Primitives.FHIRDateTimeEditable {...deriveSharedProps<dateTime>(props)} />
  ),
  time: (props) => (
    <Primitives.FHIRTimeEditable {...deriveSharedProps<string>(props)} />
  ),
  markdown: (props) => (
    <Primitives.FHIRMarkdownEditable {...deriveSharedProps<markdown>(props)} />
  ),
  uri: (props) => (
    <Primitives.FHIRUriEditable {...deriveSharedProps<uri>(props)} />
  ),
  code: (props) => (
    <Primitives.FHIRCodeEditable
      {...deriveSharedProps<code>(props)}
      fhirVersion={props.fhirVersion}
      client={props.client}
      open={true}
      system={
        getElementDefinition(props.sd, props.elementIndex).element.binding
          ?.valueSet
      }
    />
  ),
  decimal: (props) => (
    <Primitives.FHIRDecimalEditable {...deriveSharedProps<decimal>(props)} />
  ),
  integer: (props) => (
    <Primitives.FHIRIntegerEditable {...deriveSharedProps<integer>(props)} />
  ),
  base64Binary: (props) => (
    <Primitives.FHIRBase64BinaryEditable
      {...deriveSharedProps<string>(props)}
    />
  ),
  canonical: (props) => (
    <Primitives.FHIRCanonicalEditable {...deriveSharedProps<string>(props)} />
  ),
  instant: (props) => (
    <Primitives.FHIRInstantEditable {...deriveSharedProps<instant>(props)} />
  ),
  oid: (props) => (
    <Primitives.FHIROIDEditable {...deriveSharedProps<OID>(props)} />
  ),
  unsignedInt: (props) => (
    <Primitives.FHIRUnsignedIntegerEditable
      {...deriveSharedProps<number>(props)}
    />
  ),
  positiveInt: (props) => (
    <Primitives.FHIRPositiveIntegerEditable
      {...deriveSharedProps<number>(props)}
    />
  ),
  uuid: (props) => (
    <Primitives.FHIRUUIDEditable {...deriveSharedProps<UUID>(props)} />
  ),
  Address: (props) => (
    <ComplexTypes.FHIRAddressEditable {...deriveSharedProps<Address>(props)} />
  ),
  Annotation: (props) => (
    <ComplexTypes.FHIRAnnotationEditable
      {...deriveSharedProps<Annotation>(props)}
    />
  ),
  Identifier: (props) => (
    <ComplexTypes.FHIRIdentifierEditable
      {...deriveSharedProps<Identifier>(props)}
      fhirVersion={props.fhirVersion}
      client={props.client}
    />
  ),
  ContactPoint: (props) => (
    <ComplexTypes.FHIRContactPointEditable
      {...deriveSharedProps<ContactPoint>(props)}
      fhirVersion={props.fhirVersion}
      client={props.client}
    />
  ),
  Expression: (props) => (
    <ComplexTypes.FHIRExpressionEditable
      {...deriveSharedProps<Expression>(props)}
    />
  ),
  HumanName: (props) => (
    <ComplexTypes.FHIRHumanNameEditable
      {...deriveSharedProps<HumanName>(props)}
    />
  ),
  ContactDetail: (props) => (
    <ComplexTypes.FHIRContactDetailEditable
      {...deriveSharedProps<ContactDetail>(props)}
      fhirVersion={props.fhirVersion}
      client={props.client}
    />
  ),

  Period: (props) => (
    <ComplexTypes.FHIRPeriodEditable {...deriveSharedProps<Period>(props)} />
  ),
  Quantity: (props) => (
    <ComplexTypes.FHIRSimpleQuantityEditable
      value={props.value as Quantity}
      label={props.label}
      onChange={(v) => {
        props.onChange({
          op: "replace",
          path: props.pointer,
          value: v,
        });
      }}
    />
  ),
  Reference: (props) => (
    <ComplexTypes.FHIRReferenceEditable
      {...deriveSharedProps<Reference>(props)}
      fhirVersion={props.fhirVersion}
      client={props.client}
      resourceTypesAllowed={getElementDefinition(
        props.sd,
        props.elementIndex,
      ).element.type?.[0]?.targetProfile?.map((tp) => {
        const parts = tp.split("/");
        return parts[parts.length - 1] as ResourceType;
      })}
    />
  ),
  Ratio: (props) => (
    <ComplexTypes.FHIRRatioEditable {...deriveSharedProps<Ratio>(props)} />
  ),
  Range: (props) => (
    <ComplexTypes.FHIRRangeEditable {...deriveSharedProps<Range>(props)} />
  ),
  CodeableConcept: (props) => (
    <ComplexTypes.FhirCodeableConceptEditable
      {...deriveSharedProps<CodeableConcept>(props)}
      fhirVersion={props.fhirVersion}
      client={props.client}
    />
  ),
  Coding: (props) => (
    <ComplexTypes.FHIRCodingEditable
      fhirVersion={props.fhirVersion}
      client={props.client}
      {...deriveSharedProps<Coding>(props)}
    />
  ),
  Attachment: (props) => (
    <ComplexTypes.FHIRAttachmentEditable
      {...deriveSharedProps<Attachment>(props)}
    />
  ),
  Meta: (props) => (
    <ComplexTypes.FHIRMetaReadOnly
      fhirVersion={props.fhirVersion}
      client={props.client}
      {...deriveSharedProps<Meta>(props)}
    />
  ),
  // Todo [Timing, Money, Duration, MoneyQuantity, SimpleQuantity] trial [SampleData, Signature, Age, Distance]
  // MetadataTypes review and Special Purpose Data types review.
};

export const TypeComponents = Object.fromEntries(
  Object.entries(BaseTypeComponents).map(([name, component]) => [
    name,
    withFieldShell(component),
  ]),
) as Record<string, TypeComponent>;

export const isTypeRenderingSupported = (type: string) => {
  return Object.keys(TypeComponents).includes(type);
};
