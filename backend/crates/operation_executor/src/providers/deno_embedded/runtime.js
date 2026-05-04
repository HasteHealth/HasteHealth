// runtime.js

((root) => {
  const core = Deno.core;

  function argsToMessage(...args) {
    return args.map((arg) => JSON.stringify(arg)).join(" ");
  }

  root.fhir = {
    readResource: (resourceType, id) =>
      core.ops.read_resource(resourceType, id),
  };

  root.console = {
    log: (...args) => {
      core.print(`[out]: ${argsToMessage(...args)}\n`, false);
    },
    error: (...args) => {
      core.print(`[err]: ${argsToMessage(...args)}\n`, true);
    },
  };

  root._internal_ = {
    setReturnValue: (value) => {
      core.ops.set_return_value(value);
    },
  };
})(globalThis);
