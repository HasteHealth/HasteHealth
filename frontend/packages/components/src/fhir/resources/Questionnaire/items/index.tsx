import React from "react";

import {
  Attachment,
  base64Binary,
  code,
  QuestionnaireItem,
  QuestionnaireItemAnswerOption,
  QuestionnaireResponseItem,
  QuestionnaireResponseItemAnswer,
  Quantity,
  Reference,
  time,
} from "@haste-health/fhir-types/r4/types";

import { Option, Select } from "../../../../base/select";
import {
  FHIRBooleanEditable,
  FHIRDateEditable,
  FHIRDateTimeEditable,
  FHIRDecimalEditable,
  FHIRIntegerEditable,
  FHIRStringEditable,
  FHIRTimeEditable,
  FHIRUriEditable,
} from "../../../primitives";

export type QuestionnaireItemRendererProps = {
  item: QuestionnaireItem;
  responseItem: QuestionnaireResponseItem;
  answers: QuestionnaireResponseItemAnswer[];
  onAnswerChange: (
    answerIndex: number,
    nextAnswer: QuestionnaireResponseItemAnswer | undefined,
  ) => void;
  onAddAnswer: () => void;
  onRemoveAnswer: (answerIndex: number) => void;
  renderChildren: () => React.ReactNode;
};

export type QuestionnaireItemRenderer = (
  props: QuestionnaireItemRendererProps,
) => React.ReactNode;

function label(item: QuestionnaireItem): string {
  return item.text || item.linkId;
}

function rows(
  item: QuestionnaireItem,
  answers: QuestionnaireResponseItemAnswer[],
) {
  if (item.repeats) {
    return answers.length > 0
      ? answers.map((answer, answerIndex) => ({ answer, answerIndex }))
      : [{ answer: undefined, answerIndex: 0 }];
  }

  return [{ answer: answers[0], answerIndex: 0 }];
}

function header(
  item: QuestionnaireItem,
  onAddAnswer: () => void,
  showAdd: boolean,
) {
  return (
    <div className="flex items-center justify-between gap-2">
      <label className="text-sm font-medium text-slate-800">
        {label(item)}
        {item.required ? <span className="ml-1 text-red-600">*</span> : null}
      </label>
      {showAdd ? (
        <button
          type="button"
          className="rounded border border-slate-300 px-2 py-1 text-xs text-slate-700 hover:bg-slate-50"
          onClick={onAddAnswer}
        >
          Add
        </button>
      ) : null}
    </div>
  );
}

function removeButton(onRemove: () => void) {
  return (
    <button
      type="button"
      className="rounded border border-red-200 px-2 py-1 text-xs text-red-700 hover:bg-red-50"
      onClick={onRemove}
    >
      Remove
    </button>
  );
}

function itemControl(item: QuestionnaireItem): string | undefined {
  const questionnaireItemControlUrl =
    "http://hl7.org/fhir/StructureDefinition/questionnaire-itemControl";

  const extension = (item.extension || []).find(
    (ext) => ext.url === questionnaireItemControlUrl,
  );

  return extension?.valueCodeableConcept?.coding?.[0]?.code;
}

function optionLabel(option: QuestionnaireItemAnswerOption): string {
  let valueKey = Object.keys(option).find((key) => key.startsWith("value"));

  switch (valueKey) {
    case "valueCoding": {
      const coding = option.valueCoding;
      return coding?.display || coding?.code || "Coding option";
    }
    case "valueString": {
      return option.valueString ?? "Option";
    }
    case "valueInteger":
    case "valueDate":
    case "valueDateTime":
    case "valueTime": {
      return String(option[valueKey as keyof QuestionnaireItemAnswerOption]);
    }
    case "valueReference": {
      return (
        option.valueReference?.display ||
        option.valueReference?.reference ||
        "Reference"
      );
    }
    default: {
      return "Option";
    }
  }
}

