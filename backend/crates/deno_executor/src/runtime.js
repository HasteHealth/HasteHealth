// runtime.js

((globalThis) => {
  const core = Deno.core;

  // globalThis.InteropObject = core.InteropObject;
  // globalThis.console.log("INTEROP:", globalThis.InteropObject);

  function argsToMessage(...args) {
    return args.map((arg) => JSON.stringify(arg)).join(" ");
  }

  globalThis.readResource = (resourceType, id) =>
    core.ops.read_resource(resourceType, id);

  globalThis.console = {
    log: (...args) => {
      core.print(`[out]: ${argsToMessage(...args)}\n`, false);
    },
    error: (...args) => {
      core.print(`[err]: ${argsToMessage(...args)}\n`, true);
    },
  };
})(globalThis);
