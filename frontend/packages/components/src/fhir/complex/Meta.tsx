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
      <dt className="font-mono text-[10px] uppercase tracking-wider text-[var(--ci-muted)]">{label}</dt>
      <dd className="truncate font-mono text-xs text-[var(--ci-ink)]">
        {value ?? <span className="text-[var(--ci-muted)]">—</span>}
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
  const authorExtension = value?.extension?.find(
    (e) => e.url === "https://haste.health/author",
  );
  // A canonical (`https://haste.health/fhir/Membership/<id>`); versions
  // written before that hold a Reference.
  const author =
    authorExtension?.valueCanonical?.replace(
      "https://haste.health/fhir/",
      "",
    ) ?? authorExtension?.valueReference?.reference;

  const tags =
    value?.tag && value.tag.length > 0
      ? value.tag.map((t) => t.display ?? t.code ?? t.system).join(", ")
      : undefined;

  const profiles =
    value?.profile && value.profile.length > 0
      ? value.profile.join(", ")
      : undefined;

  return (
    <div className="rounded-none border border-[var(--ci-line)] bg-[var(--ci-surface-raised)] px-3 py-2.5">
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
