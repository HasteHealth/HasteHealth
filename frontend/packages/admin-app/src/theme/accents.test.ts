import assert from "node:assert/strict";
import { test } from "node:test";

import { ACCENT_THEMES, DEFAULT_ACCENT, accentAttribute } from "./accents.ts";

test("teal is the default accent and has no data attribute", () => {
  assert.equal(DEFAULT_ACCENT, "teal");
  assert.equal(accentAttribute(DEFAULT_ACCENT), null);
});

test("every registered accent has the data the picker needs", () => {
  for (const theme of ACCENT_THEMES) {
    assert.ok(theme.id);
    assert.ok(theme.label);
    assert.match(theme.swatches.light, /^#[0-9a-f]{6}$/i);
    assert.match(theme.swatches.dark, /^#[0-9a-f]{6}$/i);
  }
});

test("ember is registered and written to data-accent", () => {
  assert.ok(ACCENT_THEMES.some((theme) => theme.id === "ember"));
  assert.equal(accentAttribute("ember"), "ember");
});

test("a custom accent id passes through", () => {
  assert.equal(accentAttribute("ocean"), "ocean");
});
