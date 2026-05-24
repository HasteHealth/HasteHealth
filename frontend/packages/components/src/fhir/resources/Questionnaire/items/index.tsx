import React from "react";

import {
  Attachment,
  base64Binary,
  code,
  date,
  dateTime,
  decimal,
  integer,
  QuestionnaireItem,
  QuestionnaireItemAnswerOption,
  QuestionnaireResponseItem,
  QuestionnaireResponseItemAnswer,
  Quantity,
  Reference,
  time,
  uri,
} from "@haste-health/fhir-types/r4/types";

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
  if (option.valueCoding) {
    return (
      option.valueCoding.display || option.valueCoding.code || "Coding option"
    );
  }
  if (option.valueString !== undefined) return option.valueString;
  if (option.valueInteger !== undefined) return String(option.valueInteger);
  if (option.valueDate !== undefined) return String(option.valueDate);
  if (option.valueTime !== undefined) return String(option.valueTime);
  if (option.valueReference) {
    return (
      option.valueReference.display ||
      option.valueReference.reference ||
      "Reference"
    );
  }
  return "Option";
}

function optionToAnswer(
  option: QuestionnaireItemAnswerOption,
): QuestionnaireResponseItemAnswer | undefined {
  if (option.valueCoding) return { valueCoding: option.valueCoding };
  if (option.valueString !== undefined)
    return { valueString: option.valueString };
  if (option.valueInteger !== undefined)
    return { valueInteger: option.valueInteger };
  if (option.valueDate !== undefined) return { valueDate: option.valueDate };
  if (option.valueTime !== undefined) return { valueTime: option.valueTime };
  if (option.valueReference) return { valueReference: option.valueReference };
  return undefined;
}

function findSelectedOptionIndex(
  answer: QuestionnaireResponseItemAnswer | undefined,
  options: QuestionnaireItemAnswerOption[],
): number {
  if (!answer) return -1;

  return options.findIndex((option) => {
    if (answer.valueCoding && option.valueCoding) {
      return option.valueCoding.code === answer.valueCoding.code;
    }
    if (answer.valueString !== undefined && option.valueString !== undefined) {
      return option.valueString === answer.valueString;
    }
    if (
      answer.valueInteger !== undefined &&
      option.valueInteger !== undefined
    ) {
      return option.valueInteger === answer.valueInteger;
    }
    if (answer.valueDate !== undefined && option.valueDate !== undefined) {
      return option.valueDate === answer.valueDate;
    }
    if (answer.valueTime !== undefined && option.valueTime !== undefined) {
      return option.valueTime === answer.valueTime;
    }
    if (answer.valueReference && option.valueReference) {
      return (
        option.valueReference.reference === answer.valueReference.reference
      );
    }
    return false;
  });
}

