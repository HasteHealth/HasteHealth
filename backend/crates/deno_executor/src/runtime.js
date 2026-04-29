// runtime.js
const { core } = Deno;
// import { InteropObject } from "ext:core/ops";

// globalThis.InteropObject = InteropObject;
// globalThis.console.log("INTEROP:", globalThis.InteropObject);
function argsToMessage(...args) {
  return args.map((arg) => JSON.stringify(arg)).join(" ");
}

globalThis.stateCheck = () => core.ops.op_return_value();

globalThis.console = {
  log: (...args) => {
    core.print(`[out]: ${argsToMessage(...args)}\n`, false);
  },
  error: (...args) => {
    core.print(`[err]: ${argsToMessage(...args)}\n`, true);
  },
};
