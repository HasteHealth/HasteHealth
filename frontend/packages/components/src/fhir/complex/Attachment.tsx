import fileDownload from "js-file-download";
import React from "react";

import { Attachment } from "@haste-health/fhir-types/r4/types";

import { Button } from "../../base/button";
import { InputContainer } from "../../base/containers";
import { Input } from "../../base/input";
import { EditableProps } from "../types";

const convertFileToBase64 = (file: File): Promise<string> => {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.readAsDataURL(file);

    reader.onload = () => {
      const data = reader.result;
      if (typeof data === "string") {
        resolve(data.split(",")[1]);
      }
    };
    reader.onerror = (error) => reject(error);
  });
};

export type FHIRAttachmentEditableProps = EditableProps<Attachment>;

export const FHIRAttachmentEditable = ({
  value,
  onChange,
  issue,
  label,
}: FHIRAttachmentEditableProps) => {
  return (
    <InputContainer hideBorder label={label} issues={issue ? [issue] : []}>
      <div className="flex">
        <Input
          type="file"
          hideBorder={true}
          onChange={(e) => {
            const file = e.target?.files?.[0];
            if (!file) return;
            convertFileToBase64(file).then((data) => {
              onChange?.call(this, {
                data,
                contentType: file.type,
                size: file.size,
                title: file.name,
                creation: new Date(file.lastModified).toString(),
              } as Attachment);
            });
          }}
        />
        {value && value.data && (
          <Button
            className="ml-1"
            buttonType="secondary"
            buttonSize="medium"
            onClick={() => {
              const data = value.data;
              if (data === undefined) alert("No data to download");
              fileDownload(
                data as string,
                value?.title || "data",
                value?.contentType
              );
            }}
            children="Download"
          />
        )}
      </div>
    </InputContainer>
  );
};
