/* eslint @typescript-eslint/no-explicit-any: 0 */
import { XMarkIcon } from "@heroicons/react/24/outline";
import classNames from "classnames";
import { applyPatch } from "fast-json-patch";
import { produce } from "immer";
import React, { useMemo } from "react";

import generateJSONPatches, {
  Mutation,
} from "@haste-health/fhir-patch-building";
import {
  Loc,
  ascend,
  descend,
  pointer,
  root,
} from "@haste-health/fhir-pointer";
import {
  complexTypes,
  primitiveTypes,
  resourceTypes,
} from "@haste-health/fhir-types/r4/sets";
import {
  ElementDefinition,
  ElementDefinitionType,
  Resource,
  ResourceType,
  StructureDefinition,
  id,
} from "@haste-health/fhir-types/r4/types";

import { Add, Select } from "../../base";
import { ClientProps } from "../types";
import { TypeComponents, isTypeRenderingSupported } from "./components";
import { getElementDefinition } from "./helpers";
import { MetaProps } from "./types";

function EditorComponent(
  props: MetaProps<any, any> & { type: ElementDefinitionType }
) {
  const found = getElementDefinition(props.sd, props.elementIndex);
  const label = props.showLabel
    ? capitalize(getFieldName(found.element.path))
    : undefined;

  const Comp = TypeComponents[props.type?.code];

  return <Comp {...props} label={label} />;
}

function isLeaf(type: string | undefined): boolean {
  if (!type) return false;
  return (
    primitiveTypes.has(type) ||
    complexTypes.has(type) ||
    // Specialized primitive for .id and other strings that don't allow extensions.
    type === "http://hl7.org/fhirpath/System.String"
  );
}

function TypeChoiceTypeSelect({
  onChange,
  element,
  type,
}: Readonly<{
  element: ElementDefinition;
  onChange: (type: ElementDefinitionType) => void;
  type: ElementDefinitionType | undefined;
}>) {
  if (!element.type || element.type?.length <= 1) return undefined;
  return (
    <div className="flex flex-1">
      <Select
        value={type?.code}
        onChange={(option) => {
          const newType = element.type?.find((t) => t.code === option.value);
          if (newType) {
            onChange(newType);
          }
        }}
        options={(element.type || []).map((t) => ({
          value: t.code,
          label: t.code,
        }))}
      />
    </div>
  );
}

/*
 ** Given a position return all children indices.
 */
function getChildrenElementIndices({
  sd,
  elementIndex,
}: {
  sd: StructureDefinition;
  elementIndex: number;
}): number[] {
  const childIndices: number[] = [];
  const found = getElementDefinition(sd, elementIndex);
  if (!found.element.path)
    throw new Error("Invalid element when deriving children");

  const childRegex = new RegExp(`^${found.element.path}\\.[^\\.]+$`);

  for (
    let i = found.elementIndex + 1;
    i < (sd.snapshot?.element.length || 0);
    i++
  ) {
    if (
      sd.snapshot?.element[i]?.path &&
      childRegex.test(sd.snapshot?.element[i]?.path)
    ) {
      childIndices.push(i);
    }
  }

  return childIndices;
}

function capitalize(s: string) {
  return s[0].toUpperCase() + s.slice(1);
}

function getFieldName(path: string) {
  return path.substring(path.lastIndexOf(".") + 1).replace("[x]", "");
}

function isTypeChoice(element: ElementDefinition): boolean {
  return (element.type || []).length > 1;
}

function isIndexableObject(v: unknown): v is Record<string, unknown> {
  return typeof v === "object" && v !== null;
}

function findTypeChoiceTypeBasedOnField(
  element: ElementDefinition,
  value: Record<string, unknown> | undefined
): ElementDefinitionType {
  if (!element.type?.[0]) throw new Error(`No Type found for ${element.path}.`);

  if (!isTypeChoice(element)) return element.type?.[0];
  const fieldName = getFieldName(element.path);
  for (const type of element.type || []) {
    const typeChoiceFieldName = `${fieldName}${capitalize(type.code)}`;
    if (value?.[typeChoiceFieldName] !== undefined) return type;
  }
  return element.type?.[0];
}