function optionToAnswer(
  option: QuestionnaireItemAnswerOption,
): QuestionnaireResponseItemAnswer | undefined {
  const valueKey = Object.keys(option).find((key) => key.startsWith("value"));
  if (!valueKey) return undefined;

  return {
    [valueKey]: option[valueKey as keyof QuestionnaireItemAnswerOption],
  };
}

function findSelectedOptionIndex(
  answer: QuestionnaireResponseItemAnswer | undefined,
  options: QuestionnaireItemAnswerOption[],
): number {
  if (!answer) return -1;

  return options.findIndex((option) => {
    const valueKey = Object.keys(option).find((key) => key.startsWith("value"));
    if (!valueKey) return false;

    let value = option[valueKey as keyof QuestionnaireItemAnswerOption];
    if (typeof value === "object") {
      return (
        JSON.stringify(value) ==
        JSON.stringify(
          answer[valueKey as keyof QuestionnaireResponseItemAnswer],
        )
      );
    }

    return answer[valueKey as keyof QuestionnaireResponseItemAnswer] === value;
  });
}

function primitiveRenderer(
  renderControl: (props: {
    item: QuestionnaireItem;
    answer: QuestionnaireResponseItemAnswer | undefined;
    answerIndex: number;
    onAnswerChange: (
      answerIndex: number,
      nextAnswer: QuestionnaireResponseItemAnswer | undefined,
    ) => void;
  }) => React.ReactNode,
): QuestionnaireItemRenderer {
  return ({
    item,
    answers,
    onAnswerChange,
    onAddAnswer,
    onRemoveAnswer,
    renderChildren,
  }) => (
    <div className="space-y-2">
      {header(item, onAddAnswer, Boolean(item.repeats))}
      <div className="space-y-2">
        {rows(item, answers).map(({ answer, answerIndex }) => (
          <div
            key={`${item.linkId}-${answerIndex}`}
            className="flex items-center gap-2"
          >
            <div className="w-full">
              {renderControl({
                item,
                answer,
                answerIndex,
                onAnswerChange,
              })}
            </div>
            {item.repeats && answer
              ? removeButton(() => onRemoveAnswer(answerIndex))
              : null}
          </div>
        ))}
      </div>
      {renderChildren()}
    </div>
  );
}

const GroupRenderer: QuestionnaireItemRenderer = ({ item, renderChildren }) => (
  <div className="space-y-3 rounded-lg border border-slate-200 bg-slate-50 p-4">
    <div className="text-sm font-semibold text-slate-900">{label(item)}</div>
    <div className="space-y-4">{renderChildren()}</div>
  </div>
);

const DisplayRenderer: QuestionnaireItemRenderer = ({
  item,
  renderChildren,
}) => (
  <div className="space-y-2">
    <div className="rounded border border-slate-200 bg-white px-3 py-2 text-sm text-slate-700">
      {label(item)}
    </div>
    {renderChildren()}
  </div>
);

const BooleanRenderer = primitiveRenderer(
  ({ item, answer, answerIndex, onAnswerChange }) => (
    <FHIRBooleanEditable
      value={answer?.valueBoolean}
      label={item.repeats ? `${label(item)} ${answerIndex + 1}` : undefined}
      onChange={
        item.readOnly
          ? undefined
          : (value) =>
              onAnswerChange(answerIndex, { valueBoolean: Boolean(value) })
      }
    />
  ),
);

