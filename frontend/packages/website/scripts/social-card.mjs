#!/usr/bin/env node
/**
 * Build the social card (og:image / twitter:image).
 *
 * The card used to be a hand-made PNG committed as a binary, with nothing tying
 * it to the site's copy. It drifted: the site's H1 changed and the card kept
 * advertising "The FHIR Data Layer for AI Agents", so a link preview in Slack
 * and the page it opened named two different products. That is the same "visits
 * twice, sees two products" problem the buyer-side site review opened with.
 *
 * So the card is generated. The headline and subhead live here in one place,
 * and the proof figures are read from the conformance report that the homepage
 * and the conformance page already use, so a card cannot claim a number the
 * site does not.
 *
 * Usage, from frontend/packages/website:
 *
 *   node scripts/social-card.mjs          # write the SVG and rasterize to PNG
 *   node scripts/social-card.mjs --check  # fail if the committed SVG is stale
 *
 * Rasterizing uses macOS `qlmanage`, which is the one renderer we can rely on
 * being present without adding a dependency. On a machine without it the SVG is
 * still written and the PNG is left alone, with a warning: the SVG is the
 * source of truth and the PNG is a build product that we commit because the
 * social platforms need a raster.
 */

import { execFileSync } from "node:child_process";
import { existsSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const here = dirname(fileURLToPath(import.meta.url));
const root = resolve(here, "..");

const SVG_PATH = join(root, "static/img/social-card.svg");
const PNG_PATH = join(root, "static/img/social-card.png");
const SUPPORT_PATH = join(root, "static/test-reports/support.json");

// Open Graph's recommended size. Slack, LinkedIn and X all crop toward the
// centre, so nothing load-bearing goes near the edges.
const WIDTH = 1200;
const HEIGHT = 630;

// ---------------------------------------------------------------------------
// The copy. This is the card's whole reason to be generated: it must match the
// homepage H1 in src/pages/index.tsx and the subhead beneath it.
// ---------------------------------------------------------------------------

const HEADLINE = ["The Open-Source FHIR Server", "for AI-Native Health Apps"];
const SUBHEAD = "Epic, Oracle Health and HL7v2 in. FHIR, SQL and MCP out.";

/** The logo's teal, from static/img/logo.svg. */
const TEAL = "#0d9488";

/**
 * The proof strip, read from the same conformance report the site renders.
 *
 * Only figures the report actually contains. The commit count and licence are
 * static facts about the repository rather than measurements.
 */
function proofLine() {
  const data = JSON.parse(readFileSync(SUPPORT_PATH, "utf8"));
  const total = data.resources.length;

  // A resource type "passes" when no group failed and none is still unrun.
  // Mirrors groupsResult() in src/components/TestCoverage/data.ts.
  const passing = data.resources.filter((resource) => {
    const results = resource.groups.map((group) => group.results.postgres);
    if (results.some((result) => result === "fail")) return false;
    if (results.some((result) => result === "warn")) return false;
    return results.every((result) => result === "pass");
  }).length;

  return `${passing} / ${total} FHIR resource types pass every conformance check   ·   Apache-2.0`;
}

/**
 * The card.
 *
 * Text is drawn as SVG `<text>` in a system font stack rather than a webfont:
 * the renderer has no network, and a missing webfont would silently fall back
 * to something worse than a deliberate system face.
 */
function buildSvg() {
  const font =
    "-apple-system, BlinkMacSystemFont, 'Segoe UI', Helvetica, Arial, sans-serif";

  const headline = HEADLINE.map(
    (line, index) =>
      `    <text x="80" y="${300 + index * 68}" font-family="${font}" font-size="58" font-weight="700" fill="#ffffff" letter-spacing="-1.2">${line}</text>`,
  ).join("\n");

  return `<svg xmlns="http://www.w3.org/2000/svg" width="${WIDTH}" height="${HEIGHT}" viewBox="0 0 ${WIDTH} ${HEIGHT}" role="img">
  <title>Haste Health — ${HEADLINE.join(" ")}</title>

  <!-- Rendered standalone via file:// when rasterizing, where the SVG becomes
       the document root and picks up the UA's default body margin. Pinning it
       to the viewport keeps the screenshot flush to the card's edges. -->
  <style>
    @media screen { :root { margin: 0; padding: 0; } }
  </style>

  <defs>
    <linearGradient id="bg" x1="0" y1="0" x2="1" y2="1">
      <stop offset="0%" stop-color="#0f766e"/>
      <stop offset="55%" stop-color="#0d9488"/>
      <stop offset="100%" stop-color="#115e59"/>
    </linearGradient>
  </defs>

  <rect width="${WIDTH}" height="${HEIGHT}" fill="url(#bg)"/>

  <!-- The pulse line from the site's hero, as a quiet watermark. Kept low and
       to the right of the logo so it never crosses the headline or the proof
       strip: at a Slack thumbnail's size, a stroke through the text reads as an
       artefact rather than as texture. -->
  <path d="M600 196 L760 196 L788 196 L812 132 L840 260 L864 196 L944 196 L972 146 L996 246 L1020 196 L1200 196"
        fill="none" stroke="#ffffff" stroke-width="3" opacity="0.16"/>

  <!-- Logo mark: the three bars from static/img/logo.svg. -->
  <g transform="translate(80, 74) scale(0.46)">
    <rect x="6"  y="22" width="24" height="56"  rx="6" fill="#ffffff" opacity="0.5"/>
    <rect x="38" y="0"  width="24" height="100" rx="6" fill="#ffffff"/>
    <rect x="70" y="32" width="24" height="36"  rx="6" fill="#ffffff" opacity="0.75"/>
  </g>
  <text x="140" y="112" font-family="${font}" font-size="38" font-weight="600" fill="#ffffff">haste<tspan opacity="0.72">.health</tspan></text>

${headline}

  <text x="80" y="428" font-family="${font}" font-size="30" font-weight="400" fill="#d9f2ee">${SUBHEAD}</text>

  <!-- Proof strip. -->
  <line x1="80" y1="486" x2="1120" y2="486" stroke="#ffffff" stroke-width="1.5" opacity="0.28"/>
  <text x="80" y="536" font-family="${font}" font-size="24" font-weight="500" fill="#ffffff" opacity="0.93">${proofLine()}</text>
</svg>
`;
}

/**
 * Where to find a headless Chrome to rasterize with.
 *
 * Chrome is used rather than macOS `qlmanage`, which was the obvious choice and
 * is wrong: `qlmanage -t -s N` fits the render into an N-by-N box and returns a
 * square, so a 1200x630 card comes back 1200x1200 with the headline and the
 * proof strip cropped off. `--window-size` renders at exactly the size asked
 * for.
 */
const CHROME_CANDIDATES = [
  process.env.CHROME_PATH,
  "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
  "/Applications/Chromium.app/Contents/MacOS/Chromium",
  "/usr/bin/google-chrome",
  "/usr/bin/chromium",
  "/usr/bin/chromium-browser",
].filter(Boolean);

function findChrome() {
  return CHROME_CANDIDATES.find((candidate) => existsSync(candidate)) ?? "";
}

/** Rasterize the card at exactly WIDTH x HEIGHT. */
function rasterize(svgPath, pngPath) {
  const chrome = findChrome();

  if (!chrome) {
    console.warn(
      "! No Chrome or Chromium found: wrote the SVG but left social-card.png\n" +
        "  untouched. Set CHROME_PATH, or rasterize the SVG at exactly " +
        `${WIDTH}x${HEIGHT} elsewhere and commit the PNG, or the preview stays stale.`,
    );
    return false;
  }

  const scratch = mkdtempSync(join(tmpdir(), "social-card-"));
  try {
    execFileSync(
      chrome,
      [
        "--headless",
        "--disable-gpu",
        // The card is opaque, but without this Chrome composites onto white,
        // which would fringe the rounded logo bars.
        "--default-background-color=00000000",
        "--hide-scrollbars",
        `--window-size=${WIDTH},${HEIGHT}`,
        `--screenshot=${join(scratch, "card.png")}`,
        `file://${svgPath}`,
      ],
      { stdio: "ignore" },
    );
    const produced = join(scratch, "card.png");
    if (!existsSync(produced)) throw new Error("Chrome produced no screenshot");
    writeFileSync(pngPath, readFileSync(produced));
    return true;
  } finally {
    rmSync(scratch, { recursive: true, force: true });
  }
}

const svg = buildSvg();

if (process.argv.includes("--check")) {
  const current = existsSync(SVG_PATH) ? readFileSync(SVG_PATH, "utf8") : "";
  if (current !== svg) {
    console.error(
      "social-card.svg is stale. Run `node scripts/social-card.mjs` and commit the result.",
    );
    process.exit(1);
  }
  console.log("social-card.svg is up to date.");
  process.exit(0);
}

writeFileSync(SVG_PATH, svg);
console.log(`wrote ${SVG_PATH}`);
if (rasterize(SVG_PATH, PNG_PATH)) {
  console.log(`wrote ${PNG_PATH}`);
}
