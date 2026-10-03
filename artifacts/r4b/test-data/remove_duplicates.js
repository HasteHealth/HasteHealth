import fs from "node:fs";
import path from "path";

function getResourceTypeName(file) {
  return file.split("-")[0];
}

function getTypes(files) {
  return [...new Set(files.map(getResourceTypeName))];
}

function dedupeDirecotry(directory) {
  const files = fs.readdirSync(directory);
  const resourceTypes = getTypes(files);
  for (const resourceType of resourceTypes) {
    const resourceFiles = files.filter(
      (file) => getResourceTypeName(file) === resourceType,
    );
    if (resourceFiles.length > 1) {
      const filesToDelete = resourceFiles.slice(1);
      for (const fileToDelete of filesToDelete) {
        fs.unlinkSync(path.join(directory, fileToDelete));
      }
    }
  }
}

// Fixed to the examples next to this script rather than a CLI argument so it
// can never delete files outside this package.
dedupeDirecotry(path.join(import.meta.dirname, "examples-json"));
