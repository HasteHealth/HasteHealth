/* eslint @typescript-eslint/no-explicit-any: 0 */
import { json } from "@codemirror/lang-json";
import classNames from "classnames";
import { basicSetup } from "codemirror";
import React, { useEffect, useMemo, useState } from "react";

import { OperationDefinitionParameter } from "@haste-health/fhir-types/r4/types";

import { CodeMirror } from "../../base/codemirror";
import { JSONTextEditor } from "../../base/JSONTextEditor";
import * as Complex from "../complex";
import { FHIRCodeableConceptReadOnly } from "../complex/CodeableConceptReadOnly";
import { FHIRQuantityEditable } from "../complex/Quantity";
import { FHIRQuantityReadOnly } from "../complex/QuantityReadOnly";
import { FHIRTimingReadOnly } from "../complex/TimingReadOnly";
import * as Primitives from "../primitives";
import { JSONResourceEditor } from "../resources/JSONResourceEditor";
import { ClientProps } from "../types";
import { isResourceParameter } from "./parameters";

const jsonExtensions = [basicSetup, json()];

/** Types whose value is a quantity, edited and shown as one. */
const QUANTITY_TYPES = new Set([
  "Quantity",
  "SimpleQuantity",
  "MoneyQuantity",
  "Age",
  "Count",
  "Distance",
  "Duration",
]);

/**
 * Types whose editor binds the value straight to an input. An unset value is
 * given to them as `""`, so the input stays controlled while it is empty.
 * Dates are left out: their editors format the value with dayjs.
 */
const INPUT_TYPES = new Set([
  "string",
  "id",
  "code",
  "uri",
  "url",
  "canonical",
  "oid",
  "uuid",
  "markdown",
  "base64Binary",
  "integer",
  "unsignedInt",
  "positiveInt",
  "decimal",
  "time",
]);

type ValueEditorProps = { value: any; onChange: (value: unknown) => void };

/** Editors for types that need nothing but the value. */
const VALUE_EDITORS = new Map<string, React.ComponentType<ValueEditorProps>>([
  ["string", Primitives.FHIRStringEditable],
  ["id", Primitives.FHIRIdEditable],
  ["uri", Primitives.FHIRUriEditable],
  ["url", Primitives.FHIRUrlEditable],
  ["canonical", Primitives.FHIRCanonicalEditable],
  ["oid", Primitives.FHIROIDEditable],
  ["uuid", Primitives.FHIRUUIDEditable],
  ["markdown", Primitives.FHIRMarkdownEditable],
  ["base64Binary", Primitives.FHIRBase64BinaryEditable],
  ["integer", Primitives.FHIRIntegerEditable],
  ["unsignedInt", Primitives.FHIRUnsignedIntegerEditable],
  ["positiveInt", Primitives.FHIRPositiveIntegerEditable],
  ["decimal", Primitives.FHIRDecimalEditable],
  ["date", Primitives.FHIRDateEditable],
  ["dateTime", Primitives.FHIRDateTimeEditable],
  ["instant", Primitives.FHIRInstantEditable],
  ["time", Primitives.FHIRTimeEditable],
  ["Period", Complex.FHIRPeriodEditable],
  ["Range", Complex.FHIRRangeEditable],
  ["Ratio", Complex.FHIRRatioEditable],
  ["HumanName", Complex.FHIRHumanNameEditable],
  ["Address", Complex.FHIRAddressEditable],
  ["Annotation", Complex.FHIRAnnotationEditable],
  ["Attachment", Complex.FHIRAttachmentEditable],
  ["Expression", Complex.FHIRExpressionEditable],
  ["Timing", Complex.FHIRTimingEditable],
  ["SampledData", Complex.FHIRSampledDataEditable],
]);

/** Editors for types that also look things up through the client. */
const CLIENT_EDITORS = new Map<
  string,
  React.ComponentType<ValueEditorProps & ClientProps>
>([
  ["Coding", Complex.FHIRCodingEditable],
  ["CodeableConcept", Complex.FhirCodeableConceptEditable],
  ["Identifier", Complex.FHIRIdentifierEditable],
  ["ContactPoint", Complex.FHIRContactPointEditable],
  ["ContactDetail", Complex.FHIRContactDetailEditable],
  ["Money", Complex.FHIRMoneyEditable],
  ["Signature", Complex.FHIRSignatureEditable],
]);

export type ParameterValueEditorProps = ClientProps & {
  /** The parameter the value is for: its `type` picks the editor. */
  definition: OperationDefinitionParameter;
  value: unknown;
  onChange: (value: unknown) => void;
};

/** `true` and `false` as two buttons, so a boolean can also be left unset. */
function BooleanToggle({
  value,
  onChange,
}: Readonly<{ value: unknown; onChange: (value: boolean) => void }>) {
  return (
    <div className="inline-flex overflow-hidden rounded-md border border-slate-300 text-xs">
      {[true, false].map((option) => (
        <button
          key={String(option)}
          type="button"
          aria-pressed={value === option}
          className={classNames("px-3 py-1 font-medium", {
            "bg-brand-600 text-white": value === option,
            "bg-white text-slate-700 hover:bg-slate-50": value !== option,
          })}
          onClick={() => onChange(option)}
        >
          {String(option)}
        </button>
      ))}
    </div>
  );
}

/**
 * A value of a type there is no form for (`Dosage`, `Meta`, ...) as JSON. The
 * text is the source of truth while typing; the parsed value goes up only when
 * it is valid, and clearing the text unsets it.
 */
