import React from "react";

import { Meta } from "@haste-health/fhir-types/r4/types";

import { ClientProps } from "../types";

export interface FHIRMetaReadonlyProps extends ClientProps {
  value?: Meta;
}

function MetaField({
  label,
  value,
}: Readonly<{
  label: string;
  value: React.ReactNode;
}>) {
  return (
    <div className="min-w-0 space-y-0.5">
      <dt className="text-xs font-medium text-slate-500">{label}</dt>
      <dd className="truncate text-sm text-slate-800">
        {value ?? <span className="text-slate-400">—</span>}
      </dd>
    </div>
  );
}

function formatInstant(raw: string | undefined): string | undefined {
  if (!raw) return undefined;
  try {
    return new Intl.DateTimeFormat(undefined, {
      dateStyle: "medium",
      timeStyle: "short",
    }).format(new Date(raw));
  } catch {
    return raw;
  }
}

export const FHIRMetaReadOnly = ({ value }: FHIRMetaReadonlyProps) => {
  const author = value?.extension?.find(
    (e) => e.url === "https://haste.health/author",
  )?.valueReference?.reference;

  const tags =
    value?.tag && value.tag.length > 0
      ? value.tag.map((t) => t.display ?? t.code ?? t.system).join(", ")
      : undefined;

  const profiles =
    value?.profile && value.profile.length > 0
      ? value.profile.join(", ")
      : undefined;

  return (
    <div className="rounded-md border border-slate-200 bg-gray-50 px-3 py-2.5">
      <dl className="grid grid-cols-1 gap-x-4 gap-y-2.5 sm:grid-cols-2 lg:grid-cols-3">
        <MetaField label="Version ID" value={value?.versionId} />
        <MetaField
          label="Last Updated"
          value={formatInstant(value?.lastUpdated)}
        />
        <MetaField label="Author" value={author} />
        {tags && <MetaField label="Tags" value={tags} />}
        {profiles && (
          <MetaField
            label={profiles.includes(",") ? "Profiles" : "Profile"}
            value={profiles}
          />
        )}
      </dl>
    </div>
  );
};
