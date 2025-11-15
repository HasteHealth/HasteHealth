const CODE_REGEX = /[?&]code=[^&]+/;
const STATE_REGEX = /[?&]state=[^&]+/;
const ERROR_REGEX = /[?&]error=[^&]+/;

export const hasAuthQueryParams = (
  searchParams = window.location.search,
): boolean =>
  (CODE_REGEX.test(searchParams) && STATE_REGEX.test(searchParams)) ||
  ERROR_REGEX.test(searchParams);

/**
 * https://tools.ietf.org/html/rfc4648#section-5
 * @param string String to encode for URI
 * @returns Encoded string.
 */
function base64URIEncode(string: string) {
  return btoa(string)
    .replace(/\+/g, "-")
    .replace(/\//g, "_")
    .replace(/=+$/, "");
}

export function conditionalAddTenant(path: string, tenant?: string) {
  if (tenant) {
    return `/w/${tenant}${path}`;
  }
  return path;
}

function dec2hex(dec: number) {
  return dec.toString(16).padStart(2, "0");
}

export function generateRandomString(len: number) {
  const arr = new Uint8Array((len || 40) / 2);
  window.crypto.getRandomValues(arr);
  return Array.from(arr, dec2hex).join("");
}

export async function sha256(value: string): Promise<string> {
  const encoder = new TextEncoder();
  const data = encoder.encode(value);
  const hash = await window.crypto.subtle.digest("SHA-256", data);

  return base64URIEncode(
    String.fromCharCode.apply(
      null,
      new Uint8Array(hash) as unknown as number[],
    ),
  );
}