function primitiveInputRenderer(
  htmlType: string,
  read: (answer: QuestionnaireResponseItemAnswer | undefined) => string,
  write: (raw: string) => QuestionnaireResponseItemAnswer | undefined,
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
            <input
              type={htmlType}
              className="w-full rounded-md border border-slate-300 px-3 py-2 text-sm"
              value={read(answer)}
              readOnly={item.readOnly}
              onChange={(event) =>
                onAnswerChange(answerIndex, write(event.target.value))
              }
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

const BooleanRenderer: QuestionnaireItemRenderer = ({
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
          <label className="inline-flex items-center gap-2 text-sm text-slate-700">
            <input
              type="checkbox"
              checked={Boolean(answer?.valueBoolean)}
              disabled={item.readOnly}
              onChange={(event) =>
                onAnswerChange(answerIndex, {
                  valueBoolean: event.target.checked,
                })
              }
            />
            <span>{Boolean(answer?.valueBoolean) ? "Yes" : "No"}</span>
          </label>
          {item.repeats && answer
            ? removeButton(() => onRemoveAnswer(answerIndex))
            : null}
        </div>
      ))}
    </div>
    {renderChildren()}
  </div>
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
                <select
                  className="w-full rounded-md border border-slate-300 px-3 py-2 text-sm"
                  value={selectedIndex >= 0 ? String(selectedIndex) : ""}
                  disabled={item.readOnly}
                  onChange={(event) => {
                    if (event.target.value === "") {
                      onAnswerChange(answerIndex, undefined);
                      return;
                    }

                    const option =
                      options[Number.parseInt(event.target.value, 10)];
                    onAnswerChange(answerIndex, optionToAnswer(option));
                  }}
                >
                  <option value="">Select an option</option>
                  {options.map((option, index) => (
                    <option
                      key={`${item.linkId}-option-${index}`}
                      value={String(index)}
                    >
                      {optionLabel(option)}
                    </option>
                  ))}
                </select>
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

  return (
    <div className="space-y-2">
      {header(item, onAddAnswer, Boolean(item.repeats))}
      <div className="space-y-2">
        {rows(item, answers).map(({ answer, answerIndex }) => {
          const selectedIndex = findSelectedOptionIndex(answer, options);
          const currentString = answer?.valueString || "";

          return (
            <div
              key={`${item.linkId}-${answerIndex}`}
              className="space-y-2 rounded border border-slate-200 p-2"
            >
              {options.length > 0 ? (
                <select
                  className="w-full rounded-md border border-slate-300 px-3 py-2 text-sm"
                  value={selectedIndex >= 0 ? String(selectedIndex) : "custom"}
                  disabled={item.readOnly}
                  onChange={(event) => {
                    if (event.target.value === "custom") {
                      onAnswerChange(
                        answerIndex,
                        currentString.trim().length > 0
                          ? { valueString: currentString }
                          : undefined,
                      );
                      return;
                    }

                    const option =
                      options[Number.parseInt(event.target.value, 10)];
                    onAnswerChange(answerIndex, optionToAnswer(option));
                  }}
                >
                  {options.map((option, index) => (
                    <option
                      key={`${item.linkId}-open-option-${index}`}
                      value={String(index)}
                    >
                      {optionLabel(option)}
                    </option>
                  ))}
                  <option value="custom">Custom value</option>
                </select>
              ) : null}
              <input
                type="text"
                className="w-full rounded-md border border-slate-300 px-3 py-2 text-sm"
                value={currentString}
                readOnly={item.readOnly}
                placeholder="Enter custom value"
                onChange={(event) => {
                  const next = event.target.value;
                  onAnswerChange(
                    answerIndex,
                    next.trim().length > 0 ? { valueString: next } : undefined,
                  );
                }}
              />
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
              value={JSON.stringify(read(answer) || {}, null, 2)}
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

const DecimalRenderer = primitiveInputRenderer(
  "number",
  (answer) =>
    answer?.valueDecimal === undefined || answer.valueDecimal === null
      ? ""
      : String(answer.valueDecimal),
  (raw) => {
    if (raw.trim().length === 0) return undefined;
    const parsed = Number.parseFloat(raw);
    return Number.isNaN(parsed)
      ? undefined
      : { valueDecimal: parsed as unknown as decimal };
  },
);

const IntegerRenderer = primitiveInputRenderer(
  "number",
  (answer) =>
    answer?.valueInteger === undefined || answer.valueInteger === null
      ? ""
      : String(answer.valueInteger),
  (raw) => {
    if (raw.trim().length === 0) return undefined;
    const parsed = Number.parseInt(raw, 10);
    return Number.isNaN(parsed)
      ? undefined
      : { valueInteger: parsed as unknown as integer };
  },
);

const DateRenderer = primitiveInputRenderer(
  "date",
  (answer) => (answer?.valueDate ? String(answer.valueDate) : ""),
  (raw) =>
    raw.trim().length === 0 ? undefined : { valueDate: raw as unknown as date },
);

const DateTimeRenderer = primitiveInputRenderer(
  "datetime-local",
  (answer) => (answer?.valueDateTime ? String(answer.valueDateTime) : ""),
  (raw) =>
    raw.trim().length === 0
      ? undefined
      : { valueDateTime: raw as unknown as dateTime },
);

const TimeRenderer = primitiveInputRenderer(
  "time",
  (answer) => (answer?.valueTime ? String(answer.valueTime) : ""),
  (raw) =>
    raw.trim().length === 0 ? undefined : { valueTime: raw as unknown as time },
);

const StringRenderer = primitiveInputRenderer(
  "text",
  (answer) => answer?.valueString || "",
  (raw) => (raw.trim().length === 0 ? undefined : { valueString: raw }),
);

const TextRenderer = primitiveInputRenderer(
  "text",
  (answer) => answer?.valueString || "",
  (raw) => (raw.trim().length === 0 ? undefined : { valueString: raw }),
);

const UrlRenderer = primitiveInputRenderer(
  "url",
  (answer) => (answer?.valueUri ? String(answer.valueUri) : ""),
  (raw) =>
    raw.trim().length === 0 ? undefined : { valueUri: raw as unknown as uri },
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
  choice: ChoiceRenderer,
  "open-choice": OpenChoiceRenderer,
  attachment: AttachmentRenderer,
  reference: ReferenceRenderer,
  quantity: QuantityRenderer,
};