function getTypedFieldName(
  element: ElementDefinition,
  type: ElementDefinitionType
) {
  const fieldName = getFieldName(element.path);
  if (!isTypeChoice(element)) return fieldName;
  return `${fieldName}${capitalize(type.code)}`;
}

function getValueAndPointer(
  element: ElementDefinition,
  pointer: Loc<any, any, any>,
  value: unknown
): {
  value: unknown;
  pointer: Loc<any, any, any>;
  type: ElementDefinitionType;
} {
  if (Array.isArray(value))
    throw new Error("invalid value must be object to descend");

  const type = findTypeChoiceTypeBasedOnField(
    element,
    value as Record<string, unknown>
  );

  const field = getTypedFieldName(element, type);

  return {
    value: isIndexableObject(value) ? value[field] : undefined,
    pointer: descend(pointer, field),
    type: type,
  };
}

function DisplayInvalid({
  element,
  value,
  showInvalid,
}: {
  showInvalid: boolean;
  element: ElementDefinition;
  value: unknown;
}) {
  if (showInvalid)
    return (
      <div>
        <span className="font-semibold"> {element.path}</span>
        <span>
          {element.type && element.type[0].code}: {JSON.stringify(value)}
        </span>
      </div>
    );
  return undefined;
}

function LabelWrapper({
  sd,
  elementIndex,
  type,
  onChange,
  children,
  pointer,
  showLabel = true,
}: MetaProps<any, any> & { children: React.ReactNode }) {
  const found = getElementDefinition(sd, elementIndex);
  if (!showLabel) return children;
  return (
    <div>
      <div className="flex items-center space-x-2">
        <div className={classNames({ required: (found.element.min || 0) > 0 })}>
          {capitalize(
            getFieldName(found.element.path)
              .replace(/([A-Z])/g, " $1")
              .trim()
          )}
        </div>
        {(found.element.type || []).length > 1 && (
          <TypeChoiceTypeSelect
            element={found.element}
            type={type}
            onChange={(selectedType) => {
              if (selectedType.code !== type?.code) {
                onChange({
                  op: "replace",
                  path: pointer,
                  value: undefined,
                });
                const fieldName = getTypedFieldName(
                  found.element,
                  selectedType
                );
                const newPointer = descend(
                  ascend(pointer)?.parent as Loc<any, any, any>,
                  fieldName
                );
                onChange({
                  op: "replace",
                  path: newPointer,
                  value: {},
                });
              }
            }}
          />
        )}
      </div>

      <div className="text-gray-400 text-xs mb-1">{found.element.short}</div>

      {children}
    </div>
  );
}

const MetaValueSingular = React.memo((props: MetaProps<any, any>) => {
  const {
    fhirVersion,
    sd,
    elementIndex,
    value,
    type,
    pointer,
    showInvalid = false,
    client,
    onChange,
  } = props;

  const found = getElementDefinition(sd, elementIndex);

  const children = getChildrenElementIndices({
    sd,
    elementIndex: found.elementIndex,
  });
  if (children.length === 0) {
    if (!isLeaf(found.element.type?.[0].code)) {
      return (
        <DisplayInvalid
          element={found.element}
          value={value}
          showInvalid={showInvalid}
        />
      );
    }

    // Only render the root element not the ones underneath.
    // id is special primitive string.
    const asc = ascend(pointer);
    if (asc?.field === "id" && asc?.parent !== root(pointer)) return undefined;
    if (!type) throw new Error("Type must be defined");
    if (!isTypeRenderingSupported(type.code)) return undefined;
    return (
      <div>
        <LabelWrapper {...props}>
          <EditorComponent {...props} type={type} />
        </LabelWrapper>
      </div>
    );
  }

  return (
    <div>
      <LabelWrapper {...props}>
        <div className="p-2 border rounded-md space-y-2">
          {children.map((childIndex) => {
            const childElement = getElementDefinition(sd, childIndex);
            // Skipping extensions and nested resources for now
            if (
              childElement.element.type?.find(
                (t) => t.code === "Extension" || resourceTypes.has(t.code)
              )
            ) {
              return;
            }
            const {
              value: childValue,
              pointer: childPointer,
              type,
            } = getValueAndPointer(childElement.element, pointer, value);

            return childElement.element.max === "1" ? (
              <MetaValueSingular
                fhirVersion={fhirVersion}
                client={client}
                type={type}
                key={childPointer}
                sd={sd}
                elementIndex={childElement.elementIndex}
                onChange={onChange}
                showInvalid={showInvalid}
                pointer={childPointer}
                value={childValue}
              />
            ) : (
              <MetaValueArray
                fhirVersion={fhirVersion}
                client={client}
                type={type}
                key={childPointer}
                sd={sd}
                elementIndex={childElement.elementIndex}
                showInvalid={showInvalid}
                onChange={onChange}
                pointer={childPointer}
                value={childValue}
              />
            );
          })}
        </div>
      </LabelWrapper>
    </div>
  );
});

