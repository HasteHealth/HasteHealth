import { execFileSync } from "node:child_process";
import { readFileSync, readdirSync, writeFileSync, unlinkSync } from "node:fs";
import { join } from "node:path";
import { tmpdir } from "node:os";

const __dirname = import.meta.dirname;

function flattenData(data) {
  if (data.resourceType === "Bundle") {
    console.log(`Flattening Bundle with ${data.entry.length} entries ...`);

    return data.entry.flatMap((e) => flattenData(e.resource));
  }
  return [data];
}

function loadUSCoreProfiles() {
  const artifactsDir = join(
    __dirname,
    "../crates/artifacts/artifacts/r4/us-core",
  );
  const files = readdirSync(artifactsDir).filter((f) => f.endsWith(".json"));

  const entries = files.flatMap((filename) => {
    const resource = JSON.parse(
      readFileSync(join(artifactsDir, filename), "utf-8"),
    );

    const resources = flattenData(resource);

    if (resources.some((r) => !r.resourceType || !r.id)) {
      throw new Error(
        `Invalid resource in file ${filename}: missing resourceType or id`,
      );
    }

    return resources.map((resource) => ({
      resource,
      request: {
        method: "PUT",
        url: `${resource.resourceType}/${resource.id}`,
      },
    }));
  });

  const bundle = {
    resourceType: "Bundle",
    type: "transaction",
    entry: entries,
  };

  const tmpFile = join(tmpdir(), "us-core-transaction.json");
  writeFileSync(tmpFile, JSON.stringify(bundle));

  console.log(`Sending transaction with ${entries.length} entries ...`);

  try {
    execFileSync(
      join(__dirname, "../target/debug/haste-health"),
      ["api", "transaction", "--file", tmpFile],
      {
        stdio: "inherit",
      },
    );
  } finally {
    unlinkSync(tmpFile);
  }

  console.log("Done loading US Core artifacts.");
}

loadUSCoreProfiles();
