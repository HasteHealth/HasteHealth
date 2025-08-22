import typescript from "rollup-plugin-ts";

export default {
  input: "src/index.ts",
  external: (id) => id != "tslib" && !/^(\.?\/|\w:)/.test(id),
  output: [
    { file: "lib/index.cjs", format: "cjs" },
    { dir: "./lib", format: "es" },
  ],
  plugins: [typescript()],
};
