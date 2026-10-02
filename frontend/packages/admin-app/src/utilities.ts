import { OperationOutcome } from "@haste-health/fhir-types/r4/types";
import { ProjectId, TenantId } from "@haste-health/jwt/types";

export function deriveTenantId(): TenantId {
  const host = window.location.host;
  const tenantID = host.split(".")[0]?.split("_")[0];

  return tenantID as TenantId;
}

export function deriveProjectId(): ProjectId {
  const host = window.location.host;
  const projectId = host.split(".")[0]?.split("_")[1];

  return projectId as ProjectId;
}

export function fhirResourceDocsUrl(resourceType: string): string {
  return `https://haste.health/docs/reference/fhir/model/resources/${resourceType}`;
}

export function getErrorMessage(error: any): string {
  if ("response" in error) {
    const message = (error.response.body as OperationOutcome).issue
      .map((issue) => issue.diagnostics)
      .join("\n");

    return message;
  }
  return "Unknown Error";
}

/**
 * The URL a project's console lives at. A project is addressed by subdomain
 * (`<tenant>_<project>`), so this is another origin, not another path.
 */
export function projectUrl(projectId: string): string {
  const tenant = deriveTenantId();
  const project = deriveProjectId();

  return window.location.origin.replace(
    `${tenant}_${project}`,
    `${tenant}_${projectId}`,
  );
}

/** Opens a project's console in a new tab. */
export function openProject(projectId: string): void {
  window.open(projectUrl(projectId), "_blank");
}

/** Where a resource type's listing lives, in either console. */
export function resourceListPath(resourceType: string): string {
  return `/r/${resourceType}`;
}

/** Where a single resource's editor lives, in either console. */
export function resourceInstancePath(
  resourceType: string,
  resourceId: string,
): string {
  return `${resourceListPath(resourceType)}/${resourceId}`;
}

const MAX_SLUG_LENGTH = 40;
/** A DNS label holds 63 characters; 8 are kept spare. */
const HOSTNAME_LABEL_MAX_LENGTH = 63 - 8;
const HOSTNAME_ID_SEPARATOR = "--";

/**
 * A project id shares one DNS label with its tenant (`{tenant}--{project}`),
 * so a longer tenant id leaves less room. Matches `validate_project_id` in
 * the repository crate.
 */
function maxSlugLength(): number {
  const tenantLength = (deriveTenantId() as string | undefined)?.length ?? 0;
  const room =
    HOSTNAME_LABEL_MAX_LENGTH - HOSTNAME_ID_SEPARATOR.length - tenantLength;

  return Math.max(0, Math.min(MAX_SLUG_LENGTH, room));
}

/**
 * Turns a typed name into an id the server accepts: lowercase alphanumerics
 * and `-`, matching `ID_CHARACTERS` in the repository crate (`_` is excluded
 * for FHIR compliance). The result also has to be a hostname label, so it
 * starts and ends alphanumeric.
 *
 * Empty when a name holds nothing usable, which callers read as "no slug".
 */
export function slugifyProjectName(name: string): string {
  return (
    name
      .toLowerCase()
      // Decompose accents, so the base letter survives rather than becoming
      // a dash.
      .normalize("NFKD")
      .replace(/[̀-ͯ]/g, "")
      .replace(/[^a-z0-9]+/g, "-")
      .replace(/-+/g, "-")
      .replace(/^-|-$/g, "")
      .slice(0, maxSlugLength())
      // Slicing can leave a trailing dash behind.
      .replace(/-$/, "")
  );
}

/**
 * A slug no existing project uses. Appends `-2`, `-3` and so on, trimming the
 * base rather than the suffix to stay within `maxSlugLength`.
 */
export function uniqueProjectSlug(
  slug: string,
  taken: Iterable<string>,
): string {
  const used = new Set(taken);
  if (!used.has(slug)) {
    return slug;
  }

  const maxLength = maxSlugLength();
  for (let suffix = 2; ; suffix += 1) {
    const tail = `-${suffix}`;
    const base = slug.slice(0, maxLength - tail.length).replace(/-$/, "");
    const candidate = `${base}${tail}`;
    if (!used.has(candidate)) {
      return candidate;
    }
  }
}