const ChoiceRenderer: QuestionnaireItemRenderer = ({
  item,
  answers,
  onAnswerChange,
  onAddAnswer,
  onRemoveAnswer,
  renderChildren,
}) => {
  const options = item.answerOption || [];
  const control = itemControl(item);
  const selectOptions: Option[] = options.map((option, index) => ({
    value: index,
    label: optionLabel(option),
  }));

  return (
    <div className="space-y-2">
      {header(item, onAddAnswer, Boolean(item.repeats))}
      <div className="space-y-2">
        {rows(item, answers).map(({ answer, answerIndex }) => {
          const selectedIndex = findSelectedOptionIndex(answer, options);

          return (
            <div
              key={`${item.linkId}-${answerIndex}`}
              className="flex items-center gap-2"
            >
              {control === "radio-button" ? (
                <div className="flex w-full flex-col gap-2 rounded border border-slate-300 p-2">
                  {options.map((option, index) => (
                    <label
                      key={`${item.linkId}-radio-${index}`}
                      className="inline-flex items-center gap-2 text-sm"
                    >
                      <input
                        type="radio"
                        name={`${item.linkId}-${answerIndex}`}
                        disabled={item.readOnly}
                        checked={selectedIndex === index}
                        onChange={() =>
                          onAnswerChange(answerIndex, optionToAnswer(option))
                        }
                      />
                      <span>{optionLabel(option)}</span>
                    </label>
                  ))}
                </div>
              ) : (
                <div className="w-full">
                  <Select
                    options={selectOptions}
                    value={selectedIndex >= 0 ? selectedIndex : undefined}
                    label={
                      item.repeats
                        ? `${label(item)} ${answerIndex + 1}`
                        : undefined
                    }
                    onChange={
                      item.readOnly
                        ? undefined
                        : (selected) => {
                            if (!selected) {
                              onAnswerChange(answerIndex, undefined);
                              return;
                            }
                            const optionIndex = Number(selected.value);
                            if (Number.isNaN(optionIndex)) {
                              onAnswerChange(answerIndex, undefined);
                              return;
                            }
                            const option = options[optionIndex];
                            if (!option) {
                              onAnswerChange(answerIndex, undefined);
                              return;
                            }
                            onAnswerChange(answerIndex, optionToAnswer(option));
                          }
                    }
                  />
                </div>
              )}
              {item.repeats && answer
                ? removeButton(() => onRemoveAnswer(answerIndex))
                : null}
            </div>
          );
        })}
      </div>
      {renderChildren()}
    </div>
  );
};

const OpenChoiceRenderer: QuestionnaireItemRenderer = ({
  item,
  answers,
  onAnswerChange,
  onAddAnswer,
  onRemoveAnswer,
  renderChildren,
}) => {
  const options = item.answerOption || [];
  const selectOptions: Option[] = options.map((option, index) => ({
    value: index,
    label: optionLabel(option),
  }));

  return (
    <div className="space-y-2">
      {header(item, onAddAnswer, Boolean(item.repeats))}
      <div className="space-y-2">
        {rows(item, answers).map(({ answer, answerIndex }) => {
          const selectedIndex = findSelectedOptionIndex(answer, options);
          const currentCustom = answer?.valueString;
          const selectValue =
            selectedIndex >= 0
              ? selectedIndex
              : currentCustom && currentCustom.length > 0
                ? currentCustom
                : undefined;

          return (
            <div
              key={`${item.linkId}-${answerIndex}`}
              className="space-y-2 rounded border border-slate-200 p-2"
            >
              <Select
                options={selectOptions}
                value={selectValue}
                open
                label={
                  item.repeats ? `${label(item)} ${answerIndex + 1}` : undefined
                }
                onChange={
                  item.readOnly
                    ? undefined
                    : (selected) => {
                        if (!selected) {
                          onAnswerChange(answerIndex, undefined);
                          return;
                        }

                        if (typeof selected.value === "number") {
                          const option = options[selected.value];
                          onAnswerChange(answerIndex, optionToAnswer(option));
                          return;
                        }

                        const customValue = String(selected.value);
                        onAnswerChange(
                          answerIndex,
                          customValue.trim().length > 0
                            ? { valueString: customValue }
                            : undefined,
                        );
                      }
                }
              />
              <p className="text-xs text-slate-500">
                Type to select an option or create a custom value.
              </p>
              {item.repeats && answer
                ? removeButton(() => onRemoveAnswer(answerIndex))
                : null}
            </div>
          );
        })}
      </div>
      {renderChildren()}
    </div>
  );
};

