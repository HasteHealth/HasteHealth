#!/usr/bin/env node
import fs from "node:fs";
import path from "node:path";

// Build step: prepend a shebang to a compiled entry point so it can be run
// directly. The target must stay inside this package's build output, so the
// argument is resolved and checked against ./lib before it is touched.
const libDir = path.resolve(import.meta.dirname, "lib");
const target = path.resolve(import.meta.dirname, process.argv[2] ?? "");

if (path.relative(libDir, target).startsWith("..")) {
  throw new Error(`Refusing to write outside ${libDir}: ${target}`);
}

const data = "#!/usr/bin/env node\n\n" + fs.readFileSync(target, "utf8");
fs.writeFileSync(target, data);
