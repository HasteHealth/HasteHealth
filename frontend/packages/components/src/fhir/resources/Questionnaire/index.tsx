import React, { useEffect, useMemo } from "react";
import { useAtom } from "jotai";
import { atomWithImmer } from "jotai-immer";

import {
  canonical,
  code,
  Questionnaire,
  QuestionnaireItem,
  QuestionnaireResponse,
  QuestionnaireResponseItem,
  QuestionnaireResponseItemAnswer,
} from "@haste-health/fhir-types/r4/types";

import { QuestionnaireItemRenderer, questionnaireItemRenderers } from "./items";

export type FHIRQuestionnaireRendererProps = {
  schema: Questionnaire;
  value?: QuestionnaireResponse;
  onChange?: (qr: QuestionnaireResponse) => void;
};

function buildResponseItems(
  items: QuestionnaireItem[] | undefined,
): QuestionnaireResponseItem[] {
  return (items || []).map((item) => ({
    linkId: item.linkId,
    text: item.text,
    item: buildResponseItems(item.item),
  }));
}

function normalizeQuestionnaireResponse(
  schema: Questionnaire,
  value?: QuestionnaireResponse,
): QuestionnaireResponse {
  const base: QuestionnaireResponse =
    value ??
    ({
      resourceType: "QuestionnaireResponse",
    } as QuestionnaireResponse);

  if (!base.status) {
    base.status = "in-progress" as unknown as code;
  }

  if (!base.questionnaire && schema.url) {
    base.questionnaire = schema.url as unknown as canonical;
  }

  if (!base.item || base.item.length === 0) {
    base.item = buildResponseItems(schema.item);
  }

  return base;
}

function getResponseItemAtPath(
  items: QuestionnaireResponseItem[] | undefined,
  path: number[],
): QuestionnaireResponseItem | undefined {
  if (!items) return undefined;

  let cursor: QuestionnaireResponseItem | undefined;
  let list = items;
  for (const index of path) {
    cursor = list[index];
    if (!cursor) return undefined;
    list = cursor.item || [];
  }

  return cursor;
}

const UnsupportedItemRenderer: QuestionnaireItemRenderer = ({ item }) => (
  <div className="rounded border border-amber-200 bg-amber-50 px-3 py-2 text-xs text-amber-800">
    Unsupported Questionnaire item type: {item.type}
  </div>
);

export function FHIRQuestionnaireRenderer({
  schema,
  value,
  onChange,
}: Readonly<FHIRQuestionnaireRendererProps>) {
  const initial = useMemo(
    () => normalizeQuestionnaireResponse(schema, value),
    [schema, value],
  );

  const qrAtom = useMemo(() => {
    return atomWithImmer<QuestionnaireResponse>(initial);
  }, []);

  const [qr, setQr] = useAtom(qrAtom);

  useEffect(() => {
    onChange?.(qr);
  }, [qr, onChange]);

  function mutateResponseItem(
    path: number[],
    updater: (target: QuestionnaireResponseItem) => void,
  ) {
    setQr((draft) => {
      const target = getResponseItemAtPath(draft.item, path);
      if (!target) return;
      updater(target);
    });
  }

  function renderItem(
    itemSchema: QuestionnaireItem,
    responseItem: QuestionnaireResponseItem,
    path: number[],
  ): React.ReactNode {
    const renderer =
      questionnaireItemRenderers[itemSchema.type] || UnsupportedItemRenderer;

    return renderer({
      item: itemSchema,
      responseItem,
      answers: responseItem.answer || [],
      onAnswerChange: (
        answerIndex: number,
        nextAnswer: QuestionnaireResponseItemAnswer | undefined,
      ) => {
        mutateResponseItem(path, (target) => {
          const answers = [...(target.answer || [])];
          if (nextAnswer) {
            answers[answerIndex] = nextAnswer;
          } else {
            answers[answerIndex] = {};
          }

          const cleaned = answers.filter((answer) =>
            Boolean(answer && Object.keys(answer).length > 0),
          );

          target.answer = cleaned.length > 0 ? cleaned : undefined;
        });
      },
      onAddAnswer: () => {
        mutateResponseItem(path, (target) => {
          target.answer = [...(target.answer || []), {}];
        });
      },
      onRemoveAnswer: (answerIndex: number) => {
        mutateResponseItem(path, (target) => {
          const filtered = (target.answer || []).filter(
            (_, idx) => idx !== answerIndex,
          );
          target.answer = filtered.length > 0 ? filtered : undefined;
        });
      },
      renderChildren: () => (
        <div className="space-y-4">
          {(itemSchema.item || []).map((childItem, childIndex) => {
            const childResponse = (responseItem.item || [])[childIndex] || {
              linkId: childItem.linkId,
              text: childItem.text,
              item: buildResponseItems(childItem.item),
            };

            return (
              <div key={childItem.linkId}>
                {renderItem(childItem, childResponse, [...path, childIndex])}
              </div>
            );
          })}
        </div>
      ),
    });
  }

  return (
    <form className="space-y-4">
      {(schema.item || []).map((item, index) => {
        const responseItem = (qr.item || [])[index] || {
          linkId: item.linkId,
          text: item.text,
          item: buildResponseItems(item.item),
        };

        return (
          <div key={item.linkId}>{renderItem(item, responseItem, [index])}</div>
        );
      })}
    </form>
  );
}
