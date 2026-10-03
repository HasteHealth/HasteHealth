import { OperationOutcome } from "@haste-health/fhir-types/r4/types";
import { ProjectId, TenantId } from "@haste-health/jwt/types";

/**
 * Joins the tenant and project ids in the console's subdomain:
 * `{tenant}--{project}`. Matches `HOSTNAME_ID_SEPARATOR` in the repository
 * crate.
 */
const HOSTNAME_ID_SEPARATOR = "--";

/**
 * The separator consoles used before `--`: `{tenant}_{project}`. Old links
 * and bookmarks still carry it.
 */
const LEGACY_HOSTNAME_ID_SEPARATOR = "_";

/**
 * Where a console on a legacy `{tenant}_{project}` subdomain lives now, keeping
 * the port, path, search and hash. Undefined when the subdomain is current.
 */
export function legacyHostnameRedirectUrl(): string | undefined {
  const url = new URL(window.location.href);
  const [label, ...domain] = url.hostname.split(".");
  if (!label?.includes(LEGACY_HOSTNAME_ID_SEPARATOR)) {
    return undefined;
  }

  url.hostname = [
    label.replace(LEGACY_HOSTNAME_ID_SEPARATOR, HOSTNAME_ID_SEPARATOR),
    ...domain,
  ].join(".");

  return url.toString();
}

/** The `[tenant, project]` ids in this console's subdomain. */
function hostnameIds(): string[] {
  return window.location.host.split(".")[0]?.split(HOSTNAME_ID_SEPARATOR) ?? [];
}

export function deriveTenantId(): TenantId {
  return hostnameIds()[0] as TenantId;
}

export function deriveProjectId(): ProjectId {
  return hostnameIds()[1] as ProjectId;
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
 * (`<tenant>--<project>`), so this is another origin, not another path.
 */
export function projectUrl(projectId: string): string {
  const tenant = deriveTenantId();
  const project = deriveProjectId();

  return window.location.origin.replace(
    `${tenant}${HOSTNAME_ID_SEPARATOR}${project}`,
    `${tenant}${HOSTNAME_ID_SEPARATOR}${projectId}`,
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

/** Matches `HOSTNAME_ID_MIN_LEN` in the repository crate. */
export const MIN_SLUG_LENGTH = 3;
const MAX_SLUG_LENGTH = 40;
/** A DNS label holds 63 characters; 8 are kept spare. */
const HOSTNAME_LABEL_MAX_LENGTH = 63 - 8;

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
 * Turns a typed name into a project id the server accepts: lowercase letters
 * and digits joined by single hyphens (`HOSTNAME_ID_PATTERN` in the
 * repository crate).
 *
 * Empty when the name gives fewer than `MIN_SLUG_LENGTH` characters, which
 * callers read as "no slug".
 */
export function slugifyProjectName(name: string): string {
  const slug = name
    .toLowerCase()
    // Decompose accents, so the base letter survives rather than becoming
    // a dash.
    .normalize("NFKD")
    .replace(/[̀-ͯ]/g, "")
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-|-$/g, "")
    .slice(0, maxSlugLength())
    // Slicing can leave a trailing dash behind.
    .replace(/-$/, "");

  return slug.length < MIN_SLUG_LENGTH ? "" : slug;
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
