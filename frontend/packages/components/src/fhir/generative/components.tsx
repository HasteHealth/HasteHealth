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
  integer,
  uri,
  url,
} from "@oxidized-health/fhir-types/r4/types";

import * as ComplexTypes from "../complex";
import * as Primitives from "../primitives";
import { getElementDefinition } from "./helpers";
import { MetaProps } from "./types";

type TypeProps = MetaProps<any, any> & {
  label?: string;
};
type TypeComponent = React.FC<TypeProps>;

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

export const TypeComponents: Record<string, TypeComponent> = {
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
        props.elementIndex
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
};

export const isTypeRenderingSupported = (type: string) => {
  return Object.keys(TypeComponents).includes(type);
};