function JSONValueEditor({
  value,
  onChange,
}: Readonly<{ value: unknown; onChange: (value: unknown) => void }>) {
  const incoming = useMemo(
    () => (value === undefined ? "" : JSON.stringify(value, null, 2)),
    [value],
  );
  const [text, setText] = useState(incoming);
  const [lastSent, setLastSent] = useState<string>();

  useEffect(() => {
    if (incoming === lastSent) return;
    setText(incoming);
  }, [incoming, lastSent]);

  return (
    <div className="flex h-36 flex-col">
      <JSONTextEditor
        value={text}
        hint="JSON value"
        onChange={(next) => {
          setText(next);
          if (next.trim() === "") {
            setLastSent("");
            onChange(undefined);
            return;
          }
          try {
            const parsed = JSON.parse(next);
            setLastSent(JSON.stringify(parsed, null, 2));
            onChange(parsed);
          } catch {
            // Invalid mid edit is normal; the next valid keystroke updates.
          }
        }}
      />
    </div>
  );
}

/**
 * Edits one parameter value with the editor for its type: an input for a
 * primitive, the data type's form for a complex type, and JSON for a resource
 * or a type without a form.
 */
export function ParameterValueEditor({
  definition,
  value,
  onChange,
  client,
  fhirVersion,
}: Readonly<ParameterValueEditorProps>) {
  if (isResourceParameter(definition)) {
    return (
      <div className="flex h-56 flex-col">
        <JSONResourceEditor
          resource={value as any}
          onChange={(resource) => onChange(resource)}
        />
      </div>
    );
  }

  const type = definition.type ?? "";
  const shared = {
    value: (INPUT_TYPES.has(type) ? (value ?? "") : value) as any,
    onChange,
  };
  const clientProps = { client, fhirVersion };

  if (QUANTITY_TYPES.has(type)) {
    return <FHIRQuantityEditable {...shared} {...clientProps} />;
  }

  switch (type) {
    case "boolean":
      return <BooleanToggle value={value} onChange={onChange} />;
    case "code":
      return definition.binding?.valueSet ? (
        <Primitives.FHIRCodeEditable
          {...shared}
          {...clientProps}
          open={true}
          system={definition.binding.valueSet}
        />
      ) : (
        <Primitives.FHIRStringEditable {...shared} />
      );
    case "Reference":
      return (
        <Complex.FHIRReferenceEditable
          {...shared}
          {...clientProps}
          resourceTypesAllowed={definition.targetProfile?.map(
            (profile) => profile.split("/").pop() as any,
          )}
        />
      );
  }

  const ValueEditor = VALUE_EDITORS.get(type);
  if (ValueEditor) return <ValueEditor {...shared} />;
  const ClientEditor = CLIENT_EDITORS.get(type);
  if (ClientEditor) return <ClientEditor {...shared} {...clientProps} />;
  return <JSONValueEditor value={value} onChange={onChange} />;
}

/** A resource or other structure as read-only JSON. */
export function JSONValueView({ value }: Readonly<{ value: unknown }>) {
  const resource = value as { resourceType?: string; id?: string };
  return (
    <div className="overflow-hidden rounded-md border border-slate-200">
      {resource?.resourceType && (
        <div className="border-b border-slate-200 bg-slate-50 px-2 py-1 font-mono text-xs text-slate-600">
          {resource.resourceType}
          {resource.id ? `/${resource.id}` : ""}
        </div>
      )}
      <div className="max-h-72 overflow-auto">
        <CodeMirror
          readOnly
          extensions={jsonExtensions}
          value={JSON.stringify(value, null, 2)}
          theme={{ "&": { width: "100%" } }}
        />
      </div>
    </div>
  );
}

/**
 * Shows one parameter value read-only: the data type's display where there is
 * one, the text of a primitive, and JSON for a resource or anything else.
 */
export function ParameterValueView({
  type,
  value,
}: Readonly<{ type?: string; value: unknown }>) {
  if (value === undefined || value === null) {
    return <span className="text-sm text-slate-400">No value</span>;
  }

  if (
    typeof value === "string" ||
    typeof value === "number" ||
    typeof value === "boolean"
  ) {
    return (
      <span className="break-all font-mono text-sm text-slate-800">
        {String(value)}
      </span>
    );
  }

  const shared = { value: value as any };
  if (QUANTITY_TYPES.has(type ?? "")) {
    return <FHIRQuantityReadOnly {...shared} />;
  }

  switch (type) {
    case "Coding":
      return <Complex.FHIRCodingReadOnly {...shared} />;
    case "CodeableConcept":
      return <FHIRCodeableConceptReadOnly {...shared} />;
    case "Reference":
      return <Complex.FHIRReferenceReadOnly {...shared} />;
    case "Identifier":
      return <Complex.FHIRIdentifierReadOnly {...shared} />;
    case "HumanName":
      return <Complex.FHIRHumanNameReadOnly {...shared} />;
    case "Address":
      return <Complex.FHIRAddressReadOnly {...shared} />;
    case "Period":
      return <Complex.FHIRPeriodReadOnly {...shared} />;
    case "Range":
      return <Complex.FHIRRangeReadOnly {...shared} />;
    case "Timing":
      return <FHIRTimingReadOnly {...shared} />;
    default:
      return <JSONValueView value={value} />;
  }
}
