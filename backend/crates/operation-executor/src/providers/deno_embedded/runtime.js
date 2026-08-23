// runtime.js
//
// Global environment for custom-operation scripts. Everything here runs
// inside the V8 snapshot baked at build time; `core.ops.*` calls are wired
// up to Rust ops at runtime (see providers/deno_embedded/mod.rs).

((root) => {
  const core = Deno.core;

  function argsToMessage(...args) {
    return args.map((arg) => JSON.stringify(arg)).join(" ");
  }

  // Accepts a query string ("name=Smith&_count=10", with or without a
  // leading "?") or a plain object of search parameters and normalizes it
  // to the query string the FHIR ops expect. Array values become
  // comma-joined FHIR "OR" values, matching FHIR search semantics.
  function toQueryString(query) {
    if (query === undefined || query === null || query === "") {
      return "";
    }
    if (typeof query === "string") {
      return query.startsWith("?") ? query.slice(1) : query;
    }
    if (typeof query !== "object" || Array.isArray(query)) {
      throw new TypeError(
        "query must be a query string or a plain object of search parameters",
      );
    }

    const parts = [];
    for (const [key, value] of Object.entries(query)) {
      if (value === undefined || value === null) {
        continue;
      }
      const values = Array.isArray(value) ? value : [value];
      parts.push(
        `${encodeURIComponent(key)}=${encodeURIComponent(values.join(","))}`,
      );
    }
    return parts.join("&");
  }

  function asParameters(parameters) {
    return parameters ?? { resourceType: "Parameters", parameter: [] };
  }

  root.fhir = {
    capabilities: () => core.ops.fhir_capabilities(),

    read: (resourceType, id) => core.ops.fhir_read(resourceType, id),
    vread: (resourceType, id, versionId) =>
      core.ops.fhir_vread(resourceType, id, versionId),

    create: (resourceType, resource) =>
      core.ops.fhir_create(resourceType, resource),
    update: (resourceType, id, resource) =>
      core.ops.fhir_update(resourceType, id, resource),
    conditionalUpdate: (resourceType, query, resource) =>
      core.ops.fhir_conditional_update(
        resourceType,
        toQueryString(query),
        resource,
      ),
    patch: (resourceType, id, patch) =>
      core.ops.fhir_patch(resourceType, id, patch),

    deleteInstance: (resourceType, id) =>
      core.ops.fhir_delete_instance(resourceType, id),
    deleteType: (resourceType, query) =>
      core.ops.fhir_delete_type(resourceType, toQueryString(query)),
    deleteSystem: (query) => core.ops.fhir_delete_system(toQueryString(query)),

    searchType: (resourceType, query) =>
      core.ops.fhir_search_type(resourceType, toQueryString(query)),
    searchSystem: (query) => core.ops.fhir_search_system(toQueryString(query)),

    historyInstance: (resourceType, id, query) =>
      core.ops.fhir_history_instance(resourceType, id, toQueryString(query)),
    historyType: (resourceType, query) =>
      core.ops.fhir_history_type(resourceType, toQueryString(query)),
    historySystem: (query) =>
      core.ops.fhir_history_system(toQueryString(query)),

    invokeInstance: (resourceType, id, operation, parameters) =>
      core.ops.fhir_invoke_instance(
        resourceType,
        id,
        operation,
        asParameters(parameters),
      ),
    invokeType: (resourceType, operation, parameters) =>
      core.ops.fhir_invoke_type(
        resourceType,
        operation,
        asParameters(parameters),
      ),
    invokeSystem: (operation, parameters) =>
      core.ops.fhir_invoke_system(operation, asParameters(parameters)),

    transaction: (bundle) => core.ops.fhir_transaction(bundle),
    batch: (bundle) => core.ops.fhir_batch(bundle),
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
    getInputValue: () => {
      return core.ops.get_input_value();
    },
  };
})(globalThis);