function jsonRenderer<T>(
  typeLabel: string,
  read: (answer: QuestionnaireResponseItemAnswer | undefined) => T | undefined,
  write: (parsed: unknown) => QuestionnaireResponseItemAnswer,
): QuestionnaireItemRenderer {
  return ({
    item,
    answers,
    onAnswerChange,
    onAddAnswer,
    onRemoveAnswer,
    renderChildren,
  }) => (
    <div className="space-y-2">
      {header(item, onAddAnswer, Boolean(item.repeats))}
      <div className="space-y-2">
        {rows(item, answers).map(({ answer, answerIndex }) => (
          <div key={`${item.linkId}-${answerIndex}`} className="flex gap-2">
            <textarea
              className="h-24 w-full rounded-md border border-slate-300 px-3 py-2 font-mono text-xs"
              readOnly={item.readOnly}
              value={JSON.stringify(read(answer) ?? {}, null, 2)}
              placeholder={`${typeLabel} JSON`}
              onChange={(event) => {
                const raw = event.target.value.trim();
                if (raw.length === 0) {
                  onAnswerChange(answerIndex, undefined);
                  return;
                }

                try {
                  onAnswerChange(answerIndex, write(JSON.parse(raw)));
                } catch {
                  // Keep editing state; wait for valid JSON.
                }
              }}
            />
            {item.repeats && answer
              ? removeButton(() => onRemoveAnswer(answerIndex))
              : null}
          </div>
        ))}
      </div>
      {renderChildren()}
    </div>
  );
}

const DecimalRenderer = primitiveRenderer(
  ({ item, answer, answerIndex, onAnswerChange }) => (
    <FHIRDecimalEditable
      label={item.repeats ? `${label(item)} ${answerIndex + 1}` : undefined}
      value={answer?.valueDecimal}
      onChange={
        item.readOnly
          ? undefined
          : (value) =>
              onAnswerChange(
                answerIndex,
                value === undefined ? undefined : { valueDecimal: value },
              )
      }
    />
  ),
);

const IntegerRenderer = primitiveRenderer(
  ({ item, answer, answerIndex, onAnswerChange }) => (
    <FHIRIntegerEditable
      label={item.repeats ? `${label(item)} ${answerIndex + 1}` : undefined}
      value={answer?.valueInteger}
      onChange={
        item.readOnly
          ? undefined
          : (value) =>
              onAnswerChange(
                answerIndex,
                value === undefined ? undefined : { valueInteger: value },
              )
      }
    />
  ),
);

const DateRenderer = primitiveRenderer(
  ({ item, answer, answerIndex, onAnswerChange }) => (
    <FHIRDateEditable
      label={item.repeats ? `${label(item)} ${answerIndex + 1}` : undefined}
      value={answer?.valueDate}
      onChange={
        item.readOnly
          ? undefined
          : (value) =>
              onAnswerChange(
                answerIndex,
                value === undefined ? undefined : { valueDate: value },
              )
      }
    />
  ),
);

const DateTimeRenderer = primitiveRenderer(
  ({ item, answer, answerIndex, onAnswerChange }) => (
    <FHIRDateTimeEditable
      label={item.repeats ? `${label(item)} ${answerIndex + 1}` : undefined}
      value={answer?.valueDateTime}
      onChange={
        item.readOnly
          ? undefined
          : (value) =>
              onAnswerChange(
                answerIndex,
                value === undefined ? undefined : { valueDateTime: value },
              )
      }
    />
  ),
);

const TimeRenderer = primitiveRenderer(
  ({ item, answer, answerIndex, onAnswerChange }) => (
    <FHIRTimeEditable
      label={item.repeats ? `${label(item)} ${answerIndex + 1}` : undefined}
      value={answer?.valueTime}
      onChange={
        item.readOnly
          ? undefined
          : (value) =>
              onAnswerChange(
                answerIndex,
                value === undefined
                  ? undefined
                  : { valueTime: value as unknown as time },
              )
      }
    />
  ),
);

