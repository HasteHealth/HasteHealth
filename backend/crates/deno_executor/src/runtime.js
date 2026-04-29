// runtime.js
const { core } = Deno;
import { InteropObject } from "ext:core/ops";

function argsToMessage(...args) {
  return args.map((arg) => JSON.stringify(arg)).join(" ");
}

globalThis.InteropObject = InteropObject;

globalThis.stateCheck = core.ops.op_return_value;

globalThis.console = {
  log: (...args) => {
    core.print(`[out]: ${argsToMessage(...args)}\n`, false);
  },
  error: (...args) => {
    core.print(`[err]: ${argsToMessage(...args)}\n`, true);
  },
};

globalThis.console.log("INTEROP:", globalThis.InteropObject);

// globalThis.runjs = {
//   readFile: (path) => {
//     return core.ops.op_read_file(path);
//   },
//   writeFile: (path, contents) => {
//     return core.ops.op_write_file(path, contents);
//   },
//   //   removeFile: (path) => {
//   //     return core.ops.op_remove_file(path);
//   //   },
// };
