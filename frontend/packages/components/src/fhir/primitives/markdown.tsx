import { markdown, markdownLanguage } from "@codemirror/lang-markdown";
import React from "react";

import { markdown as fmarkdown } from "@haste-health/fhir-types/r4/types";

import { CodeMirror } from "../../base";
import { InputContainer } from "../../base/containers";
import { EditableProps } from "../types";
import { primitiveStackClass } from "./layout";

export type FHIRMarkdownEditableProps = EditableProps<fmarkdown>;

const extensions = [markdown({ base: markdownLanguage })];

export const FHIRMarkdownEditable = ({
  onChange,
  value,
  issue,
  label,
}: FHIRMarkdownEditableProps) => {
  return (
    <InputContainer label={label} issues={issue ? [issue] : []}>
      <div className={primitiveStackClass}>
        <CodeMirror
          extensions={extensions}
          value={value}
          onChange={(v) => onChange?.call(this, v as fmarkdown | undefined)}
        />
      </div>
    </InputContainer>
  );
};
