#!/usr/bin/env node
import fs from "fs";

const path = process.argv[2];
let data = "#!/usr/bin/env node\n\n";
data += fs.readFileSync(path);
fs.writeFileSync(path, data);
