import { markdown, markdownLanguage } from "@codemirror/lang-markdown";
import React from "react";

import {
  Annotation,
  markdown as fmarkdown,
} from "@haste-health/fhir-types/r4/types";

import { CodeMirror } from "../../base";
import { InputContainer } from "../../base/containers";
import { EditableProps } from "../types";

export type FHIRMarkdownEditableProps = EditableProps<Annotation>;

const extensions = [markdown({ base: markdownLanguage })];

export const FHIRAnnotationEditable = ({
  onChange,
  value,
  issue,
  label,
}: FHIRMarkdownEditableProps) => {
  return (
    <InputContainer label={label} issues={issue ? [issue] : []}>
      <CodeMirror
        extensions={extensions}
        value={value?.text}
        onChange={(text) =>
          onChange?.call(this, { ...value, text: text as fmarkdown })
        }
      />
    </InputContainer>
  );
};
