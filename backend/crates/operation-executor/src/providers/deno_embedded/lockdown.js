// lockdown.js
//
// Runs in every isolate once deno_core has finished setting it up and before
// any custom-operation code (see `build_deno_runtime` in
// providers/deno_embedded/mod.rs). It takes away what would let a script hold
// memory that the limits in mod.rs do not count: `MAX_HEAP_BYTES` bounds the
// V8 heap and `MAX_ARRAY_BUFFER_BYTES` bounds what V8 allocates through the
// isolate's ArrayBuffer allocator, but V8 maps the pages of a resizable
// ArrayBuffer, a growable SharedArrayBuffer and a WebAssembly.Memory itself.

"use strict";

(() => {
  const { construct } = Reflect;

  // Wraps a buffer constructor so it only builds fixed-length buffers, which
  // V8 allocates through the isolate's ArrayBuffer allocator.
  function fixedLengthOnly(NativeBuffer, name) {
    const FixedLengthBuffer = new Proxy(NativeBuffer, {
      construct(target, args, newTarget) {
        const options = args[1];
        if (
          options !== null &&
          (typeof options === "object" || typeof options === "function") &&
          options.maxByteLength !== undefined
        ) {
          throw new TypeError(
            `${name} does not support maxByteLength in custom operation scripts`,
          );
        }
        return construct(target, [args[0]], newTarget);
      },
    });
    // A buffer reaches its constructor through the prototype, so point that
    // at the wrapper too.
    NativeBuffer.prototype.constructor = FixedLengthBuffer;
    return FixedLengthBuffer;
  }

  globalThis.ArrayBuffer = fixedLengthOnly(ArrayBuffer, "ArrayBuffer");
  globalThis.SharedArrayBuffer = fixedLengthOnly(
    SharedArrayBuffer,
    "SharedArrayBuffer",
  );

  delete globalThis.WebAssembly;

  // deno_core's internals keep the unwrapped constructors (its primordials)
  // and expose ops that hand back buffers allocated in Rust, which the
  // allocator's limit does not count either. The `fhir`, `console` and
  // `_internal_` globals hold their own reference to what they need.
  delete globalThis.Deno;
  delete globalThis.__bootstrap;
})();
