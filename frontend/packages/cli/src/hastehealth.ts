import { Command } from "commander";

import { apiCommands } from "./commands/api.js";
import { codeGenerationCommands } from "./commands/generation.js";
import { minimizeCommands } from "./commands/minimize.js";
import { testscriptCommands } from "./commands/testscript.js";
import { validate } from "./commands/validate.js";
import { configurationCommands } from "./config.js";

const program = new Command();

program.name("HasteHealth CLI Tool").description("HasteHealth CLI interface.");

configurationCommands(program.command("config"));
codeGenerationCommands(program.command("generate"));
apiCommands(program.command("api"));
validate(program);
minimizeCommands(program.command("minimize"));
testscriptCommands(program.command("test"));

program.parse();