const StringRenderer = primitiveRenderer(
  ({ item, answer, answerIndex, onAnswerChange }) => (
    <FHIRStringEditable
      label={item.repeats ? `${label(item)} ${answerIndex + 1}` : undefined}
      value={answer?.valueString}
      disabled={Boolean(item.readOnly)}
      onChange={
        item.readOnly
          ? undefined
          : (value = "") => {
              onAnswerChange(
                answerIndex,
                value.trim().length > 0 ? { valueString: value } : undefined,
              );
            }
      }
    />
  ),
);

const TextRenderer = StringRenderer;

const UrlRenderer = primitiveRenderer(
  ({ item, answer, answerIndex, onAnswerChange }) => (
    <FHIRUriEditable
      label={item.repeats ? `${label(item)} ${answerIndex + 1}` : undefined}
      value={answer?.valueUri}
      disabled={Boolean(item.readOnly)}
      onChange={
        item.readOnly
          ? undefined
          : (value) =>
              onAnswerChange(
                answerIndex,
                value && value.trim().length > 0
                  ? { valueUri: value }
                  : undefined,
              )
      }
    />
  ),
);

const AttachmentRenderer: QuestionnaireItemRenderer = ({
  item,
  answers,
  onAnswerChange,
  onAddAnswer,
  onRemoveAnswer,
  renderChildren,
}) => (
  <div className="space-y-2">
    {header(item, onAddAnswer, Boolean(item.repeats))}
    <div className="space-y-2">
      {rows(item, answers).map(({ answer, answerIndex }) => (
        <div
          key={`${item.linkId}-${answerIndex}`}
          className="flex items-center gap-2"
        >
          <input
            type="file"
            className="w-full rounded-md border border-slate-300 px-3 py-2 text-sm"
            disabled={item.readOnly}
            onChange={(event) => {
              const file = event.target.files?.[0];
              if (!file) {
                onAnswerChange(answerIndex, undefined);
                return;
              }

              const reader = new FileReader();
              reader.onload = () => {
                const loaded = reader.result;
                if (typeof loaded !== "string") return;

                const attachment: Attachment = {
                  contentType: (file.type || undefined) as code | undefined,
                  title: file.name,
                  data: loaded as unknown as base64Binary,
                };
                onAnswerChange(answerIndex, { valueAttachment: attachment });
              };
              reader.readAsDataURL(file);
            }}
          />
          {item.repeats && answer
            ? removeButton(() => onRemoveAnswer(answerIndex))
            : null}
        </div>
      ))}
    </div>
    {renderChildren()}
  </div>
);

const ReferenceRenderer = jsonRenderer<Reference>(
  "Reference",
  (answer) => answer?.valueReference,
  (parsed) => ({ valueReference: parsed as Reference }),
);

const QuantityRenderer = jsonRenderer<Quantity>(
  "Quantity",
  (answer) => answer?.valueQuantity,
  (parsed) => ({ valueQuantity: parsed as Quantity }),
);

export const questionnaireItemRenderers: Record<
  string,
  QuestionnaireItemRenderer
> = {
  group: GroupRenderer,
  display: DisplayRenderer,
  boolean: BooleanRenderer,
  decimal: DecimalRenderer,
  integer: IntegerRenderer,
  date: DateRenderer,
  dateTime: DateTimeRenderer,
  time: TimeRenderer,
  string: StringRenderer,
  text: TextRenderer,
  url: UrlRenderer,
  uri: UrlRenderer,
  choice: ChoiceRenderer,
  "open-choice": OpenChoiceRenderer,
  attachment: AttachmentRenderer,
  reference: ReferenceRenderer,
  quantity: QuantityRenderer,
};
