#!/usr/bin/env node
// Regenerates everything on the site that is derived from somewhere else.
//
//   test-support  backend/testscripts/core + backend/testreports
//                   -> static/test-reports/support.json
//   server        the local Haste Health server (HASTE_HEALTH_* in .env)
//                   -> static/fhir/R4/metadata.json
//                      static/fhir/R4/OperationDefinition/<id>.json
//                      static/mcp/tools.json
//   fhir          artifacts/ + the two above
//                   -> docs/reference/fhir/** pages, src/fhir-model/*.json,
//                      static/fhir/R4/<Type>.json
//   social-card   support.json -> static/img/social-card.{svg,png}
//   pricing       the backend's subscription tiers (cargo) -> static/pricing/tiers.json
//
// The steps run in that order, each reading what the ones before it wrote,
// and the first failure stops the run.
//
//   pnpm generate                   everything
//   pnpm generate server fhir       only these steps
//   pnpm generate --check           fail when a committed output is stale; only
//                                   the steps that can tell (social-card, pricing)
//   pnpm generate test-support --postgres <bundle> --elasticsearch <bundle>
//                                   step options, documented in the step's module

const STEPS = ["test-support", "server", "fhir", "social-card", "pricing"];

function parse(argv) {
  const steps = [];
  const options = { check: false, args: {} };
  for (let i = 0; i < argv.length; i += 1) {
    const arg = argv[i];
    if (arg === "--check") {
      options.check = true;
    } else if (arg.startsWith("--")) {
      options.args[arg.slice(2)] = argv[i + 1];
      i += 1;
    } else {
      steps.push(arg);
    }
  }
  return { steps, options };
}

const { steps, options } = parse(process.argv.slice(2));
const unknown = steps.filter((step) => !STEPS.includes(step));
if (unknown.length > 0) {
  console.error(
    `Unknown step: ${unknown.join(", ")}. Steps: ${STEPS.join(", ")}.`,
  );
  process.exit(2);
}
const selected =
  steps.length > 0 ? STEPS.filter((step) => steps.includes(step)) : STEPS;

for (const name of selected) {
  const step = await import(`./generate/${name}.mjs`);
  if (options.check && !step.check) {
    console.log(`- ${name}: has no check, skipped`);
    continue;
  }
  const started = Date.now();
  try {
    const summary = await (options.check
      ? step.check(options)
      : step.run(options));
    const seconds = ((Date.now() - started) / 1000).toFixed(1);
    console.log(`✓ ${name} (${seconds}s): ${summary}`);
  } catch (error) {
    console.error(`✗ ${name}: ${error.message}`);
    process.exit(1);
  }
}
