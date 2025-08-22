// See [https://datatracker.ietf.org/doc/html/rfc6901#section-3] for reference.
export function escapeField(field: string) {
  return field.replace("~", "~0").replace("/", "~1");
}

export function unescapeField(field: string) {
  return field.replace("~1", "/").replace("~0", "~");
}