const MetaValueArray = React.memo((props: MetaProps<any, any>) => {
  const {
    fhirVersion,
    sd,
    elementIndex,
    value = [],
    pointer,
    onChange,
    client,
    showInvalid = false,
    type,
  } = props;
  const element = getElementDefinition(sd, elementIndex);
  if (!Array.isArray(value)) {
    throw new Error("Value must be an array or undefined");
  }

  const children = getChildrenElementIndices({
    sd,
    elementIndex: element.elementIndex,
  });

  if (!isLeaf(element.element.type?.[0].code) && children.length === 0) {
    return (
      <DisplayInvalid
        element={element.element}
        value={value}
        showInvalid={showInvalid}
      />
    );
  }
  return (
    <div>
      <LabelWrapper {...props}>
        <div className="space-y-1">
          {value.map((v, i) => (
            <div
              className={classNames(
                "relative"
                // {
                //   "bg-gray-50": i % 2 === 1,
                //   "bg-white": i % 2 === 0,
                // }
              )}
              key={`${descend(pointer, i)}`}
            >
              <MetaValueSingular
                fhirVersion={fhirVersion}
                client={client}
                type={type}
                sd={sd}
                elementIndex={element.elementIndex}
                pointer={descend(pointer, i)}
                value={v}
                showLabel={false}
                showInvalid={showInvalid}
                onChange={onChange}
              />
              {value.length > 0 && (
                <div
                  className="absolute top-1 right-1 text-slate-400 cursor-pointer hover:text-slate-500 "
                  onClick={() => {
                    onChange({
                      path: descend(pointer, i),
                      op: "remove",
                      value: v,
                    });
                  }}
                >
                  <XMarkIcon className="h-4 w-4" />
                </div>
              )}
            </div>
          ))}
        </div>
        <div className="mt-1">
          <Add
            onChange={() => {
              onChange({
                path: descend(pointer, value.length),
                op: "add",
                value: complexTypes.has(element.element.type?.[0].code || "")
                  ? {}
                  : null,
              });
            }}
          >
            Add {capitalize(getFieldName(element.element.path))}
          </Add>
        </div>
      </LabelWrapper>
    </div>
  );
});

export type Setter = (resource: Resource) => Resource;

export type FHIRGenerativeFormProps = {
  structureDefinition: StructureDefinition;
  value: Resource | undefined;
  setValue?: (s: Setter) => void;
} & ClientProps;

export const FHIRGenerativeForm = ({
  fhirVersion,
  structureDefinition,
  value,
  client,
  setValue = () => {},
}: FHIRGenerativeFormProps) => {
  const onChange = useMemo(() => {
    return (mutation: Mutation<any, any>) => {
      setValue((resource) => {
        const patches = generateJSONPatches(resource, mutation);
        const newResource = produce(resource, (value) => {
          const newValue = applyPatch(value, patches).newDocument;
          return newValue;
        });
        return newResource;
      });
    };
  }, [setValue]);

  return (
    <MetaValueSingular
      fhirVersion={fhirVersion}
      client={client}
      sd={structureDefinition}
      elementIndex={0}
      value={value}
      type={structureDefinition.snapshot?.element?.[0]?.type?.[0]}
      pointer={pointer(
        fhirVersion,
        structureDefinition.type as ResourceType,
        value?.id || ("new" as id)
      )}
      onChange={onChange}
      showLabel={false}
      showInvalid={true}
    />
  );
};
